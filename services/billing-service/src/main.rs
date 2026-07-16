//! HelixCore Billing Service — usage metering, plans, and tenant summaries.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use billing_client::{
    plan_by_id, PaymentProviderKind, Plan, TenantBillingSummary, TenantPlan, UsageSummary,
};
use chrono::Utc;
use helix_db::{PaymentIntent, PaymentStatus, PaymentStore};
use serde::Deserialize;
use service_kit::{serve_with_shutdown, ApiError, AppState, RequireAuth, ServiceBuilder};
use shared_core::ids::TenantId;
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let builder = ServiceBuilder::new("billing-service", 8083).await?;
    let addr = builder.config().listen_addr;
    let state = builder.into_state();

    let app = ServiceBuilder::base_router(state.clone()).merge(
        Router::new()
            .route("/v1/meter", post(record_meter))
            .route("/v1/plans", get(list_plans))
            .route("/v1/plans/{plan_id}", get(get_plan))
            .route("/v1/payments/provider", get(payment_provider_info))
            .route("/v1/tenants/{tenant_id}/usage/{product}", get(usage))
            .route("/v1/tenants/{tenant_id}/usage", get(usage_all))
            .route(
                "/v1/tenants/{tenant_id}/plan",
                get(get_tenant_plan).put(set_tenant_plan),
            )
            .route("/v1/tenants/{tenant_id}/summary", get(billing_summary))
            .route(
                "/v1/tenants/{tenant_id}/payments",
                get(list_payments).post(create_payment),
            )
            .route("/v1/tenants/{tenant_id}/payments/{id}", get(get_payment))
            .route(
                "/v1/tenants/{tenant_id}/payments/{id}/confirm",
                post(confirm_payment),
            )
            .route(
                "/v1/tenants/{tenant_id}/payments/{id}/cancel",
                post(cancel_payment),
            )
            .route("/v1/payments/webhook/{provider}", post(payment_webhook))
            .with_state(state.clone()),
    );

    serve_with_shutdown(addr, app, "billing-service", state.clone()).await
}

#[derive(Deserialize)]
struct MeterBody {
    tenant_id: String,
    product: String,
    metric: String,
    quantity: f64,
    unit: String,
    #[serde(default)]
    dimensions: serde_json::Value,
    #[serde(default)]
    idempotency_key: Option<String>,
}

async fn record_meter(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<MeterBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    let tid: TenantId = body
        .tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation: cannot meter other tenants").into());
    }
    state
        .clients
        .billing
        .record_usage_idempotent(
            tid,
            &body.product,
            &body.metric,
            body.quantity,
            &body.unit,
            body.dimensions,
            body.idempotency_key,
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"recorded": true}))))
}

async fn list_plans(
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<Vec<Plan>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(billing_client::plan_catalog())))
}

async fn get_plan(
    RequireAuth(principal): RequireAuth,
    Path(plan_id): Path<String>,
) -> Result<Json<ApiResponse<Plan>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let plan =
        plan_by_id(&plan_id).ok_or_else(|| HelixError::not_found(format!("plan {plan_id}")))?;
    Ok(Json(ApiResponse::ok(plan)))
}

async fn usage(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, product)): Path<(String, String)>,
) -> Result<Json<ApiResponse<Vec<UsageSummary>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation: cannot read other tenants").into());
    }
    let summary = state.clients.billing.summarize(tid, &product).await?;
    Ok(Json(ApiResponse::ok(summary)))
}

/// Summarize usage across all known product slugs + helix-core.
async fn usage_all(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation: cannot read other tenants").into());
    }
    let mut products = vec!["helix-core".to_string()];
    products.extend(
        shared_core::PRODUCT_CATALOG
            .iter()
            .map(|p| p.slug.to_string()),
    );
    let mut by_product = serde_json::Map::new();
    for product in products {
        let rows = state.clients.billing.summarize(tid, &product).await?;
        if !rows.is_empty() {
            by_product.insert(product, serde_json::to_value(rows).unwrap_or_default());
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "tenant_id": tid,
        "products": by_product
    }))))
}

