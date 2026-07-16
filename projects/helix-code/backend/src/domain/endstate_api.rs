//! End-state APIs: CI list/cancel, runners, settings, quotas, agent events, MLS devices, LSP servers, terminal.

use super::container;
use super::dap_client;
use super::git_store::GitStore;
use super::lsp_bridge;
use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use helix_db::CodeRepoStore;
use serde::Deserialize;
use service_kit::{ApiError, AppState, RequireAuth};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use uuid::Uuid;

static TERMINALS: once_cell::sync::Lazy<Mutex<HashMap<Uuid, TerminalSession>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

struct TerminalSession {
    tenant_id: Uuid,
    workdir: std::path::PathBuf,
    log: String,
    isolation: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/repos/{id}/pipeline-runs", get(list_runs))
        .route("/v1/pipeline-runs/{id}/cancel", post(cancel_run))
        .route("/v1/pipeline-artifacts/{id}/content", get(artifact_content))
        .route("/v1/runners/heartbeat", post(runner_heartbeat))
        .route("/v1/repos/{id}/agent-jobs", get(list_agent_jobs))
        .route("/v1/agent-jobs/{id}/cancel", post(cancel_agent))
        .route("/v1/agent-jobs/{id}/events", get(agent_events))
        .route("/v1/mls/devices", get(list_devices).post(register_device))
        .route("/v1/mls/key-backup", get(get_backup).put(put_backup))
        .route("/v1/mls/groups", get(list_mls_groups))
        .route("/v1/me/code-settings", get(get_settings).put(put_settings))
        .route("/v1/quotas", get(get_quotas))
        .route("/v1/lsp/servers", get(lsp_servers))
        .route("/v1/repos/{id}/terminals", post(create_terminal))
        .route(
            "/v1/terminals/{id}",
            get(read_terminal).post(write_terminal),
        )
        .route("/v1/me/breakglass", get(get_breakglass).put(put_breakglass))
        .route("/v1/extensions", get(list_extensions))
        .route("/v1/repos/{id}/debug/launch", post(debug_launch))
        .route("/v1/debug/sessions/{id}/breakpoints", post(set_breakpoints))
        .route("/v1/debug/sessions/{id}/continue", post(debug_continue))
        .route("/v1/debug/sessions/{id}/next", post(debug_next))
        .route("/v1/debug/sessions/{id}/stepIn", post(debug_step_in))
        .route("/v1/debug/sessions/{id}/stepOut", post(debug_step_out))
        .route("/v1/debug/sessions/{id}/pause", post(debug_pause))
        .route("/v1/debug/sessions/{id}/threads", get(debug_threads))
        .route("/v1/debug/sessions/{id}/stack", get(debug_stack))
        .route("/v1/debug/sessions/{id}/scopes", get(debug_scopes))
        .route("/v1/debug/sessions/{id}/variables", get(debug_variables))
        .route("/v1/debug/sessions/{id}/evaluate", post(debug_evaluate))
        .route(
            "/v1/debug/sessions/{id}",
            get(get_debug_session).delete(stop_debug),
        )
        .route("/v1/debug/adapters", get(debug_adapters))
        .route(
            "/v1/repos/{id}/deploy-keys",
            get(list_deploy_keys).post(create_deploy_key),
        )
        .route(
            "/v1/deploy-keys/{id}",
            axum::routing::delete(revoke_deploy_key),
        )
}

async fn list_runs(
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
    let items = store.list_pipeline_runs(p.tenant_id, id, 50).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn cancel_run(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let ok = store.cancel_pipeline_run(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "cancelled": ok }),
    )))
}

async fn artifact_content(
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
    let art = store
        .get_pipeline_artifact(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("artifact not found"))?;
    let bytes = state
        .clients
        .objects
        .get_object(&art.storage_key)
        .await
        .map_err(|e| HelixError::dependency(e.to_string()))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "name": art.name,
        "content_type": art.content_type,
        "byte_len": art.byte_len,
        "content_b64": b64,
        "content_utf8": String::from_utf8(bytes).ok(),
    }))))
}

#[derive(Deserialize)]
struct RunnerBody {
    name: String,
    #[serde(default)]
    labels: Vec<String>,
}

