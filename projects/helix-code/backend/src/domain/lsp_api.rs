//! E3 REST surface for LSP sessions.

use super::git_store::GitStore;
use super::lsp_bridge;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::CodeRepoStore;
use serde::Deserialize;
use service_kit::{ApiError, AppState, RequireAuth};
use shared_core::{ApiResponse, HelixError};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/repos/{id}/lsp/session",
            post(open_session).delete(close_by_repo),
        )
        .route(
            "/v1/lsp/sessions/{session_id}",
            get(get_session).delete(close_session),
        )
        .route("/v1/lsp/sessions/{session_id}/did-open", post(did_open))
        .route("/v1/lsp/sessions/{session_id}/did-change", post(did_change))
        .route(
            "/v1/lsp/sessions/{session_id}/diagnostics",
            get(diagnostics),
        )
        .route("/v1/lsp/sessions/{session_id}/hover", post(hover))
        .route("/v1/lsp/sessions/{session_id}/completion", post(completion))
        .route("/v1/lsp/sessions/{session_id}/definition", post(definition))
        .route("/v1/lsp/status", get(lsp_status))
}

async fn lsp_status(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "available": lsp_bridge::lsp_available(),
        "command": lsp_bridge::lsp_command(),
        "instance_id": lsp_bridge::instance_id(),
        "sticky": true,
        "plane": "E3"
    }))))
}

#[derive(Deserialize)]
struct OpenSessionBody {
    #[serde(default = "default_rev")]
    rev: String,
}

fn default_rev() -> String {
    "main".into()
}

async fn open_session(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<OpenSessionBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let git = GitStore::from_env();
    let bare = git.path_for_smart_http(p.tenant_id, &repo.name)?;
    let rev = if body.rev.is_empty() {
        "main".into()
    } else {
        body.rev
    };
    // Blocking: clone + initialize rust-analyzer
    let tenant = p.tenant_id;
    let name = repo.name.clone();
    let info = tokio::task::spawn_blocking(move || {
        lsp_bridge::open_session(tenant, id, &name, &bare, &rev)
    })
    .await
    .map_err(|e| HelixError::internal(format!("lsp join: {e}")))??;
    // Sticky registry for multi-instance LB routing
    if let Some(sid) = info
        .get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
    {
        let root = info
            .get("root")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let server = info
            .get("server")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let _ = store
            .register_lsp_session(
                sid,
                p.tenant_id,
                id,
                &lsp_bridge::instance_id(),
                &server,
                &root,
                7200,
            )
            .await;
    }
    Ok(Json(ApiResponse::ok(info)))
}

async fn close_by_repo(
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let closed = lsp_bridge::close_session_for_repo(p.tenant_id, id)?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "closed": closed }),
    )))
}

async fn get_session(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    match lsp_bridge::get_session(p.tenant_id, session_id) {
        Ok(s) => {
            if let Some(pool) = state.clients.db.as_ref() {
                let store = CodeRepoStore::new(pool.clone());
                let _ = store.heartbeat_lsp_session(session_id).await;
            }
            Ok(Json(ApiResponse::ok(lsp_bridge::session_info(&s))))
        }
        Err(_) => {
            // Sticky miss: session lives on another instance
            if let Some(pool) = state.clients.db.as_ref() {
                let store = CodeRepoStore::new(pool.clone());
                if let Some(reg) = store.get_lsp_session_reg(session_id).await? {
                    return Err(HelixError::dependency(format!(
                        "lsp session sticky_miss instance_id={} (route client to that instance; local={})",
                        reg.instance_id,
                        lsp_bridge::instance_id()
                    ))
                    .into());
                }
            }
            Err(HelixError::not_found("lsp session not found").into())
        }
    }
}

async fn close_session(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let _ = lsp_bridge::close_session(p.tenant_id, session_id);
    if let Some(pool) = state.clients.db.as_ref() {
        let store = CodeRepoStore::new(pool.clone());
        let _ = store.delete_lsp_session_reg(session_id).await;
    }
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "closed": session_id }),
    )))
}

