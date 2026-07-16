//! Durable job engine — stable identity, lease-based execution, checkpoints,
//! cancellation, and retry/recovery.
//!
//! The engine sits on top of `helix_db::JobRepo` and lets product code register
//! handlers by job kind. A `JobWorker` polls for claimable jobs, executes the
//! matching handler, and records heartbeats/checkpoints until the job reaches a
//! terminal state.

use async_trait::async_trait;
use chrono::Duration;
use helix_db::{Job, JobRepo, JobStatus};
use shared_core::{HelixError, HelixResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Result of running a job handler.
#[derive(Debug, Clone)]
pub enum JobOutcome {
    Completed(Option<serde_json::Value>),
    Failed(String),
}

/// Context passed to a handler so it can checkpoint progress and heartbeat.
#[derive(Clone)]
pub struct JobContext {
    repo: JobRepo,
    job_id: shared_core::ids::JobId,
}

impl JobContext {
    pub fn new(repo: JobRepo, job_id: shared_core::ids::JobId) -> Self {
        Self { repo, job_id }
    }

    /// Record a named checkpoint with an arbitrary payload.
    pub async fn checkpoint(
        &self,
        label: impl Into<String>,
        payload: serde_json::Value,
    ) -> HelixResult<()> {
        let ok = self.repo.checkpoint(self.job_id, label, payload).await?;
        if !ok {
            return Err(HelixError::not_found(format!(
                "job {} not running",
                self.job_id
            )));
        }
        Ok(())
    }

    /// Report that the job is still alive and optionally update resource usage.
    pub async fn heartbeat(
        &self,
        elapsed_ms: i64,
        resource_usage: serde_json::Value,
    ) -> HelixResult<()> {
        let ok = self
            .repo
            .heartbeat(self.job_id, elapsed_ms, resource_usage)
            .await?;
        if !ok {
            return Err(HelixError::not_found(format!(
                "job {} not running",
                self.job_id
            )));
        }
        Ok(())
    }
}

/// Implement this trait for each job kind the engine should run.
#[async_trait]
pub trait JobHandler: Send + Sync {
    async fn run(&self, job: &Job, ctx: JobContext) -> HelixResult<JobOutcome>;
}

/// Registry of handlers keyed by job kind.
#[derive(Clone, Default)]
pub struct JobRegistry {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn JobHandler>>>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, kind: impl Into<String>, handler: Arc<dyn JobHandler>) {
        let kind = kind.into();
        let mut map = self.handlers.write().expect("job registry lock poisoned");
        map.insert(kind, handler);
    }

    pub async fn get(&self, kind: &str) -> Option<Arc<dyn JobHandler>> {
        let map = self.handlers.read().expect("job registry lock poisoned");
        map.get(kind).cloned()
    }
}

/// Poll-based worker that claims and executes jobs.
#[derive(Clone)]
pub struct JobWorker {
    repo: JobRepo,
    registry: JobRegistry,
    worker_id: String,
    tenant_id: Option<shared_core::ids::TenantId>,
    lease_duration: Duration,
}

impl JobWorker {
    pub fn new(repo: JobRepo, registry: JobRegistry, worker_id: impl Into<String>) -> Self {
        Self {
            repo,
            registry,
            worker_id: worker_id.into(),
            tenant_id: None,
            lease_duration: Duration::seconds(60),
        }
    }

