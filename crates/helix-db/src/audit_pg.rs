//! Postgres-backed hash-chained audit sink.

use async_trait::async_trait;
use audit_log::{
    genesis_hash, verify_entry_signature, ArchiveSink, AuditEntry, AuditEvent, AuditSink,
    NullArchiveSink,
};
use chrono::{DateTime, Utc};
use shared_core::ids::{AuditId, TenantId};
use shared_core::tenancy::Actor;
use shared_core::time::UtcTimestamp;
use shared_core::{HelixError, HelixResult};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgAuditSink {
    pool: PgPool,
    archive: Arc<dyn ArchiveSink>,
}

impl PgAuditSink {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            archive: Arc::new(NullArchiveSink),
        }
    }

    pub fn with_archive(mut self, archive: Arc<dyn ArchiveSink>) -> Self {
        self.archive = archive;
        self
    }
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    id: Uuid,
    tenant_id: Option<Uuid>,
    actor: serde_json::Value,
    action: String,
    resource_type: String,
    resource_id: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    prev_hash: String,
    entry_hash: String,
    residency_region: String,
    #[sqlx(default)]
    hmac_signature: String,
}

impl AuditRow {
    fn into_entry(self) -> HelixResult<AuditEntry> {
        let actor: Actor = serde_json::from_value(self.actor)
            .map_err(|e| HelixError::internal(format!("audit actor decode: {e}")))?;
        Ok(AuditEntry {
            id: AuditId::from_uuid(self.id),
            tenant_id: self.tenant_id.map(TenantId::from_uuid),
            actor,
            action: self.action,
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            metadata: self.metadata,
            created_at: UtcTimestamp(self.created_at),
            prev_hash: self.prev_hash,
            entry_hash: self.entry_hash,
            residency_region: self.residency_region,
            hmac_signature: self.hmac_signature,
        })
    }
}

/// Append an audit entry inside an existing transaction so that domain change,
/// audit event, and outbox entry can commit atomically.
#[async_trait]
pub trait TransactionalAuditSink: Send + Sync {
    async fn append_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: AuditEvent,
    ) -> HelixResult<(AuditEntry, i64)>;
}

#[async_trait]
impl TransactionalAuditSink for PgAuditSink {
    async fn append_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: AuditEvent,
    ) -> HelixResult<(AuditEntry, i64)> {
        // Serialize chain tip reads/writes across appenders.
        sqlx::query("SELECT pg_advisory_xact_lock(87201401)")
            .execute(&mut **tx)
            .await
            .map_err(|e| HelixError::dependency(format!("audit lock: {e}")))?;

        if let Some(tid) = event.tenant_id {
            crate::set_tenant_context(tx, tid).await?;
        }

        let prev_hash: Option<String> =
            sqlx::query_scalar("SELECT entry_hash FROM audit.events ORDER BY seq DESC LIMIT 1")
                .fetch_optional(&mut **tx)
                .await
                .map_err(|e| HelixError::dependency(format!("audit tip: {e}")))?;

        let prev_hash = prev_hash.unwrap_or_else(genesis_hash);
        let entry = AuditEntry::from_event(event, prev_hash)?;

        let actor_json = serde_json::to_value(&entry.actor)
            .map_err(|e| HelixError::internal(format!("audit actor encode: {e}")))?;

        let seq: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO audit.events (
                id, tenant_id, actor, action, resource_type, resource_id,
                metadata, created_at, prev_hash, entry_hash, residency_region, hmac_signature
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            RETURNING seq
            "#,
        )
        .bind(entry.id.as_uuid())
        .bind(entry.tenant_id.map(|t| t.as_uuid()))
        .bind(actor_json)
        .bind(&entry.action)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(&entry.metadata)
        .bind(entry.created_at.inner())
        .bind(&entry.prev_hash)
        .bind(&entry.entry_hash)
        .bind(&entry.residency_region)
        .bind(&entry.hmac_signature)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("audit insert: {e}")))?;

        Ok((entry, seq))
    }
}