async fn runner_heartbeat(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<RunnerBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let r = store
        .upsert_runner(
            p.tenant_id,
            body.name.trim(),
            serde_json::json!(body.labels),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(r))))
}

async fn list_agent_jobs(
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
    let items = store.list_agent_jobs(p.tenant_id, id, 50).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn cancel_agent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let ok = store.cancel_agent_job(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "cancelled": ok }),
    )))
}

#[derive(Deserialize)]
struct EventQuery {
    #[serde(default)]
    after: i32,
}

async fn agent_events(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<EventQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_agent_events(p.tenant_id, id, q.after).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct DeviceBody {
    device_id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    public_identity_b64: String,
}

fn user_key(p: &shared_core::tenancy::Principal) -> String {
    format!("{}:{}", p.tenant_id.as_uuid(), p.user_id)
}

async fn register_device(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<DeviceBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let d = store
        .upsert_mls_device(
            p.tenant_id,
            &user_key(&p),
            body.device_id.trim(),
            &body.label,
            &body.public_identity_b64,
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(d))))
}

async fn list_devices(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_mls_devices(p.tenant_id, &user_key(&p)).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct BackupBody {
    ciphertext_b64: String,
}

async fn put_backup(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<BackupBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(body.ciphertext_b64.trim())
        .map_err(|e| HelixError::validation(format!("b64: {e}")))?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    store
        .put_mls_key_backup(p.tenant_id, &user_key(&p), &raw)
        .await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "stored": true, "bytes": raw.len() }),
    )))
}

async fn get_backup(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let blob = store.get_mls_key_backup(p.tenant_id, &user_key(&p)).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "present": blob.is_some(),
        "ciphertext_b64": blob.map(|b| base64::engine::general_purpose::STANDARD.encode(b)),
    }))))
}

async fn list_mls_groups(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let items = store.list_mls_groups_meta(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn get_settings(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let s = store
        .get_user_settings(p.tenant_id, p.user_id.as_uuid())
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "settings": s }))))
}

async fn put_settings(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let settings = body.get("settings").cloned().unwrap_or(body);
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    store
        .put_user_settings(p.tenant_id, p.user_id.as_uuid(), settings.clone())
        .await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "settings": settings }),
    )))
}

async fn get_quotas(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let q = store.get_or_default_quota(p.tenant_id).await?;
    let repos = store.count_repos(p.tenant_id).await?;
    let runs = store.count_pipeline_runs_month(p.tenant_id).await?;
    let agents = store.count_agent_jobs_day(p.tenant_id).await?;
    let sealed_bytes = store.sum_sealed_bytes(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "quota": q,
        "usage": {
            "repos": repos,
            "pipeline_runs_month": runs,
            "agent_jobs_day": agents,
            "sealed_bytes": sealed_bytes,
        },
        "isolation_image": container::docker_image(),
        "docker_has_forge_tools": container::image_has_forge_tools(),
    }))))
}

async fn lsp_servers(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let servers = lsp_bridge::list_language_servers();
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "servers": servers }),
    )))
}

#[derive(Deserialize)]
struct TermBody {
    #[serde(default)]
    rev: String,
}

async fn create_terminal(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<TermBody>,
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
    // clone worktree for terminal
    let root = std::env::var("HELIX_CODE_TERM_WORKDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(".data/helix-code/terminals"));
    std::fs::create_dir_all(&root).ok();
    let dest = root.join(format!("term-{}", Uuid::now_v7().simple()));
    let bare_s = bare.to_str().unwrap_or("");
    let dest_s = dest.to_str().unwrap_or("");
    let _ = Command::new("git")
        .args(["clone", "--depth", "1", bare_s, dest_s])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("git")
        .current_dir(&dest)
        .args(["checkout", &rev])
        .status();
    let tid = Uuid::now_v7();
    let inst = super::lsp_bridge::instance_id();
    let iso = super::container::resolve_terminal_isolation();
    let iso_label = super::container::isolation_label(iso).to_string();
    // Sticky registry for multi-instance
    let _ = store
        .register_process_session(
            tid,
            p.tenant_id,
            "terminal",
            &inst,
            Some(repo.id),
            serde_json::json!({ "rev": rev, "isolation": iso_label }),
            7200,
        )
        .await;
    TERMINALS.lock().unwrap().insert(
        tid,
        TerminalSession {
            tenant_id: p.tenant_id.as_uuid(),
            workdir: dest,
            log: format!("terminal ready rev={rev} isolation={iso_label} instance={inst}\n$ "),
            isolation: iso_label.clone(),
        },
    );
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "terminal_id": tid,
        "rev": rev,
        "instance_id": inst,
        "isolation": iso_label,
        "sticky": true,
    }))))
}

