//! HelixInsights API — predictive analytics & decision OS (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use helix_db::InsightsRepo;
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-insights")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder.clients().agents.register_agent(agent_framework::AgentSpec {
        name: format!("{}-assistant", product.slug),
        description: format!("{} assistant", product.title),
        system_prompt: format!(
            "You are the {} analytics assistant. Help users define datasets, metrics, and decisions.",
            product.title
        ),
        tools: vec!["echo".into(), "product_catalog".into()],
        max_steps: 8,
    });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .nest_service("/", domain_routes().with_state(state.clone()));

    let cfg = shared_core::CoreConfig::from_env("helix-insights", 8104)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-insights", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/datasets", get(list_datasets).post(create_dataset))
        .route("/v1/datasets/{id}", get(get_dataset))
        .route(
            "/v1/datasets/{id}/metrics",
            get(list_metrics).post(create_metric),
        )
        .route(
            "/v1/metrics/{id}/points",
            get(list_points).post(record_point),
        )
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "ready",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some()
    }))))
}

async fn list_datasets(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = InsightsRepo::new(pool.clone());
        let items = repo.list_datasets(p.tenant_id).await?;
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
struct CreateDataset {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_source")]
    source_type: String,
    #[serde(default)]
    schema_json: serde_json::Value,
}

fn default_source() -> String {
    "manual".into()
}

async fn create_dataset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateDataset>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable datasets"))?;
    let repo = InsightsRepo::new(pool.clone());
    let ds = repo
        .create_dataset(
            p.tenant_id,
            body.name.trim(),
            &body.description,
            &body.source_type,
            body.schema_json,
        )
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "dataset.create".into(),
            resource_type: "dataset".into(),
            resource_id: ds.id.to_string(),
            metadata: serde_json::json!({"name": ds.name, "source_type": ds.source_type}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-insights",
            "datasets.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(ds))))
}

async fn get_dataset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable datasets"))?;
    let repo = InsightsRepo::new(pool.clone());
    let ds = repo
        .get_dataset(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("dataset not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(ds))))
}

#[derive(Deserialize)]
struct CreateMetric {
    name: String,
    #[serde(default = "default_unit")]
    unit: String,
    #[serde(default = "default_agg")]
    aggregation: String,
    #[serde(default)]
    expression: String,
}

fn default_unit() -> String {
    "count".into()
}

fn default_agg() -> String {
    "sum".into()
}

async fn list_metrics(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable metrics"))?;
    let repo = InsightsRepo::new(pool.clone());
    let items = repo.list_metrics(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "dataset_id": id,
        "items": items
    }))))
}

async fn create_metric(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateMetric>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable metrics"))?;
    let repo = InsightsRepo::new(pool.clone());
    let metric = repo
        .create_metric(
            p.tenant_id,
            id,
            body.name.trim(),
            &body.unit,
            &body.aggregation,
            &body.expression,
        )
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "metric.create".into(),
            resource_type: "metric".into(),
            resource_id: metric.id.to_string(),
            metadata: serde_json::json!({
                "name": metric.name,
                "dataset_id": metric.dataset_id,
                "unit": metric.unit
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-insights",
            "metrics.created",
            1.0,
            "count",
            serde_json::json!({"dataset_id": id}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(metric))))
}

#[derive(Deserialize)]
struct RecordPoint {
    value: f64,
    #[serde(default)]
    dimensions: serde_json::Value,
}

#[derive(Deserialize)]
struct PointsQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    100
}

async fn list_points(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<PointsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for metric points"))?;
    let repo = InsightsRepo::new(pool.clone());
    let items = repo.list_points(p.tenant_id, id, q.limit).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "metric_id": id,
        "items": items
    }))))
}

async fn record_point(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<RecordPoint>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if !body.value.is_finite() {
        return Err(HelixError::validation("value must be a finite number").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for metric points"))?;
    let repo = InsightsRepo::new(pool.clone());
    let point = repo
        .record_point(p.tenant_id, id, body.value, body.dimensions)
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-insights",
            "metrics.points",
            1.0,
            "count",
            serde_json::json!({"metric_id": id}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.insights.metric.point",
            &serde_json::json!({
                "point_id": point.id,
                "metric_id": point.metric_id,
                "value": point.value
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(point))))
}
