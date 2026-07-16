//! helix-pulse API — scaffold only (build last after Core + products 1–20).
//!
//! Full cluster / Redis-class engine is intentionally NOT implemented here yet.
//! See projects/helix-pulse/VISION.md and docs/BUILD_ORDER.md.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use service_kit::{
    serve_with_shutdown, ApiError, AppState, ProductApp, ProductService, RequireAuth,
    ServiceBuilder,
};
use shared_core::{ApiResponse, HelixResult};

/// Explicit portfolio gate — cluster work is deferred.
const BUILD_PRIORITY: &str = "last";
const IMPLEMENTATION_PHASE: &str = "p0_scaffold";

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-pulse")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: format!("{}-assistant", product.slug),
            description: "HelixPulse ops assistant (scaffold)".into(),
            system_prompt:
                "You assist with HelixPulse cluster planning. Cluster is not implemented yet."
                    .into(),
            tools: vec![
                "echo".into(),
                "product_catalog".into(),
                "tenant_context".into(),
            ],
            max_steps: 6,
        });
    let addr = builder.config().listen_addr;
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .nest_service("/", domain_routes().with_state(state.clone()));

    serve_with_shutdown(addr, app, "helix-pulse", state.clone()).await
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/domain/status", get(domain_status))
        .route("/v1/pulse/vision", get(vision))
        .route("/v1/pulse/cluster", get(cluster_status))
        .route("/v1/pulse/capabilities", get(capabilities))
}

#[derive(Serialize)]
struct DomainStatus {
    domain: &'static str,
    build_priority: &'static str,
    phase: &'static str,
    cluster_implemented: bool,
    tenant: String,
    durable: bool,
    note: &'static str,
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<DomainStatus>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(DomainStatus {
        domain: "scaffold",
        build_priority: BUILD_PRIORITY,
        phase: IMPLEMENTATION_PHASE,
        cluster_implemented: false,
        tenant: p.tenant_id.to_string(),
        durable: state.clients.db.is_some(),
        note: "HelixPulse is cataloged as product 21 — full cluster after products 1–20",
    })))
}

async fn vision(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "title": "HelixPulse",
        "slug": "helix-pulse",
        "order": 21,
        "port": 8121,
        "build_priority": BUILD_PRIORITY,
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
    // Intentionally empty — full cluster is last-phase work.
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
