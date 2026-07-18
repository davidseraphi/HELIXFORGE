//! helix-synthbio API — durable store via helix_db.

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
}
