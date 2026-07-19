//! helix-vita-prime API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{CohortUpdate, DbPool, StudyUpdate, VitaRepo, VitaSummaryRow};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-vita-prime")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-vita-prime", 8118)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-vita-prime", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/studies", get(list_parents).post(create_parent))
        .route("/v1/studies/{id}", get(get_parent).patch(update_study))
        .route("/v1/studies/{id}/recruit", post(recruit_study))
        .route("/v1/studies/{id}/complete", post(complete_study))
        .route("/v1/studies/{id}/terminate", post(terminate_study))
        .route("/v1/studies/{id}/delete", post(delete_study))
        .route("/v1/studies/{id}/restore", post(restore_study))
        .route(
            "/v1/studies/{id}/cohorts",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/studies/{id}/cohorts/{cohort_id}",
            axum::routing::patch(update_cohort),
        )
        .route(
            "/v1/studies/{id}/cohorts/{cohort_id}/enroll",
            post(enroll_cohort),
        )
        .route(
            "/v1/studies/{id}/cohorts/{cohort_id}/withdraw",
            post(withdraw_cohort),
        )
        .route(
            "/v1/studies/{id}/cohorts/{cohort_id}/delete",
            post(delete_cohort),
        )
        .route(
            "/v1/studies/{id}/cohorts/{cohort_id}/restore",
            post(restore_cohort),
        )
        .route("/v1/reports/vita-summary", get(vita_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-vita-prime",
        "phase": "wave2_w18",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "studies": true,
            "cohorts": true,
            "study_lifecycle": true,
            "cohort_lifecycle": true,
            "complete_guards": true,
            "vita_summary": true,
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
            "helix-vita-prime",
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

// --- Studies ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = VitaRepo::new(pool.clone());
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
    let repo = VitaRepo::new(pool);
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
        "study.create",
        "study",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "studies.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = VitaRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("study not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateStudy {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_study(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStudy>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = VitaRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_study(
            p.tenant_id,
            id,
            StudyUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "study.update",
        "study",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "studies.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for study lifecycle transitions (recruit/complete/terminate/delete/restore).
async fn study_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = VitaRepo::new(pool);
    let item = match action {
        "recruit" => repo.recruit_study(p.tenant_id, id).await?,
        "complete" => repo.complete_study(p.tenant_id, id).await?,
        "terminate" => repo.terminate_study(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_study(p.tenant_id, id).await?,
        "restore" => repo.restore_study(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown study action").into()),
    };
    audit(
        &state,
        &p,
        &format!("study.{action}"),
        "study",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "studies.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.vita.study.lifecycle",
        serde_json::json!({
            "study_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn recruit_study(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    study_transition(state, p, id, "recruit").await
}

async fn complete_study(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    study_transition(state, p, id, "complete").await
}

async fn terminate_study(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    study_transition(state, p, id, "terminate").await
}

async fn delete_study(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    study_transition(state, p, id, "delete").await
}

async fn restore_study(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    study_transition(state, p, id, "restore").await
}

// --- Cohorts ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = VitaRepo::new(pool);
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
    let repo = VitaRepo::new(pool);
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
        "cohort.create",
        "cohort",
        item.id,
        serde_json::json!({"study_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "cohorts.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateCohort {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_cohort(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, cohort_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateCohort>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = VitaRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_cohort(
            p.tenant_id,
            id,
            cohort_id,
            CohortUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "cohort.update",
        "cohort",
        item.id,
        serde_json::json!({"study_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "cohorts.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for cohort lifecycle transitions (enroll/withdraw/delete/restore).
async fn cohort_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    cohort_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = VitaRepo::new(pool);
    let item = match action {
        "enroll" => repo.enroll_cohort(p.tenant_id, id, cohort_id).await?,
        "withdraw" => repo.withdraw_cohort(p.tenant_id, id, cohort_id).await?,
        "delete" => repo.soft_delete_cohort(p.tenant_id, id, cohort_id).await?,
        "restore" => repo.restore_cohort(p.tenant_id, id, cohort_id).await?,
        _ => return Err(HelixError::validation("unknown cohort action").into()),
    };
    audit(
        &state,
        &p,
        &format!("cohort.{action}"),
        "cohort",
        item.id,
        serde_json::json!({"study_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "cohorts.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.vita.cohort.lifecycle",
        serde_json::json!({
            "study_id": id,
            "cohort_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn enroll_cohort(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, cohort_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    cohort_transition(state, p, id, cohort_id, "enroll").await
}

async fn withdraw_cohort(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, cohort_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    cohort_transition(state, p, id, cohort_id, "withdraw").await
}

async fn delete_cohort(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, cohort_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    cohort_transition(state, p, id, cohort_id, "delete").await
}

async fn restore_cohort(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, cohort_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    cohort_transition(state, p, id, cohort_id, "restore").await
}

// --- Reports ---

async fn vita_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<VitaSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = VitaRepo::new(pool);
    let rows = repo.get_vita_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_cohort_status, next_study_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18118");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-vita-prime").expect("helix-vita-prime product known");
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
    fn study_transitions_are_guarded() {
        assert_eq!(next_study_status("draft", "recruit").unwrap(), "recruiting");
        assert_eq!(
            next_study_status("recruiting", "complete").unwrap(),
            "completed"
        );
        assert_eq!(
            next_study_status("draft", "terminate").unwrap(),
            "terminated"
        );
        assert_eq!(
            next_study_status("recruiting", "terminate").unwrap(),
            "terminated"
        );
        assert!(next_study_status("recruiting", "recruit").is_err());
        assert!(next_study_status("draft", "complete").is_err());
        assert!(next_study_status("completed", "terminate").is_err());
    }

    #[test]
    fn cohort_transitions_are_guarded() {
        assert_eq!(next_cohort_status("draft", "enroll").unwrap(), "enrolled");
        assert_eq!(
            next_cohort_status("draft", "withdraw").unwrap(),
            "withdrawn"
        );
        assert_eq!(
            next_cohort_status("enrolled", "withdraw").unwrap(),
            "withdrawn"
        );
        assert!(next_cohort_status("enrolled", "enroll").is_err());
        assert!(next_cohort_status("withdrawn", "enroll").is_err());
        assert!(next_cohort_status("withdrawn", "withdraw").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn study_and_cohort_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = VitaRepo::new(pool.clone());

        let study = repo
            .create_parent(
                tenant_id,
                "Sleep & recovery",
                "8-week observational",
                serde_json::json!({}),
            )
            .await
            .expect("create study");
        assert_eq!(study.status, "draft");

        let recruiting = repo
            .recruit_study(tenant_id, study.id)
            .await
            .expect("recruit");
        assert_eq!(recruiting.status, "recruiting");
        assert!(recruiting.recruiting_at.is_some());

        // Complete guard: a draft cohort blocks completion.
        let cohort = repo
            .create_child(
                tenant_id,
                study.id,
                "Arm A",
                "wearable tracked",
                serde_json::json!({}),
            )
            .await
            .expect("create cohort");
        assert_eq!(cohort.status, "draft");

        let blocked = repo.complete_study(tenant_id, study.id).await;
        assert!(blocked.is_err(), "complete blocked by draft cohort");

        let enrolled = repo
            .enroll_cohort(tenant_id, study.id, cohort.id)
            .await
            .expect("enroll");
        assert_eq!(enrolled.status, "enrolled");
        assert!(enrolled.enrolled_at.is_some());

        // A second cohort is withdrawn.
        let cohort2 = repo
            .create_child(tenant_id, study.id, "Arm B", "", serde_json::json!({}))
            .await
            .expect("create cohort2");
        let withdrawn = repo
            .withdraw_cohort(tenant_id, study.id, cohort2.id)
            .await
            .expect("withdraw");
        assert_eq!(withdrawn.status, "withdrawn");
        assert!(withdrawn.withdrawn_at.is_some());

        // Summary reflects both cohorts.
        let summary = repo.get_vita_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == study.id).unwrap();
        assert_eq!(row.total_cohorts, 2);
        assert_eq!(row.enrolled_cohorts, 1);
        assert_eq!(row.withdrawn_cohorts, 1);

        // Complete succeeds now.
        let completed = repo
            .complete_study(tenant_id, study.id)
            .await
            .expect("complete");
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());

        // Terminate path on a second study.
        let study2 = repo
            .create_parent(tenant_id, "Nutrition pilot", "", serde_json::json!({}))
            .await
            .expect("create study2");
        let terminated = repo
            .terminate_study(tenant_id, study2.id)
            .await
            .expect("terminate");
        assert_eq!(terminated.status, "terminated");
        assert!(terminated.terminated_at.is_some());

        // Updates.
        let renamed = repo
            .update_study(
                tenant_id,
                study.id,
                StudyUpdate {
                    name: Some("Sleep & recovery II".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update study");
        assert_eq!(renamed.name, "Sleep & recovery II");

        let cohort_updated = repo
            .update_cohort(
                tenant_id,
                study.id,
                cohort.id,
                CohortUpdate {
                    body: Some("wearable + diary".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update cohort");
        assert_eq!(cohort_updated.body, "wearable + diary");

        // Cohort delete hides it; restore returns the pre-delete status.
        repo.soft_delete_cohort(tenant_id, study.id, cohort2.id)
            .await
            .expect("delete cohort2");
        let cohorts = repo
            .list_children(tenant_id, study.id)
            .await
            .expect("list cohorts after delete");
        assert!(cohorts.iter().all(|c| c.id != cohort2.id));
        let restored_cohort = repo
            .restore_cohort(tenant_id, study.id, cohort2.id)
            .await
            .expect("restore cohort2");
        assert_eq!(restored_cohort.status, "withdrawn");

        // Study delete hides it; restore returns the pre-delete status.
        repo.soft_delete_study(tenant_id, study2.id)
            .await
            .expect("delete study2");
        let studies = repo
            .list_parents(tenant_id)
            .await
            .expect("list studies after delete");
        assert!(studies.iter().all(|s| s.id != study2.id));
        let restored = repo
            .restore_study(tenant_id, study2.id)
            .await
            .expect("restore study2");
        assert_eq!(restored.status, "terminated");
        assert!(restored.deleted_at.is_none());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn cohorts_rejected_on_deleted_study() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = VitaRepo::new(pool.clone());

        let study = repo
            .create_parent(tenant_id, "Doomed study", "", serde_json::json!({}))
            .await
            .expect("create study");
        repo.soft_delete_study(tenant_id, study.id)
            .await
            .expect("delete study");

        // 8 racing cohort creates on a soft-deleted study all fail.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.create_child(tenant_id, study.id, "leak", "", serde_json::json!({}))
                    .await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("create task panicked") {
                Ok(_) => panic!("cohort created on a deleted study"),
                Err(e) if e.code == shared_core::ErrorCode::NotFound => rejected += 1,
                Err(e) => panic!("unexpected create error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "all racing creates must be rejected");

        let cohorts = repo
            .list_children(tenant_id, study.id)
            .await
            .expect("list cohorts");
        assert_eq!(cohorts.len(), 0, "no cohort may leak onto a deleted study");
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_complete_single_winner() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = VitaRepo::new(pool.clone());

        let study = repo
            .create_parent(tenant_id, "Race complete", "", serde_json::json!({}))
            .await
            .expect("create study");
        repo.recruit_study(tenant_id, study.id)
            .await
            .expect("recruit study");

        // 8 racing completes of one recruiting study with no draft cohorts.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.complete_study(tenant_id, study.id).await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("complete task panicked") {
                Ok(_) => winners += 1,
                Err(e)
                    if e.code == shared_core::ErrorCode::Conflict
                        || e.code == shared_core::ErrorCode::Validation =>
                {
                    rejected += 1
                }
                Err(e) => panic!("unexpected complete error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing complete may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let studies = repo.list_parents(tenant_id).await.expect("list studies");
        let row = studies
            .iter()
            .find(|s| s.id == study.id)
            .expect("study listed");
        assert_eq!(row.status, "completed");
    }
}
