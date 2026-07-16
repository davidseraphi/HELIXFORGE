//! Durable marketplace payment intents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    RequiresAction,
    Paid,
    Failed,
    Cancelled,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::RequiresAction => "requires_action",
            Self::Paid => "paid",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "paid" => Self::Paid,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            "requires_action" => Self::RequiresAction,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub plan_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: PaymentStatus,
    pub provider: String,
    pub provider_ref: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PaymentStore {
    pool: PgPool,
}

impl PaymentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, intent: &PaymentIntent) -> HelixResult<PaymentIntent> {
        sqlx::query(
            r#"
            INSERT INTO helix_core.payment_intents
                (id, tenant_id, plan_id, amount_cents, currency, status, provider, provider_ref, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            "#,
        )
        .bind(intent.id)
        .bind(intent.tenant_id.as_uuid())
        .bind(&intent.plan_id)
        .bind(intent.amount_cents)
        .bind(&intent.currency)
        .bind(intent.status.as_str())
        .bind(&intent.provider)
        .bind(&intent.provider_ref)
        .bind(&intent.metadata)
        .bind(intent.created_at)
        .bind(intent.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("payment create: {e}")))?;
        Ok(intent.clone())
    }

    pub async fn get(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<PaymentIntent>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            plan_id: String,
            amount_cents: i64,
            currency: String,
            status: String,
            provider: String,
            provider_ref: Option<String>,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, plan_id, amount_cents, currency, status, provider,
                   provider_ref, metadata, created_at, updated_at
            FROM helix_core.payment_intents
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("payment get: {e}")))?;
        Ok(row.map(|r| PaymentIntent {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            plan_id: r.plan_id,
            amount_cents: r.amount_cents,
            currency: r.currency,
            status: PaymentStatus::parse(&r.status),
            provider: r.provider,
            provider_ref: r.provider_ref,
            metadata: r.metadata,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn list(&self, tenant_id: TenantId, limit: i64) -> HelixResult<Vec<PaymentIntent>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            plan_id: String,
            amount_cents: i64,
            currency: String,
            status: String,
            provider: String,
            provider_ref: Option<String>,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, plan_id, amount_cents, currency, status, provider,
                   provider_ref, metadata, created_at, updated_at
            FROM helix_core.payment_intents
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("payment list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| PaymentIntent {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                plan_id: r.plan_id,
                amount_cents: r.amount_cents,
                currency: r.currency,
                status: PaymentStatus::parse(&r.status),
                provider: r.provider,
                provider_ref: r.provider_ref,
                metadata: r.metadata,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn update_status(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        status: PaymentStatus,
        provider_ref: Option<String>,
    ) -> HelixResult<PaymentIntent> {
        let now = Utc::now();
        let res = sqlx::query(
            r#"
            UPDATE helix_core.payment_intents
            SET status = $3, provider_ref = COALESCE($4, provider_ref), updated_at = $5
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .bind(status.as_str())
        .bind(&provider_ref)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("payment update: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("payment {id}")));
        }
        self.get(tenant_id, id)
            .await?
            .ok_or_else(|| HelixError::not_found(format!("payment {id}")))
    }
}
