//! HelixSynthBio durable store — `synthbio` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Design {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimRun {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SynthbioSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_sims: i64,
    pub running_sims: i64,
    pub completed_sims: i64,
    pub failed_sims: i64,
}

#[derive(Debug, Clone, Default)]
pub struct DesignUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct SimUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a design lifecycle transition and return the resulting status.
pub fn next_design_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "submit") => Ok("review"),
        ("review", "approve") => Ok("approved"),
        ("review", "return") => Ok("draft"),
        (_, "submit") => Err(HelixError::validation(format!(
            "cannot submit a {current} design"
        ))),
        (_, "approve") => Err(HelixError::validation(format!(
            "cannot approve a {current} design"
        ))),
        (_, "return") => Err(HelixError::validation(format!(
            "cannot return a {current} design"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown design action {action}"
        ))),
    }
}

/// Validate a sim lifecycle transition and return the resulting status.
pub fn next_sim_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("open", "start") => Ok("running"),
        ("running", "complete") => Ok("completed"),
        ("running", "fail") => Ok("failed"),
        (_, "start") => Err(HelixError::validation(format!(
            "cannot start a {current} sim"
        ))),
        (_, "complete") => Err(HelixError::validation(format!(
            "cannot complete a {current} sim"
        ))),
        (_, "fail") => Err(HelixError::validation(format!(
            "cannot fail a {current} sim"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown sim action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct DesignRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    submitted_at: Option<DateTime<Utc>>,
    approved_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl DesignRow {
    fn into_design(self) -> Design {
        Design {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            submitted_at: self.submitted_at,
            approved_at: self.approved_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SimRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    failed_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl SimRow {
    fn into_sim(self) -> SimRun {
        SimRun {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            failed_at: self.failed_at,
            deleted_at: self.deleted_at,
        }
    }
}

const DESIGN_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           submitted_at, approved_at, deleted_at
    FROM synthbio.designs
"#;

const DESIGN_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              submitted_at, approved_at, deleted_at
"#;

const SIM_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, started_at, completed_at, failed_at, deleted_at
    FROM synthbio.sims
"#;

const SIM_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, started_at, completed_at, failed_at, deleted_at
"#;

#[derive(Clone)]
pub struct SynthbioRepo {
    pool: PgPool,
}

impl SynthbioRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Designs ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<Design>> {
        let rows: Vec<DesignRow> = sqlx::query_as(&format!(
            "{DESIGN_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list: {e}")))?;
        Ok(rows.into_iter().map(DesignRow::into_design).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Design> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: DesignRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO synthbio.designs
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {DESIGN_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("synthbio create: {e}")))?;
        Ok(row.into_design())
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Design>> {
        let row: Option<DesignRow> = sqlx::query_as(&format!(
            "{DESIGN_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio get: {e}")))?;
        Ok(row.map(DesignRow::into_design))
    }

    async fn fetch_design_any(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Design>> {
        let row: Option<DesignRow> =
            sqlx::query_as(&format!("{DESIGN_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("synthbio fetch design: {e}")))?;
        Ok(row.map(DesignRow::into_design))
    }

    pub async fn update_design(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        update: DesignUpdate,
    ) -> HelixResult<Design> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE synthbio.designs SET updated_at = ");
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
        builder.push_bind(design_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {DESIGN_RETURNING}"));

        let row: Option<DesignRow> = builder
            .build_query_as::<DesignRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio update design: {e}")))?;

        row.map(DesignRow::into_design)
            .ok_or_else(|| HelixError::not_found("design not found"))
    }

    pub async fn submit_design(&self, tenant_id: TenantId, design_id: Uuid) -> HelixResult<Design> {
        let design = self
            .get_parent(tenant_id, design_id)
            .await?
            .ok_or_else(|| HelixError::not_found("design not found"))?;
        let next = next_design_status(&design.status, "submit")?;
        let now = Utc::now();
        let row: Option<DesignRow> = sqlx::query_as(&format!(
            r#"
            UPDATE synthbio.designs
            SET status = $1, submitted_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {DESIGN_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio submit design: {e}")))?;

        row.map(DesignRow::into_design)
            .ok_or_else(|| HelixError::not_found("design not found"))
    }

    /// Approve a design under review. Requires at least one completed sim.
    pub async fn approve_design(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Design> {
        let design = self
            .get_parent(tenant_id, design_id)
            .await?
            .ok_or_else(|| HelixError::not_found("design not found"))?;
        let next = next_design_status(&design.status, "approve")?;

        let completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM synthbio.sims WHERE tenant_id = $1 AND parent_id = $2 AND status = 'completed' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio approve sim count: {e}")))?;
        if completed == 0 {
            return Err(HelixError::validation(
                "design needs at least one completed sim to approve",
            ));
        }

        let now = Utc::now();
        let row: Option<DesignRow> = sqlx::query_as(&format!(
            r#"
            UPDATE synthbio.designs
            SET status = $1, approved_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {DESIGN_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio approve design: {e}")))?;

        row.map(DesignRow::into_design)
            .ok_or_else(|| HelixError::not_found("design not found"))
    }

    /// Return a design under review back to draft.
    pub async fn return_design(&self, tenant_id: TenantId, design_id: Uuid) -> HelixResult<Design> {
        let design = self
            .get_parent(tenant_id, design_id)
            .await?
            .ok_or_else(|| HelixError::not_found("design not found"))?;
        let next = next_design_status(&design.status, "return")?;
        let now = Utc::now();
        let row: Option<DesignRow> = sqlx::query_as(&format!(
            r#"
            UPDATE synthbio.designs
            SET status = $1, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {DESIGN_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio return design: {e}")))?;

        row.map(DesignRow::into_design)
            .ok_or_else(|| HelixError::not_found("design not found"))
    }

    pub async fn soft_delete_design(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Design> {
        let design = self
            .get_parent(tenant_id, design_id)
            .await?
            .ok_or_else(|| HelixError::not_found("design not found"))?;
        if design.status == "deleted" {
            return Err(HelixError::validation("design is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<DesignRow> = sqlx::query_as(&format!(
            r#"
            UPDATE synthbio.designs
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {DESIGN_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio soft-delete design: {e}")))?;

        row.map(DesignRow::into_design)
            .ok_or_else(|| HelixError::not_found("design not found"))
    }

    /// Restore a soft-deleted design, returning it to its pre-delete status.
    pub async fn restore_design(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
    ) -> HelixResult<Design> {
        let design = self
            .fetch_design_any(tenant_id, design_id)
            .await?
            .ok_or_else(|| HelixError::not_found("design not found"))?;
        if design.deleted_at.is_none() {
            return Err(HelixError::validation("design is not deleted"));
        }
        let restored = if design.approved_at.is_some() {
            "approved"
        } else if design.submitted_at.is_some() {
            "review"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<DesignRow> = sqlx::query_as(&format!(
            r#"
            UPDATE synthbio.designs
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {DESIGN_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio restore design: {e}")))?;

        row.map(DesignRow::into_design)
            .ok_or_else(|| HelixError::not_found("design not found or not deleted"))
    }

    // --- Sims ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<SimRun>> {
        let rows: Vec<SimRow> = sqlx::query_as(&format!(
            "{SIM_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio list children: {e}")))?;
        Ok(rows.into_iter().map(SimRow::into_sim).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<SimRun> {
        let _parent = self
            .get_parent(tenant_id, parent_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: SimRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO synthbio.sims
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,'open',$6,$7,$7)
            {SIM_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("synthbio create child: {e}")))?;
        Ok(row.into_sim())
    }

    pub async fn get_sim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
    ) -> HelixResult<Option<SimRun>> {
        let row: Option<SimRow> = sqlx::query_as(&format!(
            "{SIM_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(sim_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio get sim: {e}")))?;
        Ok(row.map(SimRow::into_sim))
    }

    async fn fetch_sim_any(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
    ) -> HelixResult<Option<SimRun>> {
        let row: Option<SimRow> = sqlx::query_as(&format!(
            "{SIM_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(sim_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio fetch sim: {e}")))?;
        Ok(row.map(SimRow::into_sim))
    }

    pub async fn update_sim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
        update: SimUpdate,
    ) -> HelixResult<SimRun> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE synthbio.sims SET updated_at = ");
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
        builder.push_bind(design_id);
        builder.push(" AND id = ");
        builder.push_bind(sim_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {SIM_RETURNING}"));

        let row: Option<SimRow> = builder
            .build_query_as::<SimRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("synthbio update sim: {e}")))?;

        row.map(SimRow::into_sim)
            .ok_or_else(|| HelixError::not_found("sim not found"))
    }

    async fn transition_sim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
        action: &str,
    ) -> HelixResult<SimRun> {
        let sim = self
            .get_sim(tenant_id, design_id, sim_id)
            .await?
            .ok_or_else(|| HelixError::not_found("sim not found"))?;
        let next = next_sim_status(&sim.status, action)?;
        let now = Utc::now();
        let (started_at, completed_at, failed_at) = match next {
            "running" => (Some(now), None, None),
            "completed" => (sim.started_at, Some(now), None),
            "failed" => (sim.started_at, None, Some(now)),
            _ => (sim.started_at, sim.completed_at, sim.failed_at),
        };
        let row: Option<SimRow> = sqlx::query_as(&format!(
            r#"
            UPDATE synthbio.sims
            SET status = $1, started_at = $2, completed_at = $3, failed_at = $4, updated_at = $5
            WHERE tenant_id = $6 AND parent_id = $7 AND id = $8 AND deleted_at IS NULL
            {SIM_RETURNING}
            "#
        ))
        .bind(next)
        .bind(started_at)
        .bind(completed_at)
        .bind(failed_at)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(sim_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio {action} sim: {e}")))?;

        row.map(SimRow::into_sim)
            .ok_or_else(|| HelixError::not_found("sim not found"))
    }

    pub async fn start_sim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
    ) -> HelixResult<SimRun> {
        self.transition_sim(tenant_id, design_id, sim_id, "start")
            .await
    }

    pub async fn complete_sim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
    ) -> HelixResult<SimRun> {
        self.transition_sim(tenant_id, design_id, sim_id, "complete")
            .await
    }

    pub async fn fail_sim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
    ) -> HelixResult<SimRun> {
        self.transition_sim(tenant_id, design_id, sim_id, "fail")
            .await
    }

    pub async fn soft_delete_sim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
    ) -> HelixResult<SimRun> {
        let sim = self
            .get_sim(tenant_id, design_id, sim_id)
            .await?
            .ok_or_else(|| HelixError::not_found("sim not found"))?;
        if sim.status == "deleted" {
            return Err(HelixError::validation("sim is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<SimRow> = sqlx::query_as(&format!(
            r#"
            UPDATE synthbio.sims
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {SIM_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(sim_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio soft-delete sim: {e}")))?;

        row.map(SimRow::into_sim)
            .ok_or_else(|| HelixError::not_found("sim not found"))
    }

    /// Restore a soft-deleted sim, returning it to its pre-delete status.
    pub async fn restore_sim(
        &self,
        tenant_id: TenantId,
        design_id: Uuid,
        sim_id: Uuid,
    ) -> HelixResult<SimRun> {
        let sim = self
            .fetch_sim_any(tenant_id, design_id, sim_id)
            .await?
            .ok_or_else(|| HelixError::not_found("sim not found"))?;
        if sim.deleted_at.is_none() {
            return Err(HelixError::validation("sim is not deleted"));
        }
        let restored = if sim.failed_at.is_some() {
            "failed"
        } else if sim.completed_at.is_some() {
            "completed"
        } else if sim.started_at.is_some() {
            "running"
        } else {
            "open"
        };
        let now = Utc::now();
        let row: Option<SimRow> = sqlx::query_as(&format!(
            r#"
            UPDATE synthbio.sims
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {SIM_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(design_id)
        .bind(sim_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio restore sim: {e}")))?;

        row.map(SimRow::into_sim)
            .ok_or_else(|| HelixError::not_found("sim not found or not deleted"))
    }

    // --- Reports ---

    /// Per-design sim counts by status for non-deleted designs.
    pub async fn get_synthbio_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<SynthbioSummaryRow>> {
        let rows: Vec<SynthbioSummaryRow> = sqlx::query_as(
            r#"
            SELECT d.id, d.name, d.status,
                   COUNT(s.id) AS total_sims,
                   COUNT(s.id) FILTER (WHERE s.status = 'running') AS running_sims,
                   COUNT(s.id) FILTER (WHERE s.status = 'completed') AS completed_sims,
                   COUNT(s.id) FILTER (WHERE s.status = 'failed') AS failed_sims
            FROM synthbio.designs d
            LEFT JOIN synthbio.sims s
                   ON s.parent_id = d.id AND s.tenant_id = d.tenant_id
                  AND s.deleted_at IS NULL
            WHERE d.tenant_id = $1 AND d.deleted_at IS NULL
            GROUP BY d.id, d.name, d.status, d.created_at
            ORDER BY d.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("synthbio summary: {e}")))?;
        Ok(rows)
    }
}
