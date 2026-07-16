//! Helix product durable store — `vita` schema (thin widen slice).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Study {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cohort {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct VitaRepo {
    pool: PgPool,
}

impl VitaRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<Study>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            description: String,
            status: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, status, metadata, created_at
            FROM vita.studies
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Study {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                name: r.name,
                description: r.description,
                status: r.status,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Study> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO vita.studies
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita create: {e}")))?;
        Ok(Study {
            id,
            tenant_id,
            name: name.into(),
            description: description.into(),
            status: "draft".into(),
            metadata,
            created_at,
        })
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Study>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            description: String,
            status: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, status, metadata, created_at
            FROM vita.studies
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita get: {e}")))?;
        Ok(row.map(|r| Study {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            name: r.name,
            description: r.description,
            status: r.status,
            metadata: r.metadata,
            created_at: r.created_at,
        }))
    }

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Cohort>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            parent_id: Uuid,
            title: String,
            body: String,
            status: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at
            FROM vita.cohorts
            WHERE tenant_id = $1 AND parent_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita list children: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Cohort {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                parent_id: r.parent_id,
                title: r.title,
                body: r.body,
                status: r.status,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Cohort> {
        let _parent = self
            .get_parent(tenant_id, parent_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO vita.cohorts
                (id, tenant_id, parent_id, title, body, status, metadata, created_at)
            VALUES ($1,$2,$3,$4,$5,'open',$6,$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .bind(title)
        .bind(body)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vita create child: {e}")))?;
        Ok(Cohort {
            id,
            tenant_id,
            parent_id,
            title: title.into(),
            body: body.into(),
            status: "open".into(),
            metadata,
            created_at,
        })
    }
}
