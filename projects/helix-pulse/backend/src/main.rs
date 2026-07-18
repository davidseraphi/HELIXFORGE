//! helix-pulse API — durable monitors + incidents via helix_db.
//!
//! The Redis-class cluster engine remains deferred; see
//! projects/helix-pulse/VISION.md and docs/BUILD_ORDER.md.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{DbPool, IncidentUpdate, MonitorUpdate, PulseRepo, PulseSummaryRow};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-pulse")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: format!("{}-assistant", product.slug),
            description: format!("{} assistant", product.title),
            system_prompt: format!(
                "You are the {} ops assistant. Help track monitors and incidents.",
                product.title
            ),
            tools: vec!["echo".into(), "product_catalog".into()],
            max_steps: 8,
        });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-pulse", 8121)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-pulse", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/monitors", get(list_monitors).post(create_monitor))
        .route("/v1/monitors/{id}", get(get_monitor).patch(update_monitor))
        .route("/v1/monitors/{id}/activate", post(activate_monitor))
        .route("/v1/monitors/{id}/pause", post(pause_monitor))
        .route("/v1/monitors/{id}/resume", post(resume_monitor))
        .route("/v1/monitors/{id}/delete", post(delete_monitor))
        .route("/v1/monitors/{id}/restore", post(restore_monitor))
        .route(
            "/v1/monitors/{id}/incidents",
            get(list_incidents).post(create_incident),
        )
        .route(
            "/v1/monitors/{id}/incidents/{incident_id}",
            axum::routing::patch(update_incident),
        )
        .route(
            "/v1/monitors/{id}/incidents/{incident_id}/acknowledge",
            post(acknowledge_incident),
        )
        .route(
            "/v1/monitors/{id}/incidents/{incident_id}/resolve",
            post(resolve_incident),
        )
        .route(
            "/v1/monitors/{id}/incidents/{incident_id}/delete",
            post(delete_incident),
        )
        .route(
            "/v1/monitors/{id}/incidents/{incident_id}/restore",
            post(restore_incident),
        )
        .route("/v1/reports/pulse-summary", get(pulse_summary))
        .route("/v1/domain/status", get(domain_status))
        .route("/v1/pulse/vision", get(vision))
        .route("/v1/pulse/cluster", get(cluster_status))
        .route("/v1/pulse/capabilities", get(capabilities))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-pulse",
        "phase": "wave2_w21",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "monitors": true,
            "incidents": true,
            "monitor_lifecycle": true,
            "incident_lifecycle": true,
            "pause_guards": true,
            "pulse_summary": true,
            "audit": true,
            "metering": true,
            "nats": true
        }
    }))))
}

fn require_pool(state: &AppState) -> Result<DbPool, ApiError> {
    state
        .clients
        .db
        .as_ref()
        .cloned()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable pulse").into())
}

async fn audit(
    state: &AppState,
    p: &Principal,
    action: &str,
    resource_type: &str,
    resource_id: Uuid,
    metadata: serde_json::Value,
) -> Result<(), ApiError> {
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.to_string(),
            metadata,
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(())
}

async fn meter(
    state: &AppState,
    p: &Principal,
    metric: &str,
    metadata: serde_json::Value,
) -> Result<(), ApiError> {
    state
        .clients
        .billing
        .record_usage(p.tenant_id, "helix-pulse", metric, 1.0, "count", metadata)
        .await?;
    Ok(())
}

async fn publish_event(state: &AppState, topic: &str, payload: serde_json::Value) {
    state.clients.bus.publish(topic, &payload).await.ok();
}

// --- Monitors ---

async fn list_monitors(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = PulseRepo::new(pool.clone());
        let items = repo.list_monitors(p.tenant_id).await?;
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "durable": true,
            "items": items
        }))));
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": false,
        "items": []
    }))))
}

