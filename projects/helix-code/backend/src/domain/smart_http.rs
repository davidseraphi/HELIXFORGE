//! Git smart HTTP plane — real `git` pack servers over HTTP.
//!
//! Auth: `RequireAuth` (session/dev/api-key) **or** `x-helix-deploy-key: hdk_…`
//! (repo-scoped read/write deploy keys).
//!
//! Paths:
//! - `GET  /v1/git/{repo_name}/info/refs?service=git-upload-pack|git-receive-pack`
//! - `POST /v1/git/{repo_name}/git-upload-pack`
//! - `POST /v1/git/{repo_name}/git-receive-pack`

use super::branch_protection;
use super::git_store::{git_pack_advertise, git_pack_rpc, pkt_service_header, GitStore};
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use helix_db::CodeRepoStore;
use serde::Deserialize;
use service_kit::{ApiError, AppState};
use shared_core::ids::TenantId;
use shared_core::tenancy::{Principal, Scope};
use shared_core::HelixError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/git/{repo_name}/info/refs", get(info_refs))
        .route("/v1/git/{repo_name}/git-upload-pack", post(upload_pack))
        .route("/v1/git/{repo_name}/git-receive-pack", post(receive_pack))
}

#[derive(Deserialize)]
struct InfoRefsQuery {
    service: String,
}

struct GitAuth {
    tenant_id: TenantId,
    /// When set, deploy key is bound to this repo id only.
    deploy_repo_id: Option<uuid::Uuid>,
    #[allow(dead_code)]
    can_write: bool,
}

async fn resolve_git_auth(
    state: &AppState,
    headers: &HeaderMap,
    need_write: bool,
) -> Result<GitAuth, ApiError> {
    // 1) Deploy key (machine git clone/push)
    if let Some(raw) = headers
        .get("x-helix-deploy-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let pool = state
            .clients
            .db
            .as_ref()
            .ok_or_else(|| HelixError::unavailable("Postgres required for deploy keys"))?;
        let store = CodeRepoStore::new(pool.clone());
        let resolved = store
            .resolve_deploy_key(raw)
            .await?
            .ok_or_else(|| HelixError::unauthorized("invalid deploy key"))?;
        let (tenant_id, repo_id, scope) = resolved;
        let can_write = scope == "write";
        if need_write && !can_write {
            return Err(HelixError::forbidden("deploy key is read-only").into());
        }
        return Ok(GitAuth {
            tenant_id,
            deploy_repo_id: Some(repo_id),
            can_write,
        });
    }

    // 2) Standard principal (session / dev header / API key)
    let session = headers
        .get("x-session-token")
        .or_else(|| headers.get(header::AUTHORIZATION))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.strip_prefix("Bearer ").unwrap_or(s).trim().to_string());
    let dev_user = headers
        .get("x-helix-dev-user")
        .and_then(|v| v.to_str().ok());
    let p: Principal = state
        .clients
        .auth
        .resolve(session.as_deref(), dev_user)
        .await
        .map_err(ApiError)?;
    p.require_scope(Scope::Read)?;
    if need_write {
        p.require_scope(Scope::Write)?;
    }
    Ok(GitAuth {
        tenant_id: p.tenant_id,
        deploy_repo_id: None,
        can_write: p.has_scope(&Scope::Write),
    })
}

async fn info_refs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(repo_name): Path<String>,
    Query(q): Query<InfoRefsQuery>,
) -> Result<Response, ApiError> {
    let service = match q.service.as_str() {
        "git-upload-pack" | "git-receive-pack" => q.service.as_str(),
        _ => {
            return Err(HelixError::validation(
                "service must be git-upload-pack or git-receive-pack",
            )
            .into())
        }
    };
    let need_write = service == "git-receive-pack";
    let auth = resolve_git_auth(&state, &headers, need_write).await?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool.clone());
    let repo = store
        .get_by_name(auth.tenant_id, &repo_name)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    if let Some(rid) = auth.deploy_repo_id {
        if rid != repo.id {
            return Err(HelixError::forbidden("deploy key not valid for this repo").into());
        }
    }
    let git = GitStore::from_env();
    let path = git.path_for_smart_http(auth.tenant_id, &repo.name)?;
    let mut body = pkt_service_header(service);
    body.extend(git_pack_advertise(&path, service)?);

    let mut out_headers = HeaderMap::new();
    out_headers.insert(
        header::CONTENT_TYPE,
        format!("application/x-{service}-advertisement")
            .parse()
            .unwrap(),
    );
    out_headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    Ok((StatusCode::OK, out_headers, body).into_response())
}

async fn upload_pack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(repo_name): Path<String>,
    body: Bytes,
) -> Result<Response, ApiError> {
    let auth = resolve_git_auth(&state, &headers, false).await?;
    pack_rpc(
        state,
        auth.tenant_id,
        auth.deploy_repo_id,
        &repo_name,
        "git-upload-pack",
        &body,
    )
    .await
}

async fn receive_pack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(repo_name): Path<String>,
    body: Bytes,
) -> Result<Response, ApiError> {
    let auth = resolve_git_auth(&state, &headers, true).await?;
    // Enforce branch protections BEFORE git-receive-pack mutates the bare repo.
    {
        let pool = state
            .clients
            .db
            .as_ref()
            .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
        let store = CodeRepoStore::new(pool.clone());
        let repo = store
            .get_by_name(auth.tenant_id, &repo_name)
            .await?
            .ok_or_else(|| HelixError::not_found("repo not found"))?;
        if let Some(rid) = auth.deploy_repo_id {
            if rid != repo.id {
                return Err(HelixError::forbidden("deploy key not valid for this repo").into());
            }
        }
        let git = GitStore::from_env();
        let path = git.path_for_smart_http(auth.tenant_id, &repo.name)?;
        branch_protection::enforce_receive_pack(&store, auth.tenant_id, repo.id, &path, &body)
            .await?;
    }
    let resp = pack_rpc(
        state.clone(),
        auth.tenant_id,
        auth.deploy_repo_id,
        &repo_name,
        "git-receive-pack",
        &body,
    )
    .await?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = CodeRepoStore::new(pool.clone());
        if let Ok(Some(repo)) = store.get_by_name(auth.tenant_id, &repo_name).await {
            let git = GitStore::from_env();
            if let Ok(head) = git.head_sha(auth.tenant_id, &repo.name) {
                let _ = store.set_head_sha(auth.tenant_id, repo.id, &head).await;
            }
            if let Ok(refs) = git.list_refs(auth.tenant_id, &repo.name) {
                for r in refs {
                    let _ = store
                        .upsert_ref(
                            auth.tenant_id,
                            repo.id,
                            &r.name,
                            &r.target_sha,
                            r.is_symbolic,
                        )
                        .await;
                }
            }
        }
    }
    Ok(resp)
}

async fn pack_rpc(
    state: AppState,
    tenant_id: TenantId,
    deploy_repo_id: Option<uuid::Uuid>,
    repo_name: &str,
    service: &str,
    body: &[u8],
) -> Result<Response, ApiError> {
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool.clone());
    let repo = store
        .get_by_name(tenant_id, repo_name)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    if let Some(rid) = deploy_repo_id {
        if rid != repo.id {
            return Err(HelixError::forbidden("deploy key not valid for this repo").into());
        }
    }
    let git = GitStore::from_env();
    let path = git.path_for_smart_http(tenant_id, &repo.name)?;
    let out = git_pack_rpc(&path, service, body)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        format!("application/x-{service}-result").parse().unwrap(),
    );
    headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    Ok((StatusCode::OK, headers, out).into_response())
}
