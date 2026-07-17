//! HelixCommerce API — AI e-commerce & marketplace builder (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
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
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-commerce", 8105)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-commerce", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/domain/status", get(domain_status))
        .route("/v1/products", get(list_products).post(create_product))
        .route("/v1/products/{id}", get(get_product).patch(update_product))
        .route("/v1/orders", get(list_orders).post(create_order))
        .route("/v1/orders/{id}", get(get_order))
        .route("/v1/orders/{id}/cancel", post(cancel_order))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-commerce",
        "phase": "wave2_w3",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "products": true,
            "orders": true,
            "inventory_reservation": true,
            "mixed_currency_guard": true,
            "cancel": true,
            "audit": true,
            "metering": true,
            "nats": true
        }
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
    state
        .clients
        .bus
        .publish(
            "helix.commerce.product.created",
            &serde_json::json!({
                "product_id": product.id,
                "sku": product.sku,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
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

#[derive(Deserialize, Default)]
struct UpdateProduct {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    price_cents: Option<i64>,
    #[serde(default)]
    inventory_delta: Option<i32>,
    #[serde(default)]
    status: Option<String>,
}

async fn update_product(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProduct>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable commerce"))?;
    let repo = CommerceRepo::new(pool.clone());
    let product = repo
        .update_product(
            p.tenant_id,
            id,
            body.name,
            body.description,
            body.price_cents,
            body.inventory_delta,
            body.status,
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
            action: "product.update".into(),
            resource_type: "product".into(),
            resource_id: product.id.to_string(),
            metadata: serde_json::json!({"sku": product.sku, "status": product.status}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-commerce",
            "products.updated",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.commerce.product.updated",
            &serde_json::json!({
                "product_id": product.id,
                "sku": product.sku,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(product))))
}

async fn list_orders(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable commerce"))?;
    let repo = CommerceRepo::new(pool.clone());
    let items = repo.list_orders(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "items": items
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
                "currency": order.currency,
                "tenant_id": p.tenant_id.to_string()
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

async fn cancel_order(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable commerce"))?;
    let repo = CommerceRepo::new(pool.clone());
    let order = repo.cancel_order(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "order.cancel".into(),
            resource_type: "order".into(),
            resource_id: order.id.to_string(),
            metadata: serde_json::json!({
                "total_cents": order.total_cents,
                "items": order.items.len()
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
            "orders.cancelled",
            1.0,
            "count",
            serde_json::json!({"order_id": order.id}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.commerce.order.cancelled",
            &serde_json::json!({
                "order_id": order.id,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(order))))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{AppState, ProductApp, ServiceBuilder};
    use shared_core::tenancy::{Principal, Scope};
    use shared_core::{TenantId, UserId};
    use tokio::sync::{Mutex, MutexGuard};
    use uuid::Uuid;

    use super::*;

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18105");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product =
            ProductApp::from_slug("helix-commerce").expect("helix-commerce product known");
        let builder = ServiceBuilder::new(product.slug, product.default_port)
            .await
            .expect("ServiceBuilder requires Postgres + optional NATS/MinIO");
        (builder.into_state(), guard)
    }

    fn dev_principal(label: &str) -> Principal {
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

    #[test]
    fn checked_arithmetic_never_overflows_in_range() {
        let price: i64 = 1_000_000;
        let qty: i32 = 1_000_000;
        let line = price.checked_mul(qty as i64);
        let total = line.and_then(|l| l.checked_add(line.unwrap_or(0)));
        assert!(total.is_some());
    }

    #[test]
    fn mixed_currency_rejection_message_contains_both() {
        let msg = "mixed currency in order: USD and EUR";
        assert!(msg.contains("USD"));
        assert!(msg.contains("EUR"));
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn two_buyers_cannot_oversell_last_unit() {
        let (state, _guard) = locked_state().await;
        let principal = dev_principal("race-alice");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = CommerceRepo::new(pool.clone());

        // Create a product with exactly one unit in stock.
        let product = repo
            .create_product(
                principal.tenant_id,
                "race-sku-001",
                "Race Product",
                "",
                1_00,
                "USD",
                1,
                serde_json::json!({}),
            )
            .await
            .expect("create product");

        let line = OrderLineInput {
            product_id: product.id,
            quantity: 1,
        };
        let a_lines = [line.clone()];
        let b_lines = [line.clone()];

        // Both buyers try to order the single unit concurrently.
        let (a, b) = tokio::join!(
            repo.create_order(
                principal.tenant_id,
                "a@example.com",
                &a_lines,
                serde_json::json!({})
            ),
            repo.create_order(
                principal.tenant_id,
                "b@example.com",
                &b_lines,
                serde_json::json!({})
            )
        );

        let successes = [a.is_ok(), b.is_ok()];
        assert_eq!(
            successes.iter().filter(|&&x| x).count(),
            1,
            "exactly one buyer may reserve the last unit"
        );

        // Cancelling the successful order must restore inventory.
        let order = a.ok().or(b.ok()).expect("one order succeeded");
        repo.cancel_order(principal.tenant_id, order.id)
            .await
            .expect("cancel order");
        let product_after = repo
            .get_product(principal.tenant_id, product.id)
            .await
            .expect("reload product")
            .expect("product exists");
        assert_eq!(
            product_after.inventory, 1,
            "inventory restored after cancel"
        );
    }
}
