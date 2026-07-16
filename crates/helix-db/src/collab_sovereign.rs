//! Sovereign collab durability: devices, key shares, sealed CRDT, spaces,
//! attachments, residency proofs, federation, recovery ceremonies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::{TenantId, UserId};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceKey {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub device_label: String,
    pub public_key_b64: String,
    pub credential_id: Option<String>,
    pub algorithm: String,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyShare {
    pub id: Uuid,
    pub document_id: Uuid,
    pub device_key_id: Option<Uuid>,
    pub wrapped_dek: String,
    pub share_kind: String,
    pub threshold_n: Option<i32>,
    pub threshold_k: Option<i32>,
    pub shard_index: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabSpace {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub workspace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub classification: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMeta {
    pub id: Uuid,
    pub document_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub object_key: String,
    pub client_sealed: bool,
    pub sha256_hex: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyProof {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub document_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub claimed_region: String,
    pub evidence: serde_json::Value,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationReceipt {
    pub id: Uuid,
    pub remote_deployment: String,
    pub document_id: Option<Uuid>,
    pub direction: String,
    pub payload_hash: String,
    pub signature_b64: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCeremony {
    pub id: Uuid,
    pub document_id: Uuid,
    pub k: i32,
    pub n: i32,
    pub status: String,
    pub meta: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct SovereignCollabRepo {
    pool: PgPool,
}

impl SovereignCollabRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Device keys ──────────────────────────────────────────────

    pub async fn register_device_key(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        device_label: &str,
        public_key_b64: &str,
        credential_id: Option<&str>,
        algorithm: &str,
    ) -> HelixResult<DeviceKey> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.device_keys
                (id, tenant_id, user_id, device_label, public_key_b64, credential_id, algorithm, created_at, last_seen_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(device_label)
        .bind(public_key_b64)
        .bind(credential_id)
        .bind(algorithm)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("device key: {e}")))?;
        Ok(DeviceKey {
            id,
            tenant_id,
            user_id,
            device_label: device_label.into(),
            public_key_b64: public_key_b64.into(),
            credential_id: credential_id.map(str::to_string),
            algorithm: algorithm.into(),
            created_at: now,
            revoked_at: None,
            last_seen_at: Some(now),
        })
    }

    pub async fn list_device_keys(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> HelixResult<Vec<DeviceKey>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            user_id: Uuid,
            device_label: String,
            public_key_b64: String,
            credential_id: Option<String>,
            algorithm: String,
            created_at: DateTime<Utc>,
            revoked_at: Option<DateTime<Utc>>,
            last_seen_at: Option<DateTime<Utc>>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, user_id, device_label, public_key_b64, credential_id, algorithm,
                   created_at, revoked_at, last_seen_at
            FROM collab.device_keys
            WHERE tenant_id = $1 AND user_id = $2 AND revoked_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list device keys: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| DeviceKey {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                user_id: UserId::from_uuid(r.user_id),
                device_label: r.device_label,
                public_key_b64: r.public_key_b64,
                credential_id: r.credential_id,
                algorithm: r.algorithm,
                created_at: r.created_at,
                revoked_at: r.revoked_at,
                last_seen_at: r.last_seen_at,
            })
            .collect())
    }

    pub async fn revoke_device_key(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        id: Uuid,
    ) -> HelixResult<()> {
        let r = sqlx::query(
            r#"
            UPDATE collab.device_keys SET revoked_at = now()
            WHERE tenant_id = $1 AND user_id = $2 AND id = $3 AND revoked_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("revoke device: {e}")))?;
        if r.rows_affected() == 0 {
            return Err(HelixError::not_found("device key"));
        }
        Ok(())
    }

    // ── Key shares ───────────────────────────────────────────────

    pub async fn put_key_share(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        device_key_id: Option<Uuid>,
        wrapped_dek: &str,
        share_kind: &str,
        threshold_n: Option<i32>,
        threshold_k: Option<i32>,
        shard_index: Option<i32>,
    ) -> HelixResult<KeyShare> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.key_shares
                (id, tenant_id, document_id, device_key_id, wrapped_dek, share_kind,
                 threshold_n, threshold_k, shard_index, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(device_key_id)
        .bind(wrapped_dek)
        .bind(share_kind)
        .bind(threshold_n)
        .bind(threshold_k)
        .bind(shard_index)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("key share: {e}")))?;
        Ok(KeyShare {
            id,
            document_id,
            device_key_id,
            wrapped_dek: wrapped_dek.into(),
            share_kind: share_kind.into(),
            threshold_n,
            threshold_k,
            shard_index,
            created_at: now,
        })
    }

    pub async fn list_key_shares(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
    ) -> HelixResult<Vec<KeyShare>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            document_id: Uuid,
            device_key_id: Option<Uuid>,
            wrapped_dek: String,
            share_kind: String,
            threshold_n: Option<i32>,
            threshold_k: Option<i32>,
            shard_index: Option<i32>,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, document_id, device_key_id, wrapped_dek, share_kind,
                   threshold_n, threshold_k, shard_index, created_at
            FROM collab.key_shares
            WHERE tenant_id = $1 AND document_id = $2
            ORDER BY created_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list key shares: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| KeyShare {
                id: r.id,
                document_id: r.document_id,
                device_key_id: r.device_key_id,
                wrapped_dek: r.wrapped_dek,
                share_kind: r.share_kind,
                threshold_n: r.threshold_n,
                threshold_k: r.threshold_k,
                shard_index: r.shard_index,
                created_at: r.created_at,
            })
            .collect())
    }

    // ── Sealed CRDT durable ──────────────────────────────────────

    pub async fn upsert_sealed_crdt(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        sealed_state: &str,
        updated_by: Option<UserId>,
    ) -> HelixResult<()> {
        if !sealed_state.trim_start().starts_with("HC1.") {
            return Err(HelixError::validation("sealed_state must be HC1 envelope"));
        }
        sqlx::query(
            r#"
            INSERT INTO collab.sealed_crdt_state (document_id, tenant_id, sealed_state, updated_at, updated_by)
            VALUES ($1,$2,$3,now(),$4)
            ON CONFLICT (document_id) DO UPDATE
            SET sealed_state = EXCLUDED.sealed_state,
                updated_at = now(),
                updated_by = EXCLUDED.updated_by
            "#,
        )
        .bind(document_id)
        .bind(tenant_id.as_uuid())
        .bind(sealed_state)
        .bind(updated_by.map(|u| u.as_uuid()))
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("sealed crdt upsert: {e}")))?;
        Ok(())
    }

    pub async fn get_sealed_crdt(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
    ) -> HelixResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT sealed_state FROM collab.sealed_crdt_state
            WHERE document_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(document_id)
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("sealed crdt get: {e}")))?;
        Ok(row.map(|r| r.0))
    }

    // ── Classification ───────────────────────────────────────────

    pub async fn set_classification(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        classification: &str,
        sealed_comments: Option<bool>,
    ) -> HelixResult<()> {
        validate_classification(classification)?;
        if let Some(sc) = sealed_comments {
            sqlx::query(
                r#"
                UPDATE collab.documents
                SET classification = $3, sealed_comments = $4, updated_at = now()
                WHERE tenant_id = $1 AND id = $2
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(document_id)
            .bind(classification)
            .bind(sc)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("classification: {e}")))?;
        } else {
            sqlx::query(
                r#"
                UPDATE collab.documents
                SET classification = $3, updated_at = now()
                WHERE tenant_id = $1 AND id = $2
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(document_id)
            .bind(classification)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("classification: {e}")))?;
        }
        Ok(())
    }

    pub async fn get_classification(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
    ) -> HelixResult<(String, bool)> {
        let row: Option<(String, bool)> = sqlx::query_as(
            r#"
            SELECT COALESCE(classification, 'internal'), COALESCE(sealed_comments, false)
            FROM collab.documents WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("get classification: {e}")))?;
        row.ok_or_else(|| HelixError::not_found("document"))
    }

    // ── Spaces ───────────────────────────────────────────────────

    pub async fn create_space(
        &self,
        tenant_id: TenantId,
        workspace_id: Uuid,
        parent_id: Option<Uuid>,
        name: &str,
        classification: &str,
        created_by: Option<UserId>,
    ) -> HelixResult<CollabSpace> {
        validate_classification(classification)?;
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.spaces
                (id, tenant_id, workspace_id, parent_id, name, classification, created_by, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(workspace_id)
        .bind(parent_id)
        .bind(name)
        .bind(classification)
        .bind(created_by.map(|u| u.as_uuid()))
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("create space: {e}")))?;
        Ok(CollabSpace {
            id,
            tenant_id,
            workspace_id,
            parent_id,
            name: name.into(),
            classification: classification.into(),
            created_at: now,
        })
    }

    pub async fn list_spaces(
        &self,
        tenant_id: TenantId,
        workspace_id: Uuid,
    ) -> HelixResult<Vec<CollabSpace>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            workspace_id: Uuid,
            parent_id: Option<Uuid>,
            name: String,
            classification: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, workspace_id, parent_id, name, classification, created_at
            FROM collab.spaces
            WHERE tenant_id = $1 AND workspace_id = $2
            ORDER BY name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list spaces: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CollabSpace {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                workspace_id: r.workspace_id,
                parent_id: r.parent_id,
                name: r.name,
                classification: r.classification,
                created_at: r.created_at,
            })
            .collect())
    }

    // ── MLS durable user state ───────────────────────────────────

    pub async fn upsert_mls_user_blob(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        identity_label: &str,
        blob: &[u8],
        signature_public_b64: &str,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            INSERT INTO collab.mls_identities
                (id, tenant_id, user_id, identity_label, credential_blob, signature_public_b64, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,now())
            ON CONFLICT (tenant_id, user_id, identity_label) DO UPDATE
            SET credential_blob = EXCLUDED.credential_blob,
                signature_public_b64 = EXCLUDED.signature_public_b64
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(identity_label)
        .bind(blob)
        .bind(signature_public_b64)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mls user blob: {e}")))?;
        Ok(())
    }

    // (unique index on tenant_id, user_id, identity_label is in 0023 migration)

    pub async fn get_mls_user_blob(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        identity_label: &str,
    ) -> HelixResult<Option<Vec<u8>>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            r#"
            SELECT credential_blob FROM collab.mls_identities
            WHERE tenant_id = $1 AND user_id = $2 AND identity_label = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(identity_label)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mls user get: {e}")))?;
        Ok(row.map(|r| r.0))
    }

    pub async fn upsert_mls_member_state(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        group_id: &str,
        storage_json: &str,
        leaf_index: Option<i32>,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            INSERT INTO collab.mls_member_state
                (group_id, tenant_id, user_id, storage_json, leaf_index, updated_at)
            VALUES ($1,$2,$3,$4,$5,now())
            ON CONFLICT (group_id, user_id) DO UPDATE
            SET storage_json = EXCLUDED.storage_json,
                leaf_index = EXCLUDED.leaf_index,
                updated_at = now()
            "#,
        )
        .bind(group_id)
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(storage_json)
        .bind(leaf_index)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mls member state: {e}")))?;
        Ok(())
    }

    // ── Attachments ──────────────────────────────────────────────

    pub async fn register_attachment(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        filename: &str,
        content_type: &str,
        size_bytes: i64,
        object_key: &str,
        client_sealed: bool,
        sha256_hex: &str,
        created_by: Option<UserId>,
    ) -> HelixResult<AttachmentMeta> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.attachments
                (id, tenant_id, document_id, filename, content_type, size_bytes,
                 object_key, client_sealed, sha256_hex, created_by, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(filename)
        .bind(content_type)
        .bind(size_bytes)
        .bind(object_key)
        .bind(client_sealed)
        .bind(sha256_hex)
        .bind(created_by.map(|u| u.as_uuid()))
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("attachment: {e}")))?;
        Ok(AttachmentMeta {
            id,
            document_id,
            filename: filename.into(),
            content_type: content_type.into(),
            size_bytes,
            object_key: object_key.into(),
            client_sealed,
            sha256_hex: sha256_hex.into(),
            created_at: now,
        })
    }

    pub async fn get_attachment(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        attachment_id: Uuid,
    ) -> HelixResult<AttachmentMeta> {
        let items = self.list_attachments(tenant_id, document_id).await?;
        items
            .into_iter()
            .find(|a| a.id == attachment_id)
            .ok_or_else(|| HelixError::not_found("attachment"))
    }

    pub async fn delete_attachment(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        attachment_id: Uuid,
    ) -> HelixResult<String> {
        let att = self
            .get_attachment(tenant_id, document_id, attachment_id)
            .await?;
        let r = sqlx::query(
            r#"
            DELETE FROM collab.attachments
            WHERE tenant_id = $1 AND document_id = $2 AND id = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(attachment_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("delete attachment: {e}")))?;
        if r.rows_affected() == 0 {
            return Err(HelixError::not_found("attachment"));
        }
        Ok(att.object_key)
    }

    pub async fn list_attachments(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
    ) -> HelixResult<Vec<AttachmentMeta>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            document_id: Uuid,
            filename: String,
            content_type: String,
            size_bytes: i64,
            object_key: String,
            client_sealed: bool,
            sha256_hex: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, document_id, filename, content_type, size_bytes, object_key,
                   client_sealed, sha256_hex, created_at
            FROM collab.attachments
            WHERE tenant_id = $1 AND document_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list attachments: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| AttachmentMeta {
                id: r.id,
                document_id: r.document_id,
                filename: r.filename,
                content_type: r.content_type,
                size_bytes: r.size_bytes,
                object_key: r.object_key,
                client_sealed: r.client_sealed,
                sha256_hex: r.sha256_hex,
                created_at: r.created_at,
            })
            .collect())
    }

    // ── Residency ────────────────────────────────────────────────

    pub async fn add_residency_proof(
        &self,
        tenant_id: TenantId,
        document_id: Option<Uuid>,
        workspace_id: Option<Uuid>,
        claimed_region: &str,
        evidence: serde_json::Value,
        verified: bool,
    ) -> HelixResult<ResidencyProof> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.residency_proofs
                (id, tenant_id, document_id, workspace_id, claimed_region, evidence, verified, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(workspace_id)
        .bind(claimed_region)
        .bind(&evidence)
        .bind(verified)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("residency proof: {e}")))?;
        Ok(ResidencyProof {
            id,
            tenant_id,
            document_id,
            workspace_id,
            claimed_region: claimed_region.into(),
            evidence,
            verified,
            created_at: now,
        })
    }

    pub async fn list_residency_proofs(
        &self,
        tenant_id: TenantId,
        document_id: Option<Uuid>,
    ) -> HelixResult<Vec<ResidencyProof>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            document_id: Option<Uuid>,
            workspace_id: Option<Uuid>,
            claimed_region: String,
            evidence: serde_json::Value,
            verified: bool,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, document_id, workspace_id, claimed_region, evidence, verified, created_at
            FROM collab.residency_proofs
            WHERE tenant_id = $1 AND ($2::uuid IS NULL OR document_id = $2)
            ORDER BY created_at DESC
            LIMIT 50
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list residency: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| ResidencyProof {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                document_id: r.document_id,
                workspace_id: r.workspace_id,
                claimed_region: r.claimed_region,
                evidence: r.evidence,
                verified: r.verified,
                created_at: r.created_at,
            })
            .collect())
    }

    // ── Federation ───────────────────────────────────────────────

    pub async fn add_federation_receipt(
        &self,
        tenant_id: TenantId,
        remote_deployment: &str,
        document_id: Option<Uuid>,
        direction: &str,
        payload_hash: &str,
        signature_b64: &str,
    ) -> HelixResult<FederationReceipt> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.federation_receipts
                (id, tenant_id, remote_deployment, document_id, direction, payload_hash, signature_b64, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(remote_deployment)
        .bind(document_id)
        .bind(direction)
        .bind(payload_hash)
        .bind(signature_b64)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("federation receipt: {e}")))?;
        Ok(FederationReceipt {
            id,
            remote_deployment: remote_deployment.into(),
            document_id,
            direction: direction.into(),
            payload_hash: payload_hash.into(),
            signature_b64: signature_b64.into(),
            created_at: now,
        })
    }

    // ── Recovery ceremonies ──────────────────────────────────────

    pub async fn open_recovery(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        k: i32,
        n: i32,
        meta: serde_json::Value,
    ) -> HelixResult<RecoveryCeremony> {
        if k < 1 || n < k || n > 16 {
            return Err(HelixError::validation("invalid threshold k/n"));
        }
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.recovery_ceremonies
                (id, tenant_id, document_id, k, n, status, meta, created_at)
            VALUES ($1,$2,$3,$4,$5,'open',$6,$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(k)
        .bind(n)
        .bind(&meta)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("recovery open: {e}")))?;
        Ok(RecoveryCeremony {
            id,
            document_id,
            k,
            n,
            status: "open".into(),
            meta,
            created_at: now,
            completed_at: None,
        })
    }

    pub async fn complete_recovery(
        &self,
        tenant_id: TenantId,
        ceremony_id: Uuid,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            UPDATE collab.recovery_ceremonies
            SET status = 'completed', completed_at = now()
            WHERE tenant_id = $1 AND id = $2 AND status = 'open'
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(ceremony_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("recovery complete: {e}")))?;
        Ok(())
    }
}

pub fn validate_classification(c: &str) -> HelixResult<()> {
    match c {
        "public" | "internal" | "restricted" | "sovereign" => Ok(()),
        _ => Err(HelixError::validation(
            "classification must be public|internal|restricted|sovereign",
        )),
    }
}

/// Policy: classifications that require client-held e2ee (server vault forbidden).
pub fn requires_client_e2ee(classification: &str) -> bool {
    matches!(classification, "restricted" | "sovereign")
}

/// Policy: classifications that forbid cleartext durable storage.
pub fn forbids_cleartext(classification: &str) -> bool {
    matches!(classification, "restricted" | "sovereign")
}
