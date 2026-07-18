//! HelixInsights API — predictive analytics & decision OS (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use helix_db::{AggregateResult, InsightsRepo};
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
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-insights", 8104)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-insights", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/domain/status", get(domain_status))
        .route("/v1/datasets", get(list_datasets).post(create_dataset))
        .route("/v1/datasets/{id}", get(get_dataset).delete(delete_dataset))
        .route(
            "/v1/datasets/{id}/metrics",
            get(list_metrics).post(create_metric),
        )
        .route("/v1/metrics", get(list_all_metrics))
        .route("/v1/metrics/{id}", get(get_metric))
        .route(
            "/v1/metrics/{id}/points",
            get(list_points).post(record_point),
        )
        .route("/v1/metrics/{id}/aggregate", post(aggregate_metric))
        .route("/v1/metrics/{id}", delete(delete_metric))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-insights",
        "phase": "wave2_w2",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "datasets": true,
            "metrics": true,
            "points": true,
            "soft_delete": true,
            "aggregate": true,
            "audit": true,
            "metering": true,
            "nats": true
        },
        "aggregations": ["sum", "avg", "min", "max", "count"]
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
    state
        .clients
        .bus
        .publish(
            "helix.insights.dataset.created",
            &serde_json::json!({
                "dataset_id": ds.id,
                "name": ds.name,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
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

async fn delete_dataset(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable datasets"))?;
    let repo = InsightsRepo::new(pool.clone());
    let ds = repo.soft_delete_dataset(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "dataset.delete".into(),
            resource_type: "dataset".into(),
            resource_id: ds.id.to_string(),
            metadata: serde_json::json!({"name": ds.name}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.insights.dataset.deleted",
            &serde_json::json!({
                "dataset_id": ds.id,
                "name": ds.name,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(ds))))
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

async fn list_all_metrics(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable metrics"))?;
    let repo = InsightsRepo::new(pool.clone());
    let items = repo.list_metrics_for_tenant(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "items": items
    }))))
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
    state
        .clients
        .bus
        .publish(
            "helix.insights.metric.created",
            &serde_json::json!({
                "metric_id": metric.id,
                "name": metric.name,
                "dataset_id": metric.dataset_id,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(metric))))
}

async fn get_metric(
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
    let metric = repo
        .get_metric(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("metric not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(metric))))
}

async fn delete_metric(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable metrics"))?;
    let repo = InsightsRepo::new(pool.clone());
    let metric = repo.soft_delete_metric(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "metric.delete".into(),
            resource_type: "metric".into(),
            resource_id: metric.id.to_string(),
            metadata: serde_json::json!({"name": metric.name, "dataset_id": metric.dataset_id}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.insights.metric.deleted",
            &serde_json::json!({
                "metric_id": metric.id,
                "name": metric.name,
                "dataset_id": metric.dataset_id,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
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
    #[serde(default)]
    from: Option<DateTime<Utc>>,
    #[serde(default)]
    to: Option<DateTime<Utc>>,
    #[serde(default)]
    dimensions: Option<String>,
}

fn default_limit() -> i64 {
    100
}

fn parse_dimensions(raw: Option<String>) -> HelixResult<Option<serde_json::Value>> {
    match raw {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => {
            let v: serde_json::Value = serde_json::from_str(&s).map_err(|e| {
                HelixError::validation(format!("dimensions must be valid JSON: {e}"))
            })?;
            if !v.is_object() {
                return Err(HelixError::validation("dimensions must be a JSON object"));
            }
            Ok(Some(v))
        }
    }
}

async fn list_points(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<PointsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let dims = parse_dimensions(q.dimensions)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for metric points"))?;
    let repo = InsightsRepo::new(pool.clone());
    let items = repo
        .list_points_filtered(p.tenant_id, id, q.from, q.to, dims.as_ref(), q.limit)
        .await?;
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
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "point.record".into(),
            resource_type: "metric_point".into(),
            resource_id: point.id.to_string(),
            metadata: serde_json::json!({
                "metric_id": point.metric_id,
                "value": point.value
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
                "value": point.value,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(point))))
}

#[derive(Deserialize)]
struct AggregateQuery {
    aggregation: String,
    #[serde(default)]
    from: Option<DateTime<Utc>>,
    #[serde(default)]
    to: Option<DateTime<Utc>>,
    #[serde(default)]
    dimensions: Option<String>,
}

async fn aggregate_metric(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<AggregateQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let dims = parse_dimensions(body.dimensions)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for metric points"))?;
    let repo = InsightsRepo::new(pool.clone());
    let AggregateResult { value, count } = repo
        .aggregate_points(
            p.tenant_id,
            id,
            &body.aggregation,
            body.from,
            body.to,
            dims.as_ref(),
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
            action: "metric.aggregate".into(),
            resource_type: "metric".into(),
            resource_id: id.to_string(),
            metadata: serde_json::json!({
                "aggregation": body.aggregation,
                "count": count,
                "value": value
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
            "metrics.aggregated",
            1.0,
            "count",
            serde_json::json!({"metric_id": id, "aggregation": body.aggregation}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.insights.metric.aggregated",
            &serde_json::json!({
                "metric_id": id,
                "aggregation": body.aggregation,
                "count": count,
                "value": value,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "metric_id": id,
        "aggregation": body.aggregation,
        "value": value,
        "count": count
    }))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dimensions_accepts_object_json() {
        let raw = Some(r#"{"region": "local"}"#.into());
        let parsed = parse_dimensions(raw).unwrap();
        assert_eq!(parsed.unwrap()["region"], "local");
    }

    #[test]
    fn parse_dimensions_rejects_non_object() {
        let raw = Some("\"hello\"".into());
        assert!(parse_dimensions(raw).is_err());
    }

    #[test]
    fn parse_dimensions_treats_empty_as_none() {
        assert!(parse_dimensions(Some("   ".into())).unwrap().is_none());
        assert!(parse_dimensions(None).unwrap().is_none());
    }

    #[test]
    fn finite_value_required() {
        assert!(!f64::NAN.is_finite());
        assert!(!f64::INFINITY.is_finite());
        assert!(!(-f64::INFINITY).is_finite());
        assert!(42.0_f64.is_finite());
    }

    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::tenancy::{Principal, Scope};
    use shared_core::{TenantId, UserId};
    use tokio::sync::{Mutex, MutexGuard};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18104");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-insights").expect("helix-insights product known");
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

    pub fn dev_principal(label: &str) -> Principal {
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let user_id = UserId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("helixforge-user:{label}").as_bytes(),
        ));
        Principal {
            user_id,
            tenant_id,
            org_id: None,
            scopes: vec![
                Scope::Read,
                Scope::Write,
                Scope::Admin,
                Scope::AuditRead,
                Scope::Platform,
            ],
            session_id: Some(format!("dev-session:{label}")),
            residency_region: "local".into(),
        }
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn points_rejected_on_deleted_metric() {
        let (state, _guard) = locked_state().await;
        let p = dev_principal("insights-race");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = InsightsRepo::new(pool.clone());

        let ds = repo
            .create_dataset(
                p.tenant_id,
                "durability-ds",
                "",
                "manual",
                serde_json::json!({}),
            )
            .await
            .expect("create dataset");
        let metric = repo
            .create_metric(p.tenant_id, ds.id, "cpu", "percent", "avg", "")
            .await
            .expect("create metric");
        repo.record_point(p.tenant_id, metric.id, 42.0, serde_json::json!({}))
            .await
            .expect("record baseline point");

        // Delete the metric, then race 8 record attempts: all must be rejected.
        repo.soft_delete_metric(p.tenant_id, metric.id)
            .await
            .expect("soft delete metric");

        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            let tenant_id = p.tenant_id;
            handles.push(tokio::spawn(async move {
                repo.record_point(tenant_id, metric.id, 1.0, serde_json::json!({}))
                    .await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("record task panicked") {
                Err(e) if e.code == shared_core::ErrorCode::NotFound => rejected += 1,
                Ok(_) => panic!("record on a deleted metric must be rejected"),
                Err(e) => panic!("unexpected record error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "every record on a deleted metric must fail");

        let points = repo
            .list_points(p.tenant_id, metric.id, 50)
            .await
            .expect("list points");
        assert_eq!(points.len(), 1, "only the baseline point may exist");
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_records_all_landed() {
        let (state, _guard) = locked_state().await;
        let p = dev_principal("insights-sum");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = InsightsRepo::new(pool.clone());

        let ds = repo
            .create_dataset(p.tenant_id, "sum-ds", "", "manual", serde_json::json!({}))
            .await
            .expect("create dataset");
        let metric = repo
            .create_metric(p.tenant_id, ds.id, "requests", "count", "sum", "")
            .await
            .expect("create metric");

        // 8 concurrent point records on a live metric: all land, none lost.
        let mut handles = Vec::new();
        for i in 0..8u32 {
            let repo = repo.clone();
            let tenant_id = p.tenant_id;
            handles.push(tokio::spawn(async move {
                repo.record_point(tenant_id, metric.id, f64::from(i), serde_json::json!({}))
                    .await
            }));
        }
        for h in handles {
            h.await
                .expect("record task panicked")
                .expect("record on a live metric succeeds");
        }

        let points = repo
            .list_points(p.tenant_id, metric.id, 50)
            .await
            .expect("list points");
        assert_eq!(points.len(), 8, "every concurrent record must land");
        let total: f64 = points.iter().map(|pt| pt.value).sum();
        assert_eq!(total, 28.0, "sum of 0..=7");
    }
}