async fn get_tenant_plan(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<TenantPlan>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    Ok(Json(ApiResponse::ok(
        state.clients.billing.get_tenant_plan(tid).await?,
    )))
}

#[derive(Deserialize)]
struct SetPlanBody {
    plan_id: String,
}

async fn set_tenant_plan(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
    Json(body): Json<SetPlanBody>,
) -> Result<Json<ApiResponse<TenantPlan>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Admin)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    let plan = state
        .clients
        .billing
        .set_tenant_plan(tid, &body.plan_id)
        .await?;
    Ok(Json(ApiResponse::ok(plan)))
}

async fn billing_summary(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<TenantBillingSummary>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    let mut products = vec!["helix-core".to_string()];
    products.extend(
        shared_core::PRODUCT_CATALOG
            .iter()
            .map(|p| p.slug.to_string()),
    );
    let summary = state
        .clients
        .billing
        .tenant_billing_summary(tid, &products)
        .await?;
    Ok(Json(ApiResponse::ok(summary)))
}

fn payment_store(state: &AppState) -> Result<PaymentStore, HelixError> {
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for payments"))?;
    Ok(PaymentStore::new(pool.clone()))
}

fn ensure_tenant(
    principal: &shared_core::tenancy::Principal,
    tid: TenantId,
) -> Result<(), HelixError> {
    if tid != principal.tenant_id && !principal.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation"));
    }
    Ok(())
}

#[derive(Deserialize)]
struct CreatePaymentBody {
    plan_id: String,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn payment_provider_info(
    State(state): State<AppState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let cfg = &state.clients.config;
    let kind = PaymentProviderKind::parse(&cfg.payment_provider);
    Json(ApiResponse::ok(serde_json::json!({
        "provider": kind.as_str(),
        "stripe_configured": cfg.stripe_secret_key.is_some(),
        "modes": ["local_sim", "stripe"],
        "note": "Set HELIX_PAYMENT_PROVIDER=stripe and STRIPE_SECRET_KEY for Stripe path (webhook + confirm stub)"
    })))
}

/// Create a marketplace payment intent for a plan.
async fn create_payment(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
    Json(body): Json<CreatePaymentBody>,
) -> Result<Json<ApiResponse<PaymentIntent>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let plan = plan_by_id(&body.plan_id)
        .ok_or_else(|| HelixError::not_found(format!("plan {}", body.plan_id)))?;
    let provider = PaymentProviderKind::parse(&state.clients.config.payment_provider);
    let now = Utc::now();
    let intent = PaymentIntent {
        id: Uuid::now_v7(),
        tenant_id: tid,
        plan_id: plan.id.clone(),
        amount_cents: plan.price_cents_month,
        currency: body.currency.unwrap_or_else(|| "usd".into()),
        status: PaymentStatus::Pending,
        provider: provider.as_str().into(),
        provider_ref: None,
        metadata: body.metadata.unwrap_or_else(
            || serde_json::json!({"plan_name": plan.name, "marketplace": "helixforge"}),
        ),
        created_at: now,
        updated_at: now,
    };
    let store = payment_store(&state)?;
    let saved = store.create(&intent).await?;
    state.clients.metrics.inc("billing.payments.created", 1);
    let _ = state
        .clients
        .bus
        .publish(
            "helix.core.payment.created",
            &serde_json::json!({
                "id": saved.id,
                "tenant_id": tid.to_string(),
                "plan_id": saved.plan_id,
                "amount_cents": saved.amount_cents,
                "provider": saved.provider
            }),
        )
        .await;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(tid),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "payment.create".into(),
            resource_type: "payment_intent".into(),
            resource_id: saved.id.to_string(),
            metadata: serde_json::json!({"plan_id": saved.plan_id, "amount_cents": saved.amount_cents}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(saved)))
}

async fn list_payments(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<PaymentIntent>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let store = payment_store(&state)?;
    Ok(Json(ApiResponse::ok(store.list(tid, 50).await?)))
}

