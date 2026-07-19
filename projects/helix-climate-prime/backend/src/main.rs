//! helix-climate-prime API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{ClimateRepo, ClimateSummaryRow, DbPool, ScenarioUpdate, ScoreUpdate};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-climate-prime")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: format!("{}-assistant", product.slug),
            description: format!("{} assistant", product.title),
            system_prompt: format!("You are the {} assistant.", product.title),
            tools: vec!["echo".into(), "product_catalog".into()],
            max_steps: 8,
        });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-climate-prime", 8115)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-climate-prime", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/scenarios", get(list_parents).post(create_parent))
        .route("/v1/scenarios/{id}", get(get_parent).patch(update_scenario))
        .route("/v1/scenarios/{id}/activate", post(activate_scenario))
        .route("/v1/scenarios/{id}/archive", post(archive_scenario))
        .route("/v1/scenarios/{id}/reopen", post(reopen_scenario))
        .route("/v1/scenarios/{id}/delete", post(delete_scenario))
        .route("/v1/scenarios/{id}/restore", post(restore_scenario))
        .route(
            "/v1/scenarios/{id}/risk_scores",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/scenarios/{id}/risk_scores/{score_id}",
            axum::routing::patch(update_score),
        )
        .route(
            "/v1/scenarios/{id}/risk_scores/{score_id}/assess",
            post(assess_score),
        )
        .route(
            "/v1/scenarios/{id}/risk_scores/{score_id}/dismiss",
            post(dismiss_score),
        )
        .route(
            "/v1/scenarios/{id}/risk_scores/{score_id}/delete",
            post(delete_score),
        )
        .route(
            "/v1/scenarios/{id}/risk_scores/{score_id}/restore",
            post(restore_score),
        )
        .route("/v1/reports/climate-summary", get(climate_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-climate-prime",
        "phase": "wave2_w15",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "scenarios": true,
            "risk_scores": true,
            "scenario_lifecycle": true,
            "score_lifecycle": true,
            "archive_guards": true,
            "climate_summary": true,
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
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable store").into())
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
        .record_usage(
            p.tenant_id,
            "helix-climate-prime",
            metric,
            1.0,
            "count",
            metadata,
        )
        .await?;
    Ok(())
}

async fn publish_event(state: &AppState, topic: &str, payload: serde_json::Value) {
    state.clients.bus.publish(topic, &payload).await.ok();
}

// --- Scenarios ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = ClimateRepo::new(pool.clone());
        let items = repo.list_parents(p.tenant_id).await?;
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
struct CreateParent {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

async fn create_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateParent>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let item = repo
        .create_parent(
            p.tenant_id,
            body.name.trim(),
            &body.description,
            body.metadata,
        )
        .await?;
    audit(
        &state,
        &p,
        "scenario.create",
        "scenario",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "scenarios.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("scenario not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateScenario {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_scenario(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateScenario>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_scenario(
            p.tenant_id,
            id,
            ScenarioUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "scenario.update",
        "scenario",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "scenarios.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for scenario lifecycle transitions (activate/archive/reopen/delete/restore).
async fn scenario_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let item = match action {
        "activate" => repo.activate_scenario(p.tenant_id, id).await?,
        "archive" => repo.archive_scenario(p.tenant_id, id).await?,
        "reopen" => repo.reopen_scenario(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_scenario(p.tenant_id, id).await?,
        "restore" => repo.restore_scenario(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown scenario action").into()),
    };
    audit(
        &state,
        &p,
        &format!("scenario.{action}"),
        "scenario",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "scenarios.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.climate.scenario.lifecycle",
        serde_json::json!({
            "scenario_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn activate_scenario(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    scenario_transition(state, p, id, "activate").await
}

async fn archive_scenario(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    scenario_transition(state, p, id, "archive").await
}

async fn reopen_scenario(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    scenario_transition(state, p, id, "reopen").await
}

async fn delete_scenario(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    scenario_transition(state, p, id, "delete").await
}

async fn restore_scenario(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    scenario_transition(state, p, id, "restore").await
}

// --- Risk scores ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let items = repo.list_children(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "parent_id": id,
        "items": items
    }))))
}

#[derive(Deserialize)]
struct CreateChild {
    title: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

async fn create_child(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateChild>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.title.trim().is_empty() {
        return Err(HelixError::validation("title required").into());
    }
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let item = repo
        .create_child(
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
        "score.create",
        "risk_score",
        item.id,
        serde_json::json!({"scenario_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "scores.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateScore {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_score(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, score_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateScore>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_score(
            p.tenant_id,
            id,
            score_id,
            ScoreUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "score.update",
        "risk_score",
        item.id,
        serde_json::json!({"scenario_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "scores.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for score lifecycle transitions (assess/dismiss/delete/restore).
async fn score_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    score_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let item = match action {
        "assess" => repo.assess_score(p.tenant_id, id, score_id).await?,
        "dismiss" => repo.dismiss_score(p.tenant_id, id, score_id).await?,
        "delete" => repo.soft_delete_score(p.tenant_id, id, score_id).await?,
        "restore" => repo.restore_score(p.tenant_id, id, score_id).await?,
        _ => return Err(HelixError::validation("unknown score action").into()),
    };
    audit(
        &state,
        &p,
        &format!("score.{action}"),
        "risk_score",
        item.id,
        serde_json::json!({"scenario_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "scores.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.climate.score.lifecycle",
        serde_json::json!({
            "scenario_id": id,
            "score_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn assess_score(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, score_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    score_transition(state, p, id, score_id, "assess").await
}

async fn dismiss_score(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, score_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    score_transition(state, p, id, score_id, "dismiss").await
}

async fn delete_score(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, score_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    score_transition(state, p, id, score_id, "delete").await
}

async fn restore_score(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, score_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    score_transition(state, p, id, score_id, "restore").await
}

// --- Reports ---

async fn climate_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<ClimateSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = ClimateRepo::new(pool);
    let rows = repo.get_climate_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_scenario_status, next_score_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18115");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-climate-prime")
            .expect("helix-climate-prime product known");
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
    fn scenario_transitions_are_guarded() {
        assert_eq!(next_scenario_status("draft", "activate").unwrap(), "active");
        assert_eq!(
            next_scenario_status("active", "archive").unwrap(),
            "archived"
        );
        assert_eq!(
            next_scenario_status("archived", "reopen").unwrap(),
            "active"
        );
        assert!(next_scenario_status("active", "activate").is_err());
        assert!(next_scenario_status("draft", "archive").is_err());
        assert!(next_scenario_status("active", "reopen").is_err());
        assert!(next_scenario_status("deleted", "activate").is_err());
    }

    #[test]
    fn score_transitions_are_guarded() {
        assert_eq!(next_score_status("draft", "assess").unwrap(), "assessed");
        assert_eq!(next_score_status("draft", "dismiss").unwrap(), "dismissed");
        assert_eq!(
            next_score_status("assessed", "dismiss").unwrap(),
            "dismissed"
        );
        assert!(next_score_status("assessed", "assess").is_err());
        assert!(next_score_status("dismissed", "assess").is_err());
        assert!(next_score_status("dismissed", "dismiss").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn scenario_and_score_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = ClimateRepo::new(pool.clone());

        let scenario = repo
            .create_parent(
                tenant_id,
                "RCP 4.5 2050",
                "mid-range emissions",
                serde_json::json!({}),
            )
            .await
            .expect("create scenario");
        assert_eq!(scenario.status, "draft");

        let active = repo
            .activate_scenario(tenant_id, scenario.id)
            .await
            .expect("activate");
        assert_eq!(active.status, "active");
        assert!(active.activated_at.is_some());

        // Archive guard: a draft score blocks archiving.
        let score = repo
            .create_child(
                tenant_id,
                scenario.id,
                "Flood exposure",
                "region NE lowlands",
                serde_json::json!({}),
            )
            .await
            .expect("create score");
        assert_eq!(score.status, "draft");

        let blocked = repo.archive_scenario(tenant_id, scenario.id).await;
        assert!(blocked.is_err(), "archive blocked by draft score");

        let assessed = repo
            .assess_score(tenant_id, scenario.id, score.id)
            .await
            .expect("assess");
        assert_eq!(assessed.status, "assessed");
        assert!(assessed.assessed_at.is_some());

        // A second score is dismissed.
        let score2 = repo
            .create_child(
                tenant_id,
                scenario.id,
                "Heat stress",
                "",
                serde_json::json!({}),
            )
            .await
            .expect("create score2");
        let dismissed = repo
            .dismiss_score(tenant_id, scenario.id, score2.id)
            .await
            .expect("dismiss");
        assert_eq!(dismissed.status, "dismissed");
        assert!(dismissed.dismissed_at.is_some());

        // Summary reflects both scores.
        let summary = repo.get_climate_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == scenario.id).unwrap();
        assert_eq!(row.total_scores, 2);
        assert_eq!(row.assessed_scores, 1);
        assert_eq!(row.dismissed_scores, 1);

        // Archive succeeds now; reopen returns to active.
        let archived = repo
            .archive_scenario(tenant_id, scenario.id)
            .await
            .expect("archive");
        assert_eq!(archived.status, "archived");
        assert!(archived.archived_at.is_some());
        let reopened = repo
            .reopen_scenario(tenant_id, scenario.id)
            .await
            .expect("reopen");
        assert_eq!(reopened.status, "active");
        assert!(reopened.archived_at.is_none());

        // Updates.
        let renamed = repo
            .update_scenario(
                tenant_id,
                scenario.id,
                ScenarioUpdate {
                    name: Some("RCP 4.5 2050 (rev2)".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update scenario");
        assert_eq!(renamed.name, "RCP 4.5 2050 (rev2)");

        let score_updated = repo
            .update_score(
                tenant_id,
                scenario.id,
                score.id,
                ScoreUpdate {
                    body: Some("region NE lowlands + river delta".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update score");
        assert_eq!(score_updated.body, "region NE lowlands + river delta");

        // Score delete hides it; restore returns the pre-delete status.
        repo.soft_delete_score(tenant_id, scenario.id, score2.id)
            .await
            .expect("delete score2");
        let scores = repo
            .list_children(tenant_id, scenario.id)
            .await
            .expect("list scores after delete");
        assert!(scores.iter().all(|s| s.id != score2.id));
        let restored_score = repo
            .restore_score(tenant_id, scenario.id, score2.id)
            .await
            .expect("restore score2");
        assert_eq!(restored_score.status, "dismissed");

        // Scenario delete hides it; restore returns the pre-delete status.
        repo.soft_delete_scenario(tenant_id, scenario.id)
            .await
            .expect("delete scenario");
        let scenarios = repo
            .list_parents(tenant_id)
            .await
            .expect("list scenarios after delete");
        assert!(scenarios.iter().all(|s| s.id != scenario.id));
        let restored = repo
            .restore_scenario(tenant_id, scenario.id)
            .await
            .expect("restore scenario");
        assert_eq!(restored.status, "active");
        assert!(restored.deleted_at.is_none());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn scores_rejected_on_deleted_scenario() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = ClimateRepo::new(pool.clone());

        let scenario = repo
            .create_parent(tenant_id, "Doomed scenario", "", serde_json::json!({}))
            .await
            .expect("create scenario");
        repo.soft_delete_scenario(tenant_id, scenario.id)
            .await
            .expect("delete scenario");

        // 8 racing score creates on a soft-deleted scenario all fail.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.create_child(tenant_id, scenario.id, "leak", "", serde_json::json!({}))
                    .await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("create task panicked") {
                Ok(_) => panic!("score created on a deleted scenario"),
                Err(e) if e.code == shared_core::ErrorCode::NotFound => rejected += 1,
                Err(e) => panic!("unexpected create error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "all racing creates must be rejected");

        let scores = repo
            .list_children(tenant_id, scenario.id)
            .await
            .expect("list scores");
        assert_eq!(scores.len(), 0, "no score may leak onto a deleted scenario");
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_archive_single_winner() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = ClimateRepo::new(pool.clone());

        let scenario = repo
            .create_parent(tenant_id, "Race archive", "", serde_json::json!({}))
            .await
            .expect("create scenario");
        repo.activate_scenario(tenant_id, scenario.id)
            .await
            .expect("activate scenario");

        // 8 racing archives of one active scenario with no draft scores.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.archive_scenario(tenant_id, scenario.id).await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("archive task panicked") {
                Ok(_) => winners += 1,
                Err(e)
                    if e.code == shared_core::ErrorCode::Conflict
                        || e.code == shared_core::ErrorCode::Validation =>
                {
                    rejected += 1
                }
                Err(e) => panic!("unexpected archive error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing archive may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let scenarios = repo.list_parents(tenant_id).await.expect("list scenarios");
        let row = scenarios
            .iter()
            .find(|s| s.id == scenario.id)
            .expect("scenario listed");
        assert_eq!(row.status, "archived");
    }
}