#[derive(Deserialize)]
struct DidOpenBody {
    path: String,
    content: String,
    #[serde(default = "default_lang")]
    language_id: String,
}

fn default_lang() -> String {
    "rust".into()
}

async fn did_open(
    RequireAuth(p): RequireAuth,
    Path(session_id): Path<Uuid>,
    Json(body): Json<DidOpenBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let lang = if body.language_id.is_empty() {
        language_for_path(&body.path)
    } else {
        body.language_id
    };
    let tenant = p.tenant_id;
    let path = body.path.clone();
    let content = body.content.clone();
    tokio::task::spawn_blocking(move || {
        lsp_bridge::did_open(tenant, session_id, &path, &lang, &content)
    })
    .await
    .map_err(|e| HelixError::internal(format!("lsp join: {e}")))??;
    let diags = lsp_bridge::diagnostics(p.tenant_id, session_id, Some(&body.path))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "path": body.path,
        "diagnostics": diags
    }))))
}

#[derive(Deserialize)]
struct DidChangeBody {
    path: String,
    content: String,
}

async fn did_change(
    RequireAuth(p): RequireAuth,
    Path(session_id): Path<Uuid>,
    Json(body): Json<DidChangeBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let tenant = p.tenant_id;
    let path = body.path.clone();
    let content = body.content.clone();
    let version = tokio::task::spawn_blocking(move || {
        lsp_bridge::did_change(tenant, session_id, &path, &content)
    })
    .await
    .map_err(|e| HelixError::internal(format!("lsp join: {e}")))??;
    let diags = lsp_bridge::diagnostics(p.tenant_id, session_id, Some(&body.path))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "version": version,
        "diagnostics": diags
    }))))
}

#[derive(Deserialize)]
struct DiagQuery {
    path: Option<String>,
}

async fn diagnostics(
    RequireAuth(p): RequireAuth,
    Path(session_id): Path<Uuid>,
    Query(q): Query<DiagQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let items = lsp_bridge::diagnostics(p.tenant_id, session_id, q.path.as_deref())?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct PosBody {
    path: String,
    line: u32,
    character: u32,
}

async fn hover(
    RequireAuth(p): RequireAuth,
    Path(session_id): Path<Uuid>,
    Json(body): Json<PosBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let tenant = p.tenant_id;
    let path = body.path;
    let h = tokio::task::spawn_blocking(move || {
        lsp_bridge::hover(tenant, session_id, &path, body.line, body.character)
    })
    .await
    .map_err(|e| HelixError::internal(format!("lsp join: {e}")))??;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "hover": h }))))
}

async fn completion(
    RequireAuth(p): RequireAuth,
    Path(session_id): Path<Uuid>,
    Json(body): Json<PosBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let tenant = p.tenant_id;
    let path = body.path;
    let items = tokio::task::spawn_blocking(move || {
        lsp_bridge::completion(tenant, session_id, &path, body.line, body.character)
    })
    .await
    .map_err(|e| HelixError::internal(format!("lsp join: {e}")))??;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn definition(
    RequireAuth(p): RequireAuth,
    Path(session_id): Path<Uuid>,
    Json(body): Json<PosBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let tenant = p.tenant_id;
    let path = body.path;
    let items = tokio::task::spawn_blocking(move || {
        lsp_bridge::definition(tenant, session_id, &path, body.line, body.character)
    })
    .await
    .map_err(|e| HelixError::internal(format!("lsp join: {e}")))??;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

fn language_for_path(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rs") {
        "rust".into()
    } else if lower.ends_with(".ts") || lower.ends_with(".tsx") {
        "typescript".into()
    } else if lower.ends_with(".js") || lower.ends_with(".jsx") {
        "javascript".into()
    } else if lower.ends_with(".py") {
        "python".into()
    } else if lower.ends_with(".go") {
        "go".into()
    } else {
        "plaintext".into()
    }
}
