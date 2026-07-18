//! HelixWell API — AI personal & team wellness (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{CheckInUpdate, DbPool, HabitSummaryRow, HabitUpdate, WellRepo};
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
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-well", 8108)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-well", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/habits", get(list_habits).post(create_habit))
        .route("/v1/habits/{id}", get(get_habit).patch(update_habit))
        .route("/v1/habits/{id}/pause", post(pause_habit))
        .route("/v1/habits/{id}/resume", post(resume_habit))
        .route("/v1/habits/{id}/end", post(end_habit))
        .route("/v1/habits/{id}/delete", post(delete_habit))
        .route("/v1/habits/{id}/restore", post(restore_habit))
        .route("/v1/habits/{id}/logs", get(list_habit_logs).post(log_habit))
        .route("/v1/checkins", get(list_checkins).post(create_checkin))
        .route("/v1/checkins/{id}", get(get_checkin).patch(update_checkin))
        .route("/v1/checkins/{id}/delete", post(delete_checkin))
        .route("/v1/checkins/{id}/edits", get(list_checkin_edits))
        .route("/v1/reports/habit-summary", get(habit_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-well",
        "phase": "wave2_w8",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "habits": true,
            "habit_logs": true,
            "checkins": true,
            "habit_lifecycle": true,
            "optional_checkin_fields": true,
            "checkin_edit_history": true,
            "soft_delete": true,
            "habit_summary": true,
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
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable wellness").into())
}

async fn audit_habit(
    state: &AppState,
    p: &shared_core::tenancy::Principal,
    action: &str,
    habit_id: Uuid,
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
            resource_type: "habit".into(),
            resource_id: habit_id.to_string(),
            metadata,
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(())
}

async fn audit_checkin(
    state: &AppState,
    p: &shared_core::tenancy::Principal,
    action: &str,
    checkin_id: Uuid,
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
            resource_type: "checkin".into(),
            resource_id: checkin_id.to_string(),
            metadata,
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(())
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
    let pool = require_pool(&state)?;
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
    audit_habit(
        &state,
        &p,
        "habit.create",
        habit.id,
        serde_json::json!({"name": habit.name, "cadence": habit.cadence}),
    )
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
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let habit = repo
        .get_habit(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("habit not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(habit))))
}

#[derive(Deserialize, Default)]
struct UpdateHabit {
    name: Option<String>,
    description: Option<String>,
    cadence: Option<String>,
    target_per_period: Option<i32>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateHabit>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let habit = repo
        .update_habit(
            p.tenant_id,
            id,
            HabitUpdate {
                name,
                description: body.description,
                cadence: body.cadence,
                target_per_period: body.target_per_period,
                metadata: body.metadata,
            },
        )
        .await?;
    audit_habit(
        &state,
        &p,
        "habit.update",
        habit.id,
        serde_json::json!({"name": habit.name}),
    )
    .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-well",
            "habits.updated",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(habit))))
}

/// Shared handler for habit lifecycle transitions (pause/resume/end/delete/restore).
async fn habit_transition(
    state: AppState,
    p: shared_core::tenancy::Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let habit = match action {
        "pause" => repo.pause_habit(p.tenant_id, id).await?,
        "resume" => repo.resume_habit(p.tenant_id, id).await?,
        "end" => repo.end_habit(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_habit(p.tenant_id, id).await?,
        "restore" => repo.restore_habit(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown habit action").into()),
    };
    audit_habit(
        &state,
        &p,
        &format!("habit.{action}"),
        habit.id,
        serde_json::json!({"name": habit.name, "status": habit.status}),
    )
    .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-well",
            "habits.lifecycle",
            1.0,
            "count",
            serde_json::json!({"action": action}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.well.habit.lifecycle",
            &serde_json::json!({
                "habit_id": habit.id,
                "action": action,
                "status": habit.status
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(habit))))
}

async fn pause_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    habit_transition(state, p, id, "pause").await
}

async fn resume_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    habit_transition(state, p, id, "resume").await
}

async fn end_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    habit_transition(state, p, id, "end").await
}

async fn delete_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    habit_transition(state, p, id, "delete").await
}

async fn restore_habit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    habit_transition(state, p, id, "restore").await
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
    let pool = require_pool(&state)?;
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
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let items = repo.list_habit_logs(p.tenant_id, id, q.limit).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "habit_id": id,
        "items": items
    }))))
}

async fn habit_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<HabitSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let rows = repo.get_habit_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
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
    /// Skipped fields stay missing (NULL), never zero.
    mood: Option<i32>,
    energy: Option<i32>,
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
    let pool = require_pool(&state)?;
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
    audit_checkin(
        &state,
        &p,
        "checkin.create",
        checkin.id,
        serde_json::json!({
            "mood": checkin.mood,
            "energy": checkin.energy,
            "mood_skipped": checkin.mood.is_none(),
            "energy_skipped": checkin.energy.is_none()
        }),
    )
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

