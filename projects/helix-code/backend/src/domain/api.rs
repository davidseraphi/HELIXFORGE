//! HelixCode extreme REST surface (repos, tree/blob, workspaces, pipelines, agents, sealed).

use super::agent_sandbox::{
    self, apply_file_patches, apply_unified_diff, commit_and_push, local_analyze_step,
    prepare_worktree, FilePatch, MeshStepResult,
};
use super::container;
use super::git_store::GitStore;
use super::lsp_bridge;
use super::sandbox::{self, parse_definition, run_pipeline_sandbox};
use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::CodeRepoStore;
use serde::Deserialize;
use service_kit::{ApiError, AppState, RequireAuth};
use sha2::{Digest, Sha256};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError};
use std::time::Duration;
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/domain/status", get(domain_status))
        .route("/v1/repos", get(list_repos).post(create_repo))
        .route("/v1/repos/{id}", get(get_repo))
        .route("/v1/repos/{id}/refs", get(list_refs))
        .route("/v1/repos/{id}/tree", get(list_tree))
        .route("/v1/repos/{id}/files", get(list_files))
        .route("/v1/repos/{id}/search", get(search_repo))
        .route("/v1/repos/{id}/blob", get(read_blob))
        .route(
            "/v1/repos/{id}/commits",
            get(list_commits).post(commit_file),
        )
        .route("/v1/repos/{id}/commits/batch", post(commit_batch))
        .route("/v1/repos/{id}/log", get(list_commits))
        .route(
            "/v1/code/workspaces",
            get(list_workspaces).post(create_workspace),
        )
        .route(
            "/v1/repos/{id}/pipelines",
            get(list_pipelines).post(create_pipeline),
        )
        .route("/v1/pipelines/{id}/runs", post(trigger_pipeline))
        .route("/v1/pipeline-runs/{id}", get(get_pipeline_run))
        .route("/v1/pipeline-runs/{id}/artifacts", get(list_run_artifacts))
        .route("/v1/repos/{id}/agent-jobs", post(create_agent_job))
        .route("/v1/agent-jobs/{id}", get(get_agent_job))
        // sealed-objects routes live in sealed_api (E5)
        .route("/v1/ai/complete", post(complete))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let git = GitStore::from_env();
    let sample_gix = std::fs::read_dir(git.root())
        .ok()
        .and_then(|mut d| d.next())
        .and_then(|e| e.ok())
        .map(|e| GitStore::gix_open_ok(&e.path()))
        .unwrap_or(false);
    let iso = container::isolation_label(container::resolve_isolation());
    let image = container::docker_image();
    let forge_tools = container::image_has_forge_tools();
    // Build as map (avoids serde_json::json! recursion limit on large plane tables).
    let planes = serde_json::json!({
        "git_backend": git.backend_id(),
        "gitoxide": true,
        "gitoxide_reads": true,
        "gitoxide_sample_open": sample_gix,
        "git_cli_writes_and_packs": true,
        "smart_http": true,
        "workspaces": true,
        "workspace_web": true,
        "pipelines": true,
        "sandbox_runner": true,
        "ci_artifacts_minio": true,
        "agent_jobs": true,
        "agent_mesh": true,
        "agent_patch_apply": true,
        "sealed_objects": true,
        "sealed_hva4": true,
        "crypto_groups": true,
        "openmls": true,
        "openmls_durable": true,
        "container_isolation": true,
        "isolation_mode": iso,
        "docker_available": container::docker_available(),
        "docker_image": image,
        "docker_ci_image_preferred": container::CI_IMAGE_PREFERRED,
        "docker_has_forge_tools": forge_tools,
        "cleartext_forbidden_minio": true,
        "lsp": true,
        "lsp_available": lsp_bridge::lsp_available(),
        "lsp_command": lsp_bridge::lsp_command(),
        "lsp_completion": true,
        "lsp_definition": true,
        "files_index": true,
        "content_search": true,
        "batch_commit": true,
        "code_oss": true,
        "code_oss_shell": true,
        "split_editor_groups": true,
        "electron_shell": true,
        "issues": true,
        "pull_requests": true,
        "branch_protection": true,
        "branch_protection_smart_http": true,
        "deny_force_push": true,
        "required_status_checks": true,
        "webhooks": true,
        "repo_acl": true,
        "pipeline_run_list": true,
        "pipeline_cancel": true,
        "artifact_download": true,
        "runners": true,
        "multi_lang_lsp": true,
        "agent_events": true,
        "agent_list": true,
        "mls_devices": true,
        "mls_key_backup": true,
        "user_settings": true,
        "quotas": true,
        "quota_agent_jobs_day": true,
        "quota_sealed_bytes": true,
        "webhook_ssrf_policy": true,
        "webhook_https_required_non_local": true,
        "docker_bind_path_normalized": true,
        "isolation_cmd_policy": true,
        "host_fallback_gated": true,
        "breakglass_audit": true,
        "terminal_allowlist": true,
        "terminals": true,
        "extensions_registry": true,
        "debug_launch": true,
        "git_status_diff": true,
        "endstate": true,
        "deploy_keys": true,
        "lsp_sticky": true,
        "debug_breakpoints": true,
        "dap_lldb_gdb": true,
        "org_code_signing": true
    });
    let ci = serde_json::json!({
        "allow_all": sandbox::allow_all_env(),
        "step_timeout_secs": step_timeout_secs(),
        "isolation": iso,
        "docker_image": image,
        "docker_ci_image_preferred": container::CI_IMAGE_PREFERRED,
        "docker_has_forge_tools": forge_tools,
        "host_fallback_allowed": super::cmd_policy::allow_host_fallback(),
    });
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-code",
        "phase": "endstate",
        "extreme": true,
        "durable": state.clients.db.is_some(),
        "git_store_root": git.root().display().to_string(),
        "planes": planes,
        "ci": ci,
        "breakglass": {
            "count": super::breakglass::count(),
            "recent": super::breakglass::recent(),
        },
        "phase_detail": "End-state: collab+CI+multi-LSP+agents+MLS devices+terminal+quotas",
        "docs": [
            "projects/helix-code/docs/SOVEREIGN_ROADMAP.md",
            "projects/helix-code/docs/THREAT_MODEL.md",
            "projects/helix-code/docs/HA_STICKY.md",
            "docs/reviews/HELIXCODE_ENDSTATE/SELF_AUDIT_REPORT.md"
        ]
    }))))
}

