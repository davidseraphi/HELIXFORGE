//! helix-orbit-prime API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{AssetUpdate, DbPool, OrbitRepo, OrbitSummaryRow, PassUpdate};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-orbit-prime")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-orbit-prime", 8116)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-orbit-prime", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/assets", get(list_parents).post(create_parent))
        .route("/v1/assets/{id}", get(get_parent).patch(update_asset))
        .route("/v1/assets/{id}/commission", post(commission_asset))
        .route("/v1/assets/{id}/decommission", post(decommission_asset))
        .route("/v1/assets/{id}/recommission", post(recommission_asset))
        .route("/v1/assets/{id}/delete", post(delete_asset))
        .route("/v1/assets/{id}/restore", post(restore_asset))
        .route(
            "/v1/assets/{id}/passes",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/assets/{id}/passes/{pass_id}",
            axum::routing::patch(update_pass),
        )
        .route("/v1/assets/{id}/passes/{pass_id}/plan", post(plan_pass))
        .route(
            "/v1/assets/{id}/passes/{pass_id}/complete",
            post(complete_pass),
        )
        .route("/v1/assets/{id}/passes/{pass_id}/cancel", post(cancel_pass))
        .route("/v1/assets/{id}/passes/{pass_id}/delete", post(delete_pass))
        .route(
            "/v1/assets/{id}/passes/{pass_id}/restore",
            post(restore_pass),
        )
        .route("/v1/reports/orbit-summary", get(orbit_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-orbit-prime",
        "phase": "wave2_w16",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "assets": true,
            "passes": true,
            "asset_lifecycle": true,
            "pass_lifecycle": true,
            "decommission_guards": true,
            "orbit_summary": true,
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
            "helix-orbit-prime",
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

// --- Assets ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = OrbitRepo::new(pool.clone());
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
    let repo = OrbitRepo::new(pool);
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
        "asset.create",
        "asset",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "assets.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = OrbitRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("asset not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateAsset {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_asset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAsset>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = OrbitRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_asset(
            p.tenant_id,
            id,
            AssetUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "asset.update",
        "asset",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "assets.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for asset lifecycle transitions (commission/decommission/recommission/delete/restore).
async fn asset_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = OrbitRepo::new(pool);
    let item = match action {
        "commission" => repo.commission_asset(p.tenant_id, id).await?,
        "decommission" => repo.decommission_asset(p.tenant_id, id).await?,
        "recommission" => repo.recommission_asset(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_asset(p.tenant_id, id).await?,
        "restore" => repo.restore_asset(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown asset action").into()),
    };
    audit(
        &state,
        &p,
        &format!("asset.{action}"),
        "asset",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "assets.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.orbit.asset.lifecycle",
        serde_json::json!({
            "asset_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn commission_asset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    asset_transition(state, p, id, "commission").await
}

async fn decommission_asset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    asset_transition(state, p, id, "decommission").await
}

async fn recommission_asset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    asset_transition(state, p, id, "recommission").await
}

async fn delete_asset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    asset_transition(state, p, id, "delete").await
}

async fn restore_asset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    asset_transition(state, p, id, "restore").await
}

// --- Passes ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = OrbitRepo::new(pool);
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
    let repo = OrbitRepo::new(pool);
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
        "pass.create",
        "pass",
        item.id,
        serde_json::json!({"asset_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "passes.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdatePass {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_pass(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, pass_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdatePass>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = OrbitRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_pass(
            p.tenant_id,
            id,
            pass_id,
            PassUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "pass.update",
        "pass",
        item.id,
        serde_json::json!({"asset_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "passes.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for pass lifecycle transitions (plan/complete/cancel/delete/restore).
async fn pass_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    pass_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = OrbitRepo::new(pool);
    let item = match action {
        "plan" => repo.plan_pass(p.tenant_id, id, pass_id).await?,
        "complete" => repo.complete_pass(p.tenant_id, id, pass_id).await?,
        "cancel" => repo.cancel_pass(p.tenant_id, id, pass_id).await?,
        "delete" => repo.soft_delete_pass(p.tenant_id, id, pass_id).await?,
        "restore" => repo.restore_pass(p.tenant_id, id, pass_id).await?,
        _ => return Err(HelixError::validation("unknown pass action").into()),
    };
    audit(
        &state,
        &p,
        &format!("pass.{action}"),
        "pass",
        item.id,
        serde_json::json!({"asset_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "passes.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.orbit.pass.lifecycle",
        serde_json::json!({
            "asset_id": id,
            "pass_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn plan_pass(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, pass_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    pass_transition(state, p, id, pass_id, "plan").await
}

async fn complete_pass(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, pass_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    pass_transition(state, p, id, pass_id, "complete").await
}

async fn cancel_pass(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, pass_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    pass_transition(state, p, id, pass_id, "cancel").await
}

async fn delete_pass(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, pass_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    pass_transition(state, p, id, pass_id, "delete").await
}

async fn restore_pass(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, pass_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    pass_transition(state, p, id, pass_id, "restore").await
}

// --- Reports ---

async fn orbit_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<OrbitSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = OrbitRepo::new(pool);
    let rows = repo.get_orbit_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_asset_status, next_pass_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18116");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-orbit-prime").expect("helix-orbit-prime product known");
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
    fn asset_transitions_are_guarded() {
        assert_eq!(next_asset_status("draft", "commission").unwrap(), "active");
        assert_eq!(
            next_asset_status("active", "decommission").unwrap(),
            "decommissioned"
        );
        assert_eq!(
            next_asset_status("decommissioned", "recommission").unwrap(),
            "active"
        );
        assert!(next_asset_status("active", "commission").is_err());
        assert!(next_asset_status("draft", "decommission").is_err());
        assert!(next_asset_status("active", "recommission").is_err());
        assert!(next_asset_status("deleted", "commission").is_err());
    }

    #[test]
    fn pass_transitions_are_guarded() {
        assert_eq!(next_pass_status("draft", "plan").unwrap(), "planned");
        assert_eq!(
            next_pass_status("planned", "complete").unwrap(),
            "completed"
        );
        assert_eq!(next_pass_status("draft", "cancel").unwrap(), "cancelled");
        assert_eq!(next_pass_status("planned", "cancel").unwrap(), "cancelled");
        assert!(next_pass_status("planned", "plan").is_err());
        assert!(next_pass_status("draft", "complete").is_err());
        assert!(next_pass_status("completed", "cancel").is_err());
        assert!(next_pass_status("cancelled", "complete").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn asset_and_pass_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = OrbitRepo::new(pool.clone());

        let asset = repo
            .create_parent(
                tenant_id,
                "HX-1",
                "earth observation cubesat",
                serde_json::json!({}),
            )
            .await
            .expect("create asset");
        assert_eq!(asset.status, "draft");

        let active = repo
            .commission_asset(tenant_id, asset.id)
            .await
            .expect("commission");
        assert_eq!(active.status, "active");
        assert!(active.commissioned_at.is_some());

        // Decommission guard: a draft pass blocks decommissioning.
        let pass = repo
            .create_child(
                tenant_id,
                asset.id,
                "Window 043",
                "ground station north",
                serde_json::json!({}),
            )
            .await
            .expect("create pass");
        assert_eq!(pass.status, "draft");

        let blocked = repo.decommission_asset(tenant_id, asset.id).await;
        assert!(blocked.is_err(), "decommission blocked by draft pass");

        let planned = repo
            .plan_pass(tenant_id, asset.id, pass.id)
            .await
            .expect("plan");
        assert_eq!(planned.status, "planned");
        assert!(planned.planned_at.is_some());

        let blocked2 = repo.decommission_asset(tenant_id, asset.id).await;
        assert!(blocked2.is_err(), "decommission blocked by planned pass");

        let completed = repo
            .complete_pass(tenant_id, asset.id, pass.id)
            .await
            .expect("complete");
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());

        // A second pass is cancelled.
        let pass2 = repo
            .create_child(tenant_id, asset.id, "Window 044", "", serde_json::json!({}))
            .await
            .expect("create pass2");
        let cancelled = repo
            .cancel_pass(tenant_id, asset.id, pass2.id)
            .await
            .expect("cancel");
        assert_eq!(cancelled.status, "cancelled");
        assert!(cancelled.cancelled_at.is_some());

        // Summary reflects both passes.
        let summary = repo.get_orbit_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == asset.id).unwrap();
        assert_eq!(row.total_passes, 2);
        assert_eq!(row.completed_passes, 1);
        assert_eq!(row.cancelled_passes, 1);

        // Decommission succeeds now; recommission returns to active.
        let decommissioned = repo
            .decommission_asset(tenant_id, asset.id)
            .await
            .expect("decommission");
        assert_eq!(decommissioned.status, "decommissioned");
        assert!(decommissioned.decommissioned_at.is_some());
        let recommissioned = repo
            .recommission_asset(tenant_id, asset.id)
            .await
            .expect("recommission");
        assert_eq!(recommissioned.status, "active");
        assert!(recommissioned.decommissioned_at.is_none());

        // Updates.
        let renamed = repo
            .update_asset(
                tenant_id,
                asset.id,
                AssetUpdate {
                    name: Some("HX-1R".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update asset");
        assert_eq!(renamed.name, "HX-1R");

        let pass_updated = repo
            .update_pass(
                tenant_id,
                asset.id,
                pass.id,
                PassUpdate {
                    body: Some("ground station north + backup".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update pass");
        assert_eq!(pass_updated.body, "ground station north + backup");

        // Pass delete hides it; restore returns the pre-delete status.
        repo.soft_delete_pass(tenant_id, asset.id, pass2.id)
            .await
            .expect("delete pass2");
        let passes = repo
            .list_children(tenant_id, asset.id)
            .await
            .expect("list passes after delete");
        assert!(passes.iter().all(|p| p.id != pass2.id));
        let restored_pass = repo
            .restore_pass(tenant_id, asset.id, pass2.id)
            .await
            .expect("restore pass2");
        assert_eq!(restored_pass.status, "cancelled");

        // Asset delete hides it; restore returns the pre-delete status.
        repo.soft_delete_asset(tenant_id, asset.id)
            .await
            .expect("delete asset");
        let assets = repo
            .list_parents(tenant_id)
            .await
            .expect("list assets after delete");
        assert!(assets.iter().all(|a| a.id != asset.id));
        let restored = repo
            .restore_asset(tenant_id, asset.id)
            .await
            .expect("restore asset");
        assert_eq!(restored.status, "active");
        assert!(restored.deleted_at.is_none());
    }
}
