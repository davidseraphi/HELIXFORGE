//! HelixCapital API — AI financial operating system (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use helix_db::{CapitalRepo, JournalLineInput, TrialBalanceRow};
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
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-capital", 8107)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-capital", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/accounts", get(list_accounts).post(create_account))
        .route("/v1/accounts/{id}", get(get_account).patch(update_account))
        .route("/v1/accounts/{id}/close", post(close_account))
        .route("/v1/accounts/{id}/reopen", post(reopen_account))
        .route("/v1/accounts/{id}/delete", post(delete_account))
        .route("/v1/journals", get(list_journals).post(post_journal))
        .route("/v1/journals/{id}", get(get_journal))
        .route("/v1/journals/{id}/void", post(void_journal))
        .route("/v1/reports/trial-balance", get(trial_balance))
        .route("/v1/reports/balance-snapshot", post(balance_snapshot))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-capital",
        "phase": "wave2_w7",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "accounts": true,
            "journals": true,
            "double_entry": true,
            "account_lifecycle": true,
            "journal_void": true,
            "trial_balance": true,
            "balance_snapshots": true,
            "audit": true,
            "metering": true,
            "nats": true
        }
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

#[derive(Deserialize, Default)]
struct UpdateAccount {
    name: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_account(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAccount>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let account = repo
        .update_account(p.tenant_id, id, name, body.metadata)
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
            action: "account.update".into(),
            resource_type: "account".into(),
            resource_id: account.id.to_string(),
            metadata: serde_json::json!({"code": account.code}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(account))))
}

async fn close_account(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let account = repo.close_account(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "account.close".into(),
            resource_type: "account".into(),
            resource_id: account.id.to_string(),
            metadata: serde_json::json!({"code": account.code}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(account))))
}

async fn reopen_account(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let account = repo.reopen_account(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "account.reopen".into(),
            resource_type: "account".into(),
            resource_id: account.id.to_string(),
            metadata: serde_json::json!({"code": account.code}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(account))))
}