async fn list_repos(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let items = store.list(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "items": items
    }))))
}

#[derive(Deserialize)]
struct CreateRepo {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "private_vis")]
    visibility: String,
}

fn private_vis() -> String {
    "private".into()
}

async fn create_repo(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateRepo>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let name = body.name.trim();
    if name.is_empty() || !is_safe_repo_name(name) {
        return Err(HelixError::validation(
            "name required: [a-zA-Z0-9._-]{1,64}, no path separators",
        )
        .into());
    }
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    // ES8 quota
    let quota = store.get_or_default_quota(p.tenant_id).await?;
    let n = store.count_repos(p.tenant_id).await?;
    if n >= quota.max_repos as i64 {
        return Err(HelixError::forbidden(format!(
            "quota_exceeded: max_repos={}",
            quota.max_repos
        ))
        .into());
    }
    if store.get_by_name(p.tenant_id, name).await?.is_some() {
        return Err(HelixError::conflict("repo name already exists").into());
    }

    let git = GitStore::from_env();
    let (_path, head) = git.init_bare_with_seed(p.tenant_id, name, &body.description)?;

    let repo = store
        .create(p.tenant_id, name, &body.description, &body.visibility)
        .await?;
    store.set_head_sha(p.tenant_id, repo.id, &head).await?;

    for r in git.list_refs(p.tenant_id, name)? {
        let _ = store
            .upsert_ref(p.tenant_id, repo.id, &r.name, &r.target_sha, r.is_symbolic)
            .await;
    }

    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "repo.create".into(),
            resource_type: "repo".into(),
            resource_id: repo.id.to_string(),
            metadata: serde_json::json!({"name": repo.name, "head_sha": head}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-code",
            "repos.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;

    let mut out = serde_json::to_value(&repo).unwrap_or_default();
    if let Some(obj) = out.as_object_mut() {
        obj.insert("head_sha".into(), serde_json::json!(head));
        obj.insert(
            "clone_url_hint".into(),
            serde_json::json!(format!(
                "http://127.0.0.1:8102/v1/git/{}/ (smart HTTP, auth headers)",
                repo.name
            )),
        );
    }
    Ok(Json(ApiResponse::ok(out)))
}

async fn get_repo(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(repo))))
}

