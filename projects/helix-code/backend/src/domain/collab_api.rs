//! ES1 collab: issues, PRs, protections, webhooks, ACL, branches.

use super::git_store::GitStore;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{AclPermission, CodeRepoStore};
use serde::Deserialize;
use service_kit::{ApiError, AppState, RequireAuth};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/repos/{id}/issues", get(list_issues).post(create_issue))
        .route(
            "/v1/repos/{id}/issues/{number}",
            get(get_issue).patch(patch_issue),
        )
        .route("/v1/repos/{id}/pulls", get(list_prs).post(create_pr))
        .route("/v1/repos/{id}/pulls/{number}", get(get_pr))
        .route("/v1/repos/{id}/pulls/{number}/merge", post(merge_pr))
        .route("/v1/repos/{id}/pulls/{number}/reviews", post(add_review))
        .route(
            "/v1/repos/{id}/protections",
            get(list_protections).put(put_protection),
        )
        .route(
            "/v1/repos/{id}/webhooks",
            get(list_webhooks).post(create_webhook),
        )
        .route("/v1/repos/{id}/acl", get(list_acl).post(grant_acl))
        .route("/v1/repos/{id}/branches", post(create_branch))
        .route("/v1/repos/{id}/status", get(git_status))
        .route("/v1/repos/{id}/diff", get(git_diff))
}

async fn require_repo(
    store: &CodeRepoStore,
    tenant: shared_core::ids::TenantId,
    id: Uuid,
) -> Result<helix_db::CodeRepo, ApiError> {
    store
        .get(tenant, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found").into())
}

async fn meter(state: &AppState, tenant: shared_core::ids::TenantId, metric: &str, qty: f64) {
    let _ = state
        .clients
        .billing
        .record_usage(
            tenant,
            "helix-code",
            metric,
            qty,
            "count",
            serde_json::json!({}),
        )
        .await;
}

#[derive(Deserialize)]
struct IssueBody {
    title: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    labels: Vec<String>,
}

async fn create_issue(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<IssueBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let _repo = require_repo(&store, p.tenant_id, id).await?;
    let issue = store
        .create_issue(
            p.tenant_id,
            id,
            body.title.trim(),
            &body.body,
            &p.user_id.to_string(),
            serde_json::json!(body.labels),
        )
        .await?;
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "issue.create".into(),
            resource_type: "issue".into(),
            resource_id: issue.id.to_string(),
            metadata: serde_json::json!({"number": issue.number, "repo_id": id}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    meter(&state, p.tenant_id, "issues.created", 1.0).await;
    deliver_webhooks(&state, &store, p.tenant_id, id, "issue.opened", &issue).await;
    Ok(Json(ApiResponse::ok(serde_json::json!(issue))))
}

async fn list_issues(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_issues(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn get_issue(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, number)): Path<(Uuid, i32)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let item = store
        .list_issues(p.tenant_id, id)
        .await?
        .into_iter()
        .find(|i| i.number == number)
        .ok_or_else(|| HelixError::not_found("issue not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize)]
struct PatchIssue {
    #[serde(default)]
    state: Option<String>,
}

async fn patch_issue(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, number)): Path<(Uuid, i32)>,
    Json(body): Json<PatchIssue>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let st = body.state.as_deref().unwrap_or("open");
    let item = store
        .update_issue_state(p.tenant_id, id, number, st)
        .await?
        .ok_or_else(|| HelixError::not_found("issue not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}

#[derive(Deserialize)]
struct PrBody {
    title: String,
    #[serde(default)]
    body: String,
    source_branch: String,
    #[serde(default = "main_br")]
    target_branch: String,
}
fn main_br() -> String {
    "main".into()
}

async fn create_pr(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<PrBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let repo = require_repo(&store, p.tenant_id, id).await?;
    let git = GitStore::from_env();
    let head = git.head_sha(p.tenant_id, &repo.name).ok();
    let pr = store
        .create_pr(
            p.tenant_id,
            id,
            body.title.trim(),
            &body.body,
            body.source_branch.trim(),
            body.target_branch.trim(),
            &p.user_id.to_string(),
            head.as_deref(),
        )
        .await?;
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "pr.create".into(),
            resource_type: "pull_request".into(),
            resource_id: pr.id.to_string(),
            metadata: serde_json::json!({
                "number": pr.number,
                "source": pr.source_branch,
                "target": pr.target_branch
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    meter(&state, p.tenant_id, "prs.created", 1.0).await;
    deliver_webhooks(&state, &store, p.tenant_id, id, "pr.opened", &pr).await;
    Ok(Json(ApiResponse::ok(serde_json::json!(pr))))
}

async fn list_prs(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_prs(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn get_pr(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, number)): Path<(Uuid, i32)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let pr = store
        .get_pr(p.tenant_id, id, number)
        .await?
        .ok_or_else(|| HelixError::not_found("pr not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(pr))))
}

async fn merge_pr(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, number)): Path<(Uuid, i32)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let repo = require_repo(&store, p.tenant_id, id).await?;
    let pr = store
        .get_pr(p.tenant_id, id, number)
        .await?
        .ok_or_else(|| HelixError::not_found("pr not found"))?;
    if pr.state != "open" {
        return Err(HelixError::validation("pr not open").into());
    }
    if let Some(prot) = store
        .matching_protection(p.tenant_id, id, &pr.target_branch)
        .await?
    {
        if prot.require_approvals > 0 {
            let n = store.count_pr_approvals(pr.id).await?;
            if n < prot.require_approvals as i64 {
                return Err(HelixError::validation(format!(
                    "need {} approvals, have {n}",
                    prot.require_approvals
                ))
                .into());
            }
        }
    }
    let git = GitStore::from_env();
    // Head of source branch for required_status_checks
    let head_sha =
        super::branch_protection::branch_tip_sha(&git, p.tenant_id, &repo.name, &pr.source_branch)
            .or_else(|_| {
                pr.head_sha
                    .clone()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        HelixError::validation("cannot resolve PR head SHA for status checks")
                    })
            })?;
    super::branch_protection::enforce_required_status_checks(
        &store,
        p.tenant_id,
        id,
        &pr.target_branch,
        &head_sha,
    )
    .await?;
    let sha = git
        .merge_branch(
            p.tenant_id,
            &repo.name,
            &pr.source_branch,
            &pr.target_branch,
            &format!("Merge PR #{}: {}", pr.number, pr.title),
        )
        .map_err(|e| HelixError::dependency(e.to_string()))?;
    store.mark_pr_merged(p.tenant_id, id, number, &sha).await?;
    let _ = store.set_head_sha(p.tenant_id, id, &sha).await;
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "pr.merge".into(),
            resource_type: "pull_request".into(),
            resource_id: pr.id.to_string(),
            metadata: serde_json::json!({"merge_sha": sha, "number": number}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    meter(&state, p.tenant_id, "prs.merged", 1.0).await;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "merged": true,
        "merge_sha": sha,
        "number": number
    }))))
}

#[derive(Deserialize)]
struct ReviewBody {
    #[serde(default = "approve_default")]
    state: String,
    #[serde(default)]
    body: String,
}
fn approve_default() -> String {
    "comment".into()
}

async fn add_review(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, number)): Path<(Uuid, i32)>,
    Json(body): Json<ReviewBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let pr = store
        .get_pr(p.tenant_id, id, number)
        .await?
        .ok_or_else(|| HelixError::not_found("pr not found"))?;
    let rev = store
        .add_pr_review(
            p.tenant_id,
            pr.id,
            &p.user_id.to_string(),
            body.state.trim(),
            &body.body,
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(rev))))
}

