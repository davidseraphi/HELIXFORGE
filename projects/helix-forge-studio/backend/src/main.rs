//! helix-forge-studio API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{AppUpdate, DbPool, PageUpdate, StudioRepo, StudioSummaryRow};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-forge-studio")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-forge-studio", 8110)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-forge-studio", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/apps", get(list_parents).post(create_parent))
        .route("/v1/apps/{id}", get(get_parent).patch(update_app))
        .route("/v1/apps/{id}/publish", post(publish_app))
        .route("/v1/apps/{id}/unpublish", post(unpublish_app))
        .route("/v1/apps/{id}/delete", post(delete_app))
        .route("/v1/apps/{id}/restore", post(restore_app))
        .route("/v1/apps/{id}/pages", get(list_children).post(create_child))
        .route(
            "/v1/apps/{id}/pages/{page_id}",
            axum::routing::patch(update_page),
        )
        .route("/v1/apps/{id}/pages/{page_id}/archive", post(archive_page))
        .route("/v1/apps/{id}/pages/{page_id}/reopen", post(reopen_page))
        .route("/v1/apps/{id}/pages/{page_id}/delete", post(delete_page))
        .route("/v1/apps/{id}/pages/{page_id}/restore", post(restore_page))
        .route("/v1/reports/studio-summary", get(studio_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-forge-studio",
        "phase": "wave2_w10",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "apps": true,
            "pages": true,
            "app_lifecycle": true,
            "page_lifecycle": true,
            "publish_guards": true,
            "studio_summary": true,
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
            "helix-forge-studio",
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

// --- Apps ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = StudioRepo::new(pool.clone());
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
    let repo = StudioRepo::new(pool);
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
        "app.create",
        "app",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "apps.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = StudioRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("app not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateApp {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_app(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateApp>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = StudioRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_app(
            p.tenant_id,
            id,
            AppUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "app.update",
        "app",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "apps.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for app lifecycle transitions (publish/unpublish/delete/restore).
async fn app_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = StudioRepo::new(pool);
    let item = match action {
        "publish" => repo.publish_app(p.tenant_id, id).await?,
        "unpublish" => repo.unpublish_app(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_app(p.tenant_id, id).await?,
        "restore" => repo.restore_app(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown app action").into()),
    };
    audit(
        &state,
        &p,
        &format!("app.{action}"),
        "app",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "apps.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.studio.app.lifecycle",
        serde_json::json!({
            "app_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn publish_app(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    app_transition(state, p, id, "publish").await
}

async fn unpublish_app(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    app_transition(state, p, id, "unpublish").await
}

async fn delete_app(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    app_transition(state, p, id, "delete").await
}

async fn restore_app(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    app_transition(state, p, id, "restore").await
}

// --- Pages ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = StudioRepo::new(pool);
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
    let repo = StudioRepo::new(pool);
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
        "page.create",
        "page",
        item.id,
        serde_json::json!({"app_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "pages.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdatePage {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_page(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, page_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdatePage>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = StudioRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_page(
            p.tenant_id,
            id,
            page_id,
            PageUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "page.update",
        "page",
        item.id,
        serde_json::json!({"app_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "pages.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for page lifecycle transitions (archive/reopen/delete/restore).
async fn page_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    page_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = StudioRepo::new(pool);
    let item = match action {
        "archive" => repo.archive_page(p.tenant_id, id, page_id).await?,
        "reopen" => repo.reopen_page(p.tenant_id, id, page_id).await?,
        "delete" => repo.soft_delete_page(p.tenant_id, id, page_id).await?,
        "restore" => repo.restore_page(p.tenant_id, id, page_id).await?,
        _ => return Err(HelixError::validation("unknown page action").into()),
    };
    audit(
        &state,
        &p,
        &format!("page.{action}"),
        "page",
        item.id,
        serde_json::json!({"app_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "pages.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.studio.page.lifecycle",
        serde_json::json!({
            "app_id": id,
            "page_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn archive_page(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, page_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    page_transition(state, p, id, page_id, "archive").await
}

async fn reopen_page(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, page_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    page_transition(state, p, id, page_id, "reopen").await
}

async fn delete_page(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, page_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    page_transition(state, p, id, page_id, "delete").await
}

async fn restore_page(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, page_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    page_transition(state, p, id, page_id, "restore").await
}

// --- Reports ---

async fn studio_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<StudioSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = StudioRepo::new(pool);
    let rows = repo.get_studio_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_app_status, next_page_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18110");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-forge-studio").expect("helix-forge-studio product known");
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
    fn app_transitions_are_guarded() {
        assert_eq!(next_app_status("draft", "publish").unwrap(), "published");
        assert_eq!(next_app_status("published", "unpublish").unwrap(), "draft");
        assert!(next_app_status("published", "publish").is_err());
        assert!(next_app_status("draft", "unpublish").is_err());
        assert!(next_app_status("deleted", "publish").is_err());
        assert!(next_app_status("draft", "unknown").is_err());
    }

    #[test]
    fn page_transitions_are_guarded() {
        assert_eq!(next_page_status("open", "archive").unwrap(), "archived");
        assert_eq!(next_page_status("archived", "reopen").unwrap(), "open");
        assert!(next_page_status("archived", "archive").is_err());
        assert!(next_page_status("open", "reopen").is_err());
        assert!(next_page_status("deleted", "archive").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn app_and_page_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = StudioRepo::new(pool.clone());

        let app = repo
            .create_parent(tenant_id, "Shopfront", "store app", serde_json::json!({}))
            .await
            .expect("create app");
        assert_eq!(app.status, "draft");

        // Publish guard: an app with no pages cannot publish.
        let too_early = repo.publish_app(tenant_id, app.id).await;
        assert!(too_early.is_err(), "publish requires a page");

        let page = repo
            .create_child(
                tenant_id,
                app.id,
                "Home",
                "hero + grid",
                serde_json::json!({}),
            )
            .await
            .expect("create page");
        assert_eq!(page.status, "open");

        let published = repo
            .publish_app(tenant_id, app.id)
            .await
            .expect("publish with page");
        assert_eq!(published.status, "published");
        assert!(published.published_at.is_some());

        let unpublished = repo
            .unpublish_app(tenant_id, app.id)
            .await
            .expect("unpublish");
        assert_eq!(unpublished.status, "draft");

        // App update.
        let renamed = repo
            .update_app(
                tenant_id,
                app.id,
                AppUpdate {
                    name: Some("Shopfront 2".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update app");
        assert_eq!(renamed.name, "Shopfront 2");

        // Page update + archive + reopen.
        let updated = repo
            .update_page(
                tenant_id,
                app.id,
                page.id,
                PageUpdate {
                    body: Some("hero + grid + footer".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update page");
        assert_eq!(updated.body, "hero + grid + footer");
        assert!(updated.updated_at.is_some());

        let archived = repo
            .archive_page(tenant_id, app.id, page.id)
            .await
            .expect("archive page");
        assert_eq!(archived.status, "archived");
        assert!(archived.archived_at.is_some());

        // Summary reflects the archived page.
        let summary = repo.get_studio_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == app.id).unwrap();
        assert_eq!(row.total_pages, 1);
        assert_eq!(row.open_pages, 0);
        assert_eq!(row.archived_pages, 1);

        let reopened = repo
            .reopen_page(tenant_id, app.id, page.id)
            .await
            .expect("reopen page");
        assert_eq!(reopened.status, "open");
        assert!(reopened.archived_at.is_none());

        // Page delete hides it; restore brings it back open.
        repo.soft_delete_page(tenant_id, app.id, page.id)
            .await
            .expect("delete page");
        let pages = repo
            .list_children(tenant_id, app.id)
            .await
            .expect("list pages after delete");
        assert!(pages.iter().all(|p| p.id != page.id));

        // With its only page deleted, the app cannot publish.
        let no_pages = repo.publish_app(tenant_id, app.id).await;
        assert!(no_pages.is_err(), "publish requires a non-deleted page");

        let restored_page = repo
            .restore_page(tenant_id, app.id, page.id)
            .await
            .expect("restore page");
        assert_eq!(restored_page.status, "open");

        // App delete hides it; restore brings it back to its pre-delete status.
        repo.publish_app(tenant_id, app.id)
            .await
            .expect("publish before delete");
        repo.soft_delete_app(tenant_id, app.id)
            .await
            .expect("delete app");
        let apps = repo
            .list_parents(tenant_id)
            .await
            .expect("list apps after delete");
        assert!(apps.iter().all(|a| a.id != app.id));

        let restored_app = repo
            .restore_app(tenant_id, app.id)
            .await
            .expect("restore app");
        assert_eq!(restored_app.status, "published");
        assert!(restored_app.deleted_at.is_none());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn pages_rejected_on_deleted_app() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = StudioRepo::new(pool.clone());

        let app = repo
            .create_parent(tenant_id, "Doomed app", "", serde_json::json!({}))
            .await
            .expect("create app");
        repo.soft_delete_app(tenant_id, app.id)
            .await
            .expect("delete app");

        // 8 racing page creates on a soft-deleted app all fail.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.create_child(tenant_id, app.id, "leak", "", serde_json::json!({}))
                    .await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("create task panicked") {
                Ok(_) => panic!("page created on a deleted app"),
                Err(e) if e.code == shared_core::ErrorCode::NotFound => rejected += 1,
                Err(e) => panic!("unexpected create error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "all racing creates must be rejected");

        let pages = repo
            .list_children(tenant_id, app.id)
            .await
            .expect("list pages");
        assert_eq!(pages.len(), 0, "no page may leak onto a deleted app");
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_publish_single_winner() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = StudioRepo::new(pool.clone());

        let app = repo
            .create_parent(tenant_id, "Race publish", "", serde_json::json!({}))
            .await
            .expect("create app");
        repo.create_child(tenant_id, app.id, "Home", "", serde_json::json!({}))
            .await
            .expect("create page");

        // 8 racing publishes of one draft app.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.publish_app(tenant_id, app.id).await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("publish task panicked") {
                Ok(_) => winners += 1,
                Err(e)
                    if e.code == shared_core::ErrorCode::Conflict
                        || e.code == shared_core::ErrorCode::Validation =>
                {
                    rejected += 1
                }
                Err(e) => panic!("unexpected publish error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing publish may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let apps = repo.list_parents(tenant_id).await.expect("list apps");
        let row = apps.iter().find(|a| a.id == app.id).expect("app listed");
        assert_eq!(row.status, "published");
    }
}