async fn read_terminal(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = CodeRepoStore::new(pool.clone());
        let _ = store
            .require_process_local(id, "terminal", &super::lsp_bridge::instance_id())
            .await?;
    }
    let g = TERMINALS.lock().unwrap();
    let t = g
        .get(&id)
        .ok_or_else(|| HelixError::not_found("terminal not found (local process)"))?;
    if t.tenant_id != p.tenant_id.as_uuid() {
        return Err(HelixError::forbidden("terminal tenant mismatch").into());
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "log": t.log,
        "isolation": t.isolation,
        "instance_id": super::lsp_bridge::instance_id(),
    }))))
}

#[derive(Deserialize)]
struct WriteTerm {
    command: String,
}

async fn write_terminal(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<WriteTerm>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let cmd = body.command.trim();
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let _ = store
        .require_process_local(id, "terminal", &super::lsp_bridge::instance_id())
        .await?;
    let eff = super::tenant_policy::load_effective(&store, p.tenant_id).await?;
    super::terminal_policy::validate_terminal_command_ext(cmd, eff.allow_term_all)?;

    let workdir = {
        let mut g = TERMINALS.lock().unwrap();
        let t = g
            .get_mut(&id)
            .ok_or_else(|| HelixError::not_found("terminal not found (local process)"))?;
        if t.tenant_id != p.tenant_id.as_uuid() {
            return Err(HelixError::forbidden("terminal tenant mismatch").into());
        }
        t.log.push_str(cmd);
        t.log.push('\n');
        t.workdir.clone()
    };

    let mode = super::container::resolve_terminal_isolation();
    let timeout = std::time::Duration::from_secs(60);
    // Prefer isolation path when cmd_policy accepts; else host spawn after terminal_policy.
    let run_result = if super::cmd_policy::is_allowed_command(cmd) {
        super::container::run_isolated(&workdir, cmd, timeout, mode)
    } else {
        #[cfg(windows)]
        let output = Command::new("cmd")
            .arg("/C")
            .arg(cmd)
            .current_dir(&workdir)
            .output();
        #[cfg(not(windows))]
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&workdir)
            .output();
        match output {
            Ok(o) => Ok((
                o.status.code().unwrap_or(1),
                String::from_utf8_lossy(&o.stdout).into_owned(),
                String::from_utf8_lossy(&o.stderr).into_owned(),
                super::container::IsolationMode::Host,
            )),
            Err(e) => Err(HelixError::dependency(format!("term spawn: {e}"))),
        }
    };

    let mut g = TERMINALS.lock().unwrap();
    let t = g
        .get_mut(&id)
        .ok_or_else(|| HelixError::not_found("terminal not found"))?;
    match run_result {
        Ok((code, out, err, actual)) => {
            let label = super::container::isolation_label(actual);
            t.isolation = label.into();
            t.log.push_str(&format!("isolation={label}\n"));
            t.log.push_str(&out);
            if !out.ends_with('\n') && !out.is_empty() {
                t.log.push('\n');
            }
            t.log.push_str(&err);
            t.log.push_str(&format!("\nexit={code}\n$ "));
        }
        Err(e) => t.log.push_str(&format!("error: {e}\n$ ")),
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "log": t.log,
        "isolation": t.isolation,
        "instance_id": super::lsp_bridge::instance_id(),
    }))))
}

#[derive(Deserialize)]
struct BreakglassBody {
    #[serde(default)]
    allow_direct_push: bool,
    #[serde(default)]
    allow_force_push: bool,
    #[serde(default)]
    allow_ci_all: bool,
    #[serde(default)]
    allow_term_all: bool,
    #[serde(default)]
    allow_host_fallback: bool,
    #[serde(default)]
    allow_host_isolation: bool,
}

