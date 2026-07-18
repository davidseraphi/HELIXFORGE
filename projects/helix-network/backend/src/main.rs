//! HelixNetwork API — professional networking & opportunities (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{DbPool, NetworkRepo, NetworkSummaryRow, OpportunityUpdate, ProfileUpdate};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
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
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-network", 8109)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-network", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/profiles", get(list_profiles).post(create_profile))
        .route("/v1/profiles/me", get(my_profile))
        .route("/v1/profiles/{id}", get(get_profile).patch(update_profile))
        .route("/v1/profiles/{id}/deactivate", post(deactivate_profile))
        .route("/v1/profiles/{id}/reactivate", post(reactivate_profile))
        .route("/v1/profiles/{id}/delete", post(delete_profile))
        .route("/v1/profiles/{id}/restore", post(restore_profile))
        .route(
            "/v1/connections",
            get(list_connections).post(request_connection),
        )
        .route("/v1/connections/{id}/accept", post(accept_connection))
        .route("/v1/connections/{id}/decline", post(decline_connection))
        .route("/v1/connections/{id}/remove", post(remove_connection))
        .route("/v1/connections/{id}/block", post(block_connection))
        .route(
            "/v1/opportunities",
            get(list_opportunities).post(create_opportunity),
        )
        .route(
            "/v1/opportunities/{id}",
            get(get_opportunity).patch(update_opportunity),
        )
        .route("/v1/opportunities/{id}/close", post(close_opportunity))
        .route("/v1/opportunities/{id}/reopen", post(reopen_opportunity))
        .route("/v1/opportunities/{id}/delete", post(delete_opportunity))
        .route("/v1/opportunities/{id}/restore", post(restore_opportunity))
        .route("/v1/reports/network-summary", get(network_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-network",
        "phase": "wave2_w9",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "profiles": true,
            "connections": true,
            "opportunities": true,
            "profile_lifecycle": true,
            "connection_lifecycle": true,
            "opportunity_lifecycle": true,
            "blocking": true,
            "network_summary": true,
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
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable network").into())
}

async fn require_my_profile(
    state: &AppState,
    p: &Principal,
) -> Result<helix_db::Profile, ApiError> {
    let pool = require_pool(state)?;
    let repo = NetworkRepo::new(pool);
    repo.get_profile_by_user(p.tenant_id, p.user_id)
        .await?
        .ok_or_else(|| HelixError::validation("create a profile before this action").into())
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
        .record_usage(p.tenant_id, "helix-network", metric, 1.0, "count", metadata)
        .await?;
    Ok(())
}

async fn publish(state: &AppState, topic: &str, payload: serde_json::Value) {
    state.clients.bus.publish(topic, &payload).await.ok();
}

// --- Profiles ---

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
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
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
    audit(
        &state,
        &p,
        "profile.create",
        "profile",
        profile.id,
        serde_json::json!({"display_name": profile.display_name}),
    )
    .await?;
    meter(&state, &p, "profiles.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(profile))))
}

async fn my_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
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
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let profile = repo
        .get_profile(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("profile not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(profile))))
}

#[derive(Deserialize, Default)]
struct UpdateProfile {
    display_name: Option<String>,
    headline: Option<String>,
    bio: Option<String>,
    skills: Option<serde_json::Value>,
    location: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProfile>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let display_name = body
        .display_name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let profile = repo
        .update_profile(
            p.tenant_id,
            id,
            p.user_id,
            ProfileUpdate {
                display_name,
                headline: body.headline,
                bio: body.bio,
                skills: body.skills,
                location: body.location,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "profile.update",
        "profile",
        profile.id,
        serde_json::json!({"display_name": profile.display_name}),
    )
    .await?;
    meter(&state, &p, "profiles.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(profile))))
}

/// Shared handler for profile lifecycle transitions (deactivate/reactivate/delete/restore).
async fn profile_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let profile = match action {
        "deactivate" => repo.deactivate_profile(p.tenant_id, id, p.user_id).await?,
        "reactivate" => repo.reactivate_profile(p.tenant_id, id, p.user_id).await?,
        "delete" => repo.soft_delete_profile(p.tenant_id, id, p.user_id).await?,
        "restore" => repo.restore_profile(p.tenant_id, id, p.user_id).await?,
        _ => return Err(HelixError::validation("unknown profile action").into()),
    };
    audit(
        &state,
        &p,
        &format!("profile.{action}"),
        "profile",
        profile.id,
        serde_json::json!({"display_name": profile.display_name, "status": profile.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "profiles.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish(
        &state,
        "helix.network.profile.lifecycle",
        serde_json::json!({
            "profile_id": profile.id,
            "action": action,
            "status": profile.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(profile))))
}

async fn deactivate_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    profile_transition(state, p, id, "deactivate").await
}

async fn reactivate_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    profile_transition(state, p, id, "reactivate").await
}

async fn delete_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    profile_transition(state, p, id, "delete").await
}

async fn restore_profile(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    profile_transition(state, p, id, "restore").await
}

// --- Connections ---

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
    let me = require_my_profile(&state, &p).await?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let conn = repo
        .request_connection(p.tenant_id, me.id, body.to_profile_id, &body.message)
        .await?;
    audit(
        &state,
        &p,
        "connection.request",
        "connection",
        conn.id,
        serde_json::json!({
            "from": conn.from_profile_id,
            "to": conn.to_profile_id
        }),
    )
    .await?;
    meter(&state, &p, "connections.requested", serde_json::json!({})).await?;
    publish(
        &state,
        "helix.network.connection.requested",
        serde_json::json!({"connection_id": conn.id, "to": conn.to_profile_id}),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(conn))))
}

