//! helix-terra-prime API — durable store via helix_db.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{DbPool, FieldUpdate, ObservationUpdate, TerraRepo, TerraSummaryRow};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-terra-prime")?;
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

    let cfg = shared_core::CoreConfig::from_env("helix-terra-prime", 8114)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-terra-prime", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/fields", get(list_parents).post(create_parent))
        .route("/v1/fields/{id}", get(get_parent).patch(update_field))
        .route("/v1/fields/{id}/activate", post(activate_field))
        .route("/v1/fields/{id}/retire", post(retire_field))
        .route("/v1/fields/{id}/reopen", post(reopen_field))
        .route("/v1/fields/{id}/delete", post(delete_field))
        .route("/v1/fields/{id}/restore", post(restore_field))
        .route(
            "/v1/fields/{id}/observations",
            get(list_children).post(create_child),
        )
        .route(
            "/v1/fields/{id}/observations/{obs_id}",
            axum::routing::patch(update_observation),
        )
        .route(
            "/v1/fields/{id}/observations/{obs_id}/confirm",
            post(confirm_observation),
        )
        .route(
            "/v1/fields/{id}/observations/{obs_id}/dismiss",
            post(dismiss_observation),
        )
        .route(
            "/v1/fields/{id}/observations/{obs_id}/delete",
            post(delete_observation),
        )
        .route(
            "/v1/fields/{id}/observations/{obs_id}/restore",
            post(restore_observation),
        )
        .route("/v1/reports/terra-summary", get(terra_summary))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-terra-prime",
        "phase": "wave2_w14",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "fields": true,
            "observations": true,
            "field_lifecycle": true,
            "observation_lifecycle": true,
            "retire_guards": true,
            "terra_summary": true,
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
            "helix-terra-prime",
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