async fn get_payment(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, id)): Path<(String, Uuid)>,
) -> Result<Json<ApiResponse<PaymentIntent>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let store = payment_store(&state)?;
    let p = store
        .get(tid, id)
        .await?
        .ok_or_else(|| HelixError::not_found(format!("payment {id}")))?;
    Ok(Json(ApiResponse::ok(p)))
}

/// Confirm payment (local_sim always succeeds; stripe requires STRIPE_SECRET_KEY and marks requires_action unless sim).
async fn confirm_payment(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, id)): Path<(String, Uuid)>,
) -> Result<Json<ApiResponse<PaymentIntent>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let store = payment_store(&state)?;
    let existing = store
        .get(tid, id)
        .await?
        .ok_or_else(|| HelixError::not_found(format!("payment {id}")))?;
    if existing.status == PaymentStatus::Paid {
        return Ok(Json(ApiResponse::ok(existing)));
    }
    if existing.status == PaymentStatus::Cancelled {
        return Err(HelixError::conflict("payment cancelled").into());
    }

    let (status, provider_ref) = match existing.provider.as_str() {
        "stripe" => {
            // Without a live Stripe charge API call, mark requires_action unless force-sim.
            if state.clients.config.stripe_secret_key.is_some()
                && !state.clients.config.stripe_force_sim
            {
                (
                    PaymentStatus::RequiresAction,
                    Some(format!("pi_pending_{}", Uuid::now_v7())),
                )
            } else {
                (
                    PaymentStatus::Paid,
                    Some(format!("pi_sim_{}", Uuid::now_v7())),
                )
            }
        }
        _ => (PaymentStatus::Paid, Some(format!("sim_{}", Uuid::now_v7()))),
    };

    let updated = store
        .update_status(tid, id, status.clone(), provider_ref.clone())
        .await?;

    if updated.status == PaymentStatus::Paid {
        state
            .clients
            .billing
            .set_tenant_plan(tid, &updated.plan_id)
            .await?;
        state.clients.metrics.inc("billing.payments.paid", 1);
        let _ = state
            .clients
            .bus
            .publish(
                "helix.core.payment.paid",
                &serde_json::json!({
                    "id": updated.id,
                    "tenant_id": tid.to_string(),
                    "plan_id": updated.plan_id,
                    "provider_ref": updated.provider_ref
                }),
            )
            .await;
        state
            .clients
            .audit
            .append(AuditEvent {
                tenant_id: Some(tid),
                actor: Actor::User {
                    user_id: principal.user_id,
                    tenant_id: principal.tenant_id,
                },
                action: "payment.paid".into(),
                resource_type: "payment_intent".into(),
                resource_id: updated.id.to_string(),
                metadata: serde_json::json!({"plan_id": updated.plan_id, "provider": updated.provider}),
                residency_region: principal.residency_region.clone(),
            })
            .await?;
    }
    Ok(Json(ApiResponse::ok(updated)))
}

async fn cancel_payment(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, id)): Path<(String, Uuid)>,
) -> Result<Json<ApiResponse<PaymentIntent>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let store = payment_store(&state)?;
    let existing = store
        .get(tid, id)
        .await?
        .ok_or_else(|| HelixError::not_found(format!("payment {id}")))?;
    if existing.status == PaymentStatus::Paid {
        return Err(HelixError::conflict("cannot cancel paid payment").into());
    }
    let cancelled = store
        .update_status(tid, id, PaymentStatus::Cancelled, None)
        .await?;
    state.clients.metrics.inc("billing.payments.cancelled", 1);
    Ok(Json(ApiResponse::ok(cancelled)))
}

#[derive(Deserialize)]
struct WebhookBody {
    #[serde(default)]
    payment_id: Option<Uuid>,
    #[serde(default)]
    tenant_id: Option<String>,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    provider_ref: Option<String>,
}

