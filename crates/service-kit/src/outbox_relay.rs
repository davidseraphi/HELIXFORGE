//! Outbox relay — polls `helix_core.outbox` and publishes pending events to
//! NATS so domain commits become downstream messages without dual-write risk.

use chrono::Utc;
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::{Postgres, Transaction};
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::context::HelixCoreClients;

const DEFAULT_BATCH_SIZE: i64 = 100;
const DEFAULT_MAX_ATTEMPTS: i32 = 10;
const DEFAULT_INTERVAL_MS: u64 = 1_000;

#[derive(sqlx::FromRow)]
struct PendingOutboxRow {
    id: Uuid,
    tenant_id: Uuid,
    topic: String,
    payload: serde_json::Value,
    headers: serde_json::Value,
    attempt_count: i32,
}

/// Spawn a background task that relays pending outbox rows to NATS.
/// Returns `None` when Postgres is not available.
pub fn spawn_outbox_relay(clients: HelixCoreClients) -> Option<tokio::task::JoinHandle<()>> {
    let pool = clients.db?;
    let bus = clients.bus;
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(DEFAULT_INTERVAL_MS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            if let Err(e) = process_batch(&pool, &bus).await {
                error!(error = %e, "outbox relay batch failed");
            }
        }
    });
    info!("outbox relay spawned");
    Some(handle)
}

async fn process_batch(pool: &sqlx::PgPool, bus: &nats_client::HelixBus) -> HelixResult<()> {
    let mut tx: Transaction<'_, Postgres> = pool
        .begin()
        .await
        .map_err(|e| HelixError::dependency(format!("outbox relay tx: {e}")))?;

    let rows: Vec<PendingOutboxRow> = sqlx::query_as(
        r#"
        SELECT id, tenant_id, topic, payload, headers, attempt_count
        FROM helix_core.outbox
        WHERE processed_at IS NULL AND attempt_count < $1
        ORDER BY created_at
        LIMIT $2
        FOR UPDATE SKIP LOCKED
        "#,
    )
    .bind(DEFAULT_MAX_ATTEMPTS)
    .bind(DEFAULT_BATCH_SIZE)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| HelixError::dependency(format!("outbox relay fetch: {e}")))?;

    if rows.is_empty() {
        tx.commit().await.ok();
        return Ok(());
    }

    for row in rows {
        match publish_row(bus, &row).await {
            Ok(()) => {
                sqlx::query("UPDATE helix_core.outbox SET processed_at = $1 WHERE id = $2")
                    .bind(Utc::now())
                    .bind(row.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| HelixError::dependency(format!("outbox relay mark: {e}")))?;
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!(
                    outbox_id = %row.id,
                    attempt = row.attempt_count + 1,
                    error = %msg,
                    "outbox relay publish failed"
                );
                sqlx::query(
                    "UPDATE helix_core.outbox SET attempt_count = attempt_count + 1, error = $1 WHERE id = $2",
                )
                .bind(&msg)
                .bind(row.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| HelixError::dependency(format!("outbox relay retry mark: {e}")))?;
            }
        }
    }

    tx.commit()
        .await
        .map_err(|e| HelixError::dependency(format!("outbox relay commit: {e}")))?;
    Ok(())
}

async fn publish_row(bus: &nats_client::HelixBus, row: &PendingOutboxRow) -> HelixResult<()> {
    // Merge outbox headers into the envelope so downstream consumers get
    // provenance without relying on NATS header support in every backend.
    let envelope = serde_json::json!({
        "topic": row.topic,
        "tenant_id": TenantId::from_uuid(row.tenant_id).to_string(),
        "payload": row.payload,
        "headers": row.headers,
    });
    bus.publish(&row.topic, &envelope)
        .await
        .map_err(|e| HelixError::dependency(format!("outbox relay publish to {}: {e}", row.topic)))
}
