//! Tenant lifecycle repository (enterprise multi-tenancy).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    Active,
    Suspended,
    Pending,
}

impl TenantStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Pending => "pending",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "suspended" => Self::Suspended,
            "pending" => Self::Pending,
            // Fail closed: unknown/corrupted status is not active.
            _ => Self::Suspended,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRecord {
    pub id: TenantId,
    pub name: String,
    pub residency_region: String,
    pub status: TenantStatus,
    pub plan_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub suspended_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Clone)]
pub struct TenantRepo {
    pool: PgPool,
}

impl TenantRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: TenantId) -> HelixResult<Option<TenantRecord>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            name: String,
            residency_region: String,
            status: String,
            plan_id: Option<String>,
            created_at: DateTime<Utc>,
            suspended_at: Option<DateTime<Utc>>,
            metadata: serde_json::Value,
        }
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| HelixError::dependency(format!("tenant get conn: {e}")))?;
        crate::set_tenant_context(&mut conn, id).await?;

        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, name, residency_region,
                   status,
                   plan_id, created_at, suspended_at,
                   COALESCE(metadata, '{}'::jsonb) AS metadata
            FROM helix_core.tenants
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| HelixError::dependency(format!("tenant get: {e}")))?;
        Ok(row.map(|r| TenantRecord {
            id: TenantId::from_uuid(r.id),
            name: r.name,
            residency_region: r.residency_region,
            status: TenantStatus::parse(&r.status),
            plan_id: r.plan_id,
            created_at: r.created_at,
            suspended_at: r.suspended_at,
            metadata: r.metadata,
        }))
    }

    pub async fn list(&self, limit: i64) -> HelixResult<Vec<TenantRecord>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            name: String,
            residency_region: String,
            status: String,
            plan_id: Option<String>,
            created_at: DateTime<Utc>,
            suspended_at: Option<DateTime<Utc>>,
            metadata: serde_json::Value,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, name, residency_region,
                   COALESCE(status, 'active') AS status,
                   plan_id, created_at, suspended_at,
                   COALESCE(metadata, '{}'::jsonb) AS metadata
            FROM helix_core.tenants
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("tenant list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| TenantRecord {
                id: TenantId::from_uuid(r.id),
                name: r.name,
                residency_region: r.residency_region,
                status: TenantStatus::parse(&r.status),
                plan_id: r.plan_id,
                created_at: r.created_at,
                suspended_at: r.suspended_at,
                metadata: r.metadata,
            })
            .collect())
    }

    pub async fn create(
        &self,
        id: TenantId,
        name: &str,
        region: &str,
        plan_id: Option<&str>,
    ) -> HelixResult<TenantRecord> {
        let now = Utc::now();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("tenant create tx: {e}")))?;
        crate::set_tenant_context(&mut tx, id).await?;

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO helix_core.tenants
                (id, name, residency_region, status, plan_id, created_at, metadata)
            VALUES ($1,$2,$3,'active',$4,$5,'{}'::jsonb)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                residency_region = EXCLUDED.residency_region
            "#,
        )
        .bind(id.as_uuid())
        .bind(name)
        .bind(region)
        .bind(plan_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            return Err(HelixError::dependency(format!("tenant create: {e}")));
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("tenant create commit: {e}")))?;
        self.get(id)
            .await?
            .ok_or_else(|| HelixError::internal("tenant create race"))
    }

    pub async fn set_status(
        &self,
        id: TenantId,
        status: TenantStatus,
    ) -> HelixResult<TenantRecord> {
        let suspended_at = if status == TenantStatus::Suspended {
            Some(Utc::now())
        } else {
            None
        };
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("tenant status tx: {e}")))?;
        crate::set_tenant_context(&mut tx, id).await?;

        if let Err(e) = sqlx::query(
            r#"
            UPDATE helix_core.tenants
            SET status = $2, suspended_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(status.as_str())
        .bind(suspended_at)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            return Err(HelixError::dependency(format!("tenant status: {e}")));
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("tenant status commit: {e}")))?;
        self.get(id)
            .await?
            .ok_or_else(|| HelixError::not_found(format!("tenant {id}")))
    }

    pub async fn is_active(&self, id: TenantId) -> HelixResult<bool> {
        match self.get(id).await? {
            Some(t) => Ok(t.status == TenantStatus::Active),
            // Fail closed: unknown tenants are not active (Kimi P0).
            // Local bootstrap: ensure_tenant / create_tenant before durable use.
            None => Ok(false),
        }
    }

    /// Fast, direct active check that does not rely on permissive parsing.
    pub async fn is_active_sql(&self, id: TenantId) -> HelixResult<bool> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| HelixError::dependency(format!("tenant active check conn: {e}")))?;
        crate::set_tenant_context(&mut conn, id).await?;

        let active: Option<bool> = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM helix_core.tenants
                WHERE id = $1 AND status = 'active'
            )
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| HelixError::dependency(format!("tenant active check: {e}")))?;
        Ok(active.unwrap_or(false))
    }
}
