//! MinIO/S3 object refs bound to vault secret names (metadata in Postgres).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultObjectRef {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub object_key: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct VaultObjectStore {
    pool: PgPool,
}

impl VaultObjectStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Register a blob ref. Does not upload bytes — caller or MinIO client does that.
    pub async fn put_ref(
        &self,
        tenant_id: TenantId,
        name: &str,
        object_key: &str,
        content_type: &str,
        size_bytes: i64,
    ) -> HelixResult<VaultObjectRef> {
        if name.is_empty() || name.len() > 128 {
            return Err(HelixError::validation("object name length 1..=128"));
        }
        if object_key.is_empty() {
            return Err(HelixError::validation("object_key required"));
        }
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO helix_core.vault_objects
                (id, tenant_id, name, object_key, content_type, size_bytes, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            ON CONFLICT (tenant_id, name) DO UPDATE SET
                object_key = EXCLUDED.object_key,
                content_type = EXCLUDED.content_type,
                size_bytes = EXCLUDED.size_bytes,
                created_at = EXCLUDED.created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(object_key)
        .bind(content_type)
        .bind(size_bytes)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vault object put: {e}")))?;

        // Re-read so ON CONFLICT returns stable id/key
        self.get(tenant_id, name)
            .await?
            .ok_or_else(|| HelixError::internal("vault object put race"))
    }

    pub async fn get(
        &self,
        tenant_id: TenantId,
        name: &str,
    ) -> HelixResult<Option<VaultObjectRef>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            object_key: String,
            content_type: String,
            size_bytes: i64,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, object_key, content_type, size_bytes, created_at
            FROM helix_core.vault_objects
            WHERE tenant_id = $1 AND name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vault object get: {e}")))?;

        Ok(row.map(|r| VaultObjectRef {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            name: r.name,
            object_key: r.object_key,
            content_type: r.content_type,
            size_bytes: r.size_bytes,
            created_at: r.created_at,
        }))
    }

    pub async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<VaultObjectRef>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            object_key: String,
            content_type: String,
            size_bytes: i64,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, object_key, content_type, size_bytes, created_at
            FROM helix_core.vault_objects
            WHERE tenant_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vault object list: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| VaultObjectRef {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                name: r.name,
                object_key: r.object_key,
                content_type: r.content_type,
                size_bytes: r.size_bytes,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn delete(&self, tenant_id: TenantId, name: &str) -> HelixResult<()> {
        let res =
            sqlx::query("DELETE FROM helix_core.vault_objects WHERE tenant_id = $1 AND name = $2")
                .bind(tenant_id.as_uuid())
                .bind(name)
                .execute(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("vault object delete: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("object {name}")));
        }
        Ok(())
    }

    /// Canonical object key for a tenant-scoped blob (uuid only — S3-safe).
    pub fn suggest_key(tenant_id: TenantId, name: &str) -> String {
        format!("tenants/{}/vault/{}", tenant_id.as_uuid(), name)
    }
}
