//! helix-synthbio API — durable store via helix_db.

mod registry;

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{DbPool, DesignUpdate, SimUpdate, SynthbioRepo, SynthbioSummaryRow};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-synthbio")?;
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
        .merge(registry::routes())
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-synthbio", 8111)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-synthbio", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/designs", get(list_parents).post(create_parent))
        .route("/v1/designs/{id}", get(get_parent).patch(update_design))
        .route("/v1/designs/{id}/submit", post(submit_design))
        .route("/v1/designs/{id}/approve", post(approve_design))
        .route("/v1/designs/{id}/return", post(return_design))
        .route("/v1/designs/{id}/delete", post(delete_design))
        .route("/v1/designs/{id}/restore", post(restore_design))
        .route(
            "/v1/designs/{id}/sims",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/designs/{id}/sims/{sim_id}",
            axum::routing::patch(update_sim),
        )
        .route("/v1/designs/{id}/sims/{sim_id}/start", post(start_sim))
        .route(
            "/v1/designs/{id}/sims/{sim_id}/complete",
            post(complete_sim),
        )
        .route("/v1/designs/{id}/sims/{sim_id}/fail", post(fail_sim))
        .route("/v1/designs/{id}/sims/{sim_id}/delete", post(delete_sim))
        .route("/v1/designs/{id}/sims/{sim_id}/restore", post(restore_sim))
        .route("/v1/reports/synthbio-summary", get(synthbio_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-synthbio",
        "phase": "wave2_w11",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "designs": true,
            "sims": true,
            "design_lifecycle": true,
            "sim_lifecycle": true,
            "approval_guards": true,
            "synthbio_summary": true,
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
            "helix-synthbio",
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

// --- Designs ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = SynthbioRepo::new(pool.clone());
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
    let repo = SynthbioRepo::new(pool);
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
        "design.create",
        "design",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "designs.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = SynthbioRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("design not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateDesign {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_design(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateDesign>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = SynthbioRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_design(
            p.tenant_id,
            id,
            DesignUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "design.update",
        "design",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "designs.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for design lifecycle transitions (submit/approve/return/delete/restore).
async fn design_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = SynthbioRepo::new(pool);
    let item = match action {
        "submit" => repo.submit_design(p.tenant_id, id).await?,
        "approve" => repo.approve_design(p.tenant_id, id).await?,
        "return" => repo.return_design(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_design(p.tenant_id, id).await?,
        "restore" => repo.restore_design(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown design action").into()),
    };
    audit(
        &state,
        &p,
        &format!("design.{action}"),
        "design",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "designs.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.synthbio.design.lifecycle",
        serde_json::json!({
            "design_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn submit_design(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    design_transition(state, p, id, "submit").await
}

async fn approve_design(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    design_transition(state, p, id, "approve").await
}

async fn return_design(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    design_transition(state, p, id, "return").await
}

async fn delete_design(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    design_transition(state, p, id, "delete").await
}

async fn restore_design(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    design_transition(state, p, id, "restore").await
}

// --- Sims ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = SynthbioRepo::new(pool);
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
    let repo = SynthbioRepo::new(pool);
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
        "sim.create",
        "sim",
        item.id,
        serde_json::json!({"design_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "sims.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateSim {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_sim(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, sim_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateSim>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = SynthbioRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_sim(
            p.tenant_id,
            id,
            sim_id,
            SimUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "sim.update",
        "sim",
        item.id,
        serde_json::json!({"design_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "sims.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for sim lifecycle transitions (start/complete/fail/delete/restore).
async fn sim_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    sim_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = SynthbioRepo::new(pool);
    let item = match action {
        "start" => repo.start_sim(p.tenant_id, id, sim_id).await?,
        "complete" => repo.complete_sim(p.tenant_id, id, sim_id).await?,
        "fail" => repo.fail_sim(p.tenant_id, id, sim_id).await?,
        "delete" => repo.soft_delete_sim(p.tenant_id, id, sim_id).await?,
        "restore" => repo.restore_sim(p.tenant_id, id, sim_id).await?,
        _ => return Err(HelixError::validation("unknown sim action").into()),
    };
    audit(
        &state,
        &p,
        &format!("sim.{action}"),
        "sim",
        item.id,
        serde_json::json!({"design_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "sims.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.synthbio.sim.lifecycle",
        serde_json::json!({
            "design_id": id,
            "sim_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn start_sim(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, sim_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    sim_transition(state, p, id, sim_id, "start").await
}

async fn complete_sim(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, sim_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    sim_transition(state, p, id, sim_id, "complete").await
}

async fn fail_sim(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, sim_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    sim_transition(state, p, id, sim_id, "fail").await
}

async fn delete_sim(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, sim_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    sim_transition(state, p, id, sim_id, "delete").await
}

async fn restore_sim(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, sim_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    sim_transition(state, p, id, sim_id, "restore").await
}

// --- Reports ---

async fn synthbio_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<SynthbioSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = SynthbioRepo::new(pool);
    let rows = repo.get_synthbio_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_design_status, next_sim_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18111");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-synthbio").expect("helix-synthbio product known");
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
    fn design_transitions_are_guarded() {
        assert_eq!(next_design_status("draft", "submit").unwrap(), "review");
        assert_eq!(next_design_status("review", "approve").unwrap(), "approved");
        assert_eq!(next_design_status("review", "return").unwrap(), "draft");
        assert!(next_design_status("draft", "approve").is_err());
        assert!(next_design_status("approved", "submit").is_err());
        assert!(next_design_status("draft", "return").is_err());
        assert!(next_design_status("deleted", "submit").is_err());
    }

    #[test]
    fn sim_transitions_are_guarded() {
        assert_eq!(next_sim_status("open", "start").unwrap(), "running");
        assert_eq!(next_sim_status("running", "complete").unwrap(), "completed");
        assert_eq!(next_sim_status("running", "fail").unwrap(), "failed");
        assert!(next_sim_status("open", "complete").is_err());
        assert!(next_sim_status("completed", "start").is_err());
        assert!(next_sim_status("failed", "complete").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn design_and_sim_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = SynthbioRepo::new(pool.clone());

        let design = repo
            .create_parent(
                tenant_id,
                "Promoter study",
                "weak promoter variants",
                serde_json::json!({}),
            )
            .await
            .expect("create design");
        assert_eq!(design.status, "draft");

        // Approval guard: a design with no completed sim cannot be approved.
        repo.submit_design(tenant_id, design.id)
            .await
            .expect("submit");
        let too_early = repo.approve_design(tenant_id, design.id).await;
        assert!(too_early.is_err(), "approve requires a completed sim");

        // Sim run lifecycle.
        let sim = repo
            .create_child(
                tenant_id,
                design.id,
                "growth curve",
                "37C 24h",
                serde_json::json!({}),
            )
            .await
            .expect("create sim");
        assert_eq!(sim.status, "open");

        let early_complete = repo.complete_sim(tenant_id, design.id, sim.id).await;
        assert!(early_complete.is_err(), "cannot complete before start");

        let running = repo
            .start_sim(tenant_id, design.id, sim.id)
            .await
            .expect("start sim");
        assert_eq!(running.status, "running");
        assert!(running.started_at.is_some());

        let completed = repo
            .complete_sim(tenant_id, design.id, sim.id)
            .await
            .expect("complete sim");
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());

        // A second sim fails.
        let sim2 = repo
            .create_child(
                tenant_id,
                design.id,
                "toxicity screen",
                "",
                serde_json::json!({}),
            )
            .await
            .expect("create sim2");
        repo.start_sim(tenant_id, design.id, sim2.id)
            .await
            .expect("start sim2");
        let failed = repo
            .fail_sim(tenant_id, design.id, sim2.id)
            .await
            .expect("fail sim2");
        assert_eq!(failed.status, "failed");
        assert!(failed.failed_at.is_some());

        // Summary reflects both outcomes.
        let summary = repo.get_synthbio_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == design.id).unwrap();
        assert_eq!(row.total_sims, 2);
        assert_eq!(row.completed_sims, 1);
        assert_eq!(row.failed_sims, 1);

        // Approve now that a completed sim exists.
        let approved = repo
            .approve_design(tenant_id, design.id)
            .await
            .expect("approve");
        assert_eq!(approved.status, "approved");
        assert!(approved.approved_at.is_some());

        // Return path: submit -> return -> draft.
        let design2 = repo
            .create_parent(tenant_id, "Assay panel", "", serde_json::json!({}))
            .await
            .expect("create design2");
        repo.submit_design(tenant_id, design2.id)
            .await
            .expect("submit design2");
        let returned = repo
            .return_design(tenant_id, design2.id)
            .await
            .expect("return design2");
        assert_eq!(returned.status, "draft");

        // Design update.
        let renamed = repo
            .update_design(
                tenant_id,
                design.id,
                DesignUpdate {
                    name: Some("Promoter study v2".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update design");
        assert_eq!(renamed.name, "Promoter study v2");

        // Sim update.
        let sim_updated = repo
            .update_sim(
                tenant_id,
                design.id,
                sim.id,
                SimUpdate {
                    body: Some("37C 24h + controls".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update sim");
        assert_eq!(sim_updated.body, "37C 24h + controls");

        // Sim delete hides it; restore returns the pre-delete status.
        repo.soft_delete_sim(tenant_id, design.id, sim2.id)
            .await
            .expect("delete sim2");
        let sims = repo
            .list_children(tenant_id, design.id)
            .await
            .expect("list sims after delete");
        assert!(sims.iter().all(|s| s.id != sim2.id));
        let restored_sim = repo
            .restore_sim(tenant_id, design.id, sim2.id)
            .await
            .expect("restore sim2");
        assert_eq!(restored_sim.status, "failed");

        // Design delete hides it; restore returns the pre-delete status.
        repo.soft_delete_design(tenant_id, design.id)
            .await
            .expect("delete design");
        let designs = repo
            .list_parents(tenant_id)
            .await
            .expect("list designs after delete");
        assert!(designs.iter().all(|d| d.id != design.id));
        let restored = repo
            .restore_design(tenant_id, design.id)
            .await
            .expect("restore design");
        assert_eq!(restored.status, "approved");
        assert!(restored.deleted_at.is_none());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn sims_rejected_on_deleted_design() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = SynthbioRepo::new(pool.clone());

        let design = repo
            .create_parent(tenant_id, "Doomed design", "", serde_json::json!({}))
            .await
            .expect("create design");
        repo.soft_delete_design(tenant_id, design.id)
            .await
            .expect("delete design");

        // 8 racing sim creates on a soft-deleted design all fail.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.create_child(tenant_id, design.id, "leak", "", serde_json::json!({}))
                    .await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("create task panicked") {
                Ok(_) => panic!("sim created on a deleted design"),
                Err(e) if e.code == shared_core::ErrorCode::NotFound => rejected += 1,
                Err(e) => panic!("unexpected create error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "all racing creates must be rejected");

        let sims = repo
            .list_children(tenant_id, design.id)
            .await
            .expect("list sims");
        assert_eq!(sims.len(), 0, "no sim may leak onto a deleted design");
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_approve_single_winner() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = SynthbioRepo::new(pool.clone());

        let design = repo
            .create_parent(tenant_id, "Race approve", "", serde_json::json!({}))
            .await
            .expect("create design");
        repo.submit_design(tenant_id, design.id)
            .await
            .expect("submit");
        let sim = repo
            .create_child(tenant_id, design.id, "sim", "", serde_json::json!({}))
            .await
            .expect("create sim");
        repo.start_sim(tenant_id, design.id, sim.id)
            .await
            .expect("start sim");
        repo.complete_sim(tenant_id, design.id, sim.id)
            .await
            .expect("complete sim");

        // 8 racing approves of one in-review design.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.approve_design(tenant_id, design.id).await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("approve task panicked") {
                Ok(_) => winners += 1,
                Err(e)
                    if e.code == shared_core::ErrorCode::Conflict
                        || e.code == shared_core::ErrorCode::Validation =>
                {
                    rejected += 1
                }
                Err(e) => panic!("unexpected approve error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing approve may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let designs = repo.list_parents(tenant_id).await.expect("list designs");
        let row = designs
            .iter()
            .find(|d| d.id == design.id)
            .expect("design listed");
        assert_eq!(row.status, "approved");
    }

    // ——— Registry (Benchling-grade) ———

    fn registry_input(seq: &str) -> helix_db::VersionInput {
        helix_db::VersionInput {
            alphabet: "dna".into(),
            topology: "circular".into(),
            source_kind: "manual".into(),
            source_name: "bench".into(),
            sequence_text: seq.into(),
            components: vec![helix_db::Component {
                name: "prA".into(),
                role_so: "SO:0000167".into(),
                start: 10,
                end: 80,
                strand: -1,
                source: "manual".into(),
            }],
            provenance: "depositor-claimed".into(),
            notes: String::new(),
        }
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn registry_accession_versions_immutable() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = helix_db::RegistryRepo::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let design = repo
            .create_design(
                tenant_id,
                &format!("pRegistry-{suffix}"),
                "gate proof",
                "internal",
                &registry_input("ACGTACGTACGTACGT"),
                "tester",
            )
            .await
            .expect("create design");
        assert!(design.accession.starts_with("DSN-"), "{}", design.accession);
        assert_eq!(design.current_version, 1);

        let v2 = repo
            .add_version(
                tenant_id,
                design.id,
                &registry_input("ACGTACGTACGTACGTGGGG"),
                "tester",
            )
            .await
            .expect("add version");
        assert_eq!(v2.version, 2);

        let view = repo
            .design_360(tenant_id, design.id)
            .await
            .expect("360")
            .expect("design exists");
        assert_eq!(view.design.current_version, 2);
        assert_eq!(view.versions.len(), 2);
        assert!(view
            .edges
            .iter()
            .any(|e| e.relation == "derived-from" && e.child_id == v2.id));
        assert!(view.events.iter().any(|e| e.event_kind == "versioned"));
        assert_ne!(view.versions[0].content_hash, view.versions[1].content_hash);

        // DB-enforced immutability: no update path exists, and a raw UPDATE
        // must be rejected by the trigger.
        let err =
            sqlx::query("UPDATE synthbio.design_versions SET notes = 'tampered' WHERE id = $1")
                .bind(v2.id)
                .execute(pool)
                .await
                .expect_err("immutable trigger must reject UPDATE");
        assert!(
            err.to_string().contains("immutable record"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn registry_concurrent_accession_distinct() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = helix_db::RegistryRepo::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let mut handles = Vec::new();
        for i in 0..16u32 {
            let repo = repo.clone();
            let suffix = suffix.clone();
            handles.push(tokio::spawn(async move {
                repo.create_design(
                    tenant_id,
                    &format!("pRace-{suffix}-{i}"),
                    "",
                    "internal",
                    &registry_input("ACGTACGT"),
                    "tester",
                )
                .await
            }));
        }
        let mut accessions = std::collections::HashSet::new();
        for h in handles {
            let d = h
                .await
                .expect("create task panicked")
                .expect("create design");
            assert!(
                accessions.insert(d.accession.clone()),
                "duplicate accession {}",
                d.accession
            );
        }
        assert_eq!(accessions.len(), 16);
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn registry_risk_review_guards() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = helix_db::RegistryRepo::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let design = repo
            .create_design(
                tenant_id,
                &format!("pRisk-{suffix}"),
                "",
                "internal",
                &registry_input("ACGTACGT"),
                "tester",
            )
            .await
            .expect("create design");

        // Fresh case is unknown; unknown is never safe.
        let case = repo
            .ensure_risk_case(tenant_id, design.id)
            .await
            .expect("ensure case");
        assert_eq!(case.state, "unknown");

        // A decision without a named reviewer is refused.
        let err = repo
            .review_risk(
                tenant_id,
                design.id,
                &helix_db::ReviewDecision {
                    state: "allowed".into(),
                    intended_use: "bench research".into(),
                    policy_version: "v1".into(),
                    reasons: vec![],
                    conditions: String::new(),
                    expires_at: None,
                    expected_state: None,
                },
                "",
            )
            .await
            .expect_err("reviewer required");
        assert_eq!(err.code, shared_core::ErrorCode::Validation);

        // 8 racing decisions pinned to the state they saw (unknown):
        // exactly one lands; the rest conflict instead of overwriting.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.review_risk(
                    tenant_id,
                    design.id,
                    &helix_db::ReviewDecision {
                        state: "allowed".into(),
                        intended_use: "bench research".into(),
                        policy_version: "v1".into(),
                        reasons: vec!["public backbone".into()],
                        conditions: String::new(),
                        expires_at: None,
                        expected_state: Some("unknown".into()),
                    },
                    "Dr. Ada Biosafety",
                )
                .await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("review task panicked") {
                Ok(_) => winners += 1,
                Err(e)
                    if e.code == shared_core::ErrorCode::Conflict
                        || e.code == shared_core::ErrorCode::Validation =>
                {
                    rejected += 1
                }
                Err(e) => panic!("unexpected review error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing decision may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let case = repo
            .get_risk_case(tenant_id, design.id)
            .await
            .expect("get case")
            .expect("case exists");
        assert_eq!(case.state, "allowed");
        assert_eq!(case.reviewer.as_deref(), Some("Dr. Ada Biosafety"));
        assert!(case.design_version_id.is_some(), "decision pins a version");

        // An expired decision decays to unknown in the 360 view.
        let case2 = repo
            .review_risk(
                tenant_id,
                design.id,
                &helix_db::ReviewDecision {
                    state: "restricted".into(),
                    intended_use: "bench research".into(),
                    policy_version: "v1".into(),
                    reasons: vec![],
                    conditions: String::new(),
                    expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
                    expected_state: None,
                },
                "Dr. Ada Biosafety",
            )
            .await
            .expect("re-review");
        assert_eq!(case2.state, "restricted");
        let view = repo
            .design_360(tenant_id, design.id)
            .await
            .expect("360")
            .expect("design exists");
        assert_eq!(view.effective_risk, "unknown", "expired decays to unknown");
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn registry_import_quarantine_manifest() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = helix_db::RegistryRepo::new(pool.clone());

        let fixture = r#"LOCUS       pGOOD-001              120 bp    DNA     circular SYN 19-JUL-2026
DEFINITION  Good plasmid.
ACCESSION   pGOOD-001
FEATURES             Location/Qualifiers
     source          1..120
     promoter        complement(10..80)
                     /gene="prA"
     CDS             join(100..300,500..900)
                     /product="demo enzyme"
ORIGIN
        1 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
       61 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
//
LOCUS       pGOOD-002              120 bp    DNA     linear   SYN 19-JUL-2026
DEFINITION  Second good record.
ACCESSION   pGOOD-002
FEATURES             Location/Qualifiers
     source          1..120
ORIGIN
        1 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
       61 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
//
LOCUS       pBAD                   999 bp    DNA     linear   SYN 19-JUL-2026
DEFINITION  Broken record (length lie).
ACCESSION   pBAD
ORIGIN
        1 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
       61 acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt acgtacgtac gtacgtacgt
//
"#;
        let manifest = repo
            .import_records(tenant_id, "genbank", fixture, "tester")
            .await
            .expect("import");
        assert_eq!(manifest.total_records, 3);
        assert_eq!(manifest.accepted_count, 2);
        assert_eq!(manifest.rejected_count, 1);
        assert_eq!(
            manifest.accepted_count + manifest.rejected_count,
            manifest.total_records,
            "accepted + rejected must sum to input"
        );
        assert_eq!(manifest.rejected[0].record, "pBAD");
        assert!(manifest.rejected[0].reason.contains("999"));

        let good = &manifest.accepted[0];
        assert!(good.accession.starts_with("DSN-"));
        let view = repo
            .design_360(tenant_id, good.id)
            .await
            .expect("360")
            .expect("design exists");
        let v = &view.versions[0];
        assert_eq!(v.source_kind, "genbank");
        assert_eq!(v.sequence_length, 120);
        assert_eq!(v.provenance, "depositor-claimed");
        let comps = v.components.as_array().expect("components array");
        assert_eq!(comps.len(), 2, "promoter + CDS become components");
        assert!(comps.iter().any(|c| c["role_so"] == "SO:0000316"));
        assert!(comps
            .iter()
            .any(|c| c["role_so"] == "SO:0000167" && c["strand"] == -1));
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn inventory_sample_custody_lineage() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = helix_db::RegistryRepo::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let design = repo
            .create_design(
                tenant_id,
                &format!("pInv-{suffix}"),
                "",
                "internal",
                &registry_input("ACGTACGT"),
                "tester",
            )
            .await
            .expect("create design");

        let sample = repo
            .register_sample(
                tenant_id,
                &format!("prep-{suffix}"),
                "plasmid_prep",
                Some(design.id),
                "freezer-A/1-B",
                "tester",
            )
            .await
            .expect("register sample");
        assert!(sample.accession.starts_with("SMP-"), "{}", sample.accession);
        assert_eq!(sample.location, "freezer-A/1-B");

        // Custody and location move in one transaction.
        let moved = repo
            .custody_event(
                tenant_id,
                sample.id,
                "transfer",
                "bench-2",
                "tester",
                "to bench for digestion",
            )
            .await
            .expect("transfer");
        assert_eq!(moved.location, "bench-2");

        let detail = repo
            .sample_detail(tenant_id, sample.id)
            .await
            .expect("detail")
            .expect("sample exists");
        assert_eq!(
            detail.design_accession.as_deref(),
            Some(design.accession.as_str())
        );
        assert_eq!(detail.custody.len(), 2);
        assert_eq!(detail.custody[0].event, "register");
        assert_eq!(detail.custody[1].event, "transfer");
        assert_eq!(detail.custody[1].from_location, "freezer-A/1-B");
        assert_eq!(detail.custody[1].to_location, "bench-2");
        assert!(detail.edges.iter().any(|e| e.relation == "produces"));

        // Aliquot carries lineage.
        let child = repo
            .aliquot(tenant_id, sample.id, &format!("aliquot-{suffix}"), "tester")
            .await
            .expect("aliquot");
        let child_detail = repo
            .sample_detail(tenant_id, child.id)
            .await
            .expect("child detail")
            .expect("child exists");
        assert!(child_detail
            .edges
            .iter()
            .any(|e| e.relation == "derived-from"));
        assert_eq!(child_detail.sample.design_id, Some(design.id));

        // Custody is append-only: a raw UPDATE must be rejected by the trigger.
        let err = sqlx::query(
            "UPDATE synthbio.custody_events SET notes = 'tampered' WHERE sample_id = $1",
        )
        .bind(sample.id)
        .execute(pool)
        .await
        .expect_err("immutable custody trigger must reject UPDATE");
        assert!(
            err.to_string().contains("immutable record"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn inventory_concurrent_custody_serialized() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = helix_db::RegistryRepo::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let sample = repo
            .register_sample(
                tenant_id,
                &format!("race-{suffix}"),
                "strain",
                None,
                "origin",
                "tester",
            )
            .await
            .expect("register");

        // 8 racing custody events: all land in the ledger (it is append-only),
        // and the sample's location equals the LAST committed event's target.
        let mut handles = Vec::new();
        for i in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo
                    .custody_event(tenant_id, sample.id, "transfer", &format!("loc-{i}"), "tester", "")
                    .await
            }));
        }
        let mut oks = 0usize;
        for h in handles {
            match h.await.expect("custody task panicked") {
                Ok(_) => oks += 1,
                Err(e) => panic!("unexpected custody error: {e}"),
            }
        }
        assert_eq!(oks, 8, "every serialized custody event lands");

        let detail = repo
            .sample_detail(tenant_id, sample.id)
            .await
            .expect("detail")
            .expect("sample exists");
        assert_eq!(detail.custody.len(), 1 + 8);
        let last = detail.custody.last().expect("last custody event");
        assert_eq!(
            detail.sample.location, last.to_location,
            "sample location must equal the last committed custody target"
        );
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn measurement_guards_and_verdict() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = helix_db::RegistryRepo::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let sample = repo
            .register_sample(
                tenant_id,
                &format!("meas-{suffix}"),
                "strain",
                None,
                "incubator-37",
                "tester",
            )
            .await
            .expect("register sample");

        // Value-less and raw-less is refused.
        let err = repo
            .record_measurement(
                tenant_id,
                &helix_db::MeasurementInput {
                    sample_id: sample.id,
                    design_version_id: None,
                    kind: "absorbance".into(),
                    method: "plate reader".into(),
                    value: None,
                    unit: "AU".into(),
                    uncertainty: None,
                    raw: serde_json::json!({}),
                },
                "tester",
            )
            .await
            .expect_err("empty measurement refused");
        assert_eq!(err.code, shared_core::ErrorCode::Validation);

        let m = repo
            .record_measurement(
                tenant_id,
                &helix_db::MeasurementInput {
                    sample_id: sample.id,
                    design_version_id: None,
                    kind: "absorbance".into(),
                    method: "plate reader".into(),
                    value: Some(0.42),
                    unit: "AU".into(),
                    uncertainty: Some(0.01),
                    raw: serde_json::json!({"plate": "A1"}),
                },
                "tester",
            )
            .await
            .expect("record measurement");
        assert!(m.accession.starts_with("MSR-"));
        assert_eq!(m.status, "draft");
        assert_eq!(m.value, Some(0.42));
        assert_eq!(m.uncertainty, Some(0.01));

        let detail = repo
            .sample_detail(tenant_id, sample.id)
            .await
            .expect("detail")
            .expect("sample exists");
        assert!(detail.edges.iter().any(|e| e.relation == "measured"));

        // Deleted samples leak nothing.
        let doomed = repo
            .register_sample(tenant_id, &format!("doomed-{suffix}"), "oligo", None, "", "tester")
            .await
            .expect("register doomed");
        sqlx::query("UPDATE synthbio.samples SET deleted_at = now() WHERE id = $1")
            .bind(doomed.id)
            .execute(pool)
            .await
            .expect("soft delete");
        let err = repo
            .record_measurement(
                tenant_id,
                &helix_db::MeasurementInput {
                    sample_id: doomed.id,
                    design_version_id: None,
                    kind: "gel".into(),
                    method: String::new(),
                    value: Some(1.0),
                    unit: "kb".into(),
                    uncertainty: None,
                    raw: serde_json::json!({}),
                },
                "tester",
            )
            .await
            .expect_err("deleted sample rejects measurement");
        assert_eq!(err.code, shared_core::ErrorCode::NotFound);

        // 8 racing verdicts: exactly one wins.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.transition_measurement(tenant_id, m.id, "accept", "analyst")
                    .await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("verdict task panicked") {
                Ok(_) => winners += 1,
                Err(e) if e.code == shared_core::ErrorCode::Conflict => rejected += 1,
                Err(e) => panic!("unexpected verdict error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing verdict may win");
        assert_eq!(rejected, 7, "all losers must conflict");

        let err = repo
            .transition_measurement(tenant_id, m.id, "reject", "analyst")
            .await
            .expect_err("accepted measurement is terminal");
        assert_eq!(err.code, shared_core::ErrorCode::Conflict);
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn claims_evidence_attest_challenge() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = helix_db::RegistryRepo::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let design = repo
            .create_design(
                tenant_id,
                &format!("pClaim-{suffix}"),
                "",
                "internal",
                &registry_input("ACGTACGT"),
                "tester",
            )
            .await
            .expect("create design");
        let sample = repo
            .register_sample(tenant_id, &format!("cs-{suffix}"), "strain", None, "", "tester")
            .await
            .expect("register");
        let measurement = repo
            .record_measurement(
                tenant_id,
                &helix_db::MeasurementInput {
                    sample_id: sample.id,
                    design_version_id: None,
                    kind: "absorbance".into(),
                    method: "plate".into(),
                    value: Some(1.0),
                    unit: "AU".into(),
                    uncertainty: None,
                    raw: serde_json::json!({}),
                },
                "tester",
            )
            .await
            .expect("record");

        let claim = repo
            .create_claim(
                tenant_id,
                design.id,
                "The construct expresses demo enzyme at useful levels",
                "tester",
            )
            .await
            .expect("create claim");
        assert!(claim.accession.starts_with("CLM-"));
        assert_eq!(claim.status, "draft");

        let link = repo
            .link_evidence(
                tenant_id,
                claim.id,
                "measurement",
                measurement.id,
                "supports",
                "A1 well shows expression",
                "tester",
            )
            .await
            .expect("link evidence");
        assert_eq!(link.support, "supports");

        let err = repo
            .link_evidence(
                tenant_id,
                claim.id,
                "measurement",
                Uuid::now_v7(),
                "conflicts",
                "",
                "tester",
            )
            .await
            .expect_err("missing target refused");
        assert_eq!(err.code, shared_core::ErrorCode::NotFound);

        // Evidence links are append-only: raw UPDATE rejected by trigger.
        let err = sqlx::query("UPDATE synthbio.evidence_links SET note = 'x' WHERE id = $1")
            .bind(link.id)
            .execute(pool)
            .await
            .expect_err("immutable evidence trigger must reject UPDATE");
        assert!(err.to_string().contains("immutable record"), "{err}");

        // 8 racing attestations: exactly one wins.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.attest_claim(tenant_id, claim.id, "Dr. Ada PI").await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("attest task panicked") {
                Ok(_) => winners += 1,
                Err(e) if e.code == shared_core::ErrorCode::Conflict => rejected += 1,
                Err(e) => panic!("unexpected attest error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing attestation may win");
        assert_eq!(rejected, 7, "all losers must conflict");

        // Challenge after acceptance keeps history.
        let challenged = repo
            .challenge_claim(tenant_id, claim.id, "new conflicting plate", "tester")
            .await
            .expect("challenge");
        assert_eq!(challenged.status, "challenged");
        assert_eq!(challenged.attested_by.as_deref(), Some("Dr. Ada PI"));

        let claims = repo.list_claims(tenant_id, design.id).await.expect("list");
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].evidence.len(), 1);
        assert_eq!(claims[0].claim.status, "challenged");

        // ELN notes append-only.
        repo.add_note(tenant_id, design.id, "first bench note", "tester")
            .await
            .expect("note");
        let err = sqlx::query("UPDATE synthbio.notes SET body = 'edited' WHERE design_id = $1")
            .bind(design.id)
            .execute(pool)
            .await
            .expect_err("immutable notes trigger must reject UPDATE");
        assert!(err.to_string().contains("immutable record"), "{err}");
        let notes = repo.list_notes(tenant_id, design.id).await.expect("notes");
        assert_eq!(notes.len(), 1);
    }
}