/// Shared handler for connection lifecycle responses (accept/decline/remove/block).
async fn connection_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let me = require_my_profile(&state, &p).await?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let conn = match action {
        "accept" => repo.accept_connection(p.tenant_id, id, me.id).await?,
        "decline" => repo.decline_connection(p.tenant_id, id, me.id).await?,
        "remove" => repo.remove_connection(p.tenant_id, id, me.id).await?,
        "block" => repo.block_connection(p.tenant_id, id, me.id).await?,
        _ => return Err(HelixError::validation("unknown connection action").into()),
    };
    audit(
        &state,
        &p,
        &format!("connection.{action}"),
        "connection",
        conn.id,
        serde_json::json!({"status": conn.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "connections.responded",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish(
        &state,
        &format!("helix.network.connection.{action}ed"),
        serde_json::json!({"connection_id": conn.id, "status": conn.status}),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(conn))))
}

async fn accept_connection(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    connection_transition(state, p, id, "accept").await
}

async fn decline_connection(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    connection_transition(state, p, id, "decline").await
}

async fn remove_connection(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    connection_transition(state, p, id, "remove").await
}

async fn block_connection(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    connection_transition(state, p, id, "block").await
}

// --- Opportunities ---

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
    let me = require_my_profile(&state, &p).await?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
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
    audit(
        &state,
        &p,
        "opportunity.create",
        "opportunity",
        opp.id,
        serde_json::json!({"title": opp.title, "kind": opp.kind}),
    )
    .await?;
    meter(&state, &p, "opportunities.created", serde_json::json!({})).await?;
    publish(
        &state,
        "helix.network.opportunity.created",
        serde_json::json!({"opportunity_id": opp.id, "title": opp.title}),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(opp))))
}

async fn get_opportunity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let opp = repo
        .get_opportunity(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("opportunity not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(opp))))
}

#[derive(Deserialize, Default)]
struct UpdateOpportunity {
    title: Option<String>,
    description: Option<String>,
    kind: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_opportunity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateOpportunity>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let me = require_my_profile(&state, &p).await?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let opp = repo
        .update_opportunity(
            p.tenant_id,
            id,
            me.id,
            OpportunityUpdate {
                title,
                description: body.description,
                kind: body.kind,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "opportunity.update",
        "opportunity",
        opp.id,
        serde_json::json!({"title": opp.title}),
    )
    .await?;
    meter(&state, &p, "opportunities.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(opp))))
}

/// Shared handler for opportunity lifecycle transitions (close/reopen/delete/restore).
async fn opportunity_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let me = require_my_profile(&state, &p).await?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let opp = match action {
        "close" => repo.close_opportunity(p.tenant_id, id, me.id).await?,
        "reopen" => repo.reopen_opportunity(p.tenant_id, id, me.id).await?,
        "delete" => repo.soft_delete_opportunity(p.tenant_id, id, me.id).await?,
        "restore" => repo.restore_opportunity(p.tenant_id, id, me.id).await?,
        _ => return Err(HelixError::validation("unknown opportunity action").into()),
    };
    audit(
        &state,
        &p,
        &format!("opportunity.{action}"),
        "opportunity",
        opp.id,
        serde_json::json!({"title": opp.title, "status": opp.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "opportunities.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish(
        &state,
        "helix.network.opportunity.lifecycle",
        serde_json::json!({
            "opportunity_id": opp.id,
            "action": action,
            "status": opp.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(opp))))
}

async fn close_opportunity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    opportunity_transition(state, p, id, "close").await
}

async fn reopen_opportunity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    opportunity_transition(state, p, id, "reopen").await
}

async fn delete_opportunity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    opportunity_transition(state, p, id, "delete").await
}

async fn restore_opportunity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    opportunity_transition(state, p, id, "restore").await
}

