//! Durable job repository — stable identity, tenant-scoped lifecycle, lease,
//! heartbeat, checkpoint, cancel, and orphan recovery.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::{JobId, TenantId, UserId};
use shared_core::{HelixError, HelixResult, SemanticState};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Starting,
    Running,
    Waiting,
    Blocked,
    Cancelling,
    Cancelled,
    Failed,
    Completed,
    Unknown,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Blocked => "blocked",
            Self::Cancelling => "cancelling",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Completed => "completed",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "queued" => Self::Queued,
            "starting" => Self::Starting,
            "running" => Self::Running,
            "waiting" => Self::Waiting,
            "blocked" => Self::Blocked,
            "cancelling" => Self::Cancelling,
            "cancelled" => Self::Cancelled,
            "failed" => Self::Failed,
            "completed" => Self::Completed,
            _ => Self::Unknown,
        }
    }

    /// Whether the job is still allowed to do productive work.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Queued | Self::Starting | Self::Running | Self::Waiting | Self::Blocked
        )
    }

    /// Whether the job has reached a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Cancelled | Self::Failed | Self::Completed)
    }
}

impl From<JobStatus> for SemanticState {
    fn from(status: JobStatus) -> Self {
        match status {
            JobStatus::Queued | JobStatus::Starting | JobStatus::Running => Self::Active,
            JobStatus::Waiting | JobStatus::Blocked => Self::WaitingExternal,
            JobStatus::Cancelling => Self::WaitingHuman,
            JobStatus::Cancelled | JobStatus::Failed => Self::Failed,
            JobStatus::Completed => Self::Completed,
            JobStatus::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCheckpoint {
    pub label: String,
    pub recorded_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub kind: String,
    pub status: JobStatus,
    pub requested: String,
    pub stages: Vec<String>,
    pub checkpoints: Vec<JobCheckpoint>,
    pub process_identity: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub cancel_requested: bool,
    pub retry_count: i32,
    pub max_retries: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub elapsed_ms: i64,
    pub resource_usage: serde_json::Value,
    pub error: Option<String>,
    pub final_output: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct JobRepo {
    pool: sqlx::PgPool,
}

impl JobRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        tenant_id: TenantId,
        user_id: UserId,
        kind: impl Into<String>,
        requested: impl Into<String>,
        max_retries: i32,
    ) -> HelixResult<Job> {
        let id = JobId::new();
        let kind = kind.into();
        let requested = requested.into();
        let max_retries = max_retries.max(0);

        let row: JobRow = sqlx::query_as(
            r#"
            INSERT INTO helix_core.jobs (
                id, tenant_id, user_id, kind, status, requested,
                max_retries, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, 'queued', $5, $6, $7, $7)
            RETURNING
                id, tenant_id, user_id, kind, status, requested, stages, checkpoints,
                process_identity, lease_expires_at, cancel_requested, retry_count, max_retries,
                started_at, last_heartbeat_at, elapsed_ms, resource_usage, error, final_output,
                created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(&kind)
        .bind(&requested)
        .bind(max_retries)
        .bind(Utc::now())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("job create: {e}")))?;

        Ok(row.into_job())
    }

    pub async fn get(&self, tenant_id: TenantId, id: JobId) -> HelixResult<Option<Job>> {
        let row: Option<JobRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, user_id, kind, status, requested, stages, checkpoints,
                process_identity, lease_expires_at, cancel_requested, retry_count, max_retries,
                started_at, last_heartbeat_at, elapsed_ms, resource_usage, error, final_output,
                created_at, updated_at
            FROM helix_core.jobs
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("job get: {e}")))?;

