//! Shared workspace repository used by all product APIs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub product_slug: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct WorkspaceRepo {
    pool: PgPool,
}

impl WorkspaceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        tenant_id: TenantId,
        product_slug: &str,
    ) -> HelixResult<Vec<WorkspaceRecord>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            product_slug: String,
            name: String,
            created_at: DateTime<Utc>,
        }

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| HelixError::dependency(format!("workspace list conn: {e}")))?;
        crate::set_tenant_context(&mut conn, tenant_id).await?;

        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, product_slug, name, created_at
            FROM helix_core.workspaces
            WHERE tenant_id = $1 AND product_slug = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(product_slug)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| HelixError::dependency(format!("workspace list: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| WorkspaceRecord {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                product_slug: r.product_slug,
                name: r.name,
                created_at: r.created_at,
            })
            .collect())
    }

    /// List all workspaces for a tenant across every product forge.
    pub async fn list_for_tenant(&self, tenant_id: TenantId) -> HelixResult<Vec<WorkspaceRecord>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            product_slug: String,
            name: String,
            created_at: DateTime<Utc>,
        }

        let mut conn =
            self.pool.acquire().await.map_err(|e| {
                HelixError::dependency(format!("workspace list_for_tenant conn: {e}"))
            })?;
        crate::set_tenant_context(&mut conn, tenant_id).await?;

        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, product_slug, name, created_at
            FROM helix_core.workspaces
            WHERE tenant_id = $1
            ORDER BY product_slug ASC, created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| HelixError::dependency(format!("workspace list_for_tenant: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| WorkspaceRecord {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                product_slug: r.product_slug,
                name: r.name,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create(
        &self,
        tenant_id: TenantId,
        product_slug: &str,
        name: &str,
    ) -> HelixResult<WorkspaceRecord> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("workspace create tx: {e}")))?;
        crate::set_tenant_context(&mut tx, tenant_id).await?;

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO helix_core.workspaces (id, tenant_id, product_slug, name, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(product_slug)
        .bind(name)
        .bind(created_at)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            let msg = e.to_string();
            return Err(if msg.contains("unique") || msg.contains("duplicate") {
                HelixError::conflict(format!("workspace already exists: {name}"))
            } else {
                HelixError::dependency(format!("workspace create: {e}"))
            });
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("workspace create commit: {e}")))?;

        Ok(WorkspaceRecord {
            id,
            tenant_id,
            product_slug: product_slug.into(),
            name: name.into(),
            created_at,
        })
    }

    pub async fn ensure_tenant(
        &self,
        tenant_id: TenantId,
        name: &str,
        region: &str,
    ) -> HelixResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("tenant upsert tx: {e}")))?;
        crate::set_tenant_context(&mut tx, tenant_id).await?;

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO helix_core.tenants (id, name, residency_region)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(region)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            return Err(HelixError::dependency(format!("tenant upsert: {e}")));
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("tenant upsert commit: {e}")))?;
        Ok(())
    }
}
