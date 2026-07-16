//! HelixNetwork API — professional networking & opportunities (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::NetworkRepo;
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-network")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder.clients().agents.register_agent(agent_framework::AgentSpec {
        name: format!("{}-assistant", product.slug),
        description: format!("{} assistant", product.title),
        system_prompt: format!(
            "You are the {} networking assistant. Help with profiles, connections, and opportunities.",
            product.title
        ),
        tools: vec!["echo".into(), "product_catalog".into()],
        max_steps: 8,
    });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .nest_service("/", domain_routes().with_state(state.clone()));

    let cfg = shared_core::CoreConfig::from_env("helix-network", 8109)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-network", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/profiles", get(list_profiles).post(create_profile))
        .route("/v1/profiles/me", get(my_profile))
        .route("/v1/profiles/{id}", get(get_profile))
        .route(
            "/v1/connections",
            get(list_connections).post(request_connection),
        )
        .route("/v1/connections/{id}/accept", post(accept_connection))
        .route(
            "/v1/opportunities",
            get(list_opportunities).post(create_opportunity),
        )
        .route("/v1/opportunities/{id}", get(get_opportunity))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "ready",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some()
    }))))
}

async fn list_profiles(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = NetworkRepo::new(pool.clone());
        let items = repo.list_profiles(p.tenant_id).await?;
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
struct CreateProfile {
    display_name: String,
    #[serde(default)]
    headline: String,
    #[serde(default)]
    bio: String,
    #[serde(default)]
    skills: serde_json::Value,
    #[serde(default)]
    location: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

async fn create_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateProfile>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.display_name.trim().is_empty() {
        return Err(HelixError::validation("display_name required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable network"))?;
    let repo = NetworkRepo::new(pool.clone());
    let profile = repo
        .create_profile(
            p.tenant_id,
            p.user_id,
            body.display_name.trim(),
            &body.headline,
            &body.bio,
            body.skills,
            &body.location,
            body.metadata,
        )
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "profile.create".into(),
            resource_type: "profile".into(),
            resource_id: profile.id.to_string(),
            metadata: serde_json::json!({"display_name": profile.display_name}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-network",
            "profiles.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(profile))))
}

async fn my_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable network"))?;
    let repo = NetworkRepo::new(pool.clone());
    let profile = repo
        .get_profile_by_user(p.tenant_id, p.user_id)
        .await?
        .ok_or_else(|| HelixError::not_found("no profile for current user"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(profile))))
}

async fn get_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable network"))?;
    let repo = NetworkRepo::new(pool.clone());
    let profile = repo
        .get_profile(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("profile not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(profile))))
}

#[derive(Deserialize)]
struct ConnectionsQuery {
    profile_id: Option<Uuid>,
}

async fn list_connections(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Query(q): Query<ConnectionsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = NetworkRepo::new(pool.clone());
        let items = repo.list_connections(p.tenant_id, q.profile_id).await?;
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
struct RequestConnection {
    to_profile_id: Uuid,
    #[serde(default)]
    message: String,
}

async fn request_connection(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<RequestConnection>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable network"))?;
    let repo = NetworkRepo::new(pool.clone());
    let me = repo
        .get_profile_by_user(p.tenant_id, p.user_id)
        .await?
        .ok_or_else(|| HelixError::validation("create a profile before connecting"))?;
    let conn = repo
        .request_connection(p.tenant_id, me.id, body.to_profile_id, &body.message)
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "connection.request".into(),
            resource_type: "connection".into(),
            resource_id: conn.id.to_string(),
            metadata: serde_json::json!({
                "from": conn.from_profile_id,
                "to": conn.to_profile_id
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-network",
            "connections.requested",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.network.connection.requested",
            &serde_json::json!({"connection_id": conn.id, "to": conn.to_profile_id}),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(conn))))
}

async fn accept_connection(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable network"))?;
    let repo = NetworkRepo::new(pool.clone());
    let me = repo
        .get_profile_by_user(p.tenant_id, p.user_id)
        .await?
        .ok_or_else(|| HelixError::validation("create a profile before accepting"))?;
    let conn = repo.accept_connection(p.tenant_id, id, me.id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "connection.accept".into(),
            resource_type: "connection".into(),
            resource_id: conn.id.to_string(),
            metadata: serde_json::json!({"status": conn.status}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.network.connection.accepted",
            &serde_json::json!({"connection_id": conn.id}),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(conn))))
}

async fn list_opportunities(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = NetworkRepo::new(pool.clone());
        let items = repo.list_opportunities(p.tenant_id).await?;
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
struct CreateOpportunity {
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_kind")]
    kind: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

fn default_kind() -> String {
    "role".into()
}

async fn create_opportunity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateOpportunity>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.title.trim().is_empty() {
        return Err(HelixError::validation("title required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable network"))?;
    let repo = NetworkRepo::new(pool.clone());
    let me = repo
        .get_profile_by_user(p.tenant_id, p.user_id)
        .await?
        .ok_or_else(|| HelixError::validation("create a profile before posting opportunities"))?;
    let opp = repo
        .create_opportunity(
            p.tenant_id,
            me.id,
            body.title.trim(),
            &body.description,
            &body.kind,
            body.metadata,
        )
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "opportunity.create".into(),
            resource_type: "opportunity".into(),
            resource_id: opp.id.to_string(),
            metadata: serde_json::json!({"title": opp.title, "kind": opp.kind}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-network",
            "opportunities.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.network.opportunity.created",
            &serde_json::json!({"opportunity_id": opp.id, "title": opp.title}),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(opp))))
}

async fn get_opportunity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable network"))?;
    let repo = NetworkRepo::new(pool.clone());
    let opp = repo
        .get_opportunity(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("opportunity not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(opp))))
}