        Ok(row.map(|r| r.into_job()))
    }

    pub async fn list_for_tenant(&self, tenant_id: TenantId, limit: i64) -> HelixResult<Vec<Job>> {
        let rows: Vec<JobRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, user_id, kind, status, requested, stages, checkpoints,
                process_identity, lease_expires_at, cancel_requested, retry_count, max_retries,
                started_at, last_heartbeat_at, elapsed_ms, resource_usage, error, final_output,
                created_at, updated_at
            FROM helix_core.jobs
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("job list: {e}")))?;

        Ok(rows.into_iter().map(|r| r.into_job()).collect())
    }

    /// Claim the next available job for a worker. Uses `SKIP LOCKED` so
    /// multiple workers can poll safely. The returned job already has its lease
    /// extended and status advanced from `queued` to `starting`.
    pub async fn claim_next(
        &self,
        worker_id: impl Into<String>,
        lease_duration: Duration,
    ) -> HelixResult<Option<Job>> {
        self.claim_next_internal(None, worker_id, lease_duration)
            .await
    }

    /// Tenant-scoped variant of `claim_next` for product-specific workers.
    pub async fn claim_next_for_tenant(
        &self,
        tenant_id: TenantId,
        worker_id: impl Into<String>,
        lease_duration: Duration,
    ) -> HelixResult<Option<Job>> {
        self.claim_next_internal(Some(tenant_id), worker_id, lease_duration)
            .await
    }

    async fn claim_next_internal(
        &self,
        tenant_id: Option<TenantId>,
        worker_id: impl Into<String>,
        lease_duration: Duration,
    ) -> HelixResult<Option<Job>> {
        let worker_id = worker_id.into();
        let lease_seconds = lease_duration.num_seconds().max(1) as i32;

        let row: Option<JobRow> = if let Some(tenant_id) = tenant_id {
            sqlx::query_as(
                r#"
                UPDATE helix_core.jobs
                SET
                    process_identity = $1,
                    lease_expires_at = now() + make_interval(secs => $2),
                    status = CASE WHEN status = 'queued' THEN 'starting' ELSE status END,
                    started_at = COALESCE(started_at, now()),
                    last_heartbeat_at = now(),
                    updated_at = now()
                WHERE id = (
                    SELECT id FROM helix_core.jobs
                    WHERE tenant_id = $3
                      AND status IN ('queued', 'running', 'waiting', 'blocked', 'cancelling')
                      AND (lease_expires_at IS NULL OR lease_expires_at < now())
                    ORDER BY created_at ASC
                    FOR UPDATE SKIP LOCKED
                    LIMIT 1
                )
                RETURNING
                    id, tenant_id, user_id, kind, status, requested, stages, checkpoints,
                    process_identity, lease_expires_at, cancel_requested, retry_count, max_retries,
                    started_at, last_heartbeat_at, elapsed_ms, resource_usage, error, final_output,
                    created_at, updated_at
                "#,
            )
            .bind(&worker_id)
            .bind(lease_seconds)
            .bind(tenant_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                UPDATE helix_core.jobs
                SET
                    process_identity = $1,
                    lease_expires_at = now() + make_interval(secs => $2),
                    status = CASE WHEN status = 'queued' THEN 'starting' ELSE status END,
                    started_at = COALESCE(started_at, now()),
                    last_heartbeat_at = now(),
                    updated_at = now()
                WHERE id = (
                    SELECT id FROM helix_core.jobs
                    WHERE status IN ('queued', 'running', 'waiting', 'blocked', 'cancelling')
                      AND (lease_expires_at IS NULL OR lease_expires_at < now())
                      AND cancel_requested = false
                    ORDER BY created_at ASC
                    FOR UPDATE SKIP LOCKED
                    LIMIT 1
                )
                RETURNING
                    id, tenant_id, user_id, kind, status, requested, stages, checkpoints,
                    process_identity, lease_expires_at, cancel_requested, retry_count, max_retries,
                    started_at, last_heartbeat_at, elapsed_ms, resource_usage, error, final_output,
                    created_at, updated_at
                "#,
            )
            .bind(&worker_id)
            .bind(lease_seconds)
            .fetch_optional(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("job claim: {e}")))?;

        Ok(row.map(|r| r.into_job()))
    }

    pub async fn heartbeat(
        &self,
        id: JobId,
        elapsed_ms: i64,
        resource_usage: serde_json::Value,
    ) -> HelixResult<bool> {
        let n = sqlx::query(
            r#"
            UPDATE helix_core.jobs
            SET last_heartbeat_at = now(),
                elapsed_ms = $2,
                resource_usage = $3,
                updated_at = now()
            WHERE id = $1
              AND status IN ('starting', 'running', 'waiting', 'blocked', 'cancelling')
            "#,
        )
        .bind(id.as_uuid())
        .bind(elapsed_ms)
        .bind(resource_usage)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("job heartbeat: {e}")))?
        .rows_affected();

        Ok(n > 0)
    }

    pub async fn checkpoint(
        &self,
        id: JobId,
        label: impl Into<String>,
        payload: serde_json::Value,
    ) -> HelixResult<bool> {
        let label = label.into();
        let checkpoint = serde_json::json!({
            "label": label,
            "recorded_at": Utc::now().to_rfc3339(),
            "payload": payload,
        });

        let n = sqlx::query(
            r#"
            UPDATE helix_core.jobs
            SET checkpoints = checkpoints || $2::jsonb,
                updated_at = now()
            WHERE id = $1
              AND status IN ('starting', 'running', 'waiting', 'blocked', 'cancelling')
            "#,
        )
        .bind(id.as_uuid())
        .bind(checkpoint)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("job checkpoint: {e}")))?
        .rows_affected();

        Ok(n > 0)
    }

    pub async fn request_cancel(&self, tenant_id: TenantId, id: JobId) -> HelixResult<bool> {
        let n = sqlx::query(
            r#"
            UPDATE helix_core.jobs
            SET cancel_requested = true,
                status = CASE WHEN status IN ('queued', 'starting', 'running', 'waiting', 'blocked')
                              THEN 'cancelling' ELSE status END,
                updated_at = now()
            WHERE tenant_id = $1 AND id = $2 AND status NOT IN ('cancelled', 'failed', 'completed')
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("job cancel request: {e}")))?
        .rows_affected();

        Ok(n > 0)
    }

    pub async fn set_status(&self, id: JobId, status: JobStatus) -> HelixResult<bool> {
        let n =
            sqlx::query("UPDATE helix_core.jobs SET status = $2, updated_at = now() WHERE id = $1")
                .bind(id.as_uuid())
                .bind(status.as_str())
                .execute(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("job status: {e}")))?
                .rows_affected();

        Ok(n > 0)
    }

    pub async fn complete(
        &self,
        id: JobId,
        final_output: Option<serde_json::Value>,
    ) -> HelixResult<bool> {
        let n = sqlx::query(
            r#"
            UPDATE helix_core.jobs
            SET status = 'completed',
                final_output = $2,
                lease_expires_at = NULL,
                updated_at = now()
            WHERE id = $1
              AND status IN ('starting', 'running', 'waiting', 'blocked', 'cancelling')
            "#,
        )
        .bind(id.as_uuid())
        .bind(final_output)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("job complete: {e}")))?
        .rows_affected();

        Ok(n > 0)
    }

    pub async fn fail(&self, id: JobId, error: impl Into<String>) -> HelixResult<bool> {
        let error = error.into();
        let n = sqlx::query(
            r#"
            UPDATE helix_core.jobs
            SET status = 'failed',
                error = $2,
                lease_expires_at = NULL,
                updated_at = now()
            WHERE id = $1
              AND status IN ('starting', 'running', 'waiting', 'blocked', 'cancelling')
            "#,
        )
        .bind(id.as_uuid())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("job fail: {e}")))?
        .rows_affected();

        Ok(n > 0)
    }

    /// Either requeue for retry or mark failed based on retry budget.
    pub async fn retry_or_fail(
        &self,
        id: JobId,
        error: impl Into<String>,
    ) -> HelixResult<JobStatus> {
        let error = error.into();
        let status: Option<String> = sqlx::query_scalar(
            r#"
            UPDATE helix_core.jobs
            SET status = CASE WHEN retry_count < max_retries THEN 'queued' ELSE 'failed' END,
                retry_count = retry_count + 1,
                error = $2,
                lease_expires_at = NULL,
                process_identity = NULL,
                updated_at = now()
            WHERE id = $1
              AND status IN ('starting', 'running', 'waiting', 'blocked', 'cancelling')
            RETURNING status
            "#,
        )
        .bind(id.as_uuid())
        .bind(error)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("job retry/fail: {e}")))?;

        Ok(status
            .map(|s| JobStatus::parse(&s))
            .unwrap_or(JobStatus::Unknown))
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: Uuid,
    tenant_id: Uuid,
    user_id: Uuid,
    kind: String,
    status: String,
    requested: String,
    stages: serde_json::Value,
    checkpoints: serde_json::Value,
    process_identity: Option<String>,
    lease_expires_at: Option<DateTime<Utc>>,
    cancel_requested: bool,
    retry_count: i32,
    max_retries: i32,
    started_at: Option<DateTime<Utc>>,
    last_heartbeat_at: Option<DateTime<Utc>>,
    elapsed_ms: i64,
    resource_usage: serde_json::Value,
    error: Option<String>,
    final_output: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl JobRow {
    fn into_job(self) -> Job {
        let stages: Vec<String> = serde_json::from_value(self.stages).unwrap_or_default();
        let checkpoints: Vec<JobCheckpoint> =
            serde_json::from_value(self.checkpoints).unwrap_or_default();

        Job {
            id: JobId::from_uuid(self.id),
            tenant_id: TenantId::from_uuid(self.tenant_id),
            user_id: UserId::from_uuid(self.user_id),
            kind: self.kind,
            status: JobStatus::parse(&self.status),
            requested: self.requested,
            stages,
            checkpoints,
            process_identity: self.process_identity,
            lease_expires_at: self.lease_expires_at,
            cancel_requested: self.cancel_requested,
            retry_count: self.retry_count,
            max_retries: self.max_retries,
            started_at: self.started_at,
            last_heartbeat_at: self.last_heartbeat_at,
            elapsed_ms: self.elapsed_ms,
            resource_usage: self.resource_usage,
            error: self.error,
            final_output: self.final_output,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenants::TenantRepo;

    #[test]
    fn job_status_maps_to_semantic_state() {
        assert_eq!(
            SemanticState::from(JobStatus::Running),
            SemanticState::Active
        );
        assert_eq!(
            SemanticState::from(JobStatus::Waiting),
            SemanticState::WaitingExternal
        );
        assert_eq!(
            SemanticState::from(JobStatus::Blocked),
            SemanticState::WaitingExternal
        );
        assert_eq!(
            SemanticState::from(JobStatus::Cancelling),
            SemanticState::WaitingHuman
        );
        assert_eq!(
            SemanticState::from(JobStatus::Completed),
            SemanticState::Completed
        );
        assert_eq!(
            SemanticState::from(JobStatus::Failed),
            SemanticState::Failed
        );
        assert_eq!(
            SemanticState::from(JobStatus::Unknown),
            SemanticState::Unknown
        );
    }

    fn db_url() -> Option<String> {
        std::env::var("DATABASE_URL").ok()
    }

    async fn test_pool_and_tenant() -> Option<(sqlx::PgPool, TenantId, UserId)> {
        let url = db_url()?;
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "job-test", "local", None)
            .await
            .unwrap();
        Some((pool, tenant.id, UserId::new()))
    }

    #[tokio::test]
    async fn job_lifecycle_roundtrip() {
        let Some((pool, tenant_id, user_id)) = test_pool_and_tenant().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();
        let jobs = JobRepo::new(pool.clone());
        let job = jobs
            .create_in_tx(&mut tx, tenant_id, user_id, "export", "export user data", 2)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(job.status, JobStatus::Queued);

        let claimed = jobs
            .claim_next_for_tenant(tenant_id, "worker-1", Duration::seconds(60))
            .await
            .unwrap()
            .expect("job should be claimable");
        assert_eq!(claimed.id, job.id);
        assert_eq!(claimed.status, JobStatus::Starting);
        assert_eq!(claimed.process_identity.as_deref(), Some("worker-1"));

        jobs.set_status(job.id, JobStatus::Running).await.unwrap();
        jobs.heartbeat(job.id, 1200, serde_json::json!({"cpu_ms": 50}))
            .await
            .unwrap();
        jobs.checkpoint(job.id, "parsed", serde_json::json!({"rows": 42}))
            .await
            .unwrap();

        let fetched = jobs.get(tenant_id, job.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, JobStatus::Running);
        assert_eq!(fetched.elapsed_ms, 1200);
        assert_eq!(fetched.checkpoints.len(), 1);

        jobs.complete(job.id, Some(serde_json::json!({"url": "s3://..."})))
            .await
            .unwrap();
        let fetched = jobs.get(tenant_id, job.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, JobStatus::Completed);
        assert!(fetched.lease_expires_at.is_none());
    }

    #[tokio::test]
    async fn job_cancel_and_orphan_recovery() {
        let Some((pool, tenant_id, user_id)) = test_pool_and_tenant().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();
        let jobs = JobRepo::new(pool.clone());
        let job = jobs
            .create_in_tx(&mut tx, tenant_id, user_id, "import", "import csv", 0)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let claimed = jobs
            .claim_next_for_tenant(tenant_id, "worker-a", Duration::seconds(60))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(claimed.id, job.id);

        // Request cancel while running.
        assert!(jobs.request_cancel(tenant_id, job.id).await.unwrap());
        let fetched = jobs.get(tenant_id, job.id).await.unwrap().unwrap();
        assert!(fetched.cancel_requested);
        assert_eq!(fetched.status, JobStatus::Cancelling);

        // Worker sees cancel and marks cancelled.
        jobs.set_status(job.id, JobStatus::Cancelled).await.unwrap();
        let fetched = jobs.get(tenant_id, job.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, JobStatus::Cancelled);

        // A cancelled/completed job must not be reclaimable.
        let not_claimed = jobs
            .claim_next_for_tenant(tenant_id, "worker-b", Duration::seconds(60))
            .await
            .unwrap();
        assert!(not_claimed.is_none());
    }

    #[tokio::test]
    async fn job_retry_or_fail_respects_budget() {
        let Some((pool, tenant_id, user_id)) = test_pool_and_tenant().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();
        let jobs = JobRepo::new(pool.clone());
        let job = jobs
            .create_in_tx(&mut tx, tenant_id, user_id, "transform", "run transform", 2)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        jobs.claim_next_for_tenant(tenant_id, "worker-1", Duration::seconds(60))
            .await
            .unwrap()
            .unwrap();

        let status = jobs.retry_or_fail(job.id, "network timeout").await.unwrap();
        assert_eq!(status, JobStatus::Queued);
        let fetched = jobs.get(tenant_id, job.id).await.unwrap().unwrap();
        assert_eq!(fetched.retry_count, 1);

        for _ in 0..2 {
            jobs.claim_next_for_tenant(tenant_id, "worker-1", Duration::seconds(60))
                .await
                .unwrap()
                .unwrap();
            jobs.retry_or_fail(job.id, "still broken").await.unwrap();
        }

        let fetched = jobs.get(tenant_id, job.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, JobStatus::Failed);
        assert_eq!(fetched.error.as_deref(), Some("still broken"));
    }
}