// --- Fields ---

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = TerraRepo::new(pool.clone());
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
    let repo = TerraRepo::new(pool);
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
        "field.create",
        "field",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "fields.created", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = TerraRepo::new(pool);
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("field not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateField {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_field(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateField>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = TerraRepo::new(pool);
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let item = repo
        .update_field(
            p.tenant_id,
            id,
            FieldUpdate {
                name,
                description: body.description,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "field.update",
        "field",
        item.id,
        serde_json::json!({"name": item.name}),
    )
    .await?;
    meter(&state, &p, "fields.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for field lifecycle transitions (activate/retire/reopen/delete/restore).
async fn field_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = TerraRepo::new(pool);
    let item = match action {
        "activate" => repo.activate_field(p.tenant_id, id).await?,
        "retire" => repo.retire_field(p.tenant_id, id).await?,
        "reopen" => repo.reopen_field(p.tenant_id, id).await?,
        "delete" => repo.soft_delete_field(p.tenant_id, id).await?,
        "restore" => repo.restore_field(p.tenant_id, id).await?,
        _ => return Err(HelixError::validation("unknown field action").into()),
    };
    audit(
        &state,
        &p,
        &format!("field.{action}"),
        "field",
        item.id,
        serde_json::json!({"name": item.name, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "fields.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.terra.field.lifecycle",
        serde_json::json!({
            "field_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn activate_field(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    field_transition(state, p, id, "activate").await
}

async fn retire_field(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    field_transition(state, p, id, "retire").await
}

async fn reopen_field(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    field_transition(state, p, id, "reopen").await
}

async fn delete_field(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    field_transition(state, p, id, "delete").await
}

async fn restore_field(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    field_transition(state, p, id, "restore").await
}

// --- Observations ---

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = TerraRepo::new(pool);
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
    let repo = TerraRepo::new(pool);
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
        "observation.create",
        "observation",
        item.id,
        serde_json::json!({"field_id": id, "title": item.title}),
    )
    .await?;
    meter(
        &state,
        &p,
        "observations.created",
        serde_json::json!({"parent_id": id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize, Default)]
struct UpdateObservation {
    title: Option<String>,
    body: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_observation(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, obs_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateObservation>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = TerraRepo::new(pool);
    let title = body
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let item = repo
        .update_observation(
            p.tenant_id,
            id,
            obs_id,
            ObservationUpdate {
                title,
                body: body.body,
                metadata: body.metadata,
            },
        )
        .await?;
    audit(
        &state,
        &p,
        "observation.update",
        "observation",
        item.id,
        serde_json::json!({"field_id": id, "title": item.title}),
    )
    .await?;
    meter(&state, &p, "observations.updated", serde_json::json!({})).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

/// Shared handler for observation lifecycle transitions (confirm/dismiss/delete/restore).
async fn observation_transition(
    state: AppState,
    p: Principal,
    id: Uuid,
    obs_id: Uuid,
    action: &'static str,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = TerraRepo::new(pool);
    let item = match action {
        "confirm" => repo.confirm_observation(p.tenant_id, id, obs_id).await?,
        "dismiss" => repo.dismiss_observation(p.tenant_id, id, obs_id).await?,
        "delete" => {
            repo.soft_delete_observation(p.tenant_id, id, obs_id)
                .await?
        }
        "restore" => repo.restore_observation(p.tenant_id, id, obs_id).await?,
        _ => return Err(HelixError::validation("unknown observation action").into()),
    };
    audit(
        &state,
        &p,
        &format!("observation.{action}"),
        "observation",
        item.id,
        serde_json::json!({"field_id": id, "title": item.title, "status": item.status}),
    )
    .await?;
    meter(
        &state,
        &p,
        "observations.lifecycle",
        serde_json::json!({"action": action}),
    )
    .await?;
    publish_event(
        &state,
        "helix.terra.observation.lifecycle",
        serde_json::json!({
            "field_id": id,
            "observation_id": item.id,
            "action": action,
            "status": item.status
        }),
    )
    .await;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

async fn confirm_observation(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, obs_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    observation_transition(state, p, id, obs_id, "confirm").await
}

async fn dismiss_observation(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, obs_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    observation_transition(state, p, id, obs_id, "dismiss").await
}

async fn delete_observation(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, obs_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    observation_transition(state, p, id, obs_id, "delete").await
}

async fn restore_observation(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, obs_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    observation_transition(state, p, id, obs_id, "restore").await
}

// --- Reports ---

async fn terra_summary(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<TerraSummaryRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = TerraRepo::new(pool);
    let rows = repo.get_terra_summary(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};

    use super::*;
    use helix_db::{next_field_status, next_observation_status};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18114");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-terra-prime").expect("helix-terra-prime product known");
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
    fn field_transitions_are_guarded() {
        assert_eq!(next_field_status("draft", "activate").unwrap(), "active");
        assert_eq!(next_field_status("active", "retire").unwrap(), "retired");
        assert_eq!(next_field_status("retired", "reopen").unwrap(), "active");
        assert!(next_field_status("active", "activate").is_err());
        assert!(next_field_status("draft", "retire").is_err());
        assert!(next_field_status("active", "reopen").is_err());
        assert!(next_field_status("deleted", "activate").is_err());
    }

    #[test]
    fn observation_transitions_are_guarded() {
        assert_eq!(
            next_observation_status("draft", "confirm").unwrap(),
            "confirmed"
        );
        assert_eq!(
            next_observation_status("draft", "dismiss").unwrap(),
            "dismissed"
        );
        assert_eq!(
            next_observation_status("confirmed", "dismiss").unwrap(),
            "dismissed"
        );
        assert!(next_observation_status("confirmed", "confirm").is_err());
        assert!(next_observation_status("dismissed", "confirm").is_err());
        assert!(next_observation_status("dismissed", "dismiss").is_err());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn field_and_observation_lifecycle_persists() {
        let (state, _guard) = locked_state().await;
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = TerraRepo::new(pool.clone());

        let field = repo
            .create_parent(
                tenant_id,
                "North 40",
                "winter wheat plot",
                serde_json::json!({}),
            )
            .await
            .expect("create field");
        assert_eq!(field.status, "draft");

        let active = repo
            .activate_field(tenant_id, field.id)
            .await
            .expect("activate");
        assert_eq!(active.status, "active");
        assert!(active.activated_at.is_some());

        // Retire guard: a draft observation blocks retiring.
        let obs = repo
            .create_child(
                tenant_id,
                field.id,
                "Soil moisture",
                "18% at 10cm",
                serde_json::json!({}),
            )
            .await
            .expect("create observation");
        assert_eq!(obs.status, "draft");

        let blocked = repo.retire_field(tenant_id, field.id).await;
        assert!(blocked.is_err(), "retire blocked by draft observation");

        let confirmed = repo
            .confirm_observation(tenant_id, field.id, obs.id)
            .await
            .expect("confirm");
        assert_eq!(confirmed.status, "confirmed");
        assert!(confirmed.confirmed_at.is_some());

        // A second observation is dismissed.
        let obs2 = repo
            .create_child(
                tenant_id,
                field.id,
                "Weed pressure",
                "",
                serde_json::json!({}),
            )
            .await
            .expect("create obs2");
        let dismissed = repo
            .dismiss_observation(tenant_id, field.id, obs2.id)
            .await
            .expect("dismiss");
        assert_eq!(dismissed.status, "dismissed");
        assert!(dismissed.dismissed_at.is_some());

        // Summary reflects both observations.
        let summary = repo.get_terra_summary(tenant_id).await.expect("summary");
        let row = summary.iter().find(|r| r.id == field.id).unwrap();
        assert_eq!(row.total_observations, 2);
        assert_eq!(row.confirmed_observations, 1);
        assert_eq!(row.dismissed_observations, 1);

        // Retire succeeds now; reopen returns to active.
        let retired = repo
            .retire_field(tenant_id, field.id)
            .await
            .expect("retire");
        assert_eq!(retired.status, "retired");
        assert!(retired.retired_at.is_some());
        let reopened = repo
            .reopen_field(tenant_id, field.id)
            .await
            .expect("reopen");
        assert_eq!(reopened.status, "active");
        assert!(reopened.retired_at.is_none());

        // Updates.
        let renamed = repo
            .update_field(
                tenant_id,
                field.id,
                FieldUpdate {
                    name: Some("North 40 (west half)".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update field");
        assert_eq!(renamed.name, "North 40 (west half)");

        let obs_updated = repo
            .update_observation(
                tenant_id,
                field.id,
                obs.id,
                ObservationUpdate {
                    body: Some("19% at 10cm".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update observation");
        assert_eq!(obs_updated.body, "19% at 10cm");

        // Observation delete hides it; restore returns the pre-delete status.
        repo.soft_delete_observation(tenant_id, field.id, obs2.id)
            .await
            .expect("delete obs2");
        let observations = repo
            .list_children(tenant_id, field.id)
            .await
            .expect("list observations after delete");
        assert!(observations.iter().all(|o| o.id != obs2.id));
        let restored_obs = repo
            .restore_observation(tenant_id, field.id, obs2.id)
            .await
            .expect("restore obs2");
        assert_eq!(restored_obs.status, "dismissed");

        // Field delete hides it; restore returns the pre-delete status.
        repo.soft_delete_field(tenant_id, field.id)
            .await
            .expect("delete field");
        let fields = repo
            .list_parents(tenant_id)
            .await
            .expect("list fields after delete");
        assert!(fields.iter().all(|f| f.id != field.id));
        let restored = repo
            .restore_field(tenant_id, field.id)
            .await
            .expect("restore field");
        assert_eq!(restored.status, "active");
        assert!(restored.deleted_at.is_none());
    }
}