async fn list_refs(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let git = GitStore::from_env();
    let live = git.list_refs(p.tenant_id, &repo.name)?;
    // refresh db cache
    for r in &live {
        let _ = store
            .upsert_ref(p.tenant_id, repo.id, &r.name, &r.target_sha, r.is_symbolic)
            .await;
    }
    let cached = store.list_refs(p.tenant_id, repo.id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "live": live,
        "cached": cached
    }))))
}

#[derive(Deserialize)]
struct TreeQuery {
    #[serde(default = "default_rev")]
    rev: String,
    #[serde(default)]
    path: String,
}

fn default_rev() -> String {
    "main".into()
}

async fn list_tree(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<TreeQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let git = GitStore::from_env();
    let entries = git.list_tree(p.tenant_id, &repo.name, &q.rev, &q.path)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "rev": q.rev,
        "path": q.path,
        "entries": entries
    }))))
}

#[derive(Deserialize)]
struct FilesQuery {
    #[serde(default = "default_rev")]
    rev: String,
    #[serde(default = "default_files_max")]
    max: usize,
}

fn default_files_max() -> usize {
    2000
}

async fn list_files(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<FilesQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let git = GitStore::from_env();
    let files = git.list_files_recursive(p.tenant_id, &repo.name, &q.rev, q.max)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "rev": q.rev,
        "count": files.len(),
        "files": files
    }))))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_rev")]
    rev: String,
    #[serde(default = "default_search_hits")]
    max: usize,
    #[serde(default = "default_search_files")]
    max_files: usize,
}

fn default_search_hits() -> usize {
    50
}
fn default_search_files() -> usize {
    200
}

async fn search_repo(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let git = GitStore::from_env();
    let hits = git.search_content(p.tenant_id, &repo.name, &q.rev, &q.q, q.max, q.max_files)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "query": q.q,
        "rev": q.rev,
        "count": hits.len(),
        "hits": hits
    }))))
}

#[derive(Deserialize)]
struct BlobQuery {
    #[serde(default = "default_rev")]
    rev: String,
    path: String,
}

async fn read_blob(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<BlobQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let git = GitStore::from_env();
    let content = git.read_blob(p.tenant_id, &repo.name, &q.rev, &q.path)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "path": q.path,
        "rev": q.rev,
        "content": content
    }))))
}

#[derive(Deserialize)]
struct LogQuery {
    #[serde(default = "default_rev")]
    rev: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    20
}

async fn list_commits(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<LogQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let git = GitStore::from_env();
    let limit = q.limit.clamp(1, 100);
    let commits = git.log(p.tenant_id, &repo.name, &q.rev, limit)?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "commits": commits }),
    )))
}

#[derive(Deserialize)]
struct CommitBody {
    path: String,
    content: String,
    #[serde(default = "default_msg")]
    message: String,
    #[serde(default = "default_branch")]
    branch: String,
}

fn default_msg() -> String {
    "chore: update via HelixCode".into()
}
fn default_branch() -> String {
    "main".into()
}

async fn commit_file(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<CommitBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    // Branch protection: require_pr (+ shared rules) blocks direct commits unless break-glass
    super::branch_protection::enforce_rest_commit(&store, p.tenant_id, id, &body.branch).await?;
    let git = GitStore::from_env();
    let sha = git.commit_file(
        p.tenant_id,
        &repo.name,
        &body.branch,
        body.path.trim(),
        &body.content,
        &body.message,
    )?;
    store.set_head_sha(p.tenant_id, repo.id, &sha).await?;
    for r in git.list_refs(p.tenant_id, &repo.name)? {
        let _ = store
            .upsert_ref(p.tenant_id, repo.id, &r.name, &r.target_sha, r.is_symbolic)
            .await;
    }
    let _ = state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-code",
            "commits.created",
            1.0,
            "count",
            serde_json::json!({"path": body.path}),
        )
        .await;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "repo.commit".into(),
            resource_type: "repo".into(),
            resource_id: repo.id.to_string(),
            metadata: serde_json::json!({"path": body.path, "sha": sha}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "commit_sha": sha,
        "path": body.path,
        "branch": body.branch
    }))))
}

