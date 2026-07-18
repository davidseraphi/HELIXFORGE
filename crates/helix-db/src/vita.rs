//! HelixVita Prime durable store — `vita` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Study {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub recruiting_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cohort {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub enrolled_at: Option<DateTime<Utc>>,
    pub withdrawn_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VitaSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_cohorts: i64,
    pub draft_cohorts: i64,
    pub enrolled_cohorts: i64,
    pub withdrawn_cohorts: i64,
}

#[derive(Debug, Clone, Default)]
pub struct StudyUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct CohortUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a study lifecycle transition and return the resulting status.
pub fn next_study_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "recruit") => Ok("recruiting"),
        ("recruiting", "complete") => Ok("completed"),
        ("draft", "terminate") | ("recruiting", "terminate") => Ok("terminated"),
        (_, "recruit") => Err(HelixError::validation(format!(
            "cannot recruit a {current} study"
        ))),
        (_, "complete") => Err(HelixError::validation(format!(
            "cannot complete a {current} study"
        ))),
        (_, "terminate") => Err(HelixError::validation(format!(
            "cannot terminate a {current} study"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown study action {action}"
        ))),
    }
}

/// Validate a cohort lifecycle transition and return the resulting status.
pub fn next_cohort_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "enroll") => Ok("enrolled"),
        ("draft", "withdraw") | ("enrolled", "withdraw") => Ok("withdrawn"),
        (_, "enroll") => Err(HelixError::validation(format!(
            "cannot enroll a {current} cohort"
        ))),
        (_, "withdraw") => Err(HelixError::validation(format!(
            "cannot withdraw a {current} cohort"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown cohort action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct StudyRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    recruiting_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    terminated_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl StudyRow {
    fn into_study(self) -> Study {
        Study {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            recruiting_at: self.recruiting_at,
            completed_at: self.completed_at,
            terminated_at: self.terminated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CohortRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    enrolled_at: Option<DateTime<Utc>>,
    withdrawn_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl CohortRow {
    fn into_cohort(self) -> Cohort {
        Cohort {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            enrolled_at: self.enrolled_at,
            withdrawn_at: self.withdrawn_at,
            deleted_at: self.deleted_at,
        }
    }
}

const STUDY_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           recruiting_at, completed_at, terminated_at, deleted_at
    FROM vita.studies
"#;

const STUDY_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              recruiting_at, completed_at, terminated_at, deleted_at
"#;

const COHORT_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, enrolled_at, withdrawn_at, deleted_at
    FROM vita.cohorts
"#;

const COHORT_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, enrolled_at, withdrawn_at, deleted_at
"#;

#[derive(Clone)]
pub struct VitaRepo {
    pool: PgPool,
}

impl VitaRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Studies ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<Study>> {
        let rows: Vec<StudyRow> = sqlx::query_as(&format!(
            "{STUDY_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita list: {e}")))?;
        Ok(rows.into_iter().map(StudyRow::into_study).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Study> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: StudyRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO vita.studies
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {STUDY_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(&metadata)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita create: {e}")))?;
        Ok(row.into_study())
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Study>> {
        let row: Option<StudyRow> = sqlx::query_as(&format!(
            "{STUDY_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita get: {e}")))?;
        Ok(row.map(StudyRow::into_study))
    }

    async fn fetch_study_any(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Study>> {
        let row: Option<StudyRow> =
            sqlx::query_as(&format!("{STUDY_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("vita fetch study: {e}")))?;
        Ok(row.map(StudyRow::into_study))
    }

    pub async fn update_study(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
        update: StudyUpdate,
    ) -> HelixResult<Study> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE vita.studies SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(n) = update.name {
            builder.push(", name = ");
            builder.push_bind(n);
        }
        if let Some(d) = update.description {
            builder.push(", description = ");
            builder.push_bind(d);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(study_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {STUDY_RETURNING}"));

        let row: Option<StudyRow> = builder
            .build_query_as::<StudyRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("vita update study: {e}")))?;

        row.map(StudyRow::into_study)
            .ok_or_else(|| HelixError::not_found("study not found"))
    }

    pub async fn recruit_study(&self, tenant_id: TenantId, study_id: Uuid) -> HelixResult<Study> {
        let study = self
            .get_parent(tenant_id, study_id)
            .await?
            .ok_or_else(|| HelixError::not_found("study not found"))?;
        let next = next_study_status(&study.status, "recruit")?;
        let now = Utc::now();
        let row: Option<StudyRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.studies
            SET status = $1, recruiting_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {STUDY_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita recruit study: {e}")))?;

        row.map(StudyRow::into_study)
            .ok_or_else(|| HelixError::not_found("study not found"))
    }

    /// Complete a recruiting study. Rejected while draft cohorts remain.
    pub async fn complete_study(&self, tenant_id: TenantId, study_id: Uuid) -> HelixResult<Study> {
        let study = self
            .get_parent(tenant_id, study_id)
            .await?
            .ok_or_else(|| HelixError::not_found("study not found"))?;
        let next = next_study_status(&study.status, "complete")?;

        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vita.cohorts WHERE tenant_id = $1 AND parent_id = $2 AND status = 'draft' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita complete cohort count: {e}")))?;
        if drafts > 0 {
            return Err(HelixError::validation(format!(
                "study has {drafts} draft cohort(s); enroll or withdraw them first"
            )));
        }

        let now = Utc::now();
        let row: Option<StudyRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.studies
            SET status = $1, completed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {STUDY_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita complete study: {e}")))?;

        row.map(StudyRow::into_study)
            .ok_or_else(|| HelixError::not_found("study not found"))
    }

    pub async fn terminate_study(&self, tenant_id: TenantId, study_id: Uuid) -> HelixResult<Study> {
        let study = self
            .get_parent(tenant_id, study_id)
            .await?
            .ok_or_else(|| HelixError::not_found("study not found"))?;
        let next = next_study_status(&study.status, "terminate")?;
        let now = Utc::now();
        let row: Option<StudyRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.studies
            SET status = $1, terminated_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {STUDY_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita terminate study: {e}")))?;

        row.map(StudyRow::into_study)
            .ok_or_else(|| HelixError::not_found("study not found"))
    }

    pub async fn soft_delete_study(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
    ) -> HelixResult<Study> {
        let study = self
            .get_parent(tenant_id, study_id)
            .await?
            .ok_or_else(|| HelixError::not_found("study not found"))?;
        if study.status == "deleted" {
            return Err(HelixError::validation("study is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<StudyRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.studies
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {STUDY_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita soft-delete study: {e}")))?;

        row.map(StudyRow::into_study)
            .ok_or_else(|| HelixError::not_found("study not found"))
    }

    /// Restore a soft-deleted study, returning it to its pre-delete status.
    pub async fn restore_study(&self, tenant_id: TenantId, study_id: Uuid) -> HelixResult<Study> {
        let study = self
            .fetch_study_any(tenant_id, study_id)
            .await?
            .ok_or_else(|| HelixError::not_found("study not found"))?;
        if study.deleted_at.is_none() {
            return Err(HelixError::validation("study is not deleted"));
        }
        let restored = if study.terminated_at.is_some() {
            "terminated"
        } else if study.completed_at.is_some() {
            "completed"
        } else if study.recruiting_at.is_some() {
            "recruiting"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<StudyRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.studies
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {STUDY_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita restore study: {e}")))?;

        row.map(StudyRow::into_study)
            .ok_or_else(|| HelixError::not_found("study not found or not deleted"))
    }

    // --- Cohorts ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Cohort>> {
        let rows: Vec<CohortRow> = sqlx::query_as(&format!(
            "{COHORT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita list children: {e}")))?;
        Ok(rows.into_iter().map(CohortRow::into_cohort).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Cohort> {
        let _parent = self
            .get_parent(tenant_id, parent_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: CohortRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO vita.cohorts
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,'draft',$6,$7,$7)
            {COHORT_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .bind(title)
        .bind(body)
        .bind(&metadata)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita create child: {e}")))?;
        Ok(row.into_cohort())
    }

    pub async fn get_cohort(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
        cohort_id: Uuid,
    ) -> HelixResult<Option<Cohort>> {
        let row: Option<CohortRow> = sqlx::query_as(&format!(
            "{COHORT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .bind(cohort_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita get cohort: {e}")))?;
        Ok(row.map(CohortRow::into_cohort))
    }

    async fn fetch_cohort_any(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
        cohort_id: Uuid,
    ) -> HelixResult<Option<Cohort>> {
        let row: Option<CohortRow> = sqlx::query_as(&format!(
            "{COHORT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .bind(cohort_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita fetch cohort: {e}")))?;
        Ok(row.map(CohortRow::into_cohort))
    }

    pub async fn update_cohort(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
        cohort_id: Uuid,
        update: CohortUpdate,
    ) -> HelixResult<Cohort> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE vita.cohorts SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(t) = update.title {
            builder.push(", title = ");
            builder.push_bind(t);
        }
        if let Some(b) = update.body {
            builder.push(", body = ");
            builder.push_bind(b);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND parent_id = ");
        builder.push_bind(study_id);
        builder.push(" AND id = ");
        builder.push_bind(cohort_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {COHORT_RETURNING}"));

        let row: Option<CohortRow> = builder
            .build_query_as::<CohortRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("vita update cohort: {e}")))?;

        row.map(CohortRow::into_cohort)
            .ok_or_else(|| HelixError::not_found("cohort not found"))
    }

    pub async fn enroll_cohort(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
        cohort_id: Uuid,
    ) -> HelixResult<Cohort> {
        let cohort = self
            .get_cohort(tenant_id, study_id, cohort_id)
            .await?
            .ok_or_else(|| HelixError::not_found("cohort not found"))?;
        let next = next_cohort_status(&cohort.status, "enroll")?;
        let now = Utc::now();
        let row: Option<CohortRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.cohorts
            SET status = $1, enrolled_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NULL
            {COHORT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .bind(cohort_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita enroll cohort: {e}")))?;

        row.map(CohortRow::into_cohort)
            .ok_or_else(|| HelixError::not_found("cohort not found"))
    }

    pub async fn withdraw_cohort(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
        cohort_id: Uuid,
    ) -> HelixResult<Cohort> {
        let cohort = self
            .get_cohort(tenant_id, study_id, cohort_id)
            .await?
            .ok_or_else(|| HelixError::not_found("cohort not found"))?;
        let next = next_cohort_status(&cohort.status, "withdraw")?;
        let now = Utc::now();
        let row: Option<CohortRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.cohorts
            SET status = $1, withdrawn_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NULL
            {COHORT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .bind(cohort_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita withdraw cohort: {e}")))?;

        row.map(CohortRow::into_cohort)
            .ok_or_else(|| HelixError::not_found("cohort not found"))
    }

    pub async fn soft_delete_cohort(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
        cohort_id: Uuid,
    ) -> HelixResult<Cohort> {
        let cohort = self
            .get_cohort(tenant_id, study_id, cohort_id)
            .await?
            .ok_or_else(|| HelixError::not_found("cohort not found"))?;
        if cohort.status == "deleted" {
            return Err(HelixError::validation("cohort is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<CohortRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.cohorts
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {COHORT_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .bind(cohort_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita soft-delete cohort: {e}")))?;

        row.map(CohortRow::into_cohort)
            .ok_or_else(|| HelixError::not_found("cohort not found"))
    }

    /// Restore a soft-deleted cohort, returning it to its pre-delete status.
    pub async fn restore_cohort(
        &self,
        tenant_id: TenantId,
        study_id: Uuid,
        cohort_id: Uuid,
    ) -> HelixResult<Cohort> {
        let cohort = self
            .fetch_cohort_any(tenant_id, study_id, cohort_id)
            .await?
            .ok_or_else(|| HelixError::not_found("cohort not found"))?;
        if cohort.deleted_at.is_none() {
            return Err(HelixError::validation("cohort is not deleted"));
        }
        let restored = if cohort.withdrawn_at.is_some() {
            "withdrawn"
        } else if cohort.enrolled_at.is_some() {
            "enrolled"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<CohortRow> = sqlx::query_as(&format!(
            r#"
            UPDATE vita.cohorts
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {COHORT_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(study_id)
        .bind(cohort_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita restore cohort: {e}")))?;

        row.map(CohortRow::into_cohort)
            .ok_or_else(|| HelixError::not_found("cohort not found or not deleted"))
    }

    // --- Reports ---

    /// Per-study cohort counts by status for non-deleted studies.
    pub async fn get_vita_summary(&self, tenant_id: TenantId) -> HelixResult<Vec<VitaSummaryRow>> {
        let rows: Vec<VitaSummaryRow> = sqlx::query_as(
            r#"
            SELECT s.id, s.name, s.status,
                   COUNT(c.id) AS total_cohorts,
                   COUNT(c.id) FILTER (WHERE c.status = 'draft') AS draft_cohorts,
                   COUNT(c.id) FILTER (WHERE c.status = 'enrolled') AS enrolled_cohorts,
                   COUNT(c.id) FILTER (WHERE c.status = 'withdrawn') AS withdrawn_cohorts
            FROM vita.studies s
            LEFT JOIN vita.cohorts c
                   ON c.parent_id = s.id AND c.tenant_id = s.tenant_id
                  AND c.deleted_at IS NULL
            WHERE s.tenant_id = $1 AND s.deleted_at IS NULL
            GROUP BY s.id, s.name, s.status, s.created_at
            ORDER BY s.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita summary: {e}")))?;
        Ok(rows)
    }
}