async fn get_breakglass(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let flags = store.get_tenant_breakglass(p.tenant_id).await?;
    let eff = super::tenant_policy::merge_with_env(&flags);
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "tenant": flags,
        "effective": {
            "allow_direct_push": eff.allow_direct_push,
            "allow_force_push": eff.allow_force_push,
            "allow_ci_all": eff.allow_ci_all,
            "allow_term_all": eff.allow_term_all,
            "allow_host_fallback": eff.allow_host_fallback,
            "allow_host_isolation": eff.allow_host_isolation,
            "sources": eff.sources,
        }
    }))))
}

async fn put_breakglass(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<BreakglassBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let mut flags = helix_db::CodeTenantBreakglass::default_for(p.tenant_id);
    flags.allow_direct_push = body.allow_direct_push;
    flags.allow_force_push = body.allow_force_push;
    flags.allow_ci_all = body.allow_ci_all;
    flags.allow_term_all = body.allow_term_all;
    flags.allow_host_fallback = body.allow_host_fallback;
    flags.allow_host_isolation = body.allow_host_isolation;
    let saved = store
        .put_tenant_breakglass(p.tenant_id, &flags, &p.user_id.to_string())
        .await?;
    super::breakglass::record(
        "TENANT_BREAKGLASS_UPDATE",
        &format!(
            "tenant={} direct={} force={} ci={} term={} host_fb={} host_iso={}",
            p.tenant_id.as_uuid(),
            saved.allow_direct_push,
            saved.allow_force_push,
            saved.allow_ci_all,
            saved.allow_term_all,
            saved.allow_host_fallback,
            saved.allow_host_isolation
        ),
    );
    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "breakglass.tenant_update".into(),
            resource_type: "tenant_breakglass".into(),
            resource_id: p.tenant_id.as_uuid().to_string(),
            metadata: serde_json::json!({ "flags": saved }),
            residency_region: p.residency_region.clone(),
        })
        .await;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "tenant": saved }),
    )))
}

async fn list_extensions(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    // Built-in extension manifest registry (light host)
    let items = vec![
        serde_json::json!({
            "id": "helix.theme-dark",
            "name": "Helix Dark Theme",
            "version": "1.0.0",
            "contributes": { "themes": ["helix-dark"] }
        }),
        serde_json::json!({
            "id": "helix.git-basics",
            "name": "Git Basics",
            "version": "1.0.0",
            "contributes": { "commands": ["git.status", "git.diff"] }
        }),
        serde_json::json!({
            "id": "helix.rust-extras",
            "name": "Rust Extras",
            "version": "1.0.0",
            "contributes": { "languages": ["rust"] }
        }),
    ];
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

fn dbg_cfg() -> String {
    "launch".into()
}

#[derive(Deserialize)]
struct DebugLaunchExt {
    #[serde(default = "dbg_cfg")]
    config: String,
    /// Optional program path relative to checkout or absolute.
    #[serde(default)]
    program: String,
    #[serde(default)]
    args: Vec<String>,
    /// If true, start full DAP adapter (lldb-dap/gdb --interpreter=dap).
    #[serde(default = "true_bool")]
    dap: bool,
}
fn true_bool() -> bool {
    true
}

async fn debug_launch(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<DebugLaunchExt>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let probe = dap_client::adapter_probe();
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let adapter_name = probe
        .get("command")
        .and_then(|c| c.as_str())
        .unwrap_or("none");
    let inst = lsp_bridge::instance_id();
    let sess = store
        .create_debug_session(p.tenant_id, id, &body.config, adapter_name, &inst)
        .await?;
    let _ = store
        .register_process_session(
            sess.id,
            p.tenant_id,
            "dap",
            &inst,
            Some(id),
            serde_json::json!({ "adapter": adapter_name }),
            7200,
        )
        .await;

    let mut dap_info = serde_json::json!(null);
    if body.dap {
        let repo = store
            .get(p.tenant_id, id)
            .await?
            .ok_or_else(|| HelixError::not_found("repo not found"))?;
        let git = GitStore::from_env();
        // checkout worktree for debug
        let root = std::env::var("HELIX_CODE_DEBUG_WORKDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from(".data/helix-code/debug"));
        std::fs::create_dir_all(&root).ok();
        let dest = root.join(format!("dbg-{}", sess.id.simple()));
        if let Ok(bare) = git.path_for_smart_http(p.tenant_id, &repo.name) {
            let bare_s = bare.to_str().unwrap_or("");
            let dest_s = dest.to_str().unwrap_or("");
            let _ = Command::new("git")
                .args(["clone", "--depth", "1", bare_s, dest_s])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let program = if body.program.is_empty() {
            // default: try target/debug/<name> under workdir if present
            None
        } else {
            let pth = dest.join(&body.program);
            if pth.is_file() {
                Some(pth)
            } else {
                Some(std::path::PathBuf::from(&body.program))
            }
        };
        match dap_client::start_session(sess.id, program.as_deref(), Some(&dest), &body.args) {
            Ok(info) => {
                dap_info = info;
                let _ = store
                    .finish_debug_session(p.tenant_id, sess.id, "ready")
                    .await;
            }
            Err(e) => {
                // Soft-fail DAP spawn: still return durable session + probe
                dap_info = serde_json::json!({
                    "error": e.to_string(),
                    "probe": probe,
                    "note": "DAP adapter failed to start; session metadata retained"
                });
            }
        }
    }

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "session_id": sess.id,
        "config": sess.config,
        "status": sess.status,
        "adapter": sess.adapter,
        "instance_id": sess.instance_id,
        "probe": probe,
        "dap": dap_info,
        "capabilities": [
            "initialize", "launch", "setBreakpoints", "configurationDone",
            "continue", "next", "stepIn", "stepOut", "pause",
            "threads", "stackTrace", "scopes", "variables", "evaluate", "disconnect"
        ]
    }))))
}

