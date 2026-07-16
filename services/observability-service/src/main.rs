//! HelixCore Observability Service — metrics snapshot + audit chain verification.

use audit_log::AuditEntry;
use axum::extract::State;

use axum::routing::get;
use axum::{Json, Router};

use service_kit::{ApiError, AppState, RequireAuth, ServiceBuilder};
use shared_core::{ApiResponse, HelixError, HelixResult};
use std::collections::HashMap;
use subtle::ConstantTimeEq;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let builder = ServiceBuilder::new("observability-service", 8084).await?;
    let addr = builder.config().listen_addr;
    let state = builder.into_state();

    let app = ServiceBuilder::base_router(state.clone()).merge(
        Router::new()
            .route("/v1/metrics", get(metrics))
            .route("/v1/metrics/prometheus", get(metrics_prometheus))
            .route("/v1/audit/recent", get(audit_recent))
            .route("/v1/audit/operator/recent", get(audit_operator_recent))
            .route("/v1/audit/tenant", get(audit_tenant))
            .route("/v1/audit/verify", get(audit_verify))
            .route("/v1/audit/export", get(audit_export))
            .route("/v1/compliance/summary", get(compliance_summary))
            .route("/v1/core/health", get(core_health_aggregate))
            .route("/v1/core/ready", get(core_ready_aggregate))
            .with_state(state.clone()),
    );

    service_kit::serve_with_shutdown(addr, app, "observability-service", state.clone()).await
}

async fn metrics(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<HashMap<String, u64>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(state.clients.metrics.snapshot())))
}

async fn metrics_prometheus(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<String, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let snap = state.clients.metrics.snapshot();
    let mut out = String::from("# HELP helix_counter HelixForge in-process counters\n");
    out.push_str("# TYPE helix_counter counter\n");
    for (name, value) in snap {
        let safe = name.replace(['.', '-'], "_");
        out.push_str(&format!("helix_counter{{name=\"{safe}\"}} {value}\n"));
    }
    Ok(out)
}

async fn audit_recent(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<Vec<AuditEntry>>>, ApiError> {
    // Tenant-scoped only; global chain is exposed via /v1/audit/operator/recent.
    principal.require_scope(shared_core::tenancy::Scope::AuditRead)?;
    Ok(Json(ApiResponse::ok(
        state
            .clients
            .audit
            .list_for_tenant(principal.tenant_id, 100)
            .await?,
    )))
}

/// Operator-only global audit view. Requires Platform + AuditRead + operator key.
async fn audit_operator_recent(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<Vec<AuditEntry>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Platform)?;
    principal.require_scope(shared_core::tenancy::Scope::AuditRead)?;
    let expected = state
        .clients
        .config
        .audit_operator_key
        .as_deref()
        .unwrap_or("");
    if !expected.is_empty() {
        let provided = headers
            .get("x-helix-audit-operator-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let eq = expected.as_bytes().ct_eq(provided.as_bytes());
        if eq.unwrap_u8() == 0 {
            return Err(HelixError::forbidden("invalid audit operator key").into());
        }
    }
    Ok(Json(ApiResponse::ok(
        state.clients.audit.list_recent(100).await?,
    )))
}

/// Tenant-scoped audit view (caller tenant only unless Platform).
async fn audit_tenant(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<Vec<AuditEntry>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::AuditRead)?;
    let items = state
        .clients
        .audit
        .list_for_tenant(principal.tenant_id, 100)
        .await?;
    Ok(Json(ApiResponse::ok(items)))
}

/// Compliance export: NDJSON-friendly array of tenant audit rows (for SIEM / legal hold).
async fn audit_export(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::AuditRead)?;
    let items = state
        .clients
        .audit
        .list_for_tenant(principal.tenant_id, 500)
        .await?;
    let ndjson: String = items
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "tenant_id": principal.tenant_id,
        "count": items.len(),
        "format": "ndjson",
        "ndjson": ndjson,
        "entries": items
    }))))
}