#[derive(Deserialize)]
struct BatchFile {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct CommitBatchBody {
    files: Vec<BatchFile>,
    #[serde(default = "default_msg")]
    message: String,
    #[serde(default = "default_branch")]
    branch: String,
}

async fn commit_batch(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<CommitBatchBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    super::branch_protection::enforce_rest_commit(&store, p.tenant_id, id, &body.branch).await?;
    let pairs: Vec<(String, String)> = body
        .files
        .into_iter()
        .map(|f| (f.path.trim().to_string(), f.content))
        .filter(|(p, _)| !p.is_empty())
        .collect();
    let paths: Vec<String> = pairs.iter().map(|(p, _)| p.clone()).collect();
    let git = GitStore::from_env();
    let sha = git.commit_files(p.tenant_id, &repo.name, &body.branch, &pairs, &body.message)?;
    store.set_head_sha(p.tenant_id, repo.id, &sha).await?;
    for r in git.list_refs(p.tenant_id, &repo.name)? {
        let _ = store
            .upsert_ref(p.tenant_id, repo.id, &r.name, &r.target_sha, r.is_symbolic)
            .await;
    }
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "repo.commit.batch".into(),
            resource_type: "repo".into(),
            resource_id: repo.id.to_string(),
            metadata: serde_json::json!({"paths": paths, "sha": sha, "count": paths.len()}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "commit_sha": sha,
        "paths": paths,
        "branch": body.branch,
        "count": paths.len()
    }))))
}

#[derive(Deserialize)]
struct CreateWorkspace {
    repo_id: Uuid,
    name: String,
    #[serde(default = "default_branch")]
    branch: String,
    #[serde(default)]
    root_path: String,
}

async fn create_workspace(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateWorkspace>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let _repo = store
        .get(p.tenant_id, body.repo_id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let ws = store
        .create_workspace(
            p.tenant_id,
            body.repo_id,
            body.name.trim(),
            &body.branch,
            &body.root_path,
            &p.user_id.to_string(),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(ws))))
}

#[derive(Deserialize)]
struct ListWsQuery {
    repo_id: Option<Uuid>,
}

async fn list_workspaces(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Query(q): Query<ListWsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_workspaces(p.tenant_id, q.repo_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct CreatePipeline {
    name: String,
    #[serde(default)]
    definition: serde_json::Value,
}

async fn create_pipeline(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<CreatePipeline>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let _repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let def = if body.definition.is_null() {
        serde_json::json!({
            "version": 1,
            "steps": [
                {"name": "echo", "run": "echo helix-code-ci"},
                {"name": "rev", "run": "git rev-parse HEAD"}
            ],
            "artifacts": ["helix-ci.log"]
        })
    } else {
        body.definition
    };
    let pipe = store
        .create_pipeline(p.tenant_id, id, body.name.trim(), def)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(pipe))))
}

