//! HelixCommerce API — AI e-commerce & marketplace builder (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use helix_db::{CommerceRepo, OrderLineInput};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-commerce")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: format!("{}-assistant", product.slug),
            description: format!("{} assistant", product.title),
            system_prompt: format!(
                "You are the {} commerce assistant. Help users manage catalog products and orders.",
                product.title
            ),
            tools: vec!["echo".into(), "product_catalog".into()],
            max_steps: 8,
        });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .nest_service("/", domain_routes().with_state(state.clone()));

    let cfg = shared_core::CoreConfig::from_env("helix-commerce", 8105)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-commerce", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/products", get(list_products).post(create_product))
        .route("/v1/products/{id}", get(get_product))
        .route("/v1/orders", get(list_orders).post(create_order))
        .route("/v1/orders/{id}", get(get_order))
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

async fn list_products(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = CommerceRepo::new(pool.clone());
        let items = repo.list_products(p.tenant_id).await?;
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
struct CreateProduct {
    sku: String,
    name: String,
    #[serde(default)]
    description: String,
    price_cents: i64,
    #[serde(default = "default_currency")]
    currency: String,
    #[serde(default)]
    inventory: i32,
    #[serde(default)]
    metadata: serde_json::Value,
}

fn default_currency() -> String {
    "USD".into()
}

async fn create_product(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateProduct>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.sku.trim().is_empty() {
        return Err(HelixError::validation("sku required").into());
    }
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable commerce"))?;
    let repo = CommerceRepo::new(pool.clone());
    let product = repo
        .create_product(
            p.tenant_id,
            body.sku.trim(),
            body.name.trim(),
            &body.description,
            body.price_cents,
            &body.currency,
            body.inventory,
            body.metadata,
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
            action: "product.create".into(),
            resource_type: "product".into(),
            resource_id: product.id.to_string(),
            metadata: serde_json::json!({"sku": product.sku, "price_cents": product.price_cents}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-commerce",
            "products.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(product))))
}

async fn get_product(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable commerce"))?;
    let repo = CommerceRepo::new(pool.clone());
    let product = repo
        .get_product(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("product not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(product))))
}

async fn list_orders(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = CommerceRepo::new(pool.clone());
        let items = repo.list_orders(p.tenant_id).await?;
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
struct CreateOrderLine {
    product_id: Uuid,
    quantity: i32,
}

#[derive(Deserialize)]
struct CreateOrder {
    #[serde(default)]
    customer_email: String,
    items: Vec<CreateOrderLine>,
    #[serde(default)]
    metadata: serde_json::Value,
}

async fn create_order(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateOrder>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.items.is_empty() {
        return Err(HelixError::validation("items required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable commerce"))?;
    let repo = CommerceRepo::new(pool.clone());
    let lines: Vec<OrderLineInput> = body
        .items
        .into_iter()
        .map(|l| OrderLineInput {
            product_id: l.product_id,
            quantity: l.quantity,
        })
        .collect();
    let order = repo
        .create_order(p.tenant_id, &body.customer_email, &lines, body.metadata)
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
            action: "order.create".into(),
            resource_type: "order".into(),
            resource_id: order.id.to_string(),
            metadata: serde_json::json!({
                "total_cents": order.total_cents,
                "lines": order.items.len()
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-commerce",
            "orders.created",
            1.0,
            "count",
            serde_json::json!({"total_cents": order.total_cents}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.commerce.order.created",
            &serde_json::json!({
                "order_id": order.id,
                "total_cents": order.total_cents,
                "currency": order.currency
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(order))))
}

async fn get_order(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable commerce"))?;
    let repo = CommerceRepo::new(pool.clone());
    let order = repo
        .get_order(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("order not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(order))))
}
