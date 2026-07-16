//! Retention policies, legal holds, purpose bindings, recovery bin, and policy
//! exceptions.

use crate::audit_pg::{PgAuditSink, TransactionalAuditSink};
use audit_log::AuditEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::tenancy::{Actor, Principal};
use shared_core::{HelixError, HelixResult};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub retain_days: i32,
    pub action_on_expiry: String,
    pub purpose: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalHold {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub resource_type: String,
    pub resource_id: String,
    pub reason: String,
    pub placed_by: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub released_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurposeBinding {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub resource_type: String,
    pub resource_id: String,
    pub purpose: String,
    pub legal_basis: String,
    pub subject_ref: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDecision {
    pub allowed: bool,
    pub reasons: Vec<String>,
    pub legal_hold: bool,
    pub retention_blocks: bool,
}

#[derive(Clone)]
pub struct GovernanceRepo {
    pool: PgPool,
}

impl GovernanceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn set_retention(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: Option<&str>,
        retain_days: i32,
        action_on_expiry: &str,
        purpose: Option<&str>,
    ) -> HelixResult<RetentionPolicy> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO helix_core.retention_policies
                (id, tenant_id, resource_type, resource_id, retain_days, action_on_expiry, purpose, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .bind(retain_days)
        .bind(action_on_expiry)
        .bind(purpose)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("retention set: {e}")))?;

        Ok(RetentionPolicy {
            id,
            tenant_id,
            resource_type: resource_type.into(),
            resource_id: resource_id.map(str::to_string),
            retain_days,
            action_on_expiry: action_on_expiry.into(),
            purpose: purpose.map(str::to_string),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list_retention(
        &self,
        tenant_id: TenantId,
        resource_type: Option<&str>,
    ) -> HelixResult<Vec<RetentionPolicy>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            resource_type: String,
            resource_id: Option<String>,
            retain_days: i32,
            action_on_expiry: String,
            purpose: Option<String>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = if let Some(rt) = resource_type {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, resource_type, resource_id, retain_days, action_on_expiry,
                       purpose, created_at, updated_at
                FROM helix_core.retention_policies
                WHERE tenant_id = $1 AND resource_type = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(rt)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, resource_type, resource_id, retain_days, action_on_expiry,
                       purpose, created_at, updated_at
                FROM helix_core.retention_policies
                WHERE tenant_id = $1
                ORDER BY created_at DESC
                LIMIT 200
                "#,
            )
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("retention list: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| RetentionPolicy {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                resource_type: r.resource_type,
                resource_id: r.resource_id,
                retain_days: r.retain_days,
                action_on_expiry: r.action_on_expiry,
                purpose: r.purpose,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn place_hold(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
        reason: &str,
        placed_by: &str,
    ) -> HelixResult<LegalHold> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO helix_core.legal_holds
                (id, tenant_id, resource_type, resource_id, reason, placed_by, active, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,true,$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .bind(reason)
        .bind(placed_by)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("legal hold: {e}")))?;

        Ok(LegalHold {
            id,
            tenant_id,
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            reason: reason.into(),
            placed_by: placed_by.into(),
            active: true,
            created_at: now,
            released_at: None,
        })
    }

    pub async fn release_hold(&self, tenant_id: TenantId, hold_id: Uuid) -> HelixResult<()> {
        let res = sqlx::query(
            r#"
            UPDATE helix_core.legal_holds
            SET active = false, released_at = now()
            WHERE tenant_id = $1 AND id = $2 AND active = true
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(hold_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("release hold: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("legal hold {hold_id}")));
        }
        Ok(())
    }

    pub async fn has_active_hold(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
    ) -> HelixResult<bool> {
        let n: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM helix_core.legal_holds
            WHERE tenant_id = $1 AND resource_type = $2 AND resource_id = $3 AND active = true
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("hold check: {e}")))?;
        Ok(n > 0)
    }

    pub async fn bind_purpose(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
        purpose: &str,
        legal_basis: &str,
        subject_ref: Option<&str>,
    ) -> HelixResult<PurposeBinding> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO helix_core.purpose_bindings
                (id, tenant_id, resource_type, resource_id, purpose, legal_basis, subject_ref, active, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,true,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .bind(purpose)
        .bind(legal_basis)
        .bind(subject_ref)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("purpose bind: {e}")))?;

        Ok(PurposeBinding {
            id,
            tenant_id,
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            purpose: purpose.into(),
            legal_basis: legal_basis.into(),
            subject_ref: subject_ref.map(str::to_string),
            active: true,
            created_at: now,
            expires_at: None,
        })
    }

    pub async fn purposes_for(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
    ) -> HelixResult<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT purpose FROM helix_core.purpose_bindings
            WHERE tenant_id = $1 AND resource_type = $2 AND resource_id = $3
              AND active = true
              AND (expires_at IS NULL OR expires_at > now())
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("purpose list: {e}")))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// True if no purpose bindings exist OR requested purpose is bound.
    pub async fn purpose_allows(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
        purpose: &str,
    ) -> HelixResult<bool> {
        let purposes = self
            .purposes_for(tenant_id, resource_type, resource_id)
            .await?;
        if purposes.is_empty() {
            return Ok(true);
        }
        Ok(purposes.iter().any(|p| p == purpose))
    }

    /// Gate for destructive ops (delete / anonymize).
    pub async fn can_delete(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
    ) -> HelixResult<DeleteDecision> {
        let mut reasons = Vec::new();
        let hold = self
            .has_active_hold(tenant_id, resource_type, resource_id)
            .await?;
        if hold {
            reasons.push("active legal hold".into());
        }
        // Retention: if any policy action is not delete and retain_days > 0, block soft block
        // (hard delete blocked unless action_on_expiry is delete AND age exceeded — simplified:
        // block delete when retain_days > 0 for type or resource)
        let policies = self.list_retention(tenant_id, Some(resource_type)).await?;
        let retention_blocks = policies.iter().any(|p| {
            let matches = p
                .resource_id
                .as_deref()
                .map(|id| id == resource_id)
                .unwrap_or(true);
            matches && p.retain_days > 0 && p.action_on_expiry != "delete"
        });
        if retention_blocks {
            reasons.push("retention policy requires review/anonymize, not free delete".into());
        }
        Ok(DeleteDecision {
            allowed: !hold && !retention_blocks,
            reasons,
            legal_hold: hold,
            retention_blocks,
        })
    }

    // ------------------------------------------------------------------
    // Recovery bin
    // ------------------------------------------------------------------

    pub async fn soft_delete_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        tenant_id: TenantId,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        deleted_by: impl Into<String>,
        reason: Option<impl Into<String>>,
        original_table: Option<impl Into<String>>,
        original_payload: serde_json::Value,
    ) -> HelixResult<RecoveryBinEntry> {
        let resource_type = resource_type.into();
        let resource_id = resource_id.into();
        let deleted_by = deleted_by.into();
        let reason = reason.map(Into::into);
        let original_table = original_table.map(Into::into);
        let id = Uuid::now_v7();
        let deleted_at = Utc::now();
        let retain_until = deleted_at + chrono::Duration::days(30);

        let row: RecoveryBinRow = sqlx::query_as(
            r#"
            INSERT INTO helix_core.recovery_bin (
                id, tenant_id, resource_type, resource_id, original_table,
                original_payload, deleted_by, deleted_at, retain_until, reason
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id, tenant_id, resource_type, resource_id, original_table,
                original_payload, deleted_by, deleted_at, retain_until, restored_at,
                permanently_deleted_at, reason, seq
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(&resource_type)
        .bind(&resource_id)
        .bind(&original_table)
        .bind(&original_payload)
        .bind(&deleted_by)
        .bind(deleted_at)
        .bind(retain_until)
        .bind(&reason)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin insert: {e}")))?;

        Ok(row.into_entry())
    }

    pub async fn get_bin_entry(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<RecoveryBinEntry>> {
        let row: Option<RecoveryBinRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, resource_type, resource_id, original_table,
                original_payload, deleted_by, deleted_at, retain_until, restored_at,
                permanently_deleted_at, reason, seq
            FROM helix_core.recovery_bin
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin get: {e}")))?;

        Ok(row.map(|r| r.into_entry()))
    }

    pub async fn list_bin_for_tenant(
        &self,
        tenant_id: TenantId,
        limit: i64,
    ) -> HelixResult<Vec<RecoveryBinEntry>> {
        let rows: Vec<RecoveryBinRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, resource_type, resource_id, original_table,
                original_payload, deleted_by, deleted_at, retain_until, restored_at,
                permanently_deleted_at, reason, seq
            FROM helix_core.recovery_bin
            WHERE tenant_id = $1
            ORDER BY deleted_at DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin list: {e}")))?;

        Ok(rows.into_iter().map(|r| r.into_entry()).collect())
    }

    pub async fn list_expired(&self, limit: i64) -> HelixResult<Vec<RecoveryBinEntry>> {
        let rows: Vec<RecoveryBinRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, resource_type, resource_id, original_table,
                original_payload, deleted_by, deleted_at, retain_until, restored_at,
                permanently_deleted_at, reason, seq
            FROM helix_core.recovery_bin
            WHERE retain_until < now()
              AND restored_at IS NULL
              AND permanently_deleted_at IS NULL
            ORDER BY retain_until ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin expired list: {e}")))?;

        Ok(rows.into_iter().map(|r| r.into_entry()).collect())
    }

    /// Restore a recovery-bin entry. Returns the `(resource_type, resource_id)`
    /// so the caller can reactivate the row in the original table inside the
    /// same transaction.
    pub async fn restore_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<(String, String)> {
        let row: Option<RecoveryBinRestoreRow> = sqlx::query_as(
            r#"
            UPDATE helix_core.recovery_bin
            SET restored_at = now()
            WHERE tenant_id = $1 AND id = $2
              AND restored_at IS NULL
              AND permanently_deleted_at IS NULL
            RETURNING resource_type, resource_id
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin restore: {e}")))?;

        row.map(|r| (r.resource_type, r.resource_id))
            .ok_or_else(|| HelixError::not_found(format!("recovery bin entry {id}")))
    }

    /// Mark a recovery-bin entry as permanently deleted. Returns the resource
    /// identifiers so the caller can remove the original row inside the same
    /// transaction. This should only be called after audit authorization.
    pub async fn permanently_delete_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<(String, String)> {
        let row: Option<RecoveryBinRestoreRow> = sqlx::query_as(
            r#"
            UPDATE helix_core.recovery_bin
            SET permanently_deleted_at = now()
            WHERE tenant_id = $1 AND id = $2
              AND permanently_deleted_at IS NULL
            RETURNING resource_type, resource_id
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin permanent delete: {e}")))?;

        row.map(|r| (r.resource_type, r.resource_id))
            .ok_or_else(|| HelixError::not_found(format!("recovery bin entry {id}")))
    }

    /// Authority-gated permanent delete. Requires an explicit `admin` or `platform`
    /// principal, a non-empty recorded reason, and writes a hash-chained audit event.
    /// Callers still must drop the underlying resource row in the same transaction
    /// using the returned `(resource_type, resource_id)`.
    pub async fn permanently_delete(
        &self,
        principal: &Principal,
        audit: &PgAuditSink,
        tenant_id: TenantId,
        id: Uuid,
        reason: &str,
    ) -> HelixResult<(String, String)> {
        if !principal.can_permanently_delete() {
            return Err(HelixError::forbidden(
                "permanent delete requires admin or platform authority",
            ));
        }
        if reason.trim().is_empty() {
            return Err(HelixError::validation(
                "permanent delete requires a recorded reason",
            ));
        }

        let actor = Actor::User {
            user_id: principal.user_id,
            tenant_id: principal.tenant_id,
        };

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("begin permanent delete tx: {e}")))?;
        let (resource_type, resource_id) = self
            .permanently_delete_in_tx(&mut tx, tenant_id, id)
            .await?;

        let event = AuditEvent {
            tenant_id: Some(tenant_id),
            actor,
            action: "resource.permanently_deleted".into(),
            resource_type: resource_type.clone(),
            resource_id: resource_id.clone(),
            metadata: serde_json::json!({
                "bin_entry_id": id,
                "reason": reason,
                "principal_scopes": principal.scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            }),
            residency_region: principal.residency_region.clone(),
        };
        audit.append_in_tx(&mut tx, event).await?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("commit permanent delete: {e}")))?;

        Ok((resource_type, resource_id))
    }

    // ------------------------------------------------------------------
    // Policy exceptions
    // ------------------------------------------------------------------

    pub async fn create_policy_exception(
        &self,
        tenant_id: TenantId,
        resource_type: Option<impl Into<String>>,
        resource_id: Option<impl Into<String>>,
        policy_kind: impl Into<String>,
        justification: impl Into<String>,
        approved_by: impl Into<String>,
        expires_at: DateTime<Utc>,
    ) -> HelixResult<PolicyException> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let resource_type = resource_type.map(Into::into);
        let resource_id = resource_id.map(Into::into);
        let policy_kind = policy_kind.into();
        let justification = justification.into();
        let approved_by = approved_by.into();

        let row: PolicyExceptionRow = sqlx::query_as(
            r#"
            INSERT INTO helix_core.policy_exceptions (
                id, tenant_id, resource_type, resource_id, policy_kind,
                justification, approved_by, starts_at, expires_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id, tenant_id, resource_type, resource_id, policy_kind,
                justification, approved_by, starts_at, expires_at, revoked_at,
                revoked_by, created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(&resource_type)
        .bind(&resource_id)
        .bind(&policy_kind)
        .bind(&justification)
        .bind(&approved_by)
        .bind(now)
        .bind(expires_at)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("policy exception create: {e}")))?;

        Ok(row.into_exception())
    }

    pub async fn revoke_policy_exception(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        revoked_by: impl Into<String>,
    ) -> HelixResult<bool> {
        let revoked_by = revoked_by.into();
        let n = sqlx::query(
            r#"
            UPDATE helix_core.policy_exceptions
            SET revoked_at = now(), revoked_by = $3
            WHERE tenant_id = $1 AND id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .bind(&revoked_by)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("policy exception revoke: {e}")))?
        .rows_affected();

        Ok(n > 0)
    }

    pub async fn is_exception_active(
        &self,
        tenant_id: TenantId,
        policy_kind: &str,
    ) -> HelixResult<bool> {
        let n: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM helix_core.policy_exceptions
            WHERE tenant_id = $1 AND policy_kind = $2
              AND revoked_at IS NULL
              AND starts_at <= now()
              AND expires_at > now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(policy_kind)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("policy exception check: {e}")))?;

        Ok(n > 0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryBinEntry {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub resource_type: String,
    pub resource_id: String,
    pub original_table: Option<String>,
    pub original_payload: serde_json::Value,
    pub deleted_by: String,
    pub deleted_at: DateTime<Utc>,
    pub retain_until: DateTime<Utc>,
    pub restored_at: Option<DateTime<Utc>>,
    pub permanently_deleted_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
    pub seq: i64,
}

#[derive(sqlx::FromRow)]
struct RecoveryBinRow {
    id: Uuid,
    tenant_id: Uuid,
    resource_type: String,
    resource_id: String,
    original_table: Option<String>,
    original_payload: serde_json::Value,
    deleted_by: String,
    deleted_at: DateTime<Utc>,
    retain_until: DateTime<Utc>,
    restored_at: Option<DateTime<Utc>>,
    permanently_deleted_at: Option<DateTime<Utc>>,
    reason: Option<String>,
    seq: i64,
}

impl RecoveryBinRow {
    fn into_entry(self) -> RecoveryBinEntry {
        RecoveryBinEntry {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            original_table: self.original_table,
            original_payload: self.original_payload,
            deleted_by: self.deleted_by,
            deleted_at: self.deleted_at,
            retain_until: self.retain_until,
            restored_at: self.restored_at,
            permanently_deleted_at: self.permanently_deleted_at,
            reason: self.reason,
            seq: self.seq,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RecoveryBinRestoreRow {
    resource_type: String,
    resource_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyException {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub policy_kind: String,
    pub justification: String,
    pub approved_by: String,
    pub starts_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct PolicyExceptionRow {
    id: Uuid,
    tenant_id: Uuid,
    resource_type: Option<String>,
    resource_id: Option<String>,
    policy_kind: String,
    justification: String,
    approved_by: String,
    starts_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    revoked_by: Option<String>,
    created_at: DateTime<Utc>,
}

impl PolicyExceptionRow {
    fn into_exception(self) -> PolicyException {
        PolicyException {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            policy_kind: self.policy_kind,
            justification: self.justification,
            approved_by: self.approved_by,
            starts_at: self.starts_at,
            expires_at: self.expires_at,
            revoked_at: self.revoked_at,
            revoked_by: self.revoked_by,
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenants::TenantRepo;

    fn db_url() -> Option<String> {
        std::env::var("DATABASE_URL").ok()
    }

    #[tokio::test]
    async fn recovery_bin_soft_delete_and_restore() {
        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "recovery-test", "local", None)
            .await
            .unwrap();
        let governance = GovernanceRepo::new(pool.clone());

        // Commit path: bin entry is durable.
        let mut tx = pool.begin().await.unwrap();
        let entry = governance
            .soft_delete_in_tx(
                &mut tx,
                tenant.id,
                "workspace",
                "ws-1",
                "user-a",
                Some("no longer needed"),
                Some("helix_core.workspaces"),
                serde_json::json!({"name": "old workspace"}),
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let fetched = governance
            .get_bin_entry(tenant.id, entry.id)
            .await
            .unwrap()
            .expect("bin entry should exist");
        assert_eq!(fetched.resource_type, "workspace");
        assert_eq!(fetched.resource_id, "ws-1");
        assert!(fetched.restored_at.is_none());
        assert!(fetched.retain_until > fetched.deleted_at);

        // Restore returns the resource identifiers and marks the bin entry.
        let mut tx = pool.begin().await.unwrap();
        let (rt, rid) = governance
            .restore_in_tx(&mut tx, tenant.id, entry.id)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(rt, "workspace");
        assert_eq!(rid, "ws-1");

        let restored = governance
            .get_bin_entry(tenant.id, entry.id)
            .await
            .unwrap()
            .unwrap();
        assert!(restored.restored_at.is_some());
    }

    #[tokio::test]
    async fn recovery_bin_rolls_back_with_transaction() {
        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "recovery-rollback", "local", None)
            .await
            .unwrap();
        let governance = GovernanceRepo::new(pool.clone());

        let mut tx = pool.begin().await.unwrap();
        let entry = governance
            .soft_delete_in_tx(
                &mut tx,
                tenant.id,
                "workspace",
                "ws-rollback",
                "user-a",
                None::<&str>,
                None::<&str>,
                serde_json::json!({}),
            )
            .await
            .unwrap();
        tx.rollback().await.unwrap();

        assert!(governance
            .get_bin_entry(tenant.id, entry.id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn recovery_bin_expired_entries_listed() {
        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "recovery-expired", "local", None)
            .await
            .unwrap();
        let governance = GovernanceRepo::new(pool.clone());

        let mut tx = pool.begin().await.unwrap();
        let entry = governance
            .soft_delete_in_tx(
                &mut tx,
                tenant.id,
                "file",
                "file-1",
                "user-a",
                None::<&str>,
                None::<&str>,
                serde_json::json!({}),
            )
            .await
            .unwrap();
        // Force the entry to be expired.
        sqlx::query("UPDATE helix_core.recovery_bin SET retain_until = now() - interval '1 day' WHERE id = $1")
            .bind(entry.id)
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let expired = governance.list_expired(100).await.unwrap();
        assert!(expired.iter().any(|e| e.id == entry.id));
    }

    #[tokio::test]
    async fn policy_exception_lifecycle() {
        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "exception-test", "local", None)
            .await
            .unwrap();
        let governance = GovernanceRepo::new(pool.clone());

        let exception = governance
            .create_policy_exception(
                tenant.id,
                Some("workspace"),
                Some("ws-1"),
                "retention.shorten",
                "legal request",
                "admin-1",
                Utc::now() + chrono::Duration::hours(1),
            )
            .await
            .unwrap();

        assert!(governance
            .is_exception_active(tenant.id, "retention.shorten")
            .await
            .unwrap());

        assert!(governance
            .revoke_policy_exception(tenant.id, exception.id, "admin-2")
            .await
            .unwrap());

        assert!(!governance
            .is_exception_active(tenant.id, "retention.shorten")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn permanent_delete_requires_authority_reason_and_audit() {
        use shared_core::ids::UserId;
        use shared_core::tenancy::Scope;

        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "perm-delete", "local", None)
            .await
            .unwrap();
        let governance = GovernanceRepo::new(pool.clone());
        let audit = PgAuditSink::new(pool.clone());

        // Create a recovery-bin entry.
        let mut tx = pool.begin().await.unwrap();
        let entry = governance
            .soft_delete_in_tx(
                &mut tx,
                tenant.id,
                "workspace",
                "ws-del",
                "user-a",
                None::<&str>,
                None::<&str>,
                serde_json::json!({"name": "to delete"}),
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let make_principal = |scopes: Vec<Scope>| Principal {
            user_id: UserId::new(),
            tenant_id: tenant.id,
            org_id: None,
            scopes,
            session_id: None,
            residency_region: "local".into(),
        };

        // Non-admin principal is forbidden.
        let user = make_principal(vec![Scope::Read, Scope::Write]);
        let err = governance
            .permanently_delete(&user, &audit, tenant.id, entry.id, "legal hold lifted")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Forbidden"));

        // Admin principal without a reason is rejected.
        let admin = make_principal(vec![Scope::Admin]);
        let err = governance
            .permanently_delete(&admin, &audit, tenant.id, entry.id, "   ")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("recorded reason"));

        // Admin principal with a reason succeeds and emits an audit event.
        let before: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit.events WHERE tenant_id = $1 AND action = 'resource.permanently_deleted'"
        )
        .bind(tenant.id.as_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();

        governance
            .permanently_delete(&admin, &audit, tenant.id, entry.id, "legal hold lifted")
            .await
            .unwrap();

        let after: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit.events WHERE tenant_id = $1 AND action = 'resource.permanently_deleted'"
        )
        .bind(tenant.id.as_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(after, before + 1);

        let marked = governance
            .get_bin_entry(tenant.id, entry.id)
            .await
            .unwrap()
            .unwrap();
        assert!(marked.permanently_deleted_at.is_some());
    }
}
