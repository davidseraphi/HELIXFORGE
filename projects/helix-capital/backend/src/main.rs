//! HelixCapital API — AI financial operating system (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use helix_db::{CapitalRepo, JournalLineInput};
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-capital")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder.clients().agents.register_agent(agent_framework::AgentSpec {
        name: format!("{}-assistant", product.slug),
        description: format!("{} assistant", product.title),
        system_prompt: format!(
            "You are the {} finance assistant. Help manage chart of accounts and balanced journals.",
            product.title
        ),
        tools: vec!["echo".into(), "product_catalog".into()],
        max_steps: 8,
    });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .nest_service("/", domain_routes().with_state(state.clone()));

    let cfg = shared_core::CoreConfig::from_env("helix-capital", 8107)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-capital", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/accounts", get(list_accounts).post(create_account))
        .route("/v1/accounts/{id}", get(get_account))
        .route("/v1/journals", get(list_journals).post(post_journal))
        .route("/v1/journals/{id}", get(get_journal))
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

async fn list_accounts(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = CapitalRepo::new(pool.clone());
        let items = repo.list_accounts(p.tenant_id).await?;
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
struct CreateAccount {
    code: String,
    name: String,
    #[serde(default = "default_kind")]
    kind: String,
    #[serde(default = "default_currency")]
    currency: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

fn default_kind() -> String {
    "asset".into()
}

fn default_currency() -> String {
    "USD".into()
}

async fn create_account(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateAccount>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.code.trim().is_empty() {
        return Err(HelixError::validation("code required").into());
    }
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let account = repo
        .create_account(
            p.tenant_id,
            body.code.trim(),
            body.name.trim(),
            &body.kind,
            &body.currency,
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
            action: "account.create".into(),
            resource_type: "account".into(),
            resource_id: account.id.to_string(),
            metadata: serde_json::json!({"code": account.code, "kind": account.kind}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-capital",
            "accounts.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(account))))
}

async fn get_account(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let account = repo
        .get_account(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("account not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(account))))
}

async fn list_journals(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = CapitalRepo::new(pool.clone());
        let items = repo.list_journals(p.tenant_id).await?;
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
struct PostJournalLine {
    account_id: Uuid,
    side: String,
    amount_cents: i64,
    #[serde(default)]
    memo: String,
}

#[derive(Deserialize)]
struct PostJournal {
    #[serde(default)]
    memo: String,
    #[serde(default = "default_currency")]
    currency: String,
    lines: Vec<PostJournalLine>,
    #[serde(default)]
    metadata: serde_json::Value,
}

async fn post_journal(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<PostJournal>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let lines: Vec<JournalLineInput> = body
        .lines
        .into_iter()
        .map(|l| JournalLineInput {
            account_id: l.account_id,
            side: l.side,
            amount_cents: l.amount_cents,
            memo: l.memo,
        })
        .collect();
    let repo = CapitalRepo::new(pool.clone());
    let journal = repo
        .post_journal(
            p.tenant_id,
            &body.memo,
            &body.currency,
            &lines,
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
            action: "journal.post".into(),
            resource_type: "journal".into(),
            resource_id: journal.id.to_string(),
            metadata: serde_json::json!({
                "lines": journal.lines.len(),
                "memo": journal.memo
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-capital",
            "journals.posted",
            1.0,
            "count",
            serde_json::json!({"lines": journal.lines.len()}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.capital.journal.posted",
            &serde_json::json!({
                "journal_id": journal.id,
                "lines": journal.lines.len()
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(journal))))
}

async fn get_journal(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let journal = repo
        .get_journal(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("journal not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(journal))))
}
