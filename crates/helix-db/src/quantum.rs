//! HelixQuantum Forge durable store — `quantum` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumJob {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub validated_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuantumSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_circuits: i64,
    pub draft_circuits: i64,
    pub validated_circuits: i64,
    pub archived_circuits: i64,
}

#[derive(Debug, Clone, Default)]
pub struct JobUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct CircuitUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a job lifecycle transition and return the resulting status.
pub fn next_job_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "submit") => Ok("submitted"),
        ("submitted", "complete") => Ok("completed"),
        ("submitted", "fail") => Ok("failed"),
        (_, "submit") => Err(HelixError::validation(format!(
            "cannot submit a {current} job"
        ))),
        (_, "complete") => Err(HelixError::validation(format!(
            "cannot complete a {current} job"
        ))),
        (_, "fail") => Err(HelixError::validation(format!(
            "cannot fail a {current} job"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown job action {action}"
        ))),
    }
}

/// Validate a circuit lifecycle transition and return the resulting status.
pub fn next_circuit_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "validate") => Ok("validated"),
        ("draft", "archive") | ("validated", "archive") => Ok("archived"),
        (_, "validate") => Err(HelixError::validation(format!(
            "cannot validate a {current} circuit"
        ))),
        (_, "archive") => Err(HelixError::validation(format!(
            "cannot archive a {current} circuit"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown circuit action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    submitted_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    failed_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl JobRow {
    fn into_job(self) -> QuantumJob {
        QuantumJob {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            submitted_at: self.submitted_at,
            completed_at: self.completed_at,
            failed_at: self.failed_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CircuitRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    validated_at: Option<DateTime<Utc>>,
    archived_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl CircuitRow {
    fn into_circuit(self) -> Circuit {
        Circuit {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            validated_at: self.validated_at,
            archived_at: self.archived_at,
            deleted_at: self.deleted_at,
        }
    }
}

const JOB_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           submitted_at, completed_at, failed_at, deleted_at
    FROM quantum.jobs
"#;

const JOB_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              submitted_at, completed_at, failed_at, deleted_at
"#;

const CIRCUIT_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, validated_at, archived_at, deleted_at
    FROM quantum.circuits
"#;

const CIRCUIT_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, validated_at, archived_at, deleted_at
"#;

#[derive(Clone)]
pub struct QuantumRepo {
    pool: PgPool,
}

impl QuantumRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Jobs ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<QuantumJob>> {
        let rows: Vec<JobRow> = sqlx::query_as(&format!(
            "{JOB_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum list: {e}")))?;
        Ok(rows.into_iter().map(JobRow::into_job).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<QuantumJob> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: JobRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO quantum.jobs
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {JOB_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("quantum create: {e}")))?;
        Ok(row.into_job())
    }

    pub async fn get_parent(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<QuantumJob>> {
        let row: Option<JobRow> = sqlx::query_as(&format!(
            "{JOB_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum get: {e}")))?;
        Ok(row.map(JobRow::into_job))
    }

    async fn fetch_job_any(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<QuantumJob>> {
        let row: Option<JobRow> =
            sqlx::query_as(&format!("{JOB_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("quantum fetch job: {e}")))?;
        Ok(row.map(JobRow::into_job))
    }

    pub async fn update_job(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        update: JobUpdate,
    ) -> HelixResult<QuantumJob> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE quantum.jobs SET updated_at = ");
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
        builder.push_bind(job_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {JOB_RETURNING}"));

        let row: Option<JobRow> = builder
            .build_query_as::<JobRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("quantum update job: {e}")))?;

        row.map(JobRow::into_job)
            .ok_or_else(|| HelixError::not_found("job not found"))
    }

    /// Submit a draft job. Requires at least one non-deleted circuit. The
    /// draft-status and circuit-exists guards are part of the UPDATE
    /// itself, so a concurrent submit or a circuit deleted mid-flight
    /// cannot slip through a check-then-act window; the earlier reads only
    /// shape the error returned for the steady-state cases.
    pub async fn submit_job(&self, tenant_id: TenantId, job_id: Uuid) -> HelixResult<QuantumJob> {
        let job = self
            .get_parent(tenant_id, job_id)
            .await?
            .ok_or_else(|| HelixError::not_found("job not found"))?;
        let next = next_job_status(&job.status, "submit")?;

        let circuits: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM quantum.circuits WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum submit circuit count: {e}")))?;
        if circuits == 0 {
            return Err(HelixError::validation(
                "job needs at least one circuit to submit",
            ));
        }

        let now = Utc::now();
        let row: Option<JobRow> = sqlx::query_as(&format!(
            r#"
            UPDATE quantum.jobs
            SET status = $1, submitted_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = 'draft' AND deleted_at IS NULL
              AND EXISTS (
                  SELECT 1 FROM quantum.circuits c
                  WHERE c.tenant_id = $3 AND c.parent_id = $4 AND c.deleted_at IS NULL
              )
            {JOB_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum submit job: {e}")))?;

        row.map(JobRow::into_job).ok_or_else(|| {
            HelixError::conflict("job changed during submit or lost its last circuit; retry")
        })
    }

    async fn transition_job(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        action: &str,
    ) -> HelixResult<QuantumJob> {
        let job = self
            .get_parent(tenant_id, job_id)
            .await?
            .ok_or_else(|| HelixError::not_found("job not found"))?;
        let next = next_job_status(&job.status, action)?;
        let now = Utc::now();
        let (completed_at, failed_at) = match next {
            "completed" => (Some(now), None),
            "failed" => (None, Some(now)),
            _ => (job.completed_at, job.failed_at),
        };
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<JobRow> = sqlx::query_as(&format!(
            r#"
            UPDATE quantum.jobs
            SET status = $1, completed_at = $2, failed_at = $3, updated_at = $4
            WHERE tenant_id = $5 AND id = $6 AND status = $7 AND deleted_at IS NULL
            {JOB_RETURNING}
            "#
        ))
        .bind(next)
        .bind(completed_at)
        .bind(failed_at)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(&job.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum {action} job: {e}")))?;

        row.map(JobRow::into_job)
            .ok_or_else(|| HelixError::conflict("job changed during transition; retry"))
    }

    pub async fn complete_job(&self, tenant_id: TenantId, job_id: Uuid) -> HelixResult<QuantumJob> {
        self.transition_job(tenant_id, job_id, "complete").await
    }

    pub async fn fail_job(&self, tenant_id: TenantId, job_id: Uuid) -> HelixResult<QuantumJob> {
        self.transition_job(tenant_id, job_id, "fail").await
    }

    pub async fn soft_delete_job(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
    ) -> HelixResult<QuantumJob> {
        let job = self
            .get_parent(tenant_id, job_id)
            .await?
            .ok_or_else(|| HelixError::not_found("job not found"))?;
        if job.status == "deleted" {
            return Err(HelixError::validation("job is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<JobRow> = sqlx::query_as(&format!(
            r#"
            UPDATE quantum.jobs
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {JOB_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum soft-delete job: {e}")))?;

        row.map(JobRow::into_job)
            .ok_or_else(|| HelixError::not_found("job not found"))
    }

    /// Restore a soft-deleted job, returning it to its pre-delete status.
    pub async fn restore_job(&self, tenant_id: TenantId, job_id: Uuid) -> HelixResult<QuantumJob> {
        let job = self
            .fetch_job_any(tenant_id, job_id)
            .await?
            .ok_or_else(|| HelixError::not_found("job not found"))?;
        if job.deleted_at.is_none() {
            return Err(HelixError::validation("job is not deleted"));
        }
        let restored = if job.failed_at.is_some() {
            "failed"
        } else if job.completed_at.is_some() {
            "completed"
        } else if job.submitted_at.is_some() {
            "submitted"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<JobRow> = sqlx::query_as(&format!(
            r#"
            UPDATE quantum.jobs
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {JOB_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum restore job: {e}")))?;

        row.map(JobRow::into_job)
            .ok_or_else(|| HelixError::not_found("job not found or not deleted"))
    }

    // --- Circuits ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Circuit>> {
        let rows: Vec<CircuitRow> = sqlx::query_as(&format!(
            "{CIRCUIT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum list children: {e}")))?;
        Ok(rows.into_iter().map(CircuitRow::into_circuit).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Circuit> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        // The non-deleted-parent guard is part of the INSERT itself: a job
        // soft-deleted between a separate check and insert cannot leak circuits.
        let row: Option<CircuitRow> = sqlx::query_as(&format!(
            r#"
            INSERT INTO quantum.circuits
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            SELECT $1,$2,$3,$4,$5,'draft',$6,$7,$7
            FROM quantum.jobs j
            WHERE j.tenant_id = $2 AND j.id = $3 AND j.deleted_at IS NULL
            {CIRCUIT_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("quantum create child: {e}")))?;
        row.map(CircuitRow::into_circuit)
            .ok_or_else(|| HelixError::not_found("parent not found"))
    }

    pub async fn get_circuit(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        circuit_id: Uuid,
    ) -> HelixResult<Option<Circuit>> {
        let row: Option<CircuitRow> = sqlx::query_as(&format!(
            "{CIRCUIT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(circuit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum get circuit: {e}")))?;
        Ok(row.map(CircuitRow::into_circuit))
    }

    async fn fetch_circuit_any(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        circuit_id: Uuid,
    ) -> HelixResult<Option<Circuit>> {
        let row: Option<CircuitRow> = sqlx::query_as(&format!(
            "{CIRCUIT_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(circuit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum fetch circuit: {e}")))?;
        Ok(row.map(CircuitRow::into_circuit))
    }

    pub async fn update_circuit(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        circuit_id: Uuid,
        update: CircuitUpdate,
    ) -> HelixResult<Circuit> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE quantum.circuits SET updated_at = ");
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
        builder.push_bind(job_id);
        builder.push(" AND id = ");
        builder.push_bind(circuit_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {CIRCUIT_RETURNING}"));

        let row: Option<CircuitRow> = builder
            .build_query_as::<CircuitRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("quantum update circuit: {e}")))?;

        row.map(CircuitRow::into_circuit)
            .ok_or_else(|| HelixError::not_found("circuit not found"))
    }

    pub async fn validate_circuit(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        circuit_id: Uuid,
    ) -> HelixResult<Circuit> {
        let circuit = self
            .get_circuit(tenant_id, job_id, circuit_id)
            .await?
            .ok_or_else(|| HelixError::not_found("circuit not found"))?;
        let next = next_circuit_status(&circuit.status, "validate")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<CircuitRow> = sqlx::query_as(&format!(
            r#"
            UPDATE quantum.circuits
            SET status = $1, validated_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND status = $6 AND deleted_at IS NULL
            {CIRCUIT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(circuit_id)
        .bind(&circuit.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum validate circuit: {e}")))?;

        row.map(CircuitRow::into_circuit)
            .ok_or_else(|| HelixError::conflict("circuit changed during validate; retry"))
    }

    pub async fn archive_circuit(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        circuit_id: Uuid,
    ) -> HelixResult<Circuit> {
        let circuit = self
            .get_circuit(tenant_id, job_id, circuit_id)
            .await?
            .ok_or_else(|| HelixError::not_found("circuit not found"))?;
        let next = next_circuit_status(&circuit.status, "archive")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<CircuitRow> = sqlx::query_as(&format!(
            r#"
            UPDATE quantum.circuits
            SET status = $1, archived_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND status = $6 AND deleted_at IS NULL
            {CIRCUIT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(circuit_id)
        .bind(&circuit.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum archive circuit: {e}")))?;

        row.map(CircuitRow::into_circuit)
            .ok_or_else(|| HelixError::conflict("circuit changed during archive; retry"))
    }

    pub async fn soft_delete_circuit(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        circuit_id: Uuid,
    ) -> HelixResult<Circuit> {
        let circuit = self
            .get_circuit(tenant_id, job_id, circuit_id)
            .await?
            .ok_or_else(|| HelixError::not_found("circuit not found"))?;
        if circuit.status == "deleted" {
            return Err(HelixError::validation("circuit is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<CircuitRow> = sqlx::query_as(&format!(
            r#"
            UPDATE quantum.circuits
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {CIRCUIT_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(circuit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum soft-delete circuit: {e}")))?;

        row.map(CircuitRow::into_circuit)
            .ok_or_else(|| HelixError::not_found("circuit not found"))
    }

    /// Restore a soft-deleted circuit, returning it to its pre-delete status.
    pub async fn restore_circuit(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        circuit_id: Uuid,
    ) -> HelixResult<Circuit> {
        let circuit = self
            .fetch_circuit_any(tenant_id, job_id, circuit_id)
            .await?
            .ok_or_else(|| HelixError::not_found("circuit not found"))?;
        if circuit.deleted_at.is_none() {
            return Err(HelixError::validation("circuit is not deleted"));
        }
        let restored = if circuit.archived_at.is_some() {
            "archived"
        } else if circuit.validated_at.is_some() {
            "validated"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<CircuitRow> = sqlx::query_as(&format!(
            r#"
            UPDATE quantum.circuits
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {CIRCUIT_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(circuit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum restore circuit: {e}")))?;

        row.map(CircuitRow::into_circuit)
            .ok_or_else(|| HelixError::not_found("circuit not found or not deleted"))
    }

    // --- Reports ---

    /// Per-job circuit counts by status for non-deleted jobs.
    pub async fn get_quantum_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<QuantumSummaryRow>> {
        let rows: Vec<QuantumSummaryRow> = sqlx::query_as(
            r#"
            SELECT j.id, j.name, j.status,
                   COUNT(c.id) AS total_circuits,
                   COUNT(c.id) FILTER (WHERE c.status = 'draft') AS draft_circuits,
                   COUNT(c.id) FILTER (WHERE c.status = 'validated') AS validated_circuits,
                   COUNT(c.id) FILTER (WHERE c.status = 'archived') AS archived_circuits
            FROM quantum.jobs j
            LEFT JOIN quantum.circuits c
                   ON c.parent_id = j.id AND c.tenant_id = j.tenant_id
                  AND c.deleted_at IS NULL
            WHERE j.tenant_id = $1 AND j.deleted_at IS NULL
            GROUP BY j.id, j.name, j.status, j.created_at
            ORDER BY j.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("quantum summary: {e}")))?;
        Ok(rows)
    }
}