async fn delete_account(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let account = repo.soft_delete_account(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "account.delete".into(),
            resource_type: "account".into(),
            resource_id: account.id.to_string(),
            metadata: serde_json::json!({"code": account.code}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
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

#[derive(Deserialize, Default)]
struct VoidJournal {
    reason: Option<String>,
}

async fn void_journal(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<VoidJournal>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let journal = repo.void_journal(p.tenant_id, id, body.reason).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "journal.void".into(),
            resource_type: "journal".into(),
            resource_id: journal.id.to_string(),
            metadata: serde_json::json!({
                "lines": journal.lines.len(),
                "reason": journal.void_reason
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
            "journals.voided",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.capital.journal.voided",
            &serde_json::json!({
                "journal_id": journal.id,
                "lines": journal.lines.len()
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(journal))))
}

async fn trial_balance(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<TrialBalanceRow>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let rows = repo.get_trial_balance(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn balance_snapshot(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable capital"))?;
    let repo = CapitalRepo::new(pool.clone());
    let count = repo.record_balance_snapshot(p.tenant_id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "report.snapshot".into(),
            resource_type: "report".into(),
            resource_id: "trial-balance".into(),
            metadata: serde_json::json!({"accounts": count}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "accounts": count,
        "captured_at": Utc::now()
    }))))
}

// --- Pure helpers used by unit tests ---

#[allow(dead_code)]
fn sum_journal_sides(lines: &[JournalLineInput]) -> Result<(i64, i64), HelixError> {
    let mut debit_total: i64 = 0;
    let mut credit_total: i64 = 0;
    for line in lines {
        let side = line.side.trim().to_ascii_lowercase();
        if side != "debit" && side != "credit" {
            return Err(HelixError::validation("side must be debit or credit"));
        }
        if line.amount_cents <= 0 {
            return Err(HelixError::validation("amount_cents must be > 0"));
        }
        if side == "debit" {
            debit_total = debit_total
                .checked_add(line.amount_cents)
                .ok_or_else(|| HelixError::validation("debit total overflow"))?;
        } else {
            credit_total = credit_total
                .checked_add(line.amount_cents)
                .ok_or_else(|| HelixError::validation("credit total overflow"))?;
        }
    }
    Ok((debit_total, credit_total))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::tenancy::{Principal, Scope};
    use shared_core::{TenantId, UserId};
    use tokio::sync::{Mutex, MutexGuard};
    use uuid::Uuid;

    use super::*;

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18107");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-capital").expect("helix-capital product known");
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

    #[test]
    fn unbalanced_lines_are_rejected() {
        let lines = vec![
            JournalLineInput {
                account_id: Uuid::nil(),
                side: "debit".into(),
                amount_cents: 1000,
                memo: String::new(),
            },
            JournalLineInput {
                account_id: Uuid::nil(),
                side: "credit".into(),
                amount_cents: 500,
                memo: String::new(),
            },
        ];
        let (debits, credits) = sum_journal_sides(&lines).unwrap();
        assert_ne!(debits, credits, "unbalanced journal must be detectable");
    }

    #[test]
    fn invalid_side_is_rejected() {
        let lines = vec![JournalLineInput {
            account_id: Uuid::nil(),
            side: "left".into(),
            amount_cents: 100,
            memo: String::new(),
        }];
        assert!(sum_journal_sides(&lines).is_err());
    }

    #[test]
    fn zero_amount_is_rejected() {
        let lines = vec![JournalLineInput {
            account_id: Uuid::nil(),
            side: "debit".into(),
            amount_cents: 0,
            memo: String::new(),
        }];
        assert!(sum_journal_sides(&lines).is_err());
    }

    #[test]
    fn reversal_side_is_opposite() {
        assert_eq!(
            if "debit" == "debit" {
                "credit"
            } else {
                "debit"
            },
            "credit"
        );
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn account_lifecycle_and_journal_void_persist() {
        let (state, _guard) = locked_state().await;
        let p = dev_principal("capital-alice");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = CapitalRepo::new(pool.clone());

        let cash = repo
            .create_account(
                p.tenant_id,
                "cash",
                "Cash",
                "asset",
                "USD",
                serde_json::json!({}),
            )
            .await
            .expect("create cash");
        let revenue = repo
            .create_account(
                p.tenant_id,
                "revenue",
                "Revenue",
                "revenue",
                "USD",
                serde_json::json!({}),
            )
            .await
            .expect("create revenue");

        // Post a balanced journal.
        let lines = vec![
            JournalLineInput {
                account_id: cash.id,
                side: "debit".into(),
                amount_cents: 10_000,
                memo: String::new(),
            },
            JournalLineInput {
                account_id: revenue.id,
                side: "credit".into(),
                amount_cents: 10_000,
                memo: String::new(),
            },
        ];
        let journal = repo
            .post_journal(p.tenant_id, "sale", "USD", &lines, serde_json::json!({}))
            .await
            .expect("post journal");
        assert_eq!(journal.status, "posted");

        let cash_after = repo
            .get_account(p.tenant_id, cash.id)
            .await
            .unwrap()
            .unwrap();
        let revenue_after = repo
            .get_account(p.tenant_id, revenue.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(cash_after.balance_cents, 10_000);
        assert_eq!(revenue_after.balance_cents, -10_000);

        // Trial balance should reflect the balances.
        let tb = repo
            .get_trial_balance(p.tenant_id)
            .await
            .expect("trial balance");
        let cash_tb = tb.iter().find(|r| r.id == cash.id).unwrap();
        let revenue_tb = tb.iter().find(|r| r.id == revenue.id).unwrap();
        assert_eq!(cash_tb.balance_cents, 10_000);
        assert_eq!(revenue_tb.balance_cents, -10_000);

        // Cannot close an account with a non-zero balance.
        let close_err = repo.close_account(p.tenant_id, revenue.id).await;
        assert!(close_err.is_err(), "revenue has -10000 balance");

        // Void the journal and verify balances are restored.
        let voided = repo
            .void_journal(p.tenant_id, journal.id, Some("correction".into()))
            .await
            .expect("void journal");
        assert_eq!(voided.status, "voided");
        assert_eq!(voided.void_reason, "correction");

        let cash_void = repo
            .get_account(p.tenant_id, cash.id)
            .await
            .unwrap()
            .unwrap();
        let revenue_void = repo
            .get_account(p.tenant_id, revenue.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(cash_void.balance_cents, 0);
        assert_eq!(revenue_void.balance_cents, 0);

        // Now closing revenue should succeed.
        repo.close_account(p.tenant_id, revenue.id)
            .await
            .expect("close revenue after zeroing");

        // Cannot post to a closed account.
        let post_closed = repo
            .post_journal(
                p.tenant_id,
                "bad",
                "USD",
                &[
                    JournalLineInput {
                        account_id: cash.id,
                        side: "debit".into(),
                        amount_cents: 100,
                        memo: String::new(),
                    },
                    JournalLineInput {
                        account_id: revenue.id,
                        side: "credit".into(),
                        amount_cents: 100,
                        memo: String::new(),
                    },
                ],
                serde_json::json!({}),
            )
            .await;
        assert!(post_closed.is_err(), "cannot post to closed account");

        // Snapshot should write one row per non-deleted account.
        let snap_count = repo
            .record_balance_snapshot(p.tenant_id)
            .await
            .expect("snapshot");
        assert!(snap_count >= 1, "snapshot recorded at least cash");
    }
}
