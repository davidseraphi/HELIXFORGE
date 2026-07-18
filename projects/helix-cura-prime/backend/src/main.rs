//! helix-cura-prime API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{CaseUpdate, CuraRepo, CuraSummaryRow, DbPool, NoteUpdate};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-cura-prime")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-cura-prime", 8113)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-cura-prime", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/care_cases", get(list_parents).post(create_parent))
        .route("/v1/care_cases/{id}", get(get_parent).patch(update_case))
        .route("/v1/care_cases/{id}/activate", post(activate_case))
        .route("/v1/care_cases/{id}/discharge", post(discharge_case))
        .route("/v1/care_cases/{id}/reopen", post(reopen_case))
        .route("/v1/care_cases/{id}/delete", post(delete_case))
        .route("/v1/care_cases/{id}/restore", post(restore_case))
        .route(
            "/v1/care_cases/{id}/notes",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/care_cases/{id}/notes/{note_id}",
            axum::routing::patch(update_note),
        )
        .route("/v1/care_cases/{id}/notes/{note_id}/sign", post(sign_note))
        .route("/v1/care_cases/{id}/notes/{note_id}/void", post(void_note))
        .route(
            "/v1/care_cases/{id}/notes/{note_id}/delete",
            post(delete_note),
        )
        .route(
            "/v1/care_cases/{id}/notes/{note_id}/restore",
            post(restore_note),
        )
        .route("/v1/reports/cura-summary", get(cura_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-cura-prime",
        "phase": "wave2_w13",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "care_cases": true,
            "notes": true,
            "case_lifecycle": true,
            "note_lifecycle": true,
            "signed_immutable": true,
            "discharge_guards": true,
            "cura_summary": true,
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
            "helix-cura-prime",
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

// --- Cases ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = CuraRepo::new(pool.clone());
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
    let repo = CuraRepo::new(pool);
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
        "care_case.create",
        "care_case",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "care_cases.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = CuraRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("case not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateCase {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_case(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCase>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = CuraRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_case(
            p.tenant_id,
            id,
            CaseUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "care_case.update",
        "care_case",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "care_cases.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for case lifecycle transitions (activate/discharge/reopen/delete/restore).
async fn case_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = CuraRepo::new(pool);
    let item = match action {
        "activate" => repo.activate_case(p.tenant_id, id).await?,
        "discharge" => repo.discharge_case(p.tenant_id, id).await?,
        "reopen" => repo.reopen_case(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_case(p.tenant_id, id).await?,
        "restore" => repo.restore_case(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown case action").into()),
    };
    audit(
        &state,
        &p,
        &format!("care_case.{action}"),
        "care_case",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "care_cases.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.cura.case.lifecycle",
        serde_json::json!({
            "case_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn activate_case(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    case_transition(state, p, id, "activate").await
}

async fn discharge_case(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    case_transition(state, p, id, "discharge").await
}

async fn reopen_case(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    case_transition(state, p, id, "reopen").await
}

async fn delete_case(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    case_transition(state, p, id, "delete").await
}

async fn restore_case(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    case_transition(state, p, id, "restore").await
}

// --- Notes ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = CuraRepo::new(pool);
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
    let repo = CuraRepo::new(pool);
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
        "note.create",
        "note",
        item.id,
        serde_json::json!({"case_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "notes.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateNote {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_note(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, note_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateNote>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = CuraRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_note(
            p.tenant_id,
            id,
            note_id,
            NoteUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "note.update",
        "note",
        item.id,
        serde_json::json!({"case_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "notes.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for note lifecycle transitions (sign/void/delete/restore).
async fn note_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    note_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = CuraRepo::new(pool);
    let item = match action {
        "sign" => repo.sign_note(p.tenant_id, id, note_id).await?,
        "void" => repo.void_note(p.tenant_id, id, note_id).await?,
        "delete" => repo.soft_delete_note(p.tenant_id, id, note_id).await?,
        "restore" => repo.restore_note(p.tenant_id, id, note_id).await?,
        _ => return Err(HelixError::validation("unknown note action").into()),
    };
    audit(
        &state,
        &p,
        &format!("note.{action}"),
        "note",
        item.id,
        serde_json::json!({"case_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "notes.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.cura.note.lifecycle",
        serde_json::json!({
            "case_id": id,
            "note_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn sign_note(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, note_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    note_transition(state, p, id, note_id, "sign").await
}

async fn void_note(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, note_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    note_transition(state, p, id, note_id, "void").await
}

async fn delete_note(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, note_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    note_transition(state, p, id, note_id, "delete").await
}

async fn restore_note(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, note_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    note_transition(state, p, id, note_id, "restore").await
}

// --- Reports ---

async fn cura_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<CuraSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = CuraRepo::new(pool);
    let rows = repo.get_cura_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_case_status, next_note_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18113");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-cura-prime").expect("helix-cura-prime product known");
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
    fn case_transitions_are_guarded() {
        assert_eq!(next_case_status("draft", "activate").unwrap(), "active");
        assert_eq!(
            next_case_status("active", "discharge").unwrap(),
            "discharged"
        );
        assert_eq!(next_case_status("discharged", "reopen").unwrap(), "active");
        assert!(next_case_status("active", "activate").is_err());
        assert!(next_case_status("draft", "discharge").is_err());
        assert!(next_case_status("active", "reopen").is_err());
        assert!(next_case_status("deleted", "activate").is_err());
    }

    #[test]
    fn note_transitions_are_guarded() {
        assert_eq!(next_note_status("draft", "sign").unwrap(), "signed");
        assert_eq!(next_note_status("draft", "void").unwrap(), "voided");
        assert_eq!(next_note_status("signed", "void").unwrap(), "voided");
        assert!(next_note_status("signed", "sign").is_err());
        assert!(next_note_status("voided", "sign").is_err());
        assert!(next_note_status("voided", "void").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn case_and_note_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = CuraRepo::new(pool.clone());

        let case = repo
            .create_parent(
                tenant_id,
                "Case Rivera",
                "post-op followup",
                serde_json::json!({}),
            )
            .await
            .expect("create case");
        assert_eq!(case.status, "draft");

        let active = repo
            .activate_case(tenant_id, case.id)
            .await
            .expect("activate");
        assert_eq!(active.status, "active");
        assert!(active.activated_at.is_some());

        // Discharge guard: a draft note blocks discharge.
        let note = repo
            .create_child(
                tenant_id,
                case.id,
                "Round 1",
                "vitals stable",
                serde_json::json!({}),
            )
            .await
            .expect("create note");
        assert_eq!(note.status, "draft");

        let blocked = repo.discharge_case(tenant_id, case.id).await;
        assert!(blocked.is_err(), "discharge blocked by draft note");

        // Sign the note; signed notes are immutable.
        let signed = repo
            .sign_note(tenant_id, case.id, note.id)
            .await
            .expect("sign");
        assert_eq!(signed.status, "signed");
        assert!(signed.signed_at.is_some());

        let edit_signed = repo
            .update_note(
                tenant_id,
                case.id,
                note.id,
                NoteUpdate {
                    body: Some("edited after signing".into()),
                    ..Default::default()
                },
            )
            .await;
        assert!(edit_signed.is_err(), "signed note is immutable");

        // A second note is voided.
        let note2 = repo
            .create_child(tenant_id, case.id, "Round 2", "", serde_json::json!({}))
            .await
            .expect("create note2");
        let voided = repo
            .void_note(tenant_id, case.id, note2.id)
            .await
            .expect("void");
        assert_eq!(voided.status, "voided");
        assert!(voided.voided_at.is_some());

        // Summary reflects both notes.
        let summary = repo.get_cura_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == case.id).unwrap();
        assert_eq!(row.total_notes, 2);
        assert_eq!(row.signed_notes, 1);
        assert_eq!(row.voided_notes, 1);

        // Discharge succeeds now; reopen returns to active.
        let discharged = repo
            .discharge_case(tenant_id, case.id)
            .await
            .expect("discharge");
        assert_eq!(discharged.status, "discharged");
        assert!(discharged.discharged_at.is_some());
        let reopened = repo.reopen_case(tenant_id, case.id).await.expect("reopen");
        assert_eq!(reopened.status, "active");
        assert!(reopened.discharged_at.is_none());

        // Draft note edit works.
        let note3 = repo
            .create_child(
                tenant_id,
                case.id,
                "Round 3",
                "initial",
                serde_json::json!({}),
            )
            .await
            .expect("create note3");
        let edited = repo
            .update_note(
                tenant_id,
                case.id,
                note3.id,
                NoteUpdate {
                    body: Some("revised".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("edit draft note");
        assert_eq!(edited.body, "revised");

        // Note delete hides it; restore returns the pre-delete status.
        repo.soft_delete_note(tenant_id, case.id, note2.id)
            .await
            .expect("delete note2");
        let notes = repo
            .list_children(tenant_id, case.id)
            .await
            .expect("list notes after delete");
        assert!(notes.iter().all(|n| n.id != note2.id));
        let restored_note = repo
            .restore_note(tenant_id, case.id, note2.id)
            .await
            .expect("restore note2");
        assert_eq!(restored_note.status, "voided");

        // Case delete hides it; restore returns the pre-delete status.
        repo.soft_delete_case(tenant_id, case.id)
            .await
            .expect("delete case");
        let cases = repo
            .list_parents(tenant_id)
            .await
            .expect("list cases after delete");
        assert!(cases.iter().all(|c| c.id != case.id));
        let restored = repo
            .restore_case(tenant_id, case.id)
            .await
            .expect("restore");
        assert_eq!(restored.status, "active");
        assert!(restored.deleted_at.is_none());
    }
}