/// Verify HMAC-SHA256 signature of raw body with `HELIX_WEBHOOK_SECRET`.
/// Header: `X-Helix-Webhook-Signature: hex(hmac_sha256(secret, body))`
/// or Stripe-style `Stripe-Signature: t=...,v1=...` (v1 only, simplified).
fn verify_webhook_signature(
    cfg: &shared_core::config::CoreConfig,
    headers: &axum::http::HeaderMap,
    raw_body: &[u8],
) -> Result<(), HelixError> {
    let secret = cfg.webhook_secret.as_deref().unwrap_or("");
    if secret.is_empty() {
        // Local/dev only may skip if the unsafe opt-in is set AND the operator
        // explicitly enabled unsigned webhook handling.
        if cfg.local_dev_unsafe && cfg.environment == "local" && cfg.webhook_allow_unsigned {
            tracing::warn!(
                "LOCAL-DEV UNSAFE: accepting unsigned webhook (HELIX_WEBHOOK_SECRET unset). \
                 Never enable this in production."
            );
            return Ok(());
        }
        return Err(HelixError::unauthorized(
            "webhook signature required: set HELIX_WEBHOOK_SECRET",
        ));
    }
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let expected = {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| HelixError::internal("invalid webhook secret"))?;
        mac.update(raw_body);
        hex::encode(mac.finalize().into_bytes())
    };

    if let Some(sig) = headers
        .get("x-helix-webhook-signature")
        .and_then(|v| v.to_str().ok())
    {
        let got = sig.trim().strip_prefix("sha256=").unwrap_or(sig.trim());
        if constant_time_eq(got.as_bytes(), expected.as_bytes()) {
            return Ok(());
        }
        return Err(HelixError::unauthorized("invalid webhook signature"));
    }

    // Stripe-style: Stripe-Signature: t=timestamp,v1=hex
    if let Some(sig) = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
    {
        let mut t = None;
        let mut v1 = None;
        for part in sig.split(',') {
            let mut kv = part.trim().splitn(2, '=');
            match (kv.next(), kv.next()) {
                (Some("t"), Some(val)) => t = Some(val),
                (Some("v1"), Some(val)) => v1 = Some(val),
                _ => {}
            }
        }
        if let (Some(ts), Some(v1)) = (t, v1) {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|_| HelixError::internal("invalid webhook secret"))?;
            mac.update(ts.as_bytes());
            mac.update(b".");
            mac.update(raw_body);
            let stripe_expected = hex::encode(mac.finalize().into_bytes());
            if constant_time_eq(v1.as_bytes(), stripe_expected.as_bytes()) {
                return Ok(());
            }
        }
        return Err(HelixError::unauthorized("invalid Stripe-Signature"));
    }

    Err(HelixError::unauthorized(
        "missing X-Helix-Webhook-Signature or Stripe-Signature",
    ))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Provider webhook with HMAC signature verification (Kimi P0).
async fn payment_webhook(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    verify_webhook_signature(&state.clients.config, &headers, &body)?;
    let parsed: WebhookBody = serde_json::from_slice(&body)
        .map_err(|e| HelixError::validation(format!("webhook json: {e}")))?;
    let event = parsed
        .event
        .as_deref()
        .unwrap_or("payment_intent.succeeded");
    if let (Some(pid), Some(tid_s)) = (parsed.payment_id, parsed.tenant_id.as_deref()) {
        let tid: TenantId = tid_s
            .parse()
            .map_err(|_| HelixError::validation("invalid tenant_id"))?;
        let store = payment_store(&state)?;
        if event.contains("succeeded") || event.contains("paid") {
            let paid = store
                .update_status(tid, pid, PaymentStatus::Paid, parsed.provider_ref.clone())
                .await?;
            state
                .clients
                .billing
                .set_tenant_plan(tid, &paid.plan_id)
                .await?;
            let _ = state
                .clients
                .bus
                .publish(
                    "helix.core.payment.paid",
                    &serde_json::json!({"id": paid.id, "source": "webhook", "provider": provider}),
                )
                .await;
            return Ok(Json(ApiResponse::ok(serde_json::json!({
                "handled": true,
                "payment": paid
            }))));
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "handled": false,
        "provider": provider,
        "event": event,
        "note": "Provide payment_id + tenant_id for local webhook processing"
    }))))
}
