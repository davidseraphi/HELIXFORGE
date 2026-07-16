//! Durable tenant plan assignment.

use async_trait::async_trait;
use billing_client::{plan_by_id, PlanStore, TenantPlan};
use chrono::Utc;
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgPlanStore {
    pool: PgPool,
}

impl PgPlanStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlanStore for PgPlanStore {
    async fn get_plan(&self, tenant_id: TenantId) -> HelixResult<TenantPlan> {
        #[derive(sqlx::FromRow)]
        struct Row {
            plan_id: String,
            updated_at: chrono::DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            "SELECT plan_id, updated_at FROM helix_core.tenant_plans WHERE tenant_id = $1",
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("tenant plan get: {e}")))?;

        Ok(match row {
            Some(r) => TenantPlan {
                tenant_id,
                plan_id: r.plan_id,
                updated_at: r.updated_at.to_rfc3339(),
            },
            None => TenantPlan {
                tenant_id,
                plan_id: "free".into(),
                updated_at: "default".into(),
            },
        })
    }

    async fn set_plan(&self, tenant_id: TenantId, plan_id: &str) -> HelixResult<TenantPlan> {
        if plan_by_id(plan_id).is_none() {
            return Err(HelixError::validation(format!("unknown plan {plan_id}")));
        }
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO helix_core.tenant_plans (tenant_id, plan_id, updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (tenant_id) DO UPDATE SET
                plan_id = EXCLUDED.plan_id,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(plan_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("tenant plan set: {e}")))?;

        Ok(TenantPlan {
            tenant_id,
            plan_id: plan_id.into(),
            updated_at: now.to_rfc3339(),
        })
    }
}
