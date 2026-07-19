//! helix-nova-labs API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{DbPool, ExperimentUpdate, FindingUpdate, NovaRepo, NovaSummaryRow};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-nova-labs")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-nova-labs", 8120)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-nova-labs", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/experiments", get(list_parents).post(create_parent))
        .route(
            "/v1/experiments/{id}",
            get(get_parent).patch(update_experiment),
        )
        .route("/v1/experiments/{id}/start", post(start_experiment))
        .route("/v1/experiments/{id}/conclude", post(conclude_experiment))
        .route("/v1/experiments/{id}/reopen", post(reopen_experiment))
        .route("/v1/experiments/{id}/delete", post(delete_experiment))
        .route("/v1/experiments/{id}/restore", post(restore_experiment))
        .route(
            "/v1/experiments/{id}/findings",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/experiments/{id}/findings/{finding_id}",
            axum::routing::patch(update_finding),
        )
        .route(
            "/v1/experiments/{id}/findings/{finding_id}/confirm",
            post(confirm_finding),
        )
        .route(
            "/v1/experiments/{id}/findings/{finding_id}/reject",
            post(reject_finding),
        )
        .route(
            "/v1/experiments/{id}/findings/{finding_id}/delete",
            post(delete_finding),
        )
        .route(
            "/v1/experiments/{id}/findings/{finding_id}/restore",
            post(restore_finding),
        )
        .route("/v1/reports/nova-summary", get(nova_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-nova-labs",
        "phase": "wave2_w20",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "experiments": true,
            "findings": true,
            "experiment_lifecycle": true,
            "finding_lifecycle": true,
            "conclude_guards": true,
            "nova_summary": true,
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
            "helix-nova-labs",
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

// --- Experiments ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = NovaRepo::new(pool.clone());
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
    let repo = NovaRepo::new(pool);
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
        "experiment.create",
        "experiment",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "experiments.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = NovaRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("experiment not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateExperiment {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_experiment(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateExperiment>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = NovaRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_experiment(
            p.tenant_id,
            id,
            ExperimentUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "experiment.update",
        "experiment",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "experiments.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for experiment lifecycle transitions (start/conclude/reopen/delete/restore).
async fn experiment_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = NovaRepo::new(pool);
    let item = match action {
        "start" => repo.start_experiment(p.tenant_id, id).await?,
        "conclude" => repo.conclude_experiment(p.tenant_id, id).await?,
        "reopen" => repo.reopen_experiment(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_experiment(p.tenant_id, id).await?,
        "restore" => repo.restore_experiment(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown experiment action").into()),
    };
    audit(
        &state,
        &p,
        &format!("experiment.{action}"),
        "experiment",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "experiments.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.nova.experiment.lifecycle",
        serde_json::json!({
            "experiment_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn start_experiment(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    experiment_transition(state, p, id, "start").await
}

async fn conclude_experiment(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    experiment_transition(state, p, id, "conclude").await
}

async fn reopen_experiment(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    experiment_transition(state, p, id, "reopen").await
}

async fn delete_experiment(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    experiment_transition(state, p, id, "delete").await
}

async fn restore_experiment(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    experiment_transition(state, p, id, "restore").await
}

// --- Findings ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = NovaRepo::new(pool);
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
    let repo = NovaRepo::new(pool);
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
        "finding.create",
        "finding",
        item.id,
        serde_json::json!({"experiment_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "findings.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateFinding {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_finding(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, finding_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateFinding>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = NovaRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_finding(
            p.tenant_id,
            id,
            finding_id,
            FindingUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "finding.update",
        "finding",
        item.id,
        serde_json::json!({"experiment_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "findings.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for finding lifecycle transitions (confirm/reject/delete/restore).
async fn finding_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    finding_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = NovaRepo::new(pool);
    let item = match action {
        "confirm" => repo.confirm_finding(p.tenant_id, id, finding_id).await?,
        "reject" => repo.reject_finding(p.tenant_id, id, finding_id).await?,
        "delete" => {
            repo.soft_delete_finding(p.tenant_id, id, finding_id)
                .await?
        }
        "restore" => repo.restore_finding(p.tenant_id, id, finding_id).await?,
        _ => return Err(HelixError::validation("unknown finding action").into()),
    };
    audit(
        &state,
        &p,
        &format!("finding.{action}"),
        "finding",
        item.id,
        serde_json::json!({"experiment_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "findings.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.nova.finding.lifecycle",
        serde_json::json!({
            "experiment_id": id,
            "finding_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn confirm_finding(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, finding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    finding_transition(state, p, id, finding_id, "confirm").await
}

async fn reject_finding(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, finding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    finding_transition(state, p, id, finding_id, "reject").await
}

async fn delete_finding(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, finding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    finding_transition(state, p, id, finding_id, "delete").await
}

async fn restore_finding(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, finding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    finding_transition(state, p, id, finding_id, "restore").await
}

// --- Reports ---

async fn nova_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<NovaSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = NovaRepo::new(pool);
    let rows = repo.get_nova_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_experiment_status, next_finding_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18120");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-nova-labs").expect("helix-nova-labs product known");
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
    fn experiment_transitions_are_guarded() {
        assert_eq!(next_experiment_status("draft", "start").unwrap(), "running");
        assert_eq!(
            next_experiment_status("running", "conclude").unwrap(),
            "concluded"
        );
        assert_eq!(
            next_experiment_status("concluded", "reopen").unwrap(),
            "running"
        );
        assert!(next_experiment_status("running", "start").is_err());
        assert!(next_experiment_status("draft", "conclude").is_err());
        assert!(next_experiment_status("running", "reopen").is_err());
        assert!(next_experiment_status("deleted", "start").is_err());
    }

    #[test]
    fn finding_transitions_are_guarded() {
        assert_eq!(
            next_finding_status("draft", "confirm").unwrap(),
            "confirmed"
        );
        assert_eq!(next_finding_status("draft", "reject").unwrap(), "rejected");
        assert_eq!(
            next_finding_status("confirmed", "reject").unwrap(),
            "rejected"
        );
        assert!(next_finding_status("confirmed", "confirm").is_err());
        assert!(next_finding_status("rejected", "confirm").is_err());
        assert!(next_finding_status("rejected", "reject").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn experiment_and_finding_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = NovaRepo::new(pool.clone());

        let experiment = repo
            .create_parent(
                tenant_id,
                "Catalyst screen",
                "12-candidate panel",
                serde_json::json!({}),
            )
            .await
            .expect("create experiment");
        assert_eq!(experiment.status, "draft");

        let running = repo
            .start_experiment(tenant_id, experiment.id)
            .await
            .expect("start");
        assert_eq!(running.status, "running");
        assert!(running.started_at.is_some());

        // Conclude guard: a draft finding blocks conclusion.
        let finding = repo
            .create_child(
                tenant_id,
                experiment.id,
                "Candidate 4 yield",
                "71% at 60C",
                serde_json::json!({}),
            )
            .await
            .expect("create finding");
        assert_eq!(finding.status, "draft");

        let blocked = repo.conclude_experiment(tenant_id, experiment.id).await;
        assert!(blocked.is_err(), "conclude blocked by draft finding");

        let confirmed = repo
            .confirm_finding(tenant_id, experiment.id, finding.id)
            .await
            .expect("confirm");
        assert_eq!(confirmed.status, "confirmed");
        assert!(confirmed.confirmed_at.is_some());

        // A second finding is rejected.
        let finding2 = repo
            .create_child(
                tenant_id,
                experiment.id,
                "Candidate 9 anomaly",
                "",
                serde_json::json!({}),
            )
            .await
            .expect("create finding2");
        let rejected = repo
            .reject_finding(tenant_id, experiment.id, finding2.id)
            .await
            .expect("reject");
        assert_eq!(rejected.status, "rejected");
        assert!(rejected.rejected_at.is_some());

        // Summary reflects both findings.
        let summary = repo.get_nova_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == experiment.id).unwrap();
        assert_eq!(row.total_findings, 2);
        assert_eq!(row.confirmed_findings, 1);
        assert_eq!(row.rejected_findings, 1);

        // Conclude succeeds now; reopen returns to running.
        let concluded = repo
            .conclude_experiment(tenant_id, experiment.id)
            .await
            .expect("conclude");
        assert_eq!(concluded.status, "concluded");
        assert!(concluded.concluded_at.is_some());
        let reopened = repo
            .reopen_experiment(tenant_id, experiment.id)
            .await
            .expect("reopen");
        assert_eq!(reopened.status, "running");
        assert!(reopened.concluded_at.is_none());

        // Updates.
        let renamed = repo
            .update_experiment(
                tenant_id,
                experiment.id,
                ExperimentUpdate {
                    name: Some("Catalyst screen II".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update experiment");
        assert_eq!(renamed.name, "Catalyst screen II");

        let finding_updated = repo
            .update_finding(
                tenant_id,
                experiment.id,
                finding.id,
                FindingUpdate {
                    body: Some("73% at 60C".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update finding");
        assert_eq!(finding_updated.body, "73% at 60C");

        // Finding delete hides it; restore returns the pre-delete status.
        repo.soft_delete_finding(tenant_id, experiment.id, finding2.id)
            .await
            .expect("delete finding2");
        let findings = repo
            .list_children(tenant_id, experiment.id)
            .await
            .expect("list findings after delete");
        assert!(findings.iter().all(|f| f.id != finding2.id));
        let restored_finding = repo
            .restore_finding(tenant_id, experiment.id, finding2.id)
            .await
            .expect("restore finding2");
        assert_eq!(restored_finding.status, "rejected");

        // Experiment delete hides it; restore returns the pre-delete status.
        repo.soft_delete_experiment(tenant_id, experiment.id)
            .await
            .expect("delete experiment");
        let experiments = repo
            .list_parents(tenant_id)
            .await
            .expect("list experiments after delete");
        assert!(experiments.iter().all(|e| e.id != experiment.id));
        let restored = repo
            .restore_experiment(tenant_id, experiment.id)
            .await
            .expect("restore experiment");
        assert_eq!(restored.status, "running");
        assert!(restored.deleted_at.is_none());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn findings_rejected_on_deleted_experiment() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = NovaRepo::new(pool.clone());

        let experiment = repo
            .create_parent(tenant_id, "Doomed experiment", "", serde_json::json!({}))
            .await
            .expect("create experiment");
        repo.soft_delete_experiment(tenant_id, experiment.id)
            .await
            .expect("delete experiment");

        // 8 racing finding creates on a soft-deleted experiment all fail.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.create_child(tenant_id, experiment.id, "leak", "", serde_json::json!({}))
                    .await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("create task panicked") {
                Ok(_) => panic!("finding created on a deleted experiment"),
                Err(e) if e.code == shared_core::ErrorCode::NotFound => rejected += 1,
                Err(e) => panic!("unexpected create error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "all racing creates must be rejected");

        let findings = repo
            .list_children(tenant_id, experiment.id)
            .await
            .expect("list findings");
        assert_eq!(
            findings.len(),
            0,
            "no finding may leak onto a deleted experiment"
        );
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_conclude_single_winner() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = NovaRepo::new(pool.clone());

        let experiment = repo
            .create_parent(tenant_id, "Race conclude", "", serde_json::json!({}))
            .await
            .expect("create experiment");
        repo.start_experiment(tenant_id, experiment.id)
            .await
            .expect("start experiment");

        // 8 racing concludes of one running experiment with no draft findings.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.conclude_experiment(tenant_id, experiment.id).await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("conclude task panicked") {
                Ok(_) => winners += 1,
                Err(e)
                    if e.code == shared_core::ErrorCode::Conflict
                        || e.code == shared_core::ErrorCode::Validation =>
                {
                    rejected += 1
                }
                Err(e) => panic!("unexpected conclude error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing conclude may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let experiments = repo
            .list_parents(tenant_id)
            .await
            .expect("list experiments");
        let row = experiments
            .iter()
            .find(|e| e.id == experiment.id)
            .expect("experiment listed");
        assert_eq!(row.status, "concluded");
    }
}