async fn get_checkin(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let checkin = repo
        .get_checkin(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("check-in not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(checkin))))
}

#[derive(Deserialize, Default)]
struct UpdateCheckIn {
    /// Outer `None` = unchanged; explicit `null` clears the field to missing.
    #[serde(default, deserialize_with = "de_double_option")]
    mood: Option<Option<i32>>,
    #[serde(default, deserialize_with = "de_double_option")]
    energy: Option<Option<i32>>,
    notes: Option<String>,
    #[serde(default)]
    tags: Option<serde_json::Value>,
}

fn de_double_option<'de, D>(deserializer: D) -> Result<Option<Option<i32>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::<i32>::deserialize(deserializer)?))
}

async fn update_checkin(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCheckIn>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let checkin = repo
        .update_checkin(
            p.tenant_id,
            id,
            p.user_id,
            CheckInUpdate {
                mood: body.mood,
                energy: body.energy,
                notes: body.notes,
                tags: body.tags,
            },
        )
        .await?;
    audit_checkin(
        &state,
        &p,
        "checkin.update",
        checkin.id,
        serde_json::json!({"edit_version": checkin.edit_version}),
    )
    .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-well",
            "checkins.updated",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.well.checkin.updated",
            &serde_json::json!({
                "checkin_id": checkin.id,
                "edit_version": checkin.edit_version
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(checkin))))
}

async fn delete_checkin(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let checkin = repo.soft_delete_checkin(p.tenant_id, id).await?;
    audit_checkin(
        &state,
        &p,
        "checkin.delete",
        checkin.id,
        serde_json::json!({}),
    )
    .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-well",
            "checkins.deleted",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.well.checkin.deleted",
            &serde_json::json!({"checkin_id": checkin.id}),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "id": checkin.id,
        "deleted": true
    }))))
}