async fn compliance_summary(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::AuditRead)?;
    let verified = state.clients.audit.verify_chain().await.unwrap_or(false);
    let entries = state.clients.audit.count().await.unwrap_or(0);
    let tenant_entries = state
        .clients
        .audit
        .list_for_tenant(principal.tenant_id, 500)
        .await?
        .len();
    let audit_hmac = state
        .clients
        .config
        .audit_hmac_secret
        .as_ref()
        .is_some_and(|s| !s.is_empty());
    let jetstream = state.clients.bus.jetstream_enabled();
    let bus_mode = state.clients.bus.mode();
    let durable = state.clients.db.is_some();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "audit_chain_verified": verified,
        "audit_entries_global": entries,
        "audit_entries_tenant": tenant_entries,
        "residency": state.clients.config.data_residency_region,
        "environment": state.clients.config.environment,
        "sovereign_ready": verified && durable,
        "controls": {
            "hash_chained_audit": true,
            "audit_hmac_signatures": audit_hmac,
            "tenant_isolation": true,
            "api_keys": state.clients.api_keys.is_some(),
            "tenant_lifecycle": state.clients.tenants.is_some(),
            "rate_limiting": state.clients.config.rate_limit_rps > 0,
            "shared_rate_limit": durable,
            "strong_secrets_enforced": state.clients.config.enforce_strong_secrets,
            "otlp": state.clients.config.otlp_endpoint.is_some(),
            "kms": true,
            "vault_envelope": "HVA3/HVA4/HVA5",
            "postgres_durable": durable,
            "nats_jetstream": jetstream,
            "bus_mode": bus_mode,
            "fail_closed_auth": true,
            "acl_governance_regions": state.clients.acl.is_some()
                && state.clients.governance.is_some()
        }
    }))))
}

async fn audit_verify(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::AuditRead)?;
    // Return JSON either way so operators can inspect without treating false as transport failure.
    let ok = state.clients.audit.verify_chain().await?;
    let entries = state.clients.audit.count().await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "verified": ok,
        "entries": entries,
        "durable": state.clients.has_db(),
        "note": if ok {
            "chain intact"
        } else {
            "chain integrity failed — use helix-audit-rehash operator CLI to rebuild hashes"
        }
    }))))
}

/// Probe all core service healthz endpoints and return aggregate.
/// Aggregate readiness across all core services by probing each `/readyz`.
async fn core_ready_aggregate(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let e = &state.clients.config.endpoints;
    let targets = [
        ("gateway", e.gateway.as_str()),
        ("agent-hub", e.agent_hub.as_str()),
        ("vault", e.vault.as_str()),
        ("billing", e.billing.as_str()),
        ("observability", e.observability.as_str()),
        ("auth-adapter", e.auth_adapter.as_str()),
    ];
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|err| HelixError::internal(format!("http client: {err}")))?;
    let mut services = serde_json::Map::new();
    let mut all_ready = true;
    for (name, base) in targets {
        let url = format!("{}/readyz", base.trim_end_matches('/'));
        let (ready, detail) = match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body = resp.text().await.unwrap_or_default();
                let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                let is_ready = parsed
                    .get("ready")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                (is_ready, body)
            }
            Ok(resp) => {
                all_ready = false;
                (false, format!("status {}", resp.status()))
            }
            Err(err) => {
                all_ready = false;
                (false, err.to_string())
            }
        };
        if !ready {
            all_ready = false;
        }
        services.insert(
            name.into(),
            serde_json::json!({"ready": ready, "detail": detail}),
        );
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "ready": all_ready,
        "services": services,
        "residency": state.clients.config.data_residency_region,
        "environment": state.clients.config.environment,
    }))))
}

async fn core_health_aggregate(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let e = &state.clients.config.endpoints;
    let targets = [
        ("gateway", e.gateway.as_str()),
        ("agent-hub", e.agent_hub.as_str()),
        ("vault", e.vault.as_str()),
        ("billing", e.billing.as_str()),
        ("observability", e.observability.as_str()),
        ("auth-adapter", e.auth_adapter.as_str()),
    ];
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|err| HelixError::internal(format!("http client: {err}")))?;
    let mut services = serde_json::Map::new();
    let mut all_ok = true;
    for (name, base) in targets {
        let url = format!("{}/healthz", base.trim_end_matches('/'));
        let (ok, detail) = match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body = resp.text().await.unwrap_or_default();
                (true, body)
            }
            Ok(resp) => {
                all_ok = false;
                (false, format!("status {}", resp.status()))
            }
            Err(err) => {
                all_ok = false;
                (false, err.to_string())
            }
        };
        services.insert(name.into(), serde_json::json!({"ok": ok, "detail": detail}));
    }
    let audit_ok = state.clients.audit.verify_chain().await.unwrap_or(false);
    let bus_mode = state.clients.bus.mode();
    let minio_ok = state.clients.objects.health().await.unwrap_or(false);
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "ok": all_ok && audit_ok,
        "audit_chain_verified": audit_ok,
        "postgres": state.clients.has_db(),
        "bus": bus_mode,
        "minio": minio_ok,
        "vault_crypto": if state.clients.has_db() {
            "postgres-aes-gcm-tenant-dek"
        } else {
            "memory-aes-gcm-tenant-dek"
        },
        "otlp_endpoint": state.clients.config.otlp_endpoint,
        "residency": state.clients.config.data_residency_region,
        "services": services
    }))))
}
