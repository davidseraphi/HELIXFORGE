//! helix-grid-prime API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{DbPool, GridRepo, GridSummaryRow, ReadingUpdate, SiteUpdate};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-grid-prime")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-grid-prime", 8119)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-grid-prime", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/sites", get(list_parents).post(create_parent))
        .route("/v1/sites/{id}", get(get_parent).patch(update_site))
        .route("/v1/sites/{id}/energize", post(energize_site))
        .route("/v1/sites/{id}/offline", post(take_offline))
        .route("/v1/sites/{id}/online", post(bring_online))
        .route("/v1/sites/{id}/delete", post(delete_site))
        .route("/v1/sites/{id}/restore", post(restore_site))
        .route(
            "/v1/sites/{id}/readings",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/sites/{id}/readings/{reading_id}",
            axum::routing::patch(update_reading),
        )
        .route(
            "/v1/sites/{id}/readings/{reading_id}/verify",
            post(verify_reading),
        )
        .route(
            "/v1/sites/{id}/readings/{reading_id}/reject",
            post(reject_reading),
        )
        .route(
            "/v1/sites/{id}/readings/{reading_id}/delete",
            post(delete_reading),
        )
        .route(
            "/v1/sites/{id}/readings/{reading_id}/restore",
            post(restore_reading),
        )
        .route("/v1/reports/grid-summary", get(grid_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-grid-prime",
        "phase": "wave2_w19",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "sites": true,
            "readings": true,
            "site_lifecycle": true,
            "reading_lifecycle": true,
            "offline_guards": true,
            "grid_summary": true,
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
            "helix-grid-prime",
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

// --- Sites ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = GridRepo::new(pool.clone());
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
    let repo = GridRepo::new(pool);
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
        "site.create",
        "site",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "sites.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = GridRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("site not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateSite {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_site(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateSite>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = GridRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_site(
            p.tenant_id,
            id,
            SiteUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "site.update",
        "site",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "sites.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for site lifecycle transitions (energize/offline/online/delete/restore).
async fn site_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = GridRepo::new(pool);
    let item = match action {
        "energize" => repo.energize_site(p.tenant_id, id).await?,
        "offline" => repo.take_offline(p.tenant_id, id).await?,
        "online" => repo.bring_online(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_site(p.tenant_id, id).await?,
        "restore" => repo.restore_site(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown site action").into()),
    };
    audit(
        &state,
        &p,
        &format!("site.{action}"),
        "site",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "sites.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.grid.site.lifecycle",
        serde_json::json!({
            "site_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn energize_site(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    site_transition(state, p, id, "energize").await
}

async fn take_offline(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    site_transition(state, p, id, "offline").await
}

async fn bring_online(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    site_transition(state, p, id, "online").await
}

async fn delete_site(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    site_transition(state, p, id, "delete").await
}

async fn restore_site(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    site_transition(state, p, id, "restore").await
}

// --- Readings ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = GridRepo::new(pool);
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
    let repo = GridRepo::new(pool);
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
        "reading.create",
        "reading",
        item.id,
        serde_json::json!({"site_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "readings.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateReading {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_reading(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, reading_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateReading>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = GridRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_reading(
            p.tenant_id,
            id,
            reading_id,
            ReadingUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "reading.update",
        "reading",
        item.id,
        serde_json::json!({"site_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "readings.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for reading lifecycle transitions (verify/reject/delete/restore).
async fn reading_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    reading_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = GridRepo::new(pool);
    let item = match action {
        "verify" => repo.verify_reading(p.tenant_id, id, reading_id).await?,
        "reject" => repo.reject_reading(p.tenant_id, id, reading_id).await?,
        "delete" => {
            repo.soft_delete_reading(p.tenant_id, id, reading_id)
                .await?
        }
        "restore" => repo.restore_reading(p.tenant_id, id, reading_id).await?,
        _ => return Err(HelixError::validation("unknown reading action").into()),
    };
    audit(
        &state,
        &p,
        &format!("reading.{action}"),
        "reading",
        item.id,
        serde_json::json!({"site_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "readings.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.grid.reading.lifecycle",
        serde_json::json!({
            "site_id": id,
            "reading_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn verify_reading(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, reading_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    reading_transition(state, p, id, reading_id, "verify").await
}

async fn reject_reading(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, reading_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    reading_transition(state, p, id, reading_id, "reject").await
}

async fn delete_reading(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, reading_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    reading_transition(state, p, id, reading_id, "delete").await
}

async fn restore_reading(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, reading_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    reading_transition(state, p, id, reading_id, "restore").await
}

// --- Reports ---

async fn grid_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<GridSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = GridRepo::new(pool);
    let rows = repo.get_grid_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_reading_status, next_site_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18119");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-grid-prime").expect("helix-grid-prime product known");
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
    fn site_transitions_are_guarded() {
        assert_eq!(next_site_status("draft", "energize").unwrap(), "active");
        assert_eq!(next_site_status("active", "offline").unwrap(), "offline");
        assert_eq!(next_site_status("offline", "online").unwrap(), "active");
        assert!(next_site_status("active", "energize").is_err());
        assert!(next_site_status("draft", "offline").is_err());
        assert!(next_site_status("active", "online").is_err());
        assert!(next_site_status("deleted", "energize").is_err());
    }

    #[test]
    fn reading_transitions_are_guarded() {
        assert_eq!(next_reading_status("draft", "verify").unwrap(), "verified");
        assert_eq!(next_reading_status("draft", "reject").unwrap(), "rejected");
        assert_eq!(
            next_reading_status("verified", "reject").unwrap(),
            "rejected"
        );
        assert!(next_reading_status("verified", "verify").is_err());
        assert!(next_reading_status("rejected", "verify").is_err());
        assert!(next_reading_status("rejected", "reject").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn site_and_reading_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = GridRepo::new(pool.clone());

        let site = repo
            .create_parent(
                tenant_id,
                "Substation 7",
                "north feeder",
                serde_json::json!({}),
            )
            .await
            .expect("create site");
        assert_eq!(site.status, "draft");

        let active = repo
            .energize_site(tenant_id, site.id)
            .await
            .expect("energize");
        assert_eq!(active.status, "active");
        assert!(active.energized_at.is_some());

        // Offline guard: a draft reading blocks going offline.
        let reading = repo
            .create_child(
                tenant_id,
                site.id,
                "Load 10:00",
                "4.2 MW",
                serde_json::json!({}),
            )
            .await
            .expect("create reading");
        assert_eq!(reading.status, "draft");

        let blocked = repo.take_offline(tenant_id, site.id).await;
        assert!(blocked.is_err(), "offline blocked by draft reading");

        let verified = repo
            .verify_reading(tenant_id, site.id, reading.id)
            .await
            .expect("verify");
        assert_eq!(verified.status, "verified");
        assert!(verified.verified_at.is_some());

        // A second reading is rejected.
        let reading2 = repo
            .create_child(tenant_id, site.id, "Load 10:15", "", serde_json::json!({}))
            .await
            .expect("create reading2");
        let rejected = repo
            .reject_reading(tenant_id, site.id, reading2.id)
            .await
            .expect("reject");
        assert_eq!(rejected.status, "rejected");
        assert!(rejected.rejected_at.is_some());

        // Summary reflects both readings.
        let summary = repo.get_grid_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == site.id).unwrap();
        assert_eq!(row.total_readings, 2);
        assert_eq!(row.verified_readings, 1);
        assert_eq!(row.rejected_readings, 1);

        // Offline succeeds now; online returns to active.
        let offline = repo
            .take_offline(tenant_id, site.id)
            .await
            .expect("offline");
        assert_eq!(offline.status, "offline");
        assert!(offline.offline_at.is_some());
        let online = repo.bring_online(tenant_id, site.id).await.expect("online");
        assert_eq!(online.status, "active");
        assert!(online.offline_at.is_none());

        // Updates.
        let renamed = repo
            .update_site(
                tenant_id,
                site.id,
                SiteUpdate {
                    name: Some("Substation 7B".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update site");
        assert_eq!(renamed.name, "Substation 7B");

        let reading_updated = repo
            .update_reading(
                tenant_id,
                site.id,
                reading.id,
                ReadingUpdate {
                    body: Some("4.3 MW".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update reading");
        assert_eq!(reading_updated.body, "4.3 MW");

        // Reading delete hides it; restore returns the pre-delete status.
        repo.soft_delete_reading(tenant_id, site.id, reading2.id)
            .await
            .expect("delete reading2");
        let readings = repo
            .list_children(tenant_id, site.id)
            .await
            .expect("list readings after delete");
        assert!(readings.iter().all(|r| r.id != reading2.id));
        let restored_reading = repo
            .restore_reading(tenant_id, site.id, reading2.id)
            .await
            .expect("restore reading2");
        assert_eq!(restored_reading.status, "rejected");

        // Site delete hides it; restore returns the pre-delete status.
        repo.soft_delete_site(tenant_id, site.id)
            .await
            .expect("delete site");
        let sites = repo
            .list_parents(tenant_id)
            .await
            .expect("list sites after delete");
        assert!(sites.iter().all(|s| s.id != site.id));
        let restored = repo
            .restore_site(tenant_id, site.id)
            .await
            .expect("restore site");
        assert_eq!(restored.status, "active");
        assert!(restored.deleted_at.is_none());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn readings_rejected_on_deleted_site() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = GridRepo::new(pool.clone());

        let site = repo
            .create_parent(tenant_id, "Doomed site", "", serde_json::json!({}))
            .await
            .expect("create site");
        repo.soft_delete_site(tenant_id, site.id)
            .await
            .expect("delete site");

        // 8 racing reading creates on a soft-deleted site all fail.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.create_child(tenant_id, site.id, "leak", "", serde_json::json!({}))
                    .await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("create task panicked") {
                Ok(_) => panic!("reading created on a deleted site"),
                Err(e) if e.code == shared_core::ErrorCode::NotFound => rejected += 1,
                Err(e) => panic!("unexpected create error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "all racing creates must be rejected");

        let readings = repo
            .list_children(tenant_id, site.id)
            .await
            .expect("list readings");
        assert_eq!(readings.len(), 0, "no reading may leak onto a deleted site");
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_offline_single_winner() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = GridRepo::new(pool.clone());

        let site = repo
            .create_parent(tenant_id, "Race offline", "", serde_json::json!({}))
            .await
            .expect("create site");
        repo.energize_site(tenant_id, site.id)
            .await
            .expect("energize site");

        // 8 racing offlines of one active site with no draft readings.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                repo.take_offline(tenant_id, site.id).await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("offline task panicked") {
                Ok(_) => winners += 1,
                Err(e)
                    if e.code == shared_core::ErrorCode::Conflict
                        || e.code == shared_core::ErrorCode::Validation =>
                {
                    rejected += 1
                }
                Err(e) => panic!("unexpected offline error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing offline may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let sites = repo.list_parents(tenant_id).await.expect("list sites");
        let row = sites.iter().find(|s| s.id == site.id).expect("site listed");
        assert_eq!(row.status, "offline");
    }
}
