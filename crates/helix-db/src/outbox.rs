//! Durable outbox queue — committed together with domain changes so events are
//! never lost between a successful write and downstream dispatch.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxItem {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub idempotency_key: String,
    pub topic: String,
    pub payload: serde_json::Value,
    pub headers: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub attempt_count: i32,
}

#[derive(Clone)]
pub struct OutboxRepo;

impl Default for OutboxRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl OutboxRepo {
    pub fn new() -> Self {
        Self
    }

    /// Enqueue an outbox entry inside an existing transaction. The same
    /// `idempotency_key` within a tenant returns the existing row instead of
    /// creating a duplicate.
    pub async fn enqueue_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        tenant_id: TenantId,
        idempotency_key: impl Into<String>,
        topic: impl Into<String>,
        payload: serde_json::Value,
        headers: serde_json::Value,
    ) -> HelixResult<OutboxItem> {
        let idempotency_key = idempotency_key.into();
        let topic = topic.into();
        let id = Uuid::now_v7();

        let row: Option<OutboxRow> = sqlx::query_as(
            r#"
            INSERT INTO helix_core.outbox (
                id, tenant_id, idempotency_key, topic, payload, headers, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (tenant_id, idempotency_key) DO NOTHING
            RETURNING
                id, tenant_id, idempotency_key, topic, payload, headers,
                created_at, processed_at, error, attempt_count
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(&idempotency_key)
        .bind(&topic)
        .bind(&payload)
        .bind(&headers)
        .bind(Utc::now())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("outbox enqueue: {e}")))?;

        let row = match row {
            Some(r) => r,
            None => sqlx::query_as(
                r#"
                SELECT
                    id, tenant_id, idempotency_key, topic, payload, headers,
                    created_at, processed_at, error, attempt_count
                FROM helix_core.outbox
                WHERE tenant_id = $1 AND idempotency_key = $2
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(&idempotency_key)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| HelixError::dependency(format!("outbox idempotency fetch: {e}")))?,
        };

        Ok(row.into_item())
    }
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: Uuid,
    tenant_id: Uuid,
    idempotency_key: String,
    topic: String,
    payload: serde_json::Value,
    headers: serde_json::Value,
    created_at: DateTime<Utc>,
    processed_at: Option<DateTime<Utc>>,
    error: Option<String>,
    attempt_count: i32,
}

impl OutboxRow {
    fn into_item(self) -> OutboxItem {
        OutboxItem {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            idempotency_key: self.idempotency_key,
            topic: self.topic,
            payload: self.payload,
            headers: self.headers,
            created_at: self.created_at,
            processed_at: self.processed_at,
            error: self.error,
            attempt_count: self.attempt_count,
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
    async fn outbox_commits_and_rolls_back_with_transaction() {
        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "outbox-test", "local", None)
            .await
            .unwrap();

        // Commit path: row is durable.
        let mut tx = pool.begin().await.unwrap();
        let _item = OutboxRepo::new()
            .enqueue_in_tx(
                &mut tx,
                tenant.id,
                "key-1",
                "user.created",
                serde_json::json!({"user_id": "u1"}),
                serde_json::json!({}),
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let row: Option<OutboxRow> = sqlx::query_as(
            "SELECT * FROM helix_core.outbox WHERE tenant_id = $1 AND idempotency_key = $2",
        )
        .bind(tenant.id.as_uuid())
        .bind("key-1")
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(row.is_some());
        assert_eq!(row.unwrap().topic, "user.created");

        // Rollback path: row is gone.
        let mut tx = pool.begin().await.unwrap();
        OutboxRepo::new()
            .enqueue_in_tx(
                &mut tx,
                tenant.id,
                "key-rollback",
                "user.created",
                serde_json::json!({}),
                serde_json::json!({}),
            )
            .await
            .unwrap();
        tx.rollback().await.unwrap();

        let row: Option<OutboxRow> = sqlx::query_as(
            "SELECT * FROM helix_core.outbox WHERE tenant_id = $1 AND idempotency_key = $2",
        )
        .bind(tenant.id.as_uuid())
        .bind("key-rollback")
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(row.is_none());
    }

    #[tokio::test]
    async fn outbox_idempotency_within_tenant() {
        let Some(url) = db_url() else { return };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let tenant = tenants
            .create(TenantId::new(), "outbox-dup", "local", None)
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let a = OutboxRepo::new()
            .enqueue_in_tx(
                &mut tx,
                tenant.id,
                "dup-key",
                "first",
                serde_json::json!({"n": 1}),
                serde_json::json!({}),
            )
            .await
            .unwrap();
        let b = OutboxRepo::new()
            .enqueue_in_tx(
                &mut tx,
                tenant.id,
                "dup-key",
                "second",
                serde_json::json!({"n": 2}),
                serde_json::json!({}),
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(a.id, b.id);
        let row: OutboxRow = sqlx::query_as(
            "SELECT * FROM helix_core.outbox WHERE tenant_id = $1 AND idempotency_key = $2",
        )
        .bind(tenant.id.as_uuid())
        .bind("dup-key")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.topic, "first");
    }
}