// --- Reports ---

async fn network_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<NetworkSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = NetworkRepo::new(pool);
    let rows = repo.get_network_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::{TenantId, UserId};
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{can_revive_connection, next_opportunity_status, next_profile_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18109");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-network").expect("helix-network product known");
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

    fn dev_user(label: &str) -> UserId {
        UserId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("helixforge-user:{label}").as_bytes(),
        ))
    }

    #[test]
    fn profile_transitions_are_guarded() {
        assert_eq!(
            next_profile_status("active", "deactivate").unwrap(),
            "deactivated"
        );
        assert_eq!(
            next_profile_status("deactivated", "reactivate").unwrap(),
            "active"
        );
        assert!(next_profile_status("deactivated", "deactivate").is_err());
        assert!(next_profile_status("active", "reactivate").is_err());
        assert!(next_profile_status("deleted", "deactivate").is_err());
        assert!(next_profile_status("active", "unknown").is_err());
    }

    #[test]
    fn opportunity_transitions_are_guarded() {
        assert_eq!(next_opportunity_status("open", "close").unwrap(), "closed");
        assert_eq!(next_opportunity_status("closed", "reopen").unwrap(), "open");
        assert!(next_opportunity_status("closed", "close").is_err());
        assert!(next_opportunity_status("open", "reopen").is_err());
        assert!(next_opportunity_status("deleted", "close").is_err());
    }

    #[test]
    fn only_declined_or_removed_connections_revive() {
        assert!(can_revive_connection("declined"));
        assert!(can_revive_connection("removed"));
        assert!(!can_revive_connection("pending"));
        assert!(!can_revive_connection("accepted"));
        assert!(!can_revive_connection("blocked"));
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn full_connection_profile_opportunity_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = NetworkRepo::new(pool.clone());

        let alice_user = dev_user("net-alice");
        let bob_user = dev_user("net-bob");
        let carol_user = dev_user("net-carol");

        let alice = repo
            .create_profile(
                tenant_id,
                alice_user,
                "Alice",
                "builder",
                "",
                serde_json::json!([]),
                "",
                serde_json::json!({}),
            )
            .await
            .expect("create alice");
        let bob = repo
            .create_profile(
                tenant_id,
                bob_user,
                "Bob",
                "designer",
                "",
                serde_json::json!([]),
                "",
                serde_json::json!({}),
            )
            .await
            .expect("create bob");

        // Duplicate profile for the same user conflicts.
        let dup = repo
            .create_profile(
                tenant_id,
                alice_user,
                "Alice Again",
                "",
                "",
                serde_json::json!([]),
                "",
                serde_json::json!({}),
            )
            .await;
        assert!(dup.is_err(), "duplicate profile rejected");

        // Request -> decline -> revive -> accept.
        let conn = repo
            .request_connection(tenant_id, alice.id, bob.id, "hello")
            .await
            .expect("request");
        assert_eq!(conn.status, "pending");

        let wrong_decliner = repo.decline_connection(tenant_id, conn.id, alice.id).await;
        assert!(wrong_decliner.is_err(), "only the receiver may decline");

        let declined = repo
            .decline_connection(tenant_id, conn.id, bob.id)
            .await
            .expect("decline");
        assert_eq!(declined.status, "declined");
        assert!(declined.responded_at.is_some());

        let revived = repo
            .request_connection(tenant_id, alice.id, bob.id, "second try")
            .await
            .expect("revive declined");
        assert_eq!(revived.status, "pending");
        assert_eq!(revived.message, "second try");
        assert!(revived.responded_at.is_none());

        let accepted = repo
            .accept_connection(tenant_id, conn.id, bob.id)
            .await
            .expect("accept");
        assert_eq!(accepted.status, "accepted");
        assert!(accepted.responded_at.is_some());

        let dup_request = repo
            .request_connection(tenant_id, alice.id, bob.id, "again")
            .await;
        assert!(dup_request.is_err(), "already connected rejects request");

        // Summary shows the accepted connection for both profiles.
        let summary = repo.get_network_summary(tenant_id).await.expect("summary");
        let alice_row = summary.iter().find(|r| r.id == alice.id).unwrap();
        let bob_row = summary.iter().find(|r| r.id == bob.id).unwrap();
        assert_eq!(alice_row.accepted_count, 1);
        assert_eq!(bob_row.accepted_count, 1);

        // Remove, revive again, then block; blocked pairs cannot request.
        let removed = repo
            .remove_connection(tenant_id, conn.id, alice.id)
            .await
            .expect("remove");
        assert_eq!(removed.status, "removed");

        let revived2 = repo
            .request_connection(tenant_id, alice.id, bob.id, "third try")
            .await
            .expect("revive removed");
        assert_eq!(revived2.status, "pending");

        let blocked = repo
            .block_connection(tenant_id, conn.id, bob.id)
            .await
            .expect("block");
        assert_eq!(blocked.status, "blocked");
        assert_eq!(blocked.blocked_by, Some(bob.id));

        let after_block = repo
            .request_connection(tenant_id, alice.id, bob.id, "please")
            .await;
        assert!(after_block.is_err(), "blocked pair cannot request");
        let reverse_block = repo
            .request_connection(tenant_id, bob.id, alice.id, "reverse")
            .await;
        assert!(reverse_block.is_err(), "block applies in both directions");

        // Deactivated profiles cannot connect.
        let carol = repo
            .create_profile(
                tenant_id,
                carol_user,
                "Carol",
                "",
                "",
                serde_json::json!([]),
                "",
                serde_json::json!({}),
            )
            .await
            .expect("create carol");
        repo.deactivate_profile(tenant_id, alice.id, alice_user)
            .await
            .expect("deactivate alice");
        let to_deactivated = repo
            .request_connection(tenant_id, carol.id, alice.id, "hi")
            .await;
        assert!(
            to_deactivated.is_err(),
            "cannot connect to deactivated profile"
        );
        let reactivated = repo
            .reactivate_profile(tenant_id, alice.id, alice_user)
            .await
            .expect("reactivate alice");
        assert_eq!(reactivated.status, "active");

        // Profile update + soft delete + restore.
        let renamed = repo
            .update_profile(
                tenant_id,
                alice.id,
                alice_user,
                ProfileUpdate {
                    headline: Some("senior builder".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update alice");
        assert_eq!(renamed.headline, "senior builder");

        let not_owner = repo
            .update_profile(
                tenant_id,
                alice.id,
                bob_user,
                ProfileUpdate {
                    headline: Some("hijack".into()),
                    ..Default::default()
                },
            )
            .await;
        assert!(not_owner.is_err(), "non-owner cannot update profile");

        // Opportunity lifecycle.
        let opp = repo
            .create_opportunity(
                tenant_id,
                alice.id,
                "Platform role",
                "build things",
                "role",
                serde_json::json!({}),
            )
            .await
            .expect("create opportunity");
        assert_eq!(opp.status, "open");

        let updated = repo
            .update_opportunity(
                tenant_id,
                opp.id,
                alice.id,
                OpportunityUpdate {
                    title: Some("Senior platform role".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update opportunity");
        assert_eq!(updated.title, "Senior platform role");

        let closed = repo
            .close_opportunity(tenant_id, opp.id, alice.id)
            .await
            .expect("close opportunity");
        assert_eq!(closed.status, "closed");
        assert!(closed.closed_at.is_some());

        let reopened = repo
            .reopen_opportunity(tenant_id, opp.id, alice.id)
            .await
            .expect("reopen opportunity");
        assert_eq!(reopened.status, "open");
        assert!(reopened.closed_at.is_none());

        let not_owner_close = repo.close_opportunity(tenant_id, opp.id, bob.id).await;
        assert!(
            not_owner_close.is_err(),
            "non-owner cannot close opportunity"
        );

        // Summary counts the open opportunity.
        let summary2 = repo
            .get_network_summary(tenant_id)
            .await
            .expect("summary 2");
        let alice_row2 = summary2.iter().find(|r| r.id == alice.id).unwrap();
        assert_eq!(alice_row2.open_opportunities, 1);

        // Soft delete hides the opportunity; restore brings it back open.
        repo.soft_delete_opportunity(tenant_id, opp.id, alice.id)
            .await
            .expect("delete opportunity");
        let opps = repo
            .list_opportunities(tenant_id)
            .await
            .expect("list after delete");
        assert!(opps.iter().all(|o| o.id != opp.id));
        let restored_opp = repo
            .restore_opportunity(tenant_id, opp.id, alice.id)
            .await
            .expect("restore opportunity");
        assert_eq!(restored_opp.status, "open");

        // Soft delete hides the profile; restore brings it back.
        repo.soft_delete_profile(tenant_id, bob.id, bob_user)
            .await
            .expect("delete bob");
        let profiles = repo
            .list_profiles(tenant_id)
            .await
            .expect("list profiles after delete");
        assert!(profiles.iter().all(|p| p.id != bob.id));
        let restored_bob = repo
            .restore_profile(tenant_id, bob.id, bob_user)
            .await
            .expect("restore bob");
        assert_eq!(restored_bob.status, "active");
        assert!(restored_bob.deleted_at.is_none());
    }
}