#[derive(Deserialize)]
struct BpBody {
    breakpoints: Vec<serde_json::Value>,
}

async fn set_breakpoints(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<BpBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let _ = store
        .require_process_local(id, "dap", &lsp_bridge::instance_id())
        .await?;
    let bps = serde_json::json!(body.breakpoints);
    store
        .set_debug_breakpoints(p.tenant_id, id, bps.clone())
        .await?;
    // Forward to live DAP if present (group by path)
    let mut dap_results = Vec::new();
    let mut by_path: std::collections::HashMap<String, Vec<u32>> = std::collections::HashMap::new();
    for bp in &body.breakpoints {
        let path = bp
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("src/main.rs")
            .to_string();
        let line = bp.get("line").and_then(|l| l.as_u64()).unwrap_or(1) as u32;
        by_path.entry(path).or_default().push(line);
    }
    for (path, lines) in by_path {
        match dap_client::set_breakpoints(id, &path, &lines) {
            Ok(r) => dap_results.push(r),
            Err(e) => dap_results.push(serde_json::json!({ "error": e.to_string(), "path": path })),
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "session_id": id,
        "breakpoints": bps,
        "verified": true,
        "dap": dap_results
    }))))
}

async fn debug_continue(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let _ = store
        .require_process_local(id, "dap", &lsp_bridge::instance_id())
        .await?;
    let sess = store
        .get_debug_session(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("debug session not found"))?;
    let dap = dap_client::continue_exec(id)
        .map(|v| serde_json::json!({ "ok": true, "body": v }))
        .unwrap_or_else(|e| serde_json::json!({ "ok": false, "error": e.to_string() }));
    let snap = dap_client::snapshot(id).ok();
    store
        .finish_debug_session(p.tenant_id, id, "running")
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "session_id": id,
        "status": "running",
        "adapter": sess.adapter,
        "breakpoints": sess.breakpoints,
        "dap": dap,
        "snapshot": snap
    }))))
}

async fn get_debug_session(
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
    let sess = store
        .get_debug_session(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("debug session not found"))?;
    let snap = dap_client::snapshot(id).ok();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "session": sess,
        "dap_snapshot": snap
    }))))
}

