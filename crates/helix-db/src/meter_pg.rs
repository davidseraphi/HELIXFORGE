//! Postgres-backed usage metering.

use async_trait::async_trait;
use billing_client::{MeterEvent, Metering, UsageSummary};
use shared_core::ids::TenantId;
use shared_core::time::UtcTimestamp;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgMetering {
    pool: PgPool,
}

impl PgMetering {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Metering for PgMetering {
    async fn record(&self, event: MeterEvent) -> HelixResult<()> {
        if event.quantity < 0.0 {
            return Err(HelixError::validation("quantity must be >= 0"));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("meter record tx: {e}")))?;
        crate::set_tenant_context(&mut tx, event.tenant_id).await?;

        // Idempotent insert when key present (migration 0017).
        let result = sqlx::query(
            r#"
            INSERT INTO helix_core.meter_events
                (id, tenant_id, product, metric, quantity, unit, dimensions, occurred_at, idempotency_key)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(event.id)
        .bind(event.tenant_id.as_uuid())
        .bind(&event.product)
        .bind(&event.metric)
        .bind(event.quantity)
        .bind(&event.unit)
        .bind(&event.dimensions)
        .bind(event.occurred_at.inner())
        .bind(event.idempotency_key.as_deref())
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => {}
            Err(e) => {
                // If unique idempotency index hits without ON CONFLICT target match, treat as ok.
                let msg = e.to_string();
                if !msg.contains("meter_events_idempotency") && !msg.contains("duplicate key") {
                    let _ = tx.rollback().await;
                    return Err(HelixError::dependency(format!("meter insert: {e}")));
                }
            }
        };

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("meter record commit: {e}")))?;
        Ok(())
    }

    async fn summarize(
        &self,
        tenant_id: TenantId,
        product: &str,
    ) -> HelixResult<Vec<UsageSummary>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            metric: String,
            total: f64,
            unit: String,
        }

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| HelixError::dependency(format!("meter summarize conn: {e}")))?;
        crate::set_tenant_context(&mut conn, tenant_id).await?;

        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT metric, SUM(quantity)::float8 AS total, MIN(unit) AS unit
            FROM helix_core.meter_events
            WHERE tenant_id = $1 AND product = $2
            GROUP BY metric
            ORDER BY metric
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(product)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| HelixError::dependency(format!("meter summarize: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| UsageSummary {
                tenant_id,
                product: product.into(),
                metric: r.metric,
                total: r.total,
                unit: r.unit,
            })
            .collect())
    }
}

/// Helper used by integration tests.
pub async fn list_recent_meters(
    pool: &PgPool,
    tenant_id: TenantId,
    limit: i64,
) -> HelixResult<Vec<MeterEvent>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        tenant_id: Uuid,
        product: String,
        metric: String,
        quantity: f64,
        unit: String,
        dimensions: serde_json::Value,
        occurred_at: chrono::DateTime<chrono::Utc>,
    }

    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| HelixError::dependency(format!("meter list conn: {e}")))?;
    crate::set_tenant_context(&mut conn, tenant_id).await?;

    let rows: Vec<Row> = sqlx::query_as(
        r#"
        SELECT id, tenant_id, product, metric, quantity, unit, dimensions, occurred_at
        FROM helix_core.meter_events
        WHERE tenant_id = $1
        ORDER BY occurred_at DESC
        LIMIT $2
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(limit)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| HelixError::dependency(format!("meter list: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|r| MeterEvent {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            product: r.product,
            metric: r.metric,
            quantity: r.quantity,
            unit: r.unit,
            dimensions: r.dimensions,
            occurred_at: UtcTimestamp(r.occurred_at),
            idempotency_key: None,
        })
        .collect())
}
