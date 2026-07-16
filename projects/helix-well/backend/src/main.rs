//! HelixWell API — AI personal & team wellness (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use helix_db::WellRepo;
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-well")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: format!("{}-assistant", product.slug),
            description: format!("{} assistant", product.title),
            system_prompt: format!(
                "You are the {} wellness assistant. Help track habits, mood, and energy.",
                product.title
            ),
            tools: vec!["echo".into(), "product_catalog".into()],
            max_steps: 8,
        });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .nest_service("/", domain_routes().with_state(state.clone()));

    let cfg = shared_core::CoreConfig::from_env("helix-well", 8108)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-well", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/habits", get(list_habits).post(create_habit))
        .route("/v1/habits/{id}", get(get_habit))
        .route("/v1/habits/{id}/logs", get(list_habit_logs).post(log_habit))
        .route("/v1/checkins", get(list_checkins).post(create_checkin))
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

#[derive(Deserialize)]
struct MineQuery {
    #[serde(default)]
    mine: bool,
}

async fn list_habits(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Query(q): Query<MineQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = WellRepo::new(pool.clone());
        let owner = if q.mine { Some(p.user_id) } else { None };
        let items = repo.list_habits(p.tenant_id, owner).await?;
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
struct CreateHabit {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_cadence")]
    cadence: String,
    #[serde(default = "default_target")]
    target_per_period: i32,
    #[serde(default)]
    metadata: serde_json::Value,
}

fn default_cadence() -> String {
    "daily".into()
}

fn default_target() -> i32 {
    1
}

async fn create_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateHabit>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable wellness"))?;
    let repo = WellRepo::new(pool.clone());
    let habit = repo
        .create_habit(
            p.tenant_id,
            p.user_id,
            body.name.trim(),
            &body.description,
            &body.cadence,
            body.target_per_period,
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
            action: "habit.create".into(),
            resource_type: "habit".into(),
            resource_id: habit.id.to_string(),
            metadata: serde_json::json!({"name": habit.name, "cadence": habit.cadence}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-well",
            "habits.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(habit))))
}

async fn get_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable wellness"))?;
    let repo = WellRepo::new(pool.clone());
    let habit = repo
        .get_habit(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("habit not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(habit))))
}

#[derive(Deserialize)]
struct LogHabit {
    #[serde(default = "default_target")]
    quantity: i32,
    #[serde(default)]
    notes: String,
}

async fn log_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<LogHabit>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable wellness"))?;
    let repo = WellRepo::new(pool.clone());
    let log = repo
        .log_habit(p.tenant_id, p.user_id, id, body.quantity, &body.notes)
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-well",
            "habits.logged",
            1.0,
            "count",
            serde_json::json!({"habit_id": id}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.well.habit.logged",
            &serde_json::json!({"habit_id": id, "log_id": log.id, "quantity": log.quantity}),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(log))))
}

#[derive(Deserialize)]
struct LimitQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

async fn list_habit_logs(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable wellness"))?;
    let repo = WellRepo::new(pool.clone());
    let items = repo.list_habit_logs(p.tenant_id, id, q.limit).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "habit_id": id,
        "items": items
    }))))
}

#[derive(Deserialize)]
struct CheckinsQuery {
    #[serde(default)]
    mine: bool,
    #[serde(default = "default_limit")]
    limit: i64,
}

async fn list_checkins(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Query(q): Query<CheckinsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = WellRepo::new(pool.clone());
        let user = if q.mine { Some(p.user_id) } else { None };
        let items = repo.list_checkins(p.tenant_id, user, q.limit).await?;
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
struct CreateCheckIn {
    mood: i32,
    energy: i32,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    tags: serde_json::Value,
}

async fn create_checkin(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateCheckIn>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable wellness"))?;
    let tags = if body.tags.is_null() {
        serde_json::json!([])
    } else {
        body.tags
    };
    let repo = WellRepo::new(pool.clone());
    let checkin = repo
        .create_checkin(
            p.tenant_id,
            p.user_id,
            body.mood,
            body.energy,
            &body.notes,
            tags,
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
            action: "checkin.create".into(),
            resource_type: "checkin".into(),
            resource_id: checkin.id.to_string(),
            metadata: serde_json::json!({"mood": checkin.mood, "energy": checkin.energy}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-well",
            "checkins.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.well.checkin.created",
            &serde_json::json!({
                "checkin_id": checkin.id,
                "mood": checkin.mood,
                "energy": checkin.energy
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(checkin))))
}
