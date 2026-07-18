//! helix-lex-prime API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{DbPool, FilingUpdate, LexRepo, LexSummaryRow, MatterUpdate};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-lex-prime")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-lex-prime", 8112)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-lex-prime", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/matters", get(list_parents).post(create_parent))
        .route("/v1/matters/{id}", get(get_parent).patch(update_matter))
        .route("/v1/matters/{id}/open", post(open_matter))
        .route("/v1/matters/{id}/close", post(close_matter))
        .route("/v1/matters/{id}/reopen", post(reopen_matter))
        .route("/v1/matters/{id}/delete", post(delete_matter))
        .route("/v1/matters/{id}/restore", post(restore_matter))
        .route(
            "/v1/matters/{id}/filings",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/matters/{id}/filings/{filing_id}",
            axum::routing::patch(update_filing),
        )
        .route(
            "/v1/matters/{id}/filings/{filing_id}/file",
            post(file_filing),
        )
        .route(
            "/v1/matters/{id}/filings/{filing_id}/withdraw",
            post(withdraw_filing),
        )
        .route(
            "/v1/matters/{id}/filings/{filing_id}/delete",
            post(delete_filing),
        )
        .route(
            "/v1/matters/{id}/filings/{filing_id}/restore",
            post(restore_filing),
        )
        .route("/v1/reports/lex-summary", get(lex_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-lex-prime",
        "phase": "wave2_w12",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "matters": true,
            "filings": true,
            "matter_lifecycle": true,
            "filing_lifecycle": true,
            "close_guards": true,
            "lex_summary": true,
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
            "helix-lex-prime",
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

// --- Matters ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = LexRepo::new(pool.clone());
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
    let repo = LexRepo::new(pool);
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
        "matter.create",
        "matter",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "matters.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = LexRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("matter not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateMatter {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_matter(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMatter>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = LexRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_matter(
            p.tenant_id,
            id,
            MatterUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "matter.update",
        "matter",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "matters.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for matter lifecycle transitions (open/close/reopen/delete/restore).
async fn matter_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = LexRepo::new(pool);
    let item = match action {
        "open" => repo.open_matter(p.tenant_id, id).await?,
        "close" => repo.close_matter(p.tenant_id, id).await?,
        "reopen" => repo.reopen_matter(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_matter(p.tenant_id, id).await?,
        "restore" => repo.restore_matter(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown matter action").into()),
    };
    audit(
        &state,
        &p,
        &format!("matter.{action}"),
        "matter",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "matters.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.lex.matter.lifecycle",
        serde_json::json!({
            "matter_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn open_matter(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    matter_transition(state, p, id, "open").await
}

async fn close_matter(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    matter_transition(state, p, id, "close").await
}

async fn reopen_matter(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    matter_transition(state, p, id, "reopen").await
}

async fn delete_matter(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    matter_transition(state, p, id, "delete").await
}

async fn restore_matter(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    matter_transition(state, p, id, "restore").await
}

// --- Filings ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = LexRepo::new(pool);
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
    let repo = LexRepo::new(pool);
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
        "filing.create",
        "filing",
        item.id,
        serde_json::json!({"matter_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "filings.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateFiling {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_filing(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, filing_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateFiling>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = LexRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_filing(
            p.tenant_id,
            id,
            filing_id,
            FilingUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "filing.update",
        "filing",
        item.id,
        serde_json::json!({"matter_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "filings.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for filing lifecycle transitions (file/withdraw/delete/restore).
async fn filing_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    filing_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = LexRepo::new(pool);
    let item = match action {
        "file" => repo.file_filing(p.tenant_id, id, filing_id).await?,
        "withdraw" => repo.withdraw_filing(p.tenant_id, id, filing_id).await?,
        "delete" => repo.soft_delete_filing(p.tenant_id, id, filing_id).await?,
        "restore" => repo.restore_filing(p.tenant_id, id, filing_id).await?,
        _ => return Err(HelixError::validation("unknown filing action").into()),
    };
    audit(
        &state,
        &p,
        &format!("filing.{action}"),
        "filing",
        item.id,
        serde_json::json!({"matter_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "filings.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.lex.filing.lifecycle",
        serde_json::json!({
            "matter_id": id,
            "filing_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn file_filing(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, filing_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    filing_transition(state, p, id, filing_id, "file").await
}

async fn withdraw_filing(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, filing_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    filing_transition(state, p, id, filing_id, "withdraw").await
}

async fn delete_filing(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, filing_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    filing_transition(state, p, id, filing_id, "delete").await
}

async fn restore_filing(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, filing_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    filing_transition(state, p, id, filing_id, "restore").await
}

// --- Reports ---

async fn lex_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<LexSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = LexRepo::new(pool);
    let rows = repo.get_lex_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_filing_status, next_matter_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18112");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-lex-prime").expect("helix-lex-prime product known");
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
    fn matter_transitions_are_guarded() {
        assert_eq!(next_matter_status("draft", "open").unwrap(), "open");
        assert_eq!(next_matter_status("open", "close").unwrap(), "closed");
        assert_eq!(next_matter_status("closed", "reopen").unwrap(), "open");
        assert!(next_matter_status("open", "open").is_err());
        assert!(next_matter_status("draft", "close").is_err());
        assert!(next_matter_status("open", "reopen").is_err());
        assert!(next_matter_status("deleted", "open").is_err());
    }

    #[test]
    fn filing_transitions_are_guarded() {
        assert_eq!(next_filing_status("draft", "file").unwrap(), "filed");
        assert_eq!(
            next_filing_status("draft", "withdraw").unwrap(),
            "withdrawn"
        );
        assert_eq!(
            next_filing_status("filed", "withdraw").unwrap(),
            "withdrawn"
        );
        assert!(next_filing_status("filed", "file").is_err());
        assert!(next_filing_status("withdrawn", "file").is_err());
        assert!(next_filing_status("withdrawn", "withdraw").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn matter_and_filing_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = LexRepo::new(pool.clone());

        let matter = repo
            .create_parent(
                tenant_id,
                "Acme v. Doe",
                "contract dispute",
                serde_json::json!({}),
            )
            .await
            .expect("create matter");
        assert_eq!(matter.status, "draft");

        let opened = repo.open_matter(tenant_id, matter.id).await.expect("open");
        assert_eq!(opened.status, "open");
        assert!(opened.opened_at.is_some());

        // Close guard: a draft filing blocks closing.
        let filing = repo
            .create_child(
                tenant_id,
                matter.id,
                "Complaint",
                "initial pleading",
                serde_json::json!({}),
            )
            .await
            .expect("create filing");
        assert_eq!(filing.status, "draft");

        let blocked_close = repo.close_matter(tenant_id, matter.id).await;
        assert!(blocked_close.is_err(), "close blocked by draft filing");

        let filed = repo
            .file_filing(tenant_id, matter.id, filing.id)
            .await
            .expect("file");
        assert_eq!(filed.status, "filed");
        assert!(filed.filed_at.is_some());

        // A second filing is withdrawn.
        let filing2 = repo
            .create_child(
                tenant_id,
                matter.id,
                "Motion to compel",
                "",
                serde_json::json!({}),
            )
            .await
            .expect("create filing2");
        let withdrawn = repo
            .withdraw_filing(tenant_id, matter.id, filing2.id)
            .await
            .expect("withdraw");
        assert_eq!(withdrawn.status, "withdrawn");
        assert!(withdrawn.withdrawn_at.is_some());

        // Summary reflects both filings.
        let summary = repo.get_lex_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == matter.id).unwrap();
        assert_eq!(row.total_filings, 2);
        assert_eq!(row.filed_filings, 1);
        assert_eq!(row.withdrawn_filings, 1);

        // Close succeeds now; reopen returns to open.
        let closed = repo
            .close_matter(tenant_id, matter.id)
            .await
            .expect("close");
        assert_eq!(closed.status, "closed");
        assert!(closed.closed_at.is_some());
        let reopened = repo
            .reopen_matter(tenant_id, matter.id)
            .await
            .expect("reopen");
        assert_eq!(reopened.status, "open");
        assert!(reopened.closed_at.is_none());

        // Updates.
        let renamed = repo
            .update_matter(
                tenant_id,
                matter.id,
                MatterUpdate {
                    name: Some("Acme v. Doe (amended)".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update matter");
        assert_eq!(renamed.name, "Acme v. Doe (amended)");

        let filing_updated = repo
            .update_filing(
                tenant_id,
                matter.id,
                filing.id,
                FilingUpdate {
                    body: Some("amended pleading".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update filing");
        assert_eq!(filing_updated.body, "amended pleading");

        // Filing delete hides it; restore returns the pre-delete status.
        repo.soft_delete_filing(tenant_id, matter.id, filing2.id)
            .await
            .expect("delete filing2");
        let filings = repo
            .list_children(tenant_id, matter.id)
            .await
            .expect("list filings after delete");
        assert!(filings.iter().all(|f| f.id != filing2.id));
        let restored_filing = repo
            .restore_filing(tenant_id, matter.id, filing2.id)
            .await
            .expect("restore filing2");
        assert_eq!(restored_filing.status, "withdrawn");

        // Matter delete hides it; restore returns the pre-delete status.
        repo.soft_delete_matter(tenant_id, matter.id)
            .await
            .expect("delete matter");
        let matters = repo
            .list_parents(tenant_id)
            .await
            .expect("list matters after delete");
        assert!(matters.iter().all(|m| m.id != matter.id));
        let restored = repo
            .restore_matter(tenant_id, matter.id)
            .await
            .expect("restore matter");
        assert_eq!(restored.status, "open");
        assert!(restored.deleted_at.is_none());
    }
}
