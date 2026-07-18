//! helix-quantum-forge API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{CircuitUpdate, DbPool, JobUpdate, QuantumRepo, QuantumSummaryRow};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-quantum-forge")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-quantum-forge", 8117)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-quantum-forge", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/jobs", get(list_parents).post(create_parent))
        .route("/v1/jobs/{id}", get(get_parent).patch(update_job))
        .route("/v1/jobs/{id}/submit", post(submit_job))
        .route("/v1/jobs/{id}/complete", post(complete_job))
        .route("/v1/jobs/{id}/fail", post(fail_job))
        .route("/v1/jobs/{id}/delete", post(delete_job))
        .route("/v1/jobs/{id}/restore", post(restore_job))
        .route(
            "/v1/jobs/{id}/circuits",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/jobs/{id}/circuits/{circuit_id}",
            axum::routing::patch(update_circuit),
        )
        .route(
            "/v1/jobs/{id}/circuits/{circuit_id}/validate",
            post(validate_circuit),
        )
        .route(
            "/v1/jobs/{id}/circuits/{circuit_id}/archive",
            post(archive_circuit),
        )
        .route(
            "/v1/jobs/{id}/circuits/{circuit_id}/delete",
            post(delete_circuit),
        )
        .route(
            "/v1/jobs/{id}/circuits/{circuit_id}/restore",
            post(restore_circuit),
        )
        .route("/v1/reports/quantum-summary", get(quantum_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-quantum-forge",
        "phase": "wave2_w17",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "jobs": true,
            "circuits": true,
            "job_lifecycle": true,
            "circuit_lifecycle": true,
            "submit_guards": true,
            "quantum_summary": true,
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
            "helix-quantum-forge",
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

// --- Jobs ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = QuantumRepo::new(pool.clone());
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
    let repo = QuantumRepo::new(pool);
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
        "job.create",
        "job",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "jobs.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = QuantumRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("job not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateJob {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_job(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateJob>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = QuantumRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_job(
            p.tenant_id,
            id,
            JobUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "job.update",
        "job",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "jobs.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for job lifecycle transitions (submit/complete/fail/delete/restore).
async fn job_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = QuantumRepo::new(pool);
    let item = match action {
        "submit" => repo.submit_job(p.tenant_id, id).await?,
        "complete" => repo.complete_job(p.tenant_id, id).await?,
        "fail" => repo.fail_job(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_job(p.tenant_id, id).await?,
        "restore" => repo.restore_job(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown job action").into()),
    };
    audit(
        &state,
        &p,
        &format!("job.{action}"),
        "job",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "jobs.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.quantum.job.lifecycle",
        serde_json::json!({
            "job_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn submit_job(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    job_transition(state, p, id, "submit").await
}

async fn complete_job(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    job_transition(state, p, id, "complete").await
}

async fn fail_job(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    job_transition(state, p, id, "fail").await
}

async fn delete_job(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    job_transition(state, p, id, "delete").await
}

async fn restore_job(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    job_transition(state, p, id, "restore").await
}

// --- Circuits ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = QuantumRepo::new(pool);
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
    let repo = QuantumRepo::new(pool);
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
        "circuit.create",
        "circuit",
        item.id,
        serde_json::json!({"job_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "circuits.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateCircuit {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_circuit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, circuit_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateCircuit>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = QuantumRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_circuit(
            p.tenant_id,
            id,
            circuit_id,
            CircuitUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "circuit.update",
        "circuit",
        item.id,
        serde_json::json!({"job_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "circuits.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for circuit lifecycle transitions (validate/archive/delete/restore).
async fn circuit_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    circuit_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = QuantumRepo::new(pool);
    let item = match action {
        "validate" => repo.validate_circuit(p.tenant_id, id, circuit_id).await?,
        "archive" => repo.archive_circuit(p.tenant_id, id, circuit_id).await?,
        "delete" => {
            repo.soft_delete_circuit(p.tenant_id, id, circuit_id)
                .await?
        }
        "restore" => repo.restore_circuit(p.tenant_id, id, circuit_id).await?,
        _ => return Err(HelixError::validation("unknown circuit action").into()),
    };
    audit(
        &state,
        &p,
        &format!("circuit.{action}"),
        "circuit",
        item.id,
        serde_json::json!({"job_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "circuits.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.quantum.circuit.lifecycle",
        serde_json::json!({
            "job_id": id,
            "circuit_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn validate_circuit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, circuit_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    circuit_transition(state, p, id, circuit_id, "validate").await
}

async fn archive_circuit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, circuit_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    circuit_transition(state, p, id, circuit_id, "archive").await
}

async fn delete_circuit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, circuit_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    circuit_transition(state, p, id, circuit_id, "delete").await
}

async fn restore_circuit(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, circuit_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    circuit_transition(state, p, id, circuit_id, "restore").await
}

// --- Reports ---

async fn quantum_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<QuantumSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = QuantumRepo::new(pool);
    let rows = repo.get_quantum_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_circuit_status, next_job_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18117");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-quantum-forge")
            .expect("helix-quantum-forge product known");
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
    fn job_transitions_are_guarded() {
        assert_eq!(next_job_status("draft", "submit").unwrap(), "submitted");
        assert_eq!(
            next_job_status("submitted", "complete").unwrap(),
            "completed"
        );
        assert_eq!(next_job_status("submitted", "fail").unwrap(), "failed");
        assert!(next_job_status("submitted", "submit").is_err());
        assert!(next_job_status("draft", "complete").is_err());
        assert!(next_job_status("completed", "fail").is_err());
        assert!(next_job_status("deleted", "submit").is_err());
    }

    #[test]
    fn circuit_transitions_are_guarded() {
        assert_eq!(
            next_circuit_status("draft", "validate").unwrap(),
            "validated"
        );
        assert_eq!(next_circuit_status("draft", "archive").unwrap(), "archived");
        assert_eq!(
            next_circuit_status("validated", "archive").unwrap(),
            "archived"
        );
        assert!(next_circuit_status("validated", "validate").is_err());
        assert!(next_circuit_status("archived", "validate").is_err());
        assert!(next_circuit_status("archived", "archive").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn job_and_circuit_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = QuantumRepo::new(pool.clone());

        let job = repo
            .create_parent(
                tenant_id,
                "Bell state sweep",
                "2-qubit bell pairs",
                serde_json::json!({}),
            )
            .await
            .expect("create job");
        assert_eq!(job.status, "draft");

        // Submit guard: a job with no circuits cannot be submitted.
        let too_early = repo.submit_job(tenant_id, job.id).await;
        assert!(too_early.is_err(), "submit requires a circuit");

        let circuit = repo
            .create_child(
                tenant_id,
                job.id,
                "bell-01",
                "H(0) CX(0,1)",
                serde_json::json!({}),
            )
            .await
            .expect("create circuit");
        assert_eq!(circuit.status, "draft");

        let submitted = repo.submit_job(tenant_id, job.id).await.expect("submit");
        assert_eq!(submitted.status, "submitted");
        assert!(submitted.submitted_at.is_some());

        let completed = repo
            .complete_job(tenant_id, job.id)
            .await
            .expect("complete");
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());

        // A second job fails.
        let job2 = repo
            .create_parent(tenant_id, "GHZ sweep", "", serde_json::json!({}))
            .await
            .expect("create job2");
        repo.create_child(tenant_id, job2.id, "ghz-01", "", serde_json::json!({}))
            .await
            .expect("create job2 circuit");
        repo.submit_job(tenant_id, job2.id)
            .await
            .expect("submit job2");
        let failed = repo.fail_job(tenant_id, job2.id).await.expect("fail job2");
        assert_eq!(failed.status, "failed");
        assert!(failed.failed_at.is_some());

        // Circuit validate + archive.
        let validated = repo
            .validate_circuit(tenant_id, job.id, circuit.id)
            .await
            .expect("validate");
        assert_eq!(validated.status, "validated");
        assert!(validated.validated_at.is_some());

        let circuit2 = repo
            .create_child(tenant_id, job.id, "bell-02", "", serde_json::json!({}))
            .await
            .expect("create circuit2");
        let archived = repo
            .archive_circuit(tenant_id, job.id, circuit2.id)
            .await
            .expect("archive");
        assert_eq!(archived.status, "archived");
        assert!(archived.archived_at.is_some());

        // Summary reflects both circuits.
        let summary = repo.get_quantum_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == job.id).unwrap();
        assert_eq!(row.total_circuits, 2);
        assert_eq!(row.validated_circuits, 1);
        assert_eq!(row.archived_circuits, 1);

        // Updates.
        let renamed = repo
            .update_job(
                tenant_id,
                job.id,
                JobUpdate {
                    name: Some("Bell state sweep v2".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update job");
        assert_eq!(renamed.name, "Bell state sweep v2");

        let circuit_updated = repo
            .update_circuit(
                tenant_id,
                job.id,
                circuit.id,
                CircuitUpdate {
                    body: Some("H(0) CX(0,1) M(0,1)".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update circuit");
        assert_eq!(circuit_updated.body, "H(0) CX(0,1) M(0,1)");

        // Circuit delete hides it; restore returns the pre-delete status.
        repo.soft_delete_circuit(tenant_id, job.id, circuit2.id)
            .await
            .expect("delete circuit2");
        let circuits = repo
            .list_children(tenant_id, job.id)
            .await
            .expect("list circuits after delete");
        assert!(circuits.iter().all(|c| c.id != circuit2.id));
        let restored_circuit = repo
            .restore_circuit(tenant_id, job.id, circuit2.id)
            .await
            .expect("restore circuit2");
        assert_eq!(restored_circuit.status, "archived");

        // Job delete hides it; restore returns the pre-delete status.
        repo.soft_delete_job(tenant_id, job2.id)
            .await
            .expect("delete job2");
        let jobs = repo
            .list_parents(tenant_id)
            .await
            .expect("list jobs after delete");
        assert!(jobs.iter().all(|j| j.id != job2.id));
        let restored = repo.restore_job(tenant_id, job2.id).await.expect("restore");
        assert_eq!(restored.status, "failed");
        assert!(restored.deleted_at.is_none());
    }
}