async fn list_checkin_edits(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = WellRepo::new(pool.clone());
    let items = repo.list_checkin_edits(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "checkin_id": id,
        "items": items
    }))))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::tenancy::{Principal, Scope};
    use shared_core::{TenantId, UserId};
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_habit_status, validate_optional_scale};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18108");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-well").expect("helix-well product known");
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

    pub fn dev_principal(label: &str) -> Principal {
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let user_id = UserId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("helixforge-user:{label}").as_bytes(),
        ));
        Principal {
            user_id,
            tenant_id,
            org_id: None,
            scopes: vec![
                Scope::Read,
                Scope::Write,
                Scope::Admin,
                Scope::AuditRead,
                Scope::Platform,
            ],
            session_id: Some(format!("dev-session:{label}")),
            residency_region: "local".into(),
        }
    }

    #[test]
    fn skipped_scale_fields_pass_validation() {
        assert!(validate_optional_scale(None, "mood").is_ok());
        assert!(validate_optional_scale(None, "energy").is_ok());
    }

    #[test]
    fn out_of_range_scale_is_rejected() {
        assert!(validate_optional_scale(Some(0), "mood").is_err());
        assert!(validate_optional_scale(Some(11), "energy").is_err());
        assert!(validate_optional_scale(Some(1), "mood").is_ok());
        assert!(validate_optional_scale(Some(10), "energy").is_ok());
    }

    #[test]
    fn habit_transitions_are_guarded() {
        assert_eq!(next_habit_status("active", "pause").unwrap(), "paused");
        assert_eq!(next_habit_status("paused", "resume").unwrap(), "active");
        assert_eq!(next_habit_status("active", "end").unwrap(), "ended");
        assert_eq!(next_habit_status("paused", "end").unwrap(), "ended");
        assert!(next_habit_status("ended", "pause").is_err());
        assert!(next_habit_status("ended", "resume").is_err());
        assert!(next_habit_status("paused", "pause").is_err());
        assert!(next_habit_status("active", "resume").is_err());
        assert!(next_habit_status("active", "unknown").is_err());
    }

    #[test]
    fn empty_checkin_update_has_no_changes() {
        let empty = CheckInUpdate::default();
        assert!(!empty.has_changes());
        let with_notes = CheckInUpdate {
            notes: Some("n".into()),
            ..Default::default()
        };
        assert!(with_notes.has_changes());
        let cleared = CheckInUpdate {
            mood: Some(None),
            ..Default::default()
        };
        assert!(cleared.has_changes());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn habit_lifecycle_and_checkin_edits_persist() {
        let (state, _guard) = locked_state().await;
        let p = dev_principal("well-alice");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = WellRepo::new(pool.clone());

        // Habit lifecycle: create -> log -> pause -> log rejected -> resume -> log.
        let habit = repo
            .create_habit(
                p.tenant_id,
                p.user_id,
                "Walk",
                "Daily walk",
                "daily",
                1,
                serde_json::json!({}),
            )
            .await
            .expect("create habit");
        assert_eq!(habit.status, "active");

        repo.log_habit(p.tenant_id, p.user_id, habit.id, 1, "first")
            .await
            .expect("log active habit");

        let paused = repo
            .pause_habit(p.tenant_id, habit.id)
            .await
            .expect("pause habit");
        assert_eq!(paused.status, "paused");
        assert!(paused.paused_at.is_some());

        let log_paused = repo
            .log_habit(p.tenant_id, p.user_id, habit.id, 1, "nope")
            .await;
        assert!(log_paused.is_err(), "cannot log a paused habit");

        let resumed = repo
            .resume_habit(p.tenant_id, habit.id)
            .await
            .expect("resume habit");
        assert_eq!(resumed.status, "active");
        assert!(resumed.paused_at.is_none());

        repo.log_habit(p.tenant_id, p.user_id, habit.id, 2, "second")
            .await
            .expect("log resumed habit");

        // Summary reflects both logs.
        let summary = repo
            .get_habit_summary(p.tenant_id)
            .await
            .expect("habit summary");
        let row = summary.iter().find(|r| r.id == habit.id).unwrap();
        assert_eq!(row.total_logs, 2);
        assert_eq!(row.total_quantity, 3);
        assert_eq!(row.logs_last_7_days, 2);
        assert!(row.last_logged_at.is_some());

        // End is terminal for logging.
        let ended = repo
            .end_habit(p.tenant_id, habit.id)
            .await
            .expect("end habit");
        assert_eq!(ended.status, "ended");
        assert!(ended.ended_at.is_some());
        let log_ended = repo
            .log_habit(p.tenant_id, p.user_id, habit.id, 1, "nope")
            .await;
        assert!(log_ended.is_err(), "cannot log an ended habit");

        // Update still works on an ended habit.
        let renamed = repo
            .update_habit(
                p.tenant_id,
                habit.id,
                HabitUpdate {
                    name: Some("Evening walk".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update ended habit");
        assert_eq!(renamed.name, "Evening walk");

        // Check-in with a skipped field: missing is not zero.
        let checkin = repo
            .create_checkin(
                p.tenant_id,
                p.user_id,
                Some(7),
                None,
                "ok day",
                serde_json::json!([]),
            )
            .await
            .expect("create check-in");
        assert_eq!(checkin.mood, Some(7));
        assert_eq!(checkin.energy, None);
        assert_eq!(checkin.edit_version, 0);

        // Edit snapshots the previous values and bumps the version.
        let edited = repo
            .update_checkin(
                p.tenant_id,
                checkin.id,
                p.user_id,
                CheckInUpdate {
                    energy: Some(Some(5)),
                    notes: Some("ok day, tired".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("edit check-in");
        assert_eq!(edited.energy, Some(5));
        assert_eq!(edited.notes, "ok day, tired");
        assert_eq!(edited.edit_version, 1);
        assert!(edited.updated_at.is_some());

        let edits = repo
            .list_checkin_edits(p.tenant_id, checkin.id)
            .await
            .expect("list edits");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].energy, None, "snapshot holds pre-edit values");
        assert_eq!(edits[0].notes, "ok day");

        // Clearing a field returns it to missing.
        let cleared = repo
            .update_checkin(
                p.tenant_id,
                checkin.id,
                p.user_id,
                CheckInUpdate {
                    mood: Some(None),
                    ..Default::default()
                },
            )
            .await
            .expect("clear mood");
        assert_eq!(cleared.mood, None);
        assert_eq!(cleared.edit_version, 2);

        // Empty updates are rejected.
        let no_change = repo
            .update_checkin(p.tenant_id, checkin.id, p.user_id, CheckInUpdate::default())
            .await;
        assert!(no_change.is_err(), "empty update rejected");

        // Out-of-range edits are rejected.
        let bad = repo
            .update_checkin(
                p.tenant_id,
                checkin.id,
                p.user_id,
                CheckInUpdate {
                    mood: Some(Some(11)),
                    ..Default::default()
                },
            )
            .await;
        assert!(bad.is_err(), "out-of-range mood rejected");

        // Soft-deleted check-ins disappear from reads.
        repo.soft_delete_checkin(p.tenant_id, checkin.id)
            .await
            .expect("delete check-in");
        assert!(repo
            .get_checkin(p.tenant_id, checkin.id)
            .await
            .expect("get deleted")
            .is_none());
        let remaining = repo
            .list_checkins(p.tenant_id, Some(p.user_id), 50)
            .await
            .expect("list after delete");
        assert!(remaining.iter().all(|c| c.id != checkin.id));

        // Soft-deleted habits disappear; restore returns pre-delete status.
        repo.soft_delete_habit(p.tenant_id, habit.id)
            .await
            .expect("delete habit");
        let habits = repo
            .list_habits(p.tenant_id, None)
            .await
            .expect("list habits after delete");
        assert!(habits.iter().all(|h| h.id != habit.id));

        let restored = repo
            .restore_habit(p.tenant_id, habit.id)
            .await
            .expect("restore habit");
        assert_eq!(
            restored.status, "ended",
            "restore returns pre-delete status"
        );
        assert!(restored.deleted_at.is_none());
    }
}
