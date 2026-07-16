//! Atomic unit-of-work helper — commit domain change, audit event, and outbox
//! entry together, or roll them all back together.

use audit_log::{AuditEntry, AuditEvent};
use shared_core::{HelixError, HelixResult};
use sqlx::{PgPool, Postgres, Transaction};

use crate::audit_pg::TransactionalAuditSink;
use crate::jobs::{Job, JobRepo};
use crate::outbox::{OutboxItem, OutboxRepo};
use shared_core::ids::{TenantId, UserId};

/// A short-lived handle to an in-flight transaction that can append audit
/// entries, enqueue outbox items, and create durable jobs as one atomic unit.
pub struct AtomicWork<'a> {
    tx: Transaction<'a, Postgres>,
    audit: &'a dyn TransactionalAuditSink,
    outbox: OutboxRepo,
    jobs: JobRepo,
}

impl<'a> AtomicWork<'a> {
    pub async fn begin(
        pool: &'a PgPool,
        audit: &'a dyn TransactionalAuditSink,
    ) -> HelixResult<Self> {
        let tx = pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("atomic begin: {e}")))?;
        Ok(Self {
            tx,
            audit,
            outbox: OutboxRepo::new(),
            jobs: JobRepo::new(pool.clone()),
        })
    }

    /// Append an audit entry inside the unit-of-work transaction.
    pub async fn append_audit(&mut self, event: AuditEvent) -> HelixResult<AuditEntry> {
        let (entry, _seq) = self.audit.append_in_tx(&mut self.tx, event).await?;
        Ok(entry)
    }

    /// Enqueue an outbox item inside the unit-of-work transaction.
    pub async fn enqueue_outbox(
        &mut self,
        tenant_id: TenantId,
        idempotency_key: impl Into<String>,
        topic: impl Into<String>,
        payload: serde_json::Value,
        headers: serde_json::Value,
    ) -> HelixResult<OutboxItem> {
        self.outbox
            .enqueue_in_tx(
                &mut self.tx,
                tenant_id,
                idempotency_key,
                topic,
                payload,
                headers,
            )
            .await
    }

    /// Create a durable job inside the unit-of-work transaction.
    pub async fn create_job(
        &mut self,
        tenant_id: TenantId,
        user_id: UserId,
        kind: impl Into<String>,
        requested: impl Into<String>,
        max_retries: i32,
    ) -> HelixResult<Job> {
        self.jobs
            .create_in_tx(
                &mut self.tx,
                tenant_id,
                user_id,
                kind,
                requested,
                max_retries,
            )
            .await
    }

    /// Commit the transaction. Returns an error if the commit fails, leaving
    /// the database unchanged.
    pub async fn commit(self) -> HelixResult<()> {
        self.tx
            .commit()
            .await
            .map_err(|e| HelixError::dependency(format!("atomic commit: {e}")))?;
        Ok(())
    }

    /// Roll back the transaction explicitly. Useful when a caller wants to
    /// abort after a domain-level validation failure.
    pub async fn rollback(self) -> HelixResult<()> {
        self.tx
            .rollback()
            .await
            .map_err(|e| HelixError::dependency(format!("atomic rollback: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_pg::PgAuditSink;
    use crate::tenants::TenantRepo;
    use audit_log::AuditEvent;
    use shared_core::tenancy::Actor;

    fn db_url() -> Option<String> {
        std::env::var("DATABASE_URL").ok()
    }

    #[tokio::test]
    async fn audit_outbox_job_commit_together() {
        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let audit = PgAuditSink::new(pool.clone());
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "atomic-test", "local", None)
            .await
            .unwrap();
        let user = UserId::new();

        let mut work = AtomicWork::begin(&pool, &audit).await.unwrap();
        let job = work
            .create_job(tenant.id, user, "export", "export data", 3)
            .await
            .unwrap();
        let _entry = work
            .append_audit(AuditEvent {
                tenant_id: Some(tenant.id),
                actor: Actor::System {
                    reason: "atomic-test".into(),
                },
                action: "job.created".into(),
                resource_type: "job".into(),
                resource_id: job.id.to_string(),
                metadata: serde_json::json!({"kind": "export"}),
                residency_region: "local".into(),
            })
            .await
            .unwrap();
        let _outbox = work
            .enqueue_outbox(
                tenant.id,
                format!("job:{}:created", job.id),
                "job.created",
                serde_json::json!({"job_id": job.id.to_string()}),
                serde_json::json!({}),
            )
            .await
            .unwrap();
        work.commit().await.unwrap();

        // Verify all three rows are durable.
        let fetched_job = crate::jobs::JobRepo::new(pool.clone())
            .get(tenant.id, job.id)
            .await
            .unwrap();
        assert!(fetched_job.is_some());

        let audit_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit.events WHERE resource_id = $1")
                .bind(job.id.to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(audit_count, 1);

        let outbox_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM helix_core.outbox WHERE tenant_id = $1")
                .bind(tenant.id.as_uuid())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(outbox_count, 1);
    }

    #[tokio::test]
    async fn audit_outbox_job_rollback_together() {
        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let audit = PgAuditSink::new(pool.clone());
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "atomic-rollback", "local", None)
            .await
            .unwrap();
        let user = UserId::new();

        let mut work = AtomicWork::begin(&pool, &audit).await.unwrap();
        let job = work
            .create_job(tenant.id, user, "export", "export data", 3)
            .await
            .unwrap();
        let _entry = work
            .append_audit(AuditEvent {
                tenant_id: Some(tenant.id),
                actor: Actor::System {
                    reason: "atomic-test".into(),
                },
                action: "job.created".into(),
                resource_type: "job".into(),
                resource_id: job.id.to_string(),
                metadata: serde_json::json!({}),
                residency_region: "local".into(),
            })
            .await
            .unwrap();
        let _outbox = work
            .enqueue_outbox(
                tenant.id,
                "rollback-key",
                "job.created",
                serde_json::json!({}),
                serde_json::json!({}),
            )
            .await
            .unwrap();
        work.rollback().await.unwrap();

        assert!(crate::jobs::JobRepo::new(pool.clone())
            .get(tenant.id, job.id)
            .await
            .unwrap()
            .is_none());
        let audit_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit.events WHERE resource_id = $1")
                .bind(job.id.to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(audit_count, 0);
        let outbox_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM helix_core.outbox WHERE tenant_id = $1")
                .bind(tenant.id.as_uuid())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(outbox_count, 0);
    }
}