#[derive(Deserialize)]
struct ProtectionBody {
    branch_pattern: String,
    #[serde(default = "true_d")]
    require_pr: bool,
    #[serde(default)]
    require_approvals: i32,
    #[serde(default = "true_d")]
    deny_force_push: bool,
    #[serde(default)]
    required_status_checks: Vec<String>,
}
fn true_d() -> bool {
    true
}

async fn put_protection(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<ProtectionBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let _ = require_repo(&store, p.tenant_id, id).await?;
    let prot = store
        .upsert_protection(
            p.tenant_id,
            id,
            body.branch_pattern.trim(),
            body.require_pr,
            body.require_approvals,
            body.deny_force_push,
            serde_json::json!(body.required_status_checks),
        )
        .await?;
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "branch.protect".into(),
            resource_type: "branch_protection".into(),
            resource_id: prot.id.to_string(),
            metadata: serde_json::json!({"pattern": prot.branch_pattern}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(prot))))
}

async fn list_protections(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_protections(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct WebhookBody {
    url: String,
    #[serde(default)]
    secret: String,
    #[serde(default)]
    events: Vec<String>,
}

async fn create_webhook(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<WebhookBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let _ = require_repo(&store, p.tenant_id, id).await?;
    super::webhook_policy::validate_webhook_url(body.url.trim())?;
    let events = if body.events.is_empty() {
        serde_json::json!(["*"])
    } else {
        serde_json::json!(body.events)
    };
    let wh = store
        .create_webhook(p.tenant_id, id, body.url.trim(), &body.secret, events)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(wh))))
}

async fn list_webhooks(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_webhooks(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct AclBody {
    principal_kind: String,
    principal_id: String,
    #[serde(default = "write_perm")]
    permission: String,
}
fn write_perm() -> String {
    "write".into()
}

async fn grant_acl(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<AclBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let acl = state
        .clients
        .acl
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("ACL store unavailable"))?;
    let perm = match body.permission.as_str() {
        "admin" => AclPermission::Admin,
        "read" => AclPermission::Read,
        _ => AclPermission::Write,
    };
    let granted_by = p.user_id.to_string();
    let entry = acl
        .grant(
            p.tenant_id,
            "repo",
            &id.to_string(),
            &body.principal_kind,
            &body.principal_id,
            &[perm],
            Some(granted_by.as_str()),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(entry))))
}