#[derive(Deserialize)]
struct CreateMonitor {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

async fn create_monitor(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateMonitor>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let item = repo
        .create_monitor(
            p.tenant_id,
            body.name.trim(),
            &body.description,
            body.metadata,
        )
        .await?;
    audit(
        &state,
        &p,
        "monitor.create",
        "monitor",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "monitors.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_monitor(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let item = repo
        .get_monitor(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("monitor not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateMonitor {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_monitor(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMonitor>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_monitor(
            p.tenant_id,
            id,
            MonitorUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "monitor.update",
        "monitor",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "monitors.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for monitor lifecycle transitions (activate/pause/resume/delete/restore).
async fn monitor_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let item = match action {
        "activate" => repo.activate_monitor(p.tenant_id, id).await?,
        "pause" => repo.pause_monitor(p.tenant_id, id).await?,
        "resume" => repo.resume_monitor(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_monitor(p.tenant_id, id).await?,
        "restore" => repo.restore_monitor(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown monitor action").into()),
    };
    audit(
        &state,
        &p,
        &format!("monitor.{action}"),
        "monitor",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "monitors.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.pulse.monitor.lifecycle",
        serde_json::json!({
            "monitor_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn activate_monitor(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    monitor_transition(state, p, id, "activate").await
}

async fn pause_monitor(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    monitor_transition(state, p, id, "pause").await
}

async fn resume_monitor(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    monitor_transition(state, p, id, "resume").await
}

async fn delete_monitor(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    monitor_transition(state, p, id, "delete").await
}

async fn restore_monitor(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    monitor_transition(state, p, id, "restore").await
}

// --- Incidents ---

async fn list_incidents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let items = repo.list_incidents(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "parent_id": id,
        "items": items
    }))))
}

#[derive(Deserialize)]
struct CreateIncident {
    title: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

async fn create_incident(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateIncident>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.title.trim().is_empty() {
        return Err(HelixError::validation("title required").into());
    }
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let item = repo
        .create_incident(
            p.tenant_id,
            id,
            body.title.trim(),
            &body.body,
            body.metadata,
        )
        .await?;
    audit(
        &state,
        &p,
        "incident.create",
        "incident",
        item.id,
        serde_json::json!({"monitor_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "incidents.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    publish_event(
        &state,
        "helix.pulse.incident.opened",
        serde_json::json!({"monitor_id": id, "incident_id": item.id}),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateIncident {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_incident(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, incident_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateIncident>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_incident(
            p.tenant_id,
            id,
            incident_id,
            IncidentUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "incident.update",
        "incident",
        item.id,
        serde_json::json!({"monitor_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "incidents.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for incident lifecycle transitions (acknowledge/resolve/delete/restore).
async fn incident_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    incident_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let item = match action {
        "acknowledge" => {
            repo.acknowledge_incident(p.tenant_id, id, incident_id)
                .await?
        }
        "resolve" => repo.resolve_incident(p.tenant_id, id, incident_id).await?,
        "delete" => {
            repo.soft_delete_incident(p.tenant_id, id, incident_id)
                .await?
        }
        "restore" => repo.restore_incident(p.tenant_id, id, incident_id).await?,
        _ => return Err(HelixError::validation("unknown incident action").into()),
    };
    audit(
        &state,
        &p,
        &format!("incident.{action}"),
        "incident",
        item.id,
        serde_json::json!({"monitor_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "incidents.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.pulse.incident.lifecycle",
        serde_json::json!({
            "monitor_id": id,
            "incident_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn acknowledge_incident(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    incident_transition(state, p, id, incident_id, "acknowledge").await
}

async fn resolve_incident(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    incident_transition(state, p, id, incident_id, "resolve").await
}

async fn delete_incident(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    incident_transition(state, p, id, incident_id, "delete").await
}

async fn restore_incident(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    incident_transition(state, p, id, incident_id, "restore").await
}

// --- Reports ---

async fn pulse_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<PulseSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = PulseRepo::new(pool);
    let rows = repo.get_pulse_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

// --- Informational scaffold endpoints (cluster engine deferred) ---

async fn vision(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "title": "HelixPulse",
        "slug": "helix-pulse",
        "order": 21,
        "port": 8121,
        "build_priority": "last",
        "north_star": "Sovereign multi-tenant distributed memory plane (modern Redis-class)",
        "not": "Day-one Redis clone or Core dependency",
        "phases": ["p0_scaffold", "p1_embedded", "p2_protocol_subset", "p3_cluster", "p4_multi_region"],
        "use_until_then": {
            "rate_limit": "service_kit in-process / NATS KV",
            "messaging": "NATS JetStream",
            "secrets": "vault-service",
            "durable": "postgres"
        },
        "docs": [
            "projects/helix-pulse/VISION.md",
            "projects/helix-pulse/docs/BUILD_ORDER.md"
        ]
    }))))
}

async fn cluster_status(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "implemented": false,
        "phase_required": "p3_cluster",
        "nodes": [],
        "shard_map": null,
        "message": "Cluster engine deferred. See BUILD_ORDER.md — build after products 1–20."
    }))))
}

async fn capabilities(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "kv": false,
        "ttl": false,
        "incr": false,
        "pubsub": false,
        "streams": false,
        "resp_gateway": false,
        "cluster": false,
        "multi_region": false,
        "tenant_isolation": "planned",
        "envelope_crypto": "planned",
        "audit": "planned"
    }))))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_incident_status, next_monitor_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18121");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-pulse").expect("helix-pulse product known");
        let builder = ServiceBuilder::new(product.slug, product.default_port)
            .await
            .expect("ServiceBuilder requires Postgres + optional NATS/MinIO");
        let state = builder.into_state();

        // Integration tests run against a freshly-migrated, empty Postgres.
        // The dev principal's tenant is deterministic but not seeded, so create
        // it here before any audited operation tries to reference it.
        let local_dev_tenant = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        if let Some(tenants) = state.clients.tenants.as_ref() {
            let _ = tenants
                .create(local_dev_tenant, "local-dev", "local", None)
                .await;
        }

        (state, guard)
    }

    #[test]
    fn monitor_transitions_are_guarded() {
        assert_eq!(next_monitor_status("draft", "activate").unwrap(), "active");
        assert_eq!(next_monitor_status("active", "pause").unwrap(), "paused");
        assert_eq!(next_monitor_status("paused", "resume").unwrap(), "active");
        assert!(next_monitor_status("active", "activate").is_err());
        assert!(next_monitor_status("draft", "pause").is_err());
        assert!(next_monitor_status("active", "resume").is_err());
        assert!(next_monitor_status("deleted", "activate").is_err());
    }

    #[test]
    fn incident_transitions_are_guarded() {
        assert_eq!(
            next_incident_status("open", "acknowledge").unwrap(),
            "acknowledged"
        );
        assert_eq!(next_incident_status("open", "resolve").unwrap(), "resolved");
        assert_eq!(
            next_incident_status("acknowledged", "resolve").unwrap(),
            "resolved"
        );
        assert!(next_incident_status("acknowledged", "acknowledge").is_err());
        assert!(next_incident_status("resolved", "acknowledge").is_err());
        assert!(next_incident_status("resolved", "resolve").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn monitor_and_incident_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = PulseRepo::new(pool.clone());

        let monitor = repo
            .create_monitor(
                tenant_id,
                "gateway-8080",
                "core gateway health",
                serde_json::json!({}),
            )
            .await
            .expect("create monitor");
        assert_eq!(monitor.status, "draft");

        let active = repo
            .activate_monitor(tenant_id, monitor.id)
            .await
            .expect("activate");
        assert_eq!(active.status, "active");
        assert!(active.activated_at.is_some());

        // Pause guard: an open incident blocks pausing.
        let incident = repo
            .create_incident(
                tenant_id,
                monitor.id,
                "Latency spike",
                "p95 over 800ms",
                serde_json::json!({}),
            )
            .await
            .expect("create incident");
        assert_eq!(incident.status, "open");

        let blocked = repo.pause_monitor(tenant_id, monitor.id).await;
        assert!(blocked.is_err(), "pause blocked by open incident");

        let acknowledged = repo
            .acknowledge_incident(tenant_id, monitor.id, incident.id)
            .await
            .expect("acknowledge");
        assert_eq!(acknowledged.status, "acknowledged");
        assert!(acknowledged.acknowledged_at.is_some());

        // Pause works once no incident is open; resume returns to active.
        let paused = repo
            .pause_monitor(tenant_id, monitor.id)
            .await
            .expect("pause");
        assert_eq!(paused.status, "paused");
        assert!(paused.paused_at.is_some());
        let resumed = repo
            .resume_monitor(tenant_id, monitor.id)
            .await
            .expect("resume");
        assert_eq!(resumed.status, "active");
        assert!(resumed.paused_at.is_none());

        // Resolve the incident.
        let resolved = repo
            .resolve_incident(tenant_id, monitor.id, incident.id)
            .await
            .expect("resolve");
        assert_eq!(resolved.status, "resolved");
        assert!(resolved.resolved_at.is_some());

        // Summary reflects the incident.
        let summary = repo.get_pulse_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == monitor.id).unwrap();
        assert_eq!(row.total_incidents, 1);
        assert_eq!(row.open_incidents, 0);
        assert_eq!(row.resolved_incidents, 1);

        // Updates.
        let renamed = repo
            .update_monitor(
                tenant_id,
                monitor.id,
                MonitorUpdate {
                    description: Some("core gateway + auth health".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update monitor");
        assert_eq!(renamed.description, "core gateway + auth health");

        let incident_updated = repo
            .update_incident(
                tenant_id,
                monitor.id,
                incident.id,
                IncidentUpdate {
                    body: Some("p95 over 800ms for 4m".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update incident");
        assert_eq!(incident_updated.body, "p95 over 800ms for 4m");

        // Incident delete hides it; restore returns the pre-delete status.
        repo.soft_delete_incident(tenant_id, monitor.id, incident.id)
            .await
            .expect("delete incident");
        let incidents = repo
            .list_incidents(tenant_id, monitor.id)
            .await
            .expect("list incidents after delete");
        assert!(incidents.iter().all(|i| i.id != incident.id));
        let restored_incident = repo
            .restore_incident(tenant_id, monitor.id, incident.id)
            .await
            .expect("restore incident");
        assert_eq!(restored_incident.status, "resolved");

        // Monitor delete hides it; restore returns the pre-delete status.
        repo.soft_delete_monitor(tenant_id, monitor.id)
            .await
            .expect("delete monitor");
        let monitors = repo
            .list_monitors(tenant_id)
            .await
            .expect("list monitors after delete");
        assert!(monitors.iter().all(|m| m.id != monitor.id));
        let restored = repo
            .restore_monitor(tenant_id, monitor.id)
            .await
            .expect("restore monitor");
        assert_eq!(restored.status, "active");
        assert!(restored.deleted_at.is_none());
    }
}