    /// Restrict this worker to a single tenant.
    pub fn for_tenant(mut self, tenant_id: shared_core::ids::TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Set how long a claimed lease lasts before another worker can take over.
    pub fn with_lease_duration(mut self, duration: Duration) -> Self {
        self.lease_duration = duration;
        self
    }

    /// Try to claim and execute exactly one job. Returns the job id if work was
    /// done, or `None` when no jobs are claimable.
    pub async fn run_once(&self) -> HelixResult<Option<shared_core::ids::JobId>> {
        let maybe_job = match self.tenant_id {
            Some(tenant_id) => {
                self.repo
                    .claim_next_for_tenant(tenant_id, &self.worker_id, self.lease_duration)
                    .await?
            }
            None => {
                self.repo
                    .claim_next(&self.worker_id, self.lease_duration)
                    .await?
            }
        };

        let job = match maybe_job {
            Some(j) => j,
            None => return Ok(None),
        };

        // Honour a cancel request before starting productive work.
        if job.cancel_requested {
            self.repo.set_status(job.id, JobStatus::Cancelled).await?;
            return Ok(Some(job.id));
        }

        let handler = self.registry.get(&job.kind).await.ok_or_else(|| {
            HelixError::internal(format!("no handler registered for job kind '{}'", job.kind))
        })?;

        self.repo.set_status(job.id, JobStatus::Running).await?;
        let ctx = JobContext::new(self.repo.clone(), job.id);

        let result = handler.run(&job, ctx).await;

        match result {
            Ok(JobOutcome::Completed(output)) => {
                self.repo.complete(job.id, output).await?;
            }
            Ok(JobOutcome::Failed(error)) => {
                self.handle_failure(job.id, error).await?;
            }
            Err(error) => {
                self.handle_failure(job.id, error.message).await?;
            }
        }

        Ok(Some(job.id))
    }

    async fn handle_failure(&self, id: shared_core::ids::JobId, error: String) -> HelixResult<()> {
        let status = self.repo.retry_or_fail(id, error).await?;
        if status == JobStatus::Failed {
            tracing::warn!(job_id = %id, "job reached max retries and failed");
        }
        Ok(())
    }

    /// Run continuously until the shutdown signal fires, polling every
    /// `poll_interval`. Stops cleanly after the current job finishes.
    pub async fn run(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
        poll_interval: std::time::Duration,
    ) -> HelixResult<()> {
        loop {
            if *shutdown.borrow() {
                break;
            }

            match self.run_once().await {
                Ok(Some(id)) => tracing::info!(job_id = %id, "executed job"),
                Ok(None) => {
                    tokio::select! {
                        _ = tokio::time::sleep(poll_interval) => {}
                        _ = shutdown.changed() => break,
                    }
                }
                Err(error) => {
                    tracing::error!(error = %error, "job worker error");
                    tokio::time::sleep(poll_interval).await;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use audit_log::AuditEvent;
    use chrono::Duration;
    use helix_db::{AtomicWork, JobRepo, TenantRepo};
    use shared_core::ids::{TenantId, UserId};
    use shared_core::tenancy::Actor;

    fn db_url() -> Option<String> {
        std::env::var("DATABASE_URL").ok()
    }

    struct EchoHandler;

    #[async_trait]
    impl JobHandler for EchoHandler {
        async fn run(&self, job: &Job, ctx: JobContext) -> HelixResult<JobOutcome> {
            ctx.heartbeat(10, serde_json::json!({"cpu_ms": 1})).await?;
            ctx.checkpoint("echoed", serde_json::json!({"requested": job.requested}))
                .await?;
            Ok(JobOutcome::Completed(Some(serde_json::json!({
                "echo": job.requested
            }))))
        }
    }

    struct FailingHandler;

    #[async_trait]
    impl JobHandler for FailingHandler {
        async fn run(&self, _job: &Job, _ctx: JobContext) -> HelixResult<JobOutcome> {
            Ok(JobOutcome::Failed("boom".into()))
        }
    }

    #[tokio::test]
    async fn worker_runs_registered_handler() {
        let Some(url) = db_url() else { return };
        let pool = helix_db::connect_and_migrate(&url).await.unwrap();
        let audit = helix_db::PgAuditSink::new(pool.clone());
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "job-engine", "local", None)
            .await
            .unwrap();
        let user = UserId::new();

        let mut work = AtomicWork::begin(&pool, &audit).await.unwrap();
        let job = work
            .create_job(tenant.id, user, "echo", "hello world", 0)
            .await
            .unwrap();
        let _ = work
            .append_audit(AuditEvent {
                tenant_id: Some(tenant.id),
                actor: Actor::System {
                    reason: "job-engine-test".into(),
                },
                action: "job.queued".into(),
                resource_type: "job".into(),
                resource_id: job.id.to_string(),
                metadata: serde_json::json!({}),
                residency_region: "local".into(),
            })
            .await
            .unwrap();
        work.commit().await.unwrap();

        let registry = JobRegistry::new();
        registry.register("echo", Arc::new(EchoHandler));
        let worker = JobWorker::new(JobRepo::new(pool.clone()), registry, "test-worker")
            .for_tenant(tenant.id)
            .with_lease_duration(Duration::seconds(60));

        let ran = worker.run_once().await.unwrap();
        assert_eq!(ran, Some(job.id));

        let repo = JobRepo::new(pool.clone());
        let finished = repo.get(tenant.id, job.id).await.unwrap().unwrap();
        assert_eq!(finished.status, JobStatus::Completed);
        assert_eq!(finished.checkpoints.len(), 1);
    }

    #[tokio::test]
    async fn worker_retries_then_fails() {
        let Some(url) = db_url() else { return };
        let pool = helix_db::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "job-engine-fail", "local", None)
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let job = JobRepo::new(pool.clone())
            .create_in_tx(&mut tx, tenant.id, UserId::new(), "fail", "will fail", 1)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let registry = JobRegistry::new();
        registry.register("fail", Arc::new(FailingHandler));
        let worker = JobWorker::new(JobRepo::new(pool.clone()), registry, "test-worker")
            .for_tenant(tenant.id)
            .with_lease_duration(Duration::seconds(60));

        // First run requeues (retry_count 0 < max_retries 1).
        worker.run_once().await.unwrap();
        let repo = JobRepo::new(pool.clone());
        let after_first = repo.get(tenant.id, job.id).await.unwrap().unwrap();
        assert_eq!(after_first.status, JobStatus::Queued);
        assert_eq!(after_first.retry_count, 1);

        // Second run fails permanently.
        worker.run_once().await.unwrap();
        let after_second = repo.get(tenant.id, job.id).await.unwrap().unwrap();
        assert_eq!(after_second.status, JobStatus::Failed);
    }

    #[tokio::test]
    async fn worker_respects_cancel_request() {
        let Some(url) = db_url() else { return };
        let pool = helix_db::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "job-engine-cancel", "local", None)
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let job = JobRepo::new(pool.clone())
            .create_in_tx(&mut tx, tenant.id, UserId::new(), "echo", "cancel me", 0)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let repo = JobRepo::new(pool.clone());
        repo.request_cancel(tenant.id, job.id).await.unwrap();

        let registry = JobRegistry::new();
        registry.register("echo", Arc::new(EchoHandler));
        let worker = JobWorker::new(repo.clone(), registry, "test-worker").for_tenant(tenant.id);

        worker.run_once().await.unwrap();
        let finished = repo.get(tenant.id, job.id).await.unwrap().unwrap();
        assert_eq!(finished.status, JobStatus::Cancelled);
    }
}
