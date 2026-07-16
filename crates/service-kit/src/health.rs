use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use observability::HealthStatus;
use serde::Serialize;
use shared_core::semantic_state::SemanticState;
use std::collections::BTreeMap;

use crate::context::AppState;

/// Honest state of a single readiness check.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckState {
    Healthy,
    Degraded,
    Unknown,
    NotConfigured,
}

impl CheckState {
    /// A check in `Healthy` or `NotConfigured` (local-only opt-out) does not
    /// block readiness. `Degraded` or `Unknown` block readiness outside local.
    pub fn is_ready(&self, is_local: bool) -> bool {
        matches!(self, Self::Healthy | Self::NotConfigured) || is_local
    }
}

impl From<CheckState> for SemanticState {
    fn from(state: CheckState) -> Self {
        match state {
            CheckState::Healthy => Self::Completed,
            CheckState::Degraded => Self::Failed,
            CheckState::Unknown => Self::Unknown,
            CheckState::NotConfigured => Self::WaitingHuman,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckDetail {
    pub state: CheckState,
    pub detail: String,
}

#[derive(Serialize)]
pub struct ReadyBody {
    pub ready: bool,
    pub service: String,
    pub environment: String,
    pub residency: String,
    pub checks: BTreeMap<String, CheckDetail>,
}

pub fn health_router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/meta", get(meta))
}

async fn healthz(State(state): State<AppState>) -> Json<HealthStatus> {
    // Auth health uses a short-timeout Kratos probe (never blocks liveness long).
    let auth = state.clients.auth.health().await.ok();
    let mut status = HealthStatus::healthy(state.clients.config.service_name.clone());
    if let Some(a) = auth {
        status = status.with_check(
            "auth",
            true,
            format!("{} kratos={}", a.mode, a.kratos_reachable),
        );
    }
    let bus_ok = state.clients.bus.is_connected();
    status = status.with_check(
        "bus",
        bus_ok,
        if bus_ok { "nats" } else { "memory-fallback" },
    );
    // Re-probe Postgres so /healthz does not report stale startup state.
    let db_ok = if let Some(pool) = state.clients.db.as_ref() {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
            .is_ok()
    } else {
        state.clients.db_status.connected
    };
    status = status.with_check("postgres", db_ok, state.clients.db_status.detail.clone());
    if let Some(ref ep) = state.clients.config.otlp_endpoint {
        status = status.with_check(
            "otlp",
            observability::otlp_enabled(),
            format!("{ep} active={}", observability::otlp_enabled()),
        );
    }
    let minio_ok = state.clients.objects.health().await.unwrap_or(false);
    status = status.with_check(
        "minio",
        minio_ok,
        format!(
            "{} bucket={}",
            state.clients.config.minio_endpoint, state.clients.config.minio_bucket
        ),
    );
    Json(status)
}

async fn readyz(State(state): State<AppState>) -> Json<ReadyBody> {
    let is_local = state.clients.config.environment == "local";

    let db_check = db_check(&state, is_local).await;
    let nats_check = nats_check(&state, is_local).await;
    let minio_check = minio_check(&state, is_local).await;
    let auth_check = auth_check(&state, is_local).await;

    let mut checks = BTreeMap::new();
    checks.insert("db".into(), db_check);
    checks.insert("nats".into(), nats_check);
    checks.insert("minio".into(), minio_check);
    checks.insert("auth".into(), auth_check);

    // Outside local mode, every required dependency must be Healthy.
    // Local mode is optimistic (memory fallbacks are acceptable for dev) but
    // still reports honest check labels.
    let ready = is_local || checks.values().all(|c| c.state.is_ready(is_local));

    Json(ReadyBody {
        ready,
        service: state.clients.config.service_name.clone(),
        environment: state.clients.config.environment.clone(),
        residency: state.clients.config.data_residency_region.clone(),
        checks,
    })
}

async fn db_check(state: &AppState, is_local: bool) -> CheckDetail {
    match state.clients.db.as_ref() {
        Some(pool) => match sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
        {
            Ok(_) => CheckDetail {
                state: CheckState::Healthy,
                detail: "connected".into(),
            },
            Err(e) => CheckDetail {
                state: CheckState::Degraded,
                detail: format!("probe failed: {e}"),
            },
        },
        None => CheckDetail {
            state: if is_local {
                CheckState::NotConfigured
            } else {
                CheckState::Unknown
            },
            detail: state.clients.db_status.detail.clone(),
        },
    }
}

async fn nats_check(state: &AppState, is_local: bool) -> CheckDetail {
    if state.clients.bus.is_connected() {
        CheckDetail {
            state: CheckState::Healthy,
            detail: format!("mode={}", state.clients.bus.mode()),
        }
    } else {
        CheckDetail {
            state: if is_local {
                CheckState::NotConfigured
            } else {
                CheckState::Degraded
            },
            detail: format!("mode={}", state.clients.bus.mode()),
        }
    }
}

async fn minio_check(state: &AppState, is_local: bool) -> CheckDetail {
    match state.clients.objects.health().await {
        Ok(true) => CheckDetail {
            state: CheckState::Healthy,
            detail: state.clients.config.minio_endpoint.clone(),
        },
        Ok(false) => CheckDetail {
            state: CheckState::Degraded,
            detail: format!("{} reported unhealthy", state.clients.config.minio_endpoint),
        },
        Err(e) => CheckDetail {
            state: if is_local {
                CheckState::NotConfigured
            } else {
                CheckState::Degraded
            },
            detail: format!("{} unreachable: {e}", state.clients.config.minio_endpoint),
        },
    }
}

async fn auth_check(state: &AppState, is_local: bool) -> CheckDetail {
    match state.clients.auth.health().await {
        Ok(h) if h.kratos_reachable || h.dev_headers_allowed => CheckDetail {
            state: CheckState::Healthy,
            detail: format!("mode={} kratos={}", h.mode, h.kratos_reachable),
        },
        Ok(h) => CheckDetail {
            state: if is_local {
                CheckState::NotConfigured
            } else {
                CheckState::Unknown
            },
            detail: format!("mode={} kratos={}", h.mode, h.kratos_reachable),
        },
        Err(e) => CheckDetail {
            state: if is_local {
                CheckState::NotConfigured
            } else {
                CheckState::Degraded
            },
            detail: format!("probe failed: {e}"),
        },
    }
}

async fn meta(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": state.clients.config.service_name,
        "environment": state.clients.config.environment,
        "residency": state.clients.config.data_residency_region,
        "products": shared_core::PRODUCT_CATALOG.len(),
        "endpoints": state.clients.config.endpoints,
        "enterprise": {
            "rate_limit_rps": state.clients.config.rate_limit_rps,
            "max_body_bytes": state.clients.config.max_body_bytes,
            "enforce_strong_secrets": state.clients.config.enforce_strong_secrets,
            "api_keys": state.clients.api_keys.is_some(),
            "tenant_lifecycle": state.clients.tenants.is_some(),
        },
        "db": {
            "connected": state.clients.db_status.connected,
            "migrated": state.clients.db_status.migrated,
            "detail": state.clients.db_status.detail,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_state_maps_to_semantic_state() {
        assert_eq!(
            SemanticState::from(CheckState::Healthy),
            SemanticState::Completed
        );
        assert_eq!(
            SemanticState::from(CheckState::Degraded),
            SemanticState::Failed
        );
        assert_eq!(
            SemanticState::from(CheckState::Unknown),
            SemanticState::Unknown
        );
        assert_eq!(
            SemanticState::from(CheckState::NotConfigured),
            SemanticState::WaitingHuman
        );
    }
}