async fn debug_threads(
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let t = dap_client::threads(id)?;
    Ok(Json(ApiResponse::ok(t)))
}

#[derive(Deserialize)]
struct StackQuery {
    #[serde(default = "one_u64")]
    thread_id: u64,
}
fn one_u64() -> u64 {
    1
}

#[derive(Deserialize)]
struct ThreadBody {
    #[serde(default = "one_u64")]
    thread_id: u64,
}

async fn debug_stack(
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<StackQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let st = dap_client::stack_trace(id, q.thread_id)?;
    Ok(Json(ApiResponse::ok(st)))
}

async fn require_dap_local(state: &AppState, session_id: Uuid) -> Result<(), ApiError> {
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    store
        .require_process_local(session_id, "dap", &lsp_bridge::instance_id())
        .await?;
    Ok(())
}

async fn debug_next(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<ThreadBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    require_dap_local(&state, id).await?;
    let r = dap_client::next(id, body.thread_id)?;
    let snap = dap_client::snapshot(id).ok();
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "result": r, "snapshot": snap }),
    )))
}

async fn debug_step_in(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<ThreadBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    require_dap_local(&state, id).await?;
    let r = dap_client::step_in(id, body.thread_id)?;
    let snap = dap_client::snapshot(id).ok();
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "result": r, "snapshot": snap }),
    )))
}

async fn debug_step_out(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<ThreadBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    require_dap_local(&state, id).await?;
    let r = dap_client::step_out(id, body.thread_id)?;
    let snap = dap_client::snapshot(id).ok();
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "result": r, "snapshot": snap }),
    )))
}

async fn debug_pause(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<ThreadBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    require_dap_local(&state, id).await?;
    let r = dap_client::pause(id, body.thread_id)?;
    let snap = dap_client::snapshot(id).ok();
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "result": r, "snapshot": snap }),
    )))
}

#[derive(Deserialize)]
struct ScopesQuery {
    #[serde(default = "one_u64")]
    frame_id: u64,
}

async fn debug_scopes(
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<ScopesQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(dap_client::scopes(id, q.frame_id)?)))
}

#[derive(Deserialize)]
struct VariablesQuery {
    #[serde(default = "one_u64")]
    variables_reference: u64,
}

async fn debug_variables(
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<VariablesQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(dap_client::variables(
        id,
        q.variables_reference,
    )?)))
}

#[derive(Deserialize)]
struct EvalBody {
    expression: String,
    #[serde(default)]
    frame_id: Option<u64>,
    #[serde(default = "eval_ctx")]
    context: String,
}
fn eval_ctx() -> String {
    "repl".into()
}

async fn debug_evaluate(
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<EvalBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let r = dap_client::evaluate(id, &body.expression, body.frame_id, &body.context)?;
    Ok(Json(ApiResponse::ok(r)))
}

async fn debug_adapters(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(dap_client::adapter_probe())))
}

async fn stop_debug(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let _ = dap_client::disconnect(id);
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    store
        .finish_debug_session(p.tenant_id, id, "stopped")
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "stopped": id }))))
}

// DAP threads / stackTrace convenience
// GET /v1/debug/sessions/{id}/threads|stack

#[derive(Deserialize)]
struct DeployKeyBody {
    name: String,
    #[serde(default = "read_scope")]
    scope: String,
}
fn read_scope() -> String {
    "read".into()
}

async fn create_deploy_key(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<DeployKeyBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let _ = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let issued = store
        .create_deploy_key(p.tenant_id, id, body.name.trim(), &body.scope)
        .await?;
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: shared_core::tenancy::Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "deploy_key.create".into(),
            resource_type: "deploy_key".into(),
            resource_id: issued.key.id.to_string(),
            metadata: serde_json::json!({
                "scope": issued.key.scope,
                "prefix": issued.key.token_prefix
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "key": issued.key,
        "token": issued.token,
        "usage": "git -c http.extraHeader=\"x-helix-deploy-key: <token>\" clone …"
    }))))
}

async fn list_deploy_keys(
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
    let items = store.list_deploy_keys(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn revoke_deploy_key(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let ok = store.revoke_deploy_key(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "revoked": ok }))))
}
