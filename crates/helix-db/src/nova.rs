//! HelixNova Labs durable store — `nova` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub concluded_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NovaSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_findings: i64,
    pub draft_findings: i64,
    pub confirmed_findings: i64,
    pub rejected_findings: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ExperimentUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct FindingUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate an experiment lifecycle transition and return the resulting status.
pub fn next_experiment_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "start") => Ok("running"),
        ("running", "conclude") => Ok("concluded"),
        ("concluded", "reopen") => Ok("running"),
        (_, "start") => Err(HelixError::validation(format!(
            "cannot start a {current} experiment"
        ))),
        (_, "conclude") => Err(HelixError::validation(format!(
            "cannot conclude a {current} experiment"
        ))),
        (_, "reopen") => Err(HelixError::validation(format!(
            "cannot reopen a {current} experiment"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown experiment action {action}"
        ))),
    }
}

/// Validate a finding lifecycle transition and return the resulting status.
pub fn next_finding_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "confirm") => Ok("confirmed"),
        ("draft", "reject") | ("confirmed", "reject") => Ok("rejected"),
        (_, "confirm") => Err(HelixError::validation(format!(
            "cannot confirm a {current} finding"
        ))),
        (_, "reject") => Err(HelixError::validation(format!(
            "cannot reject a {current} finding"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown finding action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct ExperimentRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    concluded_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ExperimentRow {
    fn into_experiment(self) -> Experiment {
        Experiment {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            started_at: self.started_at,
            concluded_at: self.concluded_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FindingRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    confirmed_at: Option<DateTime<Utc>>,
    rejected_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl FindingRow {
    fn into_finding(self) -> Finding {
        Finding {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            confirmed_at: self.confirmed_at,
            rejected_at: self.rejected_at,
            deleted_at: self.deleted_at,
        }
    }
}

const EXPERIMENT_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           started_at, concluded_at, deleted_at
    FROM nova.experiments
"#;

const EXPERIMENT_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              started_at, concluded_at, deleted_at
"#;

const FINDING_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, confirmed_at, rejected_at, deleted_at
    FROM nova.findings
"#;

const FINDING_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, confirmed_at, rejected_at, deleted_at
"#;

#[derive(Clone)]
pub struct NovaRepo {
    pool: PgPool,
}

impl NovaRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Experiments ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<Experiment>> {
        let rows: Vec<ExperimentRow> = sqlx::query_as(&format!(
            "{EXPERIMENT_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(ExperimentRow::into_experiment)
            .collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Experiment> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: ExperimentRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO nova.experiments
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {EXPERIMENT_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("nova create: {e}")))?;
        Ok(row.into_experiment())
    }

    pub async fn get_parent(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<Experiment>> {
        let row: Option<ExperimentRow> = sqlx::query_as(&format!(
            "{EXPERIMENT_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova get: {e}")))?;
        Ok(row.map(ExperimentRow::into_experiment))
    }

    async fn fetch_experiment_any(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<Experiment>> {
        let row: Option<ExperimentRow> = sqlx::query_as(&format!(
            "{EXPERIMENT_SELECT} WHERE tenant_id = $1 AND id = $2"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova fetch experiment: {e}")))?;
        Ok(row.map(ExperimentRow::into_experiment))
    }

    pub async fn update_experiment(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
        update: ExperimentUpdate,
    ) -> HelixResult<Experiment> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE nova.experiments SET updated_at = ");
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
        builder.push_bind(experiment_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {EXPERIMENT_RETURNING}"));

        let row: Option<ExperimentRow> = builder
            .build_query_as::<ExperimentRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("nova update experiment: {e}")))?;

        row.map(ExperimentRow::into_experiment)
            .ok_or_else(|| HelixError::not_found("experiment not found"))
    }

    pub async fn start_experiment(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
    ) -> HelixResult<Experiment> {
        let experiment = self
            .get_parent(tenant_id, experiment_id)
            .await?
            .ok_or_else(|| HelixError::not_found("experiment not found"))?;
        let next = next_experiment_status(&experiment.status, "start")?;
        let now = Utc::now();
        let row: Option<ExperimentRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.experiments
            SET status = $1, started_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {EXPERIMENT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova start experiment: {e}")))?;

        row.map(ExperimentRow::into_experiment)
            .ok_or_else(|| HelixError::not_found("experiment not found"))
    }

    /// Conclude a running experiment. Rejected while draft findings remain.
    pub async fn conclude_experiment(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
    ) -> HelixResult<Experiment> {
        let experiment = self
            .get_parent(tenant_id, experiment_id)
            .await?
            .ok_or_else(|| HelixError::not_found("experiment not found"))?;
        let next = next_experiment_status(&experiment.status, "conclude")?;

        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM nova.findings WHERE tenant_id = $1 AND parent_id = $2 AND status = 'draft' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova conclude finding count: {e}")))?;
        if drafts > 0 {
            return Err(HelixError::validation(format!(
                "experiment has {drafts} draft finding(s); confirm or reject them first"
            )));
        }

        let now = Utc::now();
        let row: Option<ExperimentRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.experiments
            SET status = $1, concluded_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {EXPERIMENT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova conclude experiment: {e}")))?;

        row.map(ExperimentRow::into_experiment)
            .ok_or_else(|| HelixError::not_found("experiment not found"))
    }

    pub async fn reopen_experiment(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
    ) -> HelixResult<Experiment> {
        let experiment = self
            .get_parent(tenant_id, experiment_id)
            .await?
            .ok_or_else(|| HelixError::not_found("experiment not found"))?;
        let next = next_experiment_status(&experiment.status, "reopen")?;
        let now = Utc::now();
        let row: Option<ExperimentRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.experiments
            SET status = $1, concluded_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {EXPERIMENT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova reopen experiment: {e}")))?;

        row.map(ExperimentRow::into_experiment)
            .ok_or_else(|| HelixError::not_found("experiment not found"))
    }

    pub async fn soft_delete_experiment(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
    ) -> HelixResult<Experiment> {
        let experiment = self
            .get_parent(tenant_id, experiment_id)
            .await?
            .ok_or_else(|| HelixError::not_found("experiment not found"))?;
        if experiment.status == "deleted" {
            return Err(HelixError::validation("experiment is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<ExperimentRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.experiments
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {EXPERIMENT_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova soft-delete experiment: {e}")))?;

        row.map(ExperimentRow::into_experiment)
            .ok_or_else(|| HelixError::not_found("experiment not found"))
    }

    /// Restore a soft-deleted experiment, returning it to its pre-delete status.
    pub async fn restore_experiment(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
    ) -> HelixResult<Experiment> {
        let experiment = self
            .fetch_experiment_any(tenant_id, experiment_id)
            .await?
            .ok_or_else(|| HelixError::not_found("experiment not found"))?;
        if experiment.deleted_at.is_none() {
            return Err(HelixError::validation("experiment is not deleted"));
        }
        let restored = if experiment.concluded_at.is_some() {
            "concluded"
        } else if experiment.started_at.is_some() {
            "running"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<ExperimentRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.experiments
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {EXPERIMENT_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova restore experiment: {e}")))?;

        row.map(ExperimentRow::into_experiment)
            .ok_or_else(|| HelixError::not_found("experiment not found or not deleted"))
    }

    // --- Findings ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Finding>> {
        let rows: Vec<FindingRow> = sqlx::query_as(&format!(
            "{FINDING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova list children: {e}")))?;
        Ok(rows.into_iter().map(FindingRow::into_finding).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Finding> {
        let _parent = self
            .get_parent(tenant_id, parent_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: FindingRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO nova.findings
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,'draft',$6,$7,$7)
            {FINDING_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("nova create child: {e}")))?;
        Ok(row.into_finding())
    }

    pub async fn get_finding(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
        finding_id: Uuid,
    ) -> HelixResult<Option<Finding>> {
        let row: Option<FindingRow> = sqlx::query_as(&format!(
            "{FINDING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .bind(finding_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova get finding: {e}")))?;
        Ok(row.map(FindingRow::into_finding))
    }

    async fn fetch_finding_any(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
        finding_id: Uuid,
    ) -> HelixResult<Option<Finding>> {
        let row: Option<FindingRow> = sqlx::query_as(&format!(
            "{FINDING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .bind(finding_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova fetch finding: {e}")))?;
        Ok(row.map(FindingRow::into_finding))
    }

    pub async fn update_finding(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
        finding_id: Uuid,
        update: FindingUpdate,
    ) -> HelixResult<Finding> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE nova.findings SET updated_at = ");
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
        builder.push_bind(experiment_id);
        builder.push(" AND id = ");
        builder.push_bind(finding_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {FINDING_RETURNING}"));

        let row: Option<FindingRow> = builder
            .build_query_as::<FindingRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("nova update finding: {e}")))?;

        row.map(FindingRow::into_finding)
            .ok_or_else(|| HelixError::not_found("finding not found"))
    }

    pub async fn confirm_finding(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
        finding_id: Uuid,
    ) -> HelixResult<Finding> {
        let finding = self
            .get_finding(tenant_id, experiment_id, finding_id)
            .await?
            .ok_or_else(|| HelixError::not_found("finding not found"))?;
        let next = next_finding_status(&finding.status, "confirm")?;
        let now = Utc::now();
        let row: Option<FindingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.findings
            SET status = $1, confirmed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NULL
            {FINDING_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .bind(finding_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova confirm finding: {e}")))?;

        row.map(FindingRow::into_finding)
            .ok_or_else(|| HelixError::not_found("finding not found"))
    }

    pub async fn reject_finding(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
        finding_id: Uuid,
    ) -> HelixResult<Finding> {
        let finding = self
            .get_finding(tenant_id, experiment_id, finding_id)
            .await?
            .ok_or_else(|| HelixError::not_found("finding not found"))?;
        let next = next_finding_status(&finding.status, "reject")?;
        let now = Utc::now();
        let row: Option<FindingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.findings
            SET status = $1, rejected_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NULL
            {FINDING_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .bind(finding_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova reject finding: {e}")))?;

        row.map(FindingRow::into_finding)
            .ok_or_else(|| HelixError::not_found("finding not found"))
    }

    pub async fn soft_delete_finding(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
        finding_id: Uuid,
    ) -> HelixResult<Finding> {
        let finding = self
            .get_finding(tenant_id, experiment_id, finding_id)
            .await?
            .ok_or_else(|| HelixError::not_found("finding not found"))?;
        if finding.status == "deleted" {
            return Err(HelixError::validation("finding is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<FindingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.findings
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {FINDING_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .bind(finding_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova soft-delete finding: {e}")))?;

        row.map(FindingRow::into_finding)
            .ok_or_else(|| HelixError::not_found("finding not found"))
    }

    /// Restore a soft-deleted finding, returning it to its pre-delete status.
    pub async fn restore_finding(
        &self,
        tenant_id: TenantId,
        experiment_id: Uuid,
        finding_id: Uuid,
    ) -> HelixResult<Finding> {
        let finding = self
            .fetch_finding_any(tenant_id, experiment_id, finding_id)
            .await?
            .ok_or_else(|| HelixError::not_found("finding not found"))?;
        if finding.deleted_at.is_none() {
            return Err(HelixError::validation("finding is not deleted"));
        }
        let restored = if finding.rejected_at.is_some() {
            "rejected"
        } else if finding.confirmed_at.is_some() {
            "confirmed"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<FindingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE nova.findings
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {FINDING_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(experiment_id)
        .bind(finding_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova restore finding: {e}")))?;

        row.map(FindingRow::into_finding)
            .ok_or_else(|| HelixError::not_found("finding not found or not deleted"))
    }

    // --- Reports ---

    /// Per-experiment finding counts by status for non-deleted experiments.
    pub async fn get_nova_summary(&self, tenant_id: TenantId) -> HelixResult<Vec<NovaSummaryRow>> {
        let rows: Vec<NovaSummaryRow> = sqlx::query_as(
            r#"
            SELECT e.id, e.name, e.status,
                   COUNT(f.id) AS total_findings,
                   COUNT(f.id) FILTER (WHERE f.status = 'draft') AS draft_findings,
                   COUNT(f.id) FILTER (WHERE f.status = 'confirmed') AS confirmed_findings,
                   COUNT(f.id) FILTER (WHERE f.status = 'rejected') AS rejected_findings
            FROM nova.experiments e
            LEFT JOIN nova.findings f
                   ON f.parent_id = e.id AND f.tenant_id = e.tenant_id
                  AND f.deleted_at IS NULL
            WHERE e.tenant_id = $1 AND e.deleted_at IS NULL
            GROUP BY e.id, e.name, e.status, e.created_at
            ORDER BY e.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("nova summary: {e}")))?;
        Ok(rows)
    }
}
