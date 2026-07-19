//! HelixTerra Prime durable store — `terra` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub retired_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
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
    pub dismissed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TerraSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_observations: i64,
    pub draft_observations: i64,
    pub confirmed_observations: i64,
    pub dismissed_observations: i64,
}

#[derive(Debug, Clone, Default)]
pub struct FieldUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct ObservationUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a field lifecycle transition and return the resulting status.
pub fn next_field_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "activate") => Ok("active"),
        ("active", "retire") => Ok("retired"),
        ("retired", "reopen") => Ok("active"),
        (_, "activate") => Err(HelixError::validation(format!(
            "cannot activate a {current} field"
        ))),
        (_, "retire") => Err(HelixError::validation(format!(
            "cannot retire a {current} field"
        ))),
        (_, "reopen") => Err(HelixError::validation(format!(
            "cannot reopen a {current} field"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown field action {action}"
        ))),
    }
}

/// Validate an observation lifecycle transition and return the resulting status.
pub fn next_observation_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "confirm") => Ok("confirmed"),
        ("draft", "dismiss") | ("confirmed", "dismiss") => Ok("dismissed"),
        (_, "confirm") => Err(HelixError::validation(format!(
            "cannot confirm a {current} observation"
        ))),
        (_, "dismiss") => Err(HelixError::validation(format!(
            "cannot dismiss a {current} observation"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown observation action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct FieldRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    activated_at: Option<DateTime<Utc>>,
    retired_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl FieldRow {
    fn into_field(self) -> Field {
        Field {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            activated_at: self.activated_at,
            retired_at: self.retired_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ObservationRow {
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
    dismissed_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ObservationRow {
    fn into_observation(self) -> Observation {
        Observation {
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
            dismissed_at: self.dismissed_at,
            deleted_at: self.deleted_at,
        }
    }
}

const FIELD_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           activated_at, retired_at, deleted_at
    FROM terra.fields
"#;

const FIELD_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              activated_at, retired_at, deleted_at
"#;

const OBSERVATION_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, confirmed_at, dismissed_at, deleted_at
    FROM terra.observations
"#;

const OBSERVATION_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, confirmed_at, dismissed_at, deleted_at
"#;

#[derive(Clone)]
pub struct TerraRepo {
    pool: PgPool,
}

impl TerraRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Fields ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<Field>> {
        let rows: Vec<FieldRow> = sqlx::query_as(&format!(
            "{FIELD_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra list: {e}")))?;
        Ok(rows.into_iter().map(FieldRow::into_field).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Field> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: FieldRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO terra.fields
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {FIELD_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("terra create: {e}")))?;
        Ok(row.into_field())
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Field>> {
        let row: Option<FieldRow> = sqlx::query_as(&format!(
            "{FIELD_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra get: {e}")))?;
        Ok(row.map(FieldRow::into_field))
    }

    async fn fetch_field_any(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Field>> {
        let row: Option<FieldRow> =
            sqlx::query_as(&format!("{FIELD_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("terra fetch field: {e}")))?;
        Ok(row.map(FieldRow::into_field))
    }

    pub async fn update_field(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
        update: FieldUpdate,
    ) -> HelixResult<Field> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE terra.fields SET updated_at = ");
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
        builder.push_bind(field_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {FIELD_RETURNING}"));

        let row: Option<FieldRow> = builder
            .build_query_as::<FieldRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("terra update field: {e}")))?;

        row.map(FieldRow::into_field)
            .ok_or_else(|| HelixError::not_found("field not found"))
    }

    pub async fn activate_field(&self, tenant_id: TenantId, field_id: Uuid) -> HelixResult<Field> {
        let field = self
            .get_parent(tenant_id, field_id)
            .await?
            .ok_or_else(|| HelixError::not_found("field not found"))?;
        let next = next_field_status(&field.status, "activate")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<FieldRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.fields
            SET status = $1, activated_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {FIELD_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .bind(&field.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra activate field: {e}")))?;

        row.map(FieldRow::into_field)
            .ok_or_else(|| HelixError::conflict("field changed during activate; retry"))
    }

    /// Retire an active field. Rejected while draft observations remain.
    /// The active-status and no-draft-observations guards are part of the
    /// UPDATE itself, so a concurrent retire or an observation created
    /// mid-flight cannot slip through a check-then-act window; the earlier
    /// reads only shape the error returned for the steady-state cases.
    pub async fn retire_field(&self, tenant_id: TenantId, field_id: Uuid) -> HelixResult<Field> {
        let field = self
            .get_parent(tenant_id, field_id)
            .await?
            .ok_or_else(|| HelixError::not_found("field not found"))?;
        let next = next_field_status(&field.status, "retire")?;

        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM terra.observations WHERE tenant_id = $1 AND parent_id = $2 AND status = 'draft' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra retire observation count: {e}")))?;
        if drafts > 0 {
            return Err(HelixError::validation(format!(
                "field has {drafts} draft observation(s); confirm or dismiss them first"
            )));
        }

        let now = Utc::now();
        let row: Option<FieldRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.fields
            SET status = $1, retired_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = 'active' AND deleted_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM terra.observations o
                  WHERE o.tenant_id = $3 AND o.parent_id = $4
                    AND o.status = 'draft' AND o.deleted_at IS NULL
              )
            {FIELD_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra retire field: {e}")))?;

        row.map(FieldRow::into_field).ok_or_else(|| {
            HelixError::conflict("field changed during retire or gained a draft observation; retry")
        })
    }

    pub async fn reopen_field(&self, tenant_id: TenantId, field_id: Uuid) -> HelixResult<Field> {
        let field = self
            .get_parent(tenant_id, field_id)
            .await?
            .ok_or_else(|| HelixError::not_found("field not found"))?;
        let next = next_field_status(&field.status, "reopen")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<FieldRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.fields
            SET status = $1, retired_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {FIELD_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .bind(&field.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra reopen field: {e}")))?;

        row.map(FieldRow::into_field)
            .ok_or_else(|| HelixError::conflict("field changed during reopen; retry"))
    }

    pub async fn soft_delete_field(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
    ) -> HelixResult<Field> {
        let field = self
            .get_parent(tenant_id, field_id)
            .await?
            .ok_or_else(|| HelixError::not_found("field not found"))?;
        if field.status == "deleted" {
            return Err(HelixError::validation("field is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<FieldRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.fields
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {FIELD_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra soft-delete field: {e}")))?;

        row.map(FieldRow::into_field)
            .ok_or_else(|| HelixError::not_found("field not found"))
    }

    /// Restore a soft-deleted field, returning it to its pre-delete status.
    pub async fn restore_field(&self, tenant_id: TenantId, field_id: Uuid) -> HelixResult<Field> {
        let field = self
            .fetch_field_any(tenant_id, field_id)
            .await?
            .ok_or_else(|| HelixError::not_found("field not found"))?;
        if field.deleted_at.is_none() {
            return Err(HelixError::validation("field is not deleted"));
        }
        let restored = if field.retired_at.is_some() {
            "retired"
        } else if field.activated_at.is_some() {
            "active"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<FieldRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.fields
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {FIELD_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra restore field: {e}")))?;

        row.map(FieldRow::into_field)
            .ok_or_else(|| HelixError::not_found("field not found or not deleted"))
    }

    // --- Observations ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Observation>> {
        let rows: Vec<ObservationRow> = sqlx::query_as(&format!(
            "{OBSERVATION_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra list children: {e}")))?;
        Ok(rows
            .into_iter()
            .map(ObservationRow::into_observation)
            .collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Observation> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        // The non-deleted-parent guard is part of the INSERT itself: a field
        // soft-deleted between a separate check and insert cannot leak observations.
        let row: Option<ObservationRow> = sqlx::query_as(&format!(
            r#"
            INSERT INTO terra.observations
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            SELECT $1,$2,$3,$4,$5,'draft',$6,$7,$7
            FROM terra.fields f
            WHERE f.tenant_id = $2 AND f.id = $3 AND f.deleted_at IS NULL
            {OBSERVATION_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .bind(title)
        .bind(body)
        .bind(&metadata)
        .bind(created_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra create child: {e}")))?;
        row.map(ObservationRow::into_observation)
            .ok_or_else(|| HelixError::not_found("parent not found"))
    }

    pub async fn get_observation(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
        observation_id: Uuid,
    ) -> HelixResult<Option<Observation>> {
        let row: Option<ObservationRow> = sqlx::query_as(&format!(
            "{OBSERVATION_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .bind(observation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra get observation: {e}")))?;
        Ok(row.map(ObservationRow::into_observation))
    }

    async fn fetch_observation_any(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
        observation_id: Uuid,
    ) -> HelixResult<Option<Observation>> {
        let row: Option<ObservationRow> = sqlx::query_as(&format!(
            "{OBSERVATION_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .bind(observation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra fetch observation: {e}")))?;
        Ok(row.map(ObservationRow::into_observation))
    }

    pub async fn update_observation(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
        observation_id: Uuid,
        update: ObservationUpdate,
    ) -> HelixResult<Observation> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE terra.observations SET updated_at = ");
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
        builder.push_bind(field_id);
        builder.push(" AND id = ");
        builder.push_bind(observation_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {OBSERVATION_RETURNING}"));

        let row: Option<ObservationRow> = builder
            .build_query_as::<ObservationRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("terra update observation: {e}")))?;

        row.map(ObservationRow::into_observation)
            .ok_or_else(|| HelixError::not_found("observation not found"))
    }

    pub async fn confirm_observation(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
        observation_id: Uuid,
    ) -> HelixResult<Observation> {
        let obs = self
            .get_observation(tenant_id, field_id, observation_id)
            .await?
            .ok_or_else(|| HelixError::not_found("observation not found"))?;
        let next = next_observation_status(&obs.status, "confirm")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<ObservationRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.observations
            SET status = $1, confirmed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND status = $6 AND deleted_at IS NULL
            {OBSERVATION_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .bind(observation_id)
        .bind(&obs.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra confirm observation: {e}")))?;

        row.map(ObservationRow::into_observation)
            .ok_or_else(|| HelixError::conflict("observation changed during confirm; retry"))
    }

    pub async fn dismiss_observation(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
        observation_id: Uuid,
    ) -> HelixResult<Observation> {
        let obs = self
            .get_observation(tenant_id, field_id, observation_id)
            .await?
            .ok_or_else(|| HelixError::not_found("observation not found"))?;
        let next = next_observation_status(&obs.status, "dismiss")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<ObservationRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.observations
            SET status = $1, dismissed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND status = $6 AND deleted_at IS NULL
            {OBSERVATION_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .bind(observation_id)
        .bind(&obs.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra dismiss observation: {e}")))?;

        row.map(ObservationRow::into_observation)
            .ok_or_else(|| HelixError::conflict("observation changed during dismiss; retry"))
    }

    pub async fn soft_delete_observation(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
        observation_id: Uuid,
    ) -> HelixResult<Observation> {
        let obs = self
            .get_observation(tenant_id, field_id, observation_id)
            .await?
            .ok_or_else(|| HelixError::not_found("observation not found"))?;
        if obs.status == "deleted" {
            return Err(HelixError::validation("observation is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<ObservationRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.observations
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {OBSERVATION_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .bind(observation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra soft-delete observation: {e}")))?;

        row.map(ObservationRow::into_observation)
            .ok_or_else(|| HelixError::not_found("observation not found"))
    }

    /// Restore a soft-deleted observation, returning it to its pre-delete status.
    pub async fn restore_observation(
        &self,
        tenant_id: TenantId,
        field_id: Uuid,
        observation_id: Uuid,
    ) -> HelixResult<Observation> {
        let obs = self
            .fetch_observation_any(tenant_id, field_id, observation_id)
            .await?
            .ok_or_else(|| HelixError::not_found("observation not found"))?;
        if obs.deleted_at.is_none() {
            return Err(HelixError::validation("observation is not deleted"));
        }
        let restored = if obs.dismissed_at.is_some() {
            "dismissed"
        } else if obs.confirmed_at.is_some() {
            "confirmed"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<ObservationRow> = sqlx::query_as(&format!(
            r#"
            UPDATE terra.observations
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {OBSERVATION_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(field_id)
        .bind(observation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra restore observation: {e}")))?;

        row.map(ObservationRow::into_observation)
            .ok_or_else(|| HelixError::not_found("observation not found or not deleted"))
    }

    // --- Reports ---

    /// Per-field observation counts by status for non-deleted fields.
    pub async fn get_terra_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<TerraSummaryRow>> {
        let rows: Vec<TerraSummaryRow> = sqlx::query_as(
            r#"
            SELECT f.id, f.name, f.status,
                   COUNT(o.id) AS total_observations,
                   COUNT(o.id) FILTER (WHERE o.status = 'draft') AS draft_observations,
                   COUNT(o.id) FILTER (WHERE o.status = 'confirmed') AS confirmed_observations,
                   COUNT(o.id) FILTER (WHERE o.status = 'dismissed') AS dismissed_observations
            FROM terra.fields f
            LEFT JOIN terra.observations o
                   ON o.parent_id = f.id AND o.tenant_id = f.tenant_id
                  AND o.deleted_at IS NULL
            WHERE f.tenant_id = $1 AND f.deleted_at IS NULL
            GROUP BY f.id, f.name, f.status, f.created_at
            ORDER BY f.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("terra summary: {e}")))?;
        Ok(rows)
    }
}