async fn list_acl(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let acl = state
        .clients
        .acl
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("ACL store unavailable"))?;
    let items = acl
        .list_for_resource(p.tenant_id, "repo", &id.to_string())
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct BranchBody {
    name: String,
    #[serde(default = "main_br")]
    from: String,
}

async fn create_branch(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<BranchBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let repo = require_repo(&store, p.tenant_id, id).await?;
    let git = GitStore::from_env();
    let sha = git.create_branch(p.tenant_id, &repo.name, body.name.trim(), body.from.trim())?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({"name": body.name, "sha": sha}),
    )))
}

async fn git_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let repo = require_repo(&store, p.tenant_id, id).await?;
    let git = GitStore::from_env();
    let status = git.status_summary(p.tenant_id, &repo.name)?;
    Ok(Json(ApiResponse::ok(status)))
}

#[derive(Deserialize)]
struct DiffQuery {
    #[serde(default)]
    path: String,
    #[serde(default = "main_br")]
    rev: String,
}

async fn git_diff(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    axum::extract::Query(q): axum::extract::Query<DiffQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let repo = require_repo(&store, p.tenant_id, id).await?;
    let git = GitStore::from_env();
    let diff = git.diff_path(p.tenant_id, &repo.name, &q.rev, &q.path)?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({"path": q.path, "rev": q.rev, "diff": diff}),
    )))
}

async fn deliver_webhooks<T: serde::Serialize>(
    state: &AppState,
    store: &CodeRepoStore,
    tenant_id: shared_core::ids::TenantId,
    repo_id: Uuid,
    event: &str,
    payload: &T,
) {
    let hooks = match store.list_webhooks(tenant_id, repo_id).await {
        Ok(h) => h,
        Err(_) => return,
    };
    let body = serde_json::json!({"event": event, "payload": payload});
    let body_bytes = serde_json::to_vec(&body).unwrap_or_default();
    for h in hooks {
        if !h.active {
            continue;
        }
        // SSRF: re-resolve at deliver time (DNS rebinding mitigation for policy)
        let (url, ips) = match super::webhook_policy::parse_and_resolve(&h.url) {
            Ok(v) => v,
            Err(e) => {
                let _ = store
                    .record_webhook_delivery(
                        tenant_id,
                        h.id,
                        event,
                        "blocked_ssrf",
                        None,
                        serde_json::json!({"error": e.to_string(), "url": h.url}),
                    )
                    .await;
                continue;
            }
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        // Sign once
        let mut sig_header: Option<String> = None;
        if !h.secret.is_empty() {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            type HmacSha256 = Hmac<Sha256>;
            if let Ok(mut mac) = HmacSha256::new_from_slice(h.secret.as_bytes()) {
                mac.update(&body_bytes);
                sig_header = Some(format!(
                    "sha256={}",
                    hex::encode(mac.finalize().into_bytes())
                ));
            }
        }

        // http: pin to first allowed IP + Host header
        // https: use original URL after policy (TLS SNI needs hostname; IPs re-checked)
        let status;
        let mut code: Option<i32> = None;
        let send_result = if url.scheme() == "http" {
            if let Some(ip) = ips.first().copied() {
                if let Ok((target, host_header, _)) =
                    super::webhook_policy::pinned_request_target(&url, ip)
                {
                    let mut req = client
                        .post(&target)
                        .header("Host", host_header)
                        .body(body_bytes.clone());
                    if let Some(ref s) = sig_header {
                        req = req.header("X-Helix-Webhook-Signature", s);
                    }
                    req.send().await
                } else {
                    let mut req = client.post(url.as_str()).body(body_bytes.clone());
                    if let Some(ref s) = sig_header {
                        req = req.header("X-Helix-Webhook-Signature", s);
                    }
                    req.send().await
                }
            } else {
                let mut req = client.post(url.as_str()).body(body_bytes.clone());
                if let Some(ref s) = sig_header {
                    req = req.header("X-Helix-Webhook-Signature", s);
                }
                req.send().await
            }
        } else {
            let mut req = client.post(url.as_str()).body(body_bytes.clone());
            if let Some(ref s) = sig_header {
                req = req.header("X-Helix-Webhook-Signature", s);
            }
            req.send().await
        };

        match send_result {
            Ok(resp) => {
                code = Some(resp.status().as_u16() as i32);
                status = if resp.status().is_success() {
                    "delivered"
                } else {
                    "http_error"
                };
            }
            Err(_) => status = "delivery_failed",
        }

        let _ = store
            .record_webhook_delivery(tenant_id, h.id, event, status, code, body.clone())
            .await;
        let _ = state
            .clients
            .bus
            .publish(
                "helix.helix-code.webhook.delivered",
                &serde_json::json!({
                    "webhook_id": h.id,
                    "event": event,
                    "status": status,
                    "resolved_ips": ips.iter().map(|i| i.to_string()).collect::<Vec<_>>(),
                }),
            )
            .await;
    }
}