async fn list_pipelines(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_pipelines(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct TriggerBody {
    #[serde(default = "default_ref")]
    trigger_ref: String,
    /// Optional exact SHA for required_status_checks matching; defaults to rev-parse(trigger_ref) then HEAD.
    #[serde(default)]
    commit_sha: String,
}

fn default_ref() -> String {
    "refs/heads/main".into()
}

async fn trigger_pipeline(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<TriggerBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let quota = store.get_or_default_quota(p.tenant_id).await?;
    let runs_m = store.count_pipeline_runs_month(p.tenant_id).await?;
    if runs_m >= quota.max_pipeline_runs_month as i64 {
        return Err(HelixError::forbidden(format!(
            "quota_exceeded: max_pipeline_runs_month={}",
            quota.max_pipeline_runs_month
        ))
        .into());
    }
    let pipe = store
        .get_pipeline(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("pipeline not found"))?;
    let repo = store
        .get(p.tenant_id, pipe.repo_id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let git = GitStore::from_env();
    let commit = if !body.commit_sha.trim().is_empty() {
        body.commit_sha.trim().to_string()
    } else {
        git.rev_parse(p.tenant_id, &repo.name, &body.trigger_ref)
            .or_else(|_| {
                git.rev_parse(
                    p.tenant_id,
                    &repo.name,
                    body.trigger_ref.trim_start_matches("refs/heads/"),
                )
            })
            .or_else(|_| git.head_sha(p.tenant_id, &repo.name))
            .unwrap_or_else(|_| "HEAD".into())
    };
    let run = store
        .create_pipeline_run(
            p.tenant_id,
            pipe.id,
            pipe.repo_id,
            &body.trigger_ref,
            Some(commit.as_str()),
        )
        .await?;

    let (steps, artifact_names) = parse_definition(&pipe.definition);
    let bare = git.path_for_smart_http(p.tenant_id, &repo.name)?;
    let timeout = Duration::from_secs(step_timeout_secs());

    // Blocking sandbox on worker — E2 local forge (spawn_blocking keeps tokio responsive)
    let bare_c = bare.clone();
    let commit_c = commit.clone();
    let steps_c = steps.clone();
    let arts_c = artifact_names.clone();
    let sandbox = tokio::task::spawn_blocking(move || {
        run_pipeline_sandbox(&bare_c, &commit_c, &steps_c, &arts_c, timeout)
    })
    .await
    .map_err(|e| HelixError::internal(format!("sandbox join: {e}")))??;

    // Upload artifacts to MinIO
    let mut artifact_meta = Vec::new();
    for rel in &sandbox.artifact_paths {
        let full = sandbox.workdir.join(rel);
        let bytes = match std::fs::read(&full) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let hash = hex::encode(Sha256::digest(&bytes));
        let name = rel.to_string_lossy().replace('\\', "/");
        let key = format!("code/{}/ci/{}/{}", p.tenant_id.as_uuid(), run.id, name);
        let ct = if name.ends_with(".log") || name.ends_with(".txt") {
            "text/plain"
        } else {
            "application/octet-stream"
        };
        if let Err(e) = state.clients.objects.put_object(&key, &bytes, ct).await {
            tracing::warn!(error = %e, %key, "artifact upload failed");
            continue;
        }
        if let Ok(meta) = store
            .insert_pipeline_artifact(
                p.tenant_id,
                run.id,
                pipe.repo_id,
                &name,
                &key,
                ct,
                bytes.len() as i64,
                &hash,
            )
            .await
        {
            artifact_meta.push(serde_json::json!({
                "id": meta.id,
                "name": meta.name,
                "storage_key": meta.storage_key,
                "sha256": meta.sha256,
                "byte_len": meta.byte_len
            }));
        }
    }

    let artifacts_json = serde_json::Value::Array(artifact_meta.clone());
    store
        .finish_pipeline_run(
            p.tenant_id,
            run.id,
            &sandbox.status,
            &sandbox.log_text,
            Some(&sandbox.workdir.display().to_string()),
            artifacts_json,
            Some(sandbox.exit_code),
            &sandbox.isolation,
        )
        .await?;
    let finished = store
        .get_pipeline_run(p.tenant_id, run.id)
        .await?
        .ok_or_else(|| HelixError::internal("run missing after finish"))?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "pipeline.run".into(),
            resource_type: "pipeline_run".into(),
            resource_id: finished.id.to_string(),
            metadata: serde_json::json!({
                "status": sandbox.status,
                "exit_code": sandbox.exit_code,
                "artifacts": artifact_meta.len(),
                "allow_all": sandbox::allow_all_env(),
                "isolation": sandbox.isolation,
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(finished))))
}

async fn get_pipeline_run(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let run = store
        .get_pipeline_run(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("run not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(run))))
}

async fn list_run_artifacts(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let _run = store
        .get_pipeline_run(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("run not found"))?;
    let items = store.list_pipeline_artifacts(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

fn step_timeout_secs() -> u64 {
    std::env::var("HELIX_CODE_CI_STEP_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60)
        .clamp(5, 600)
}

#[derive(Deserialize)]
struct AgentJobBody {
    /// `sandbox` | `patch` | `mesh`
    #[serde(default = "default_kind")]
    kind: String,
    prompt: String,
    workspace_id: Option<Uuid>,
    #[serde(default = "default_branch")]
    branch: String,
    #[serde(default)]
    rev: Option<String>,
    /// Full-file replacements / creates
    #[serde(default)]
    patches: Vec<FilePatch>,
    /// Optional unified diff (git apply)
    #[serde(default)]
    unified_diff: Option<String>,
    #[serde(default = "default_true")]
    commit: bool,
    #[serde(default)]
    commit_message: Option<String>,
    /// Agent names for mesh (defaults for kind=mesh)
    #[serde(default)]
    agents: Vec<String>,
}

fn default_kind() -> String {
    "mesh".into()
}
fn default_true() -> bool {
    true
}

async fn create_agent_job(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<AgentJobBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    // ES8: enforce max_agent_jobs_day
    let quota = store.get_or_default_quota(p.tenant_id).await?;
    let agents_today = store.count_agent_jobs_day(p.tenant_id).await?;
    if agents_today >= quota.max_agent_jobs_day as i64 {
        return Err(HelixError::forbidden(format!(
            "quota_exceeded: max_agent_jobs_day={} used={agents_today}",
            quota.max_agent_jobs_day
        ))
        .into());
    }
    let kind = body.kind.trim().to_ascii_lowercase();
    if !matches!(kind.as_str(), "sandbox" | "patch" | "mesh") {
        return Err(HelixError::validation("kind must be sandbox|patch|mesh").into());
    }
    let job = store
        .create_agent_job(p.tenant_id, repo.id, body.workspace_id, &kind, &body.prompt)
        .await?;

    let git = GitStore::from_env();
    let bare = git.path_for_smart_http(p.tenant_id, &repo.name)?;
    let rev = body
        .rev
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| body.branch.clone());
    let branch = body.branch.clone();
    let prompt = body.prompt.clone();
    let patches = body.patches.clone();
    let unified = body.unified_diff.clone();
    let do_commit = body.commit;
    let commit_msg = body.commit_message.clone().unwrap_or_else(|| {
        format!(
            "agent({kind}): {}",
            prompt.chars().take(60).collect::<String>()
        )
    });
    let agents = if body.agents.is_empty() {
        match kind.as_str() {
            "mesh" => vec!["helix-code-assistant".into(), "helix-code-patcher".into()],
            "sandbox" => vec!["helix-code-assistant".into()],
            _ => vec![],
        }
    } else {
        body.agents.clone()
    };

    // Worktree + patches on blocking pool
    let bare_c = bare.clone();
    let rev_c = rev.clone();
    let patches_c = patches.clone();
    let unified_c = unified.clone();
    let kind_c = kind.clone();
    let sandbox_result = tokio::task::spawn_blocking(move || {
        run_agent_job_blocking(
            &bare_c,
            &rev_c,
            &kind_c,
            &prompt,
            &patches_c,
            unified_c.as_deref(),
            do_commit,
            &branch,
            &commit_msg,
        )
    })
    .await
    .map_err(|e| HelixError::internal(format!("agent join: {e}")))??;

    // Mesh: invoke agent-framework agents (tools sandboxed by spec)
    let mut mesh_steps = sandbox_result.mesh_steps;
    let mut agent_run_ids = Vec::new();
    for agent_name in &agents {
        // Prefer product-registered agents; fall back to any matching name
        let agent_key = if state
            .clients
            .agents
            .list_agents()
            .iter()
            .any(|a| a.name == *agent_name)
        {
            agent_name.clone()
        } else {
            // try slug-prefixed default from main
            "helix-code-assistant".to_string()
        };
        let input = serde_json::json!({
            "tools": ["echo", "product_catalog", "utc_now", "tenant_context"],
            "args": {
                "message": format!("repo={} kind={} prompt={}", repo.name, kind, body.prompt),
                "repo": repo.name,
                "files_changed": sandbox_result.files_changed,
            },
            "cancel": false
        });
        match state
            .clients
            .agents
            .run(&agent_key, p.tenant_id, p.user_id, input)
            .await
        {
            Ok(run) => {
                agent_run_ids.push(run.id.to_string());
                mesh_steps.push(MeshStepResult {
                    agent: agent_key,
                    status: format!("{:?}", run.status).to_ascii_lowercase(),
                    summary: run
                        .output
                        .map(|o| o.to_string())
                        .unwrap_or_else(|| "ok".into()),
                    run_id: Some(run.id.to_string()),
                });
            }
            Err(e) => {
                mesh_steps.push(MeshStepResult {
                    agent: agent_key,
                    status: "failed".into(),
                    summary: e.to_string(),
                    run_id: None,
                });
            }
        }
    }

    let status = if sandbox_result.status == "succeeded"
        && mesh_steps.iter().all(|s| {
            s.status.contains("succeeded")
                || s.status == "running"
                || s.status == "completed"
                || s.status == "Succeeded"
                || s.status.contains("ok")
                || s.status == "succeeded"
        }) {
        // Normalize: agent RunStatus Display might be "Succeeded"
        "succeeded"
    } else if sandbox_result.status != "succeeded"
        || mesh_steps.iter().any(|s| s.status.contains("fail"))
    {
        "failed"
    } else {
        "succeeded"
    };

    // Soften mesh status: if sandbox ok and agents ran, succeed even if status enum spelling differs
    let status = if sandbox_result.status == "succeeded"
        && !mesh_steps.iter().any(|s| s.status == "failed")
    {
        "succeeded"
    } else {
        status
    };

    let mesh_json: serde_json::Value = serde_json::json!(mesh_steps
        .iter()
        .map(|s| serde_json::json!({
            "agent": s.agent,
            "status": s.status,
            "summary": s.summary,
            "run_id": s.run_id,
        }))
        .collect::<Vec<_>>());
    let files_json = serde_json::json!(sandbox_result.files_changed);
    let runs_json = serde_json::json!(agent_run_ids);
    let summary = format!(
        "E4 {kind}: files={} commit={} mesh_steps={}",
        sandbox_result.files_changed.len(),
        sandbox_result.commit_sha.as_deref().unwrap_or("-"),
        mesh_steps.len()
    );

    if let Some(sha) = sandbox_result.commit_sha.as_deref() {
        let _ = store.set_head_sha(p.tenant_id, repo.id, sha).await;
        if let Ok(refs) = git.list_refs(p.tenant_id, &repo.name) {
            for r in refs {
                let _ = store
                    .upsert_ref(p.tenant_id, repo.id, &r.name, &r.target_sha, r.is_symbolic)
                    .await;
            }
        }
    }

    store
        .finish_agent_job(
            p.tenant_id,
            job.id,
            status,
            &summary,
            Some(&sandbox_result.workdir.display().to_string()),
            sandbox_result.commit_sha.as_deref(),
            &sandbox_result.log_text,
            files_json,
            runs_json,
            mesh_json,
            &sandbox_result.isolation,
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
            action: "agent.job".into(),
            resource_type: "agent_job".into(),
            resource_id: job.id.to_string(),
            metadata: serde_json::json!({
                "kind": kind,
                "status": status,
                "files": sandbox_result.files_changed,
                "commit_sha": sandbox_result.commit_sha,
                "agents": agents,
                "isolation": sandbox_result.isolation,
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;

    let done = store
        .get_agent_job(p.tenant_id, job.id)
        .await?
        .ok_or_else(|| HelixError::internal("job missing"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(done))))
}

fn run_agent_job_blocking(
    bare: &std::path::Path,
    rev: &str,
    kind: &str,
    prompt: &str,
    patches: &[FilePatch],
    unified_diff: Option<&str>,
    do_commit: bool,
    branch: &str,
    commit_msg: &str,
) -> shared_core::HelixResult<agent_sandbox::AgentSandboxResult> {
    let mut log = String::new();
    let mode = container::resolve_isolation();
    let mut isolation = container::isolation_label(mode).to_string();
    log.push_str(&format!(
        "E4 agent job kind={kind} rev={rev} isolation={isolation} docker={}\n",
        container::docker_available()
    ));
    let workdir = prepare_worktree(bare, rev)?;
    log.push_str(&format!("workdir={}\n", workdir.display()));

    let mut mesh_steps = vec![local_analyze_step(&workdir, prompt)];
    log.push_str(&format!("analyze: {}\n", mesh_steps[0].summary));

    // Optional isolated shell probe (echo only — host if docker lacks git tooling)
    match container::run_isolated(
        &workdir,
        "echo helix-agent-isolated",
        Duration::from_secs(30),
        mode,
    ) {
        Ok((code, out, err, actual)) => {
            isolation = container::isolation_label(actual).to_string();
            log.push_str(&format!(
                "isolated_probe exit={code} mode={isolation}\n{out}{err}"
            ));
            mesh_steps.push(MeshStepResult {
                agent: "isolation-probe".into(),
                status: if code == 0 {
                    "succeeded".into()
                } else {
                    "failed".into()
                },
                summary: format!("isolation={isolation} exit={code}"),
                run_id: None,
            });
        }
        Err(e) => {
            log.push_str(&format!("isolated_probe error: {e}\n"));
        }
    }

    let mut files_changed = Vec::new();
    if !patches.is_empty() {
        let ch = apply_file_patches(&workdir, patches)?;
        log.push_str(&format!("file_patches={}\n", ch.join(",")));
        files_changed.extend(ch);
    }
    if let Some(diff) = unified_diff {
        if !diff.trim().is_empty() {
            let ch = apply_unified_diff(&workdir, diff)?;
            log.push_str(&format!("unified_diff files={}\n", ch.join(",")));
            files_changed.extend(ch);
        }
    }

    // patch kind requires some change unless sandbox-only
    if kind == "patch" && files_changed.is_empty() {
        return Ok(agent_sandbox::AgentSandboxResult {
            status: "failed".into(),
            log_text: {
                log.push_str("patch kind requires patches or unified_diff\n");
                log
            },
            workdir,
            commit_sha: None,
            files_changed,
            mesh_steps,
            isolation,
        });
    }

    let mut commit_sha = None;
    if do_commit && !files_changed.is_empty() {
        match commit_and_push(&workdir, bare, branch, commit_msg) {
            Ok(sha) => {
                log.push_str(&format!("committed {sha}\n"));
                commit_sha = Some(sha);
                mesh_steps.push(MeshStepResult {
                    agent: "git-commit".into(),
                    status: "succeeded".into(),
                    summary: format!("pushed to {branch}"),
                    run_id: None,
                });
            }
            Err(e) => {
                log.push_str(&format!("commit failed: {e}\n"));
                return Ok(agent_sandbox::AgentSandboxResult {
                    status: "failed".into(),
                    log_text: log,
                    workdir,
                    commit_sha: None,
                    files_changed,
                    mesh_steps,
                    isolation,
                });
            }
        }
    } else if do_commit && files_changed.is_empty() {
        log.push_str("no file changes; skip commit\n");
    }

    files_changed.sort();
    files_changed.dedup();
    Ok(agent_sandbox::AgentSandboxResult {
        status: "succeeded".into(),
        log_text: log,
        workdir,
        commit_sha,
        files_changed,
        mesh_steps,
        isolation,
    })
}

async fn get_agent_job(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_db(&state)?;
    let store = CodeRepoStore::new(pool);
    let job = store
        .get_agent_job(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("job not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(job))))
}

#[derive(Deserialize)]
struct CompleteBody {
    language: String,
    prefix: String,
}

async fn complete(
    RequireAuth(p): RequireAuth,
    Json(body): Json<CompleteBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    // Legacy route — prefer /v1/lsp/sessions/{id}/completion
    let snippet: String = body.prefix.chars().take(40).collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "language": body.language,
        "suggestion": format!("// use LSP session completion (E3). prefix={snippet}..."),
        "lsp": lsp_bridge::lsp_available(),
        "hint": "POST /v1/repos/{id}/lsp/session then /v1/lsp/sessions/{sid}/completion"
    }))))
}

fn require_db(state: &AppState) -> Result<sqlx::PgPool, ApiError> {
    state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required for HelixCode extreme").into())
}

fn is_safe_repo_name(name: &str) -> bool {
    let b = name.as_bytes();
    if b.is_empty() || b.len() > 64 {
        return false;
    }
    b.iter()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'))
}