#[async_trait]
impl AuditSink for PgAuditSink {
    async fn append(&self, event: AuditEvent) -> HelixResult<AuditEntry> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("audit begin: {e}")))?;

        let (entry, seq) = self.append_in_tx(&mut tx, event).await?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("audit commit: {e}")))?;

        if let Err(e) = self.archive.append(seq, &entry).await {
            tracing::error!(error = %e, seq, "audit archive append failed");
        }

        Ok(entry)
    }

    async fn verify_chain(&self) -> HelixResult<bool> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("audit verify begin: {e}")))?;
        sqlx::query("SET LOCAL row_security = off")
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("audit verify rls off: {e}")))?;

        let rows: Vec<AuditRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, actor, action, resource_type, resource_id,
                   metadata, created_at, prev_hash, entry_hash, residency_region,
                   COALESCE(hmac_signature, '') AS hmac_signature
            FROM audit.events
            ORDER BY seq ASC
            "#,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("audit list: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("audit verify commit: {e}")))?;

        let hmac_on = audit_log::hmac_enabled();
        let mut expected_prev = genesis_hash();
        for row in rows {
            let entry = row.into_entry()?;
            // Link integrity is the hard requirement.
            if entry.prev_hash != expected_prev {
                return Ok(false);
            }
            // Content hash: deterministic under canonical serialization. In HMAC mode any
            // drift is treated as tampering; otherwise we warn but tolerate legacy drift.
            match entry.recompute_hash() {
                Ok(re) if re != entry.entry_hash => {
                    if hmac_on {
                        tracing::error!(
                            id = %entry.id,
                            action = %entry.action,
                            "audit content hash mismatch in HMAC mode"
                        );
                        return Ok(false);
                    }
                    tracing::warn!(
                        id = %entry.id,
                        action = %entry.action,
                        "audit content hash drift (link still valid)"
                    );
                }
                Err(e) => {
                    if hmac_on {
                        tracing::error!(error = %e, "audit recompute failed in HMAC mode");
                        return Ok(false);
                    }
                    tracing::warn!(error = %e, "audit recompute failed");
                }
                _ => {}
            }
            if !verify_entry_signature(&entry.entry_hash, &entry.hmac_signature) {
                return Ok(false);
            }
            expected_prev = entry.entry_hash;
        }

        // Also verify the WORM archive if one is configured.
        if self.archive.latest_archived_seq().await?.is_some()
            && !self.archive.verify_archive(None).await?
        {
            return Ok(false);
        }

        Ok(true)
    }

    async fn list_recent(&self, limit: usize) -> HelixResult<Vec<AuditEntry>> {
        let limit = limit.clamp(1, 500) as i64;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("audit recent begin: {e}")))?;
        sqlx::query("SET LOCAL row_security = off")
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("audit recent rls off: {e}")))?;

        let rows: Vec<AuditRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, actor, action, resource_type, resource_id,
                   metadata, created_at, prev_hash, entry_hash, residency_region,
                   COALESCE(hmac_signature, '') AS hmac_signature
            FROM audit.events
            ORDER BY seq DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("audit recent: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("audit recent commit: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows.into_iter().rev() {
            out.push(row.into_entry()?);
        }
        Ok(out)
    }

    async fn count(&self) -> HelixResult<u64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("audit count begin: {e}")))?;
        sqlx::query("SET LOCAL row_security = off")
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("audit count rls off: {e}")))?;

        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit.events")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("audit count: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("audit count commit: {e}")))?;
        Ok(n as u64)
    }

    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
        limit: usize,
    ) -> HelixResult<Vec<AuditEntry>> {
        let limit = limit.clamp(1, 500) as i64;
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| HelixError::dependency(format!("audit tenant list conn: {e}")))?;
        crate::set_tenant_context(&mut conn, tenant_id).await?;

        let rows: Vec<AuditRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, actor, action, resource_type, resource_id,
                   metadata, created_at, prev_hash, entry_hash, residency_region,
                   COALESCE(hmac_signature, '') AS hmac_signature
            FROM audit.events
            WHERE tenant_id = $1
            ORDER BY seq DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(limit)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| HelixError::dependency(format!("audit tenant list: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows.into_iter().rev() {
            out.push(row.into_entry()?);
        }
        Ok(out)
    }
}

impl PgAuditSink {
    /// Rebuild prev/entry hashes from stored bodies (local recovery after format drift).
    pub async fn rehash_chain(&self) -> HelixResult<u64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("audit rehash begin: {e}")))?;

        sqlx::query("SELECT pg_advisory_xact_lock(87201402)")
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("audit rehash lock: {e}")))?;

        sqlx::query("SET LOCAL row_security = off")
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("audit rehash rls off: {e}")))?;

        let rows: Vec<AuditRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, actor, action, resource_type, resource_id,
                   metadata, created_at, prev_hash, entry_hash, residency_region,
                   COALESCE(hmac_signature, '') AS hmac_signature
            FROM audit.events
            ORDER BY seq ASC
            "#,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("audit rehash list: {e}")))?;

        let mut expected_prev = genesis_hash();
        let mut n = 0u64;
        for row in rows {
            let mut entry = row.into_entry()?;
            entry.prev_hash = expected_prev.clone();
            entry.entry_hash = entry.recompute_hash()?;
            entry.hmac_signature = audit_log::sign_entry_hash(&entry.entry_hash);
            sqlx::query(
                r#"
                UPDATE audit.events
                SET prev_hash = $1, entry_hash = $2, hmac_signature = $3
                WHERE id = $4
                "#,
            )
            .bind(&entry.prev_hash)
            .bind(&entry.entry_hash)
            .bind(&entry.hmac_signature)
            .bind(entry.id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("audit rehash update: {e}")))?;
            expected_prev = entry.entry_hash;
            n += 1;
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("audit rehash commit: {e}")))?;
        Ok(n)
    }
}
