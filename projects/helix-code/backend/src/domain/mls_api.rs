//! HelixCode OpenMLS REST (RFC 9420) — multi-tenant forge group join + sealed app messages.

use super::mls_engine::{MlsEngine, MlsIdentityOut};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use helix_db::CodeRepoStore;
use once_cell::sync::Lazy;
use serde::Deserialize;
use service_kit::{ApiError, AppState, RequireAuth};
use sha2::Digest;
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;
use vault_client::{vault_open_raw, vault_seal_raw};

static MLS: Lazy<MlsEngine> = Lazy::new(MlsEngine::new);
// Persist blobs on each mutating op when DB present — also hydrate on demand.

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/mls/identity", post(ensure_identity))
        .route("/v1/mls/key-package", post(key_package))
        .route("/v1/mls/groups", post(create_group))
        .route("/v1/mls/groups/{group_id}/add", post(add_member))
        .route("/v1/mls/groups/{group_id}/join", post(join_group))
        .route("/v1/mls/groups/{group_id}", get(group_info))
        .route("/v1/mls/groups/{group_id}/encrypt", post(encrypt_msg))
        .route("/v1/mls/groups/{group_id}/decrypt", post(decrypt_msg))
        .route("/v1/repos/{id}/mls-sealed", post(put_mls_sealed))
        .route("/v1/mls/status", get(mls_status))
}

fn user_key(p: &shared_core::tenancy::Principal) -> String {
    format!("{}:{}", p.tenant_id.as_uuid(), p.user_id)
}

async fn hydrate(state: &AppState, p: &shared_core::tenancy::Principal) -> HelixResult<()> {
    let uk = user_key(p);
    if MLS.has_user(&uk) {
        return Ok(());
    }
    if let Some(pool) = state.clients.db.as_ref() {
        let store = CodeRepoStore::new(pool.clone());
        if let Some(blob) = store.get_mls_user_blob(p.tenant_id, &uk).await? {
            MLS.import_user_blob(&uk, &blob)?;
        }
    }
    Ok(())
}

async fn persist(state: &AppState, p: &shared_core::tenancy::Principal) -> HelixResult<()> {
    let uk = user_key(p);
    if !MLS.has_user(&uk) {
        return Ok(());
    }
    let blob = MLS.export_user_blob(&uk)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = CodeRepoStore::new(pool.clone());
        store.upsert_mls_user_blob(p.tenant_id, &uk, &blob).await?;
    }
    Ok(())
}

async fn mls_status(
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "openmls": true,
        "ciphersuite": "MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519",
        "export_label": "helix-code-dek",
        "user_hydrated": MLS.has_user(&user_key(&p)),
    }))))
}

#[derive(Deserialize)]
struct IdentityBody {
    #[serde(default = "default_label")]
    label: String,
}

fn default_label() -> String {
    "forge".into()
}

async fn ensure_identity(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<IdentityBody>,
) -> Result<Json<ApiResponse<MlsIdentityOut>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate(&state, &p).await?;
    let out = MLS.ensure_identity(&user_key(&p), &body.label)?;
    persist(&state, &p).await?;
    Ok(Json(ApiResponse::ok(out)))
}

async fn key_package(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate(&state, &p).await?;
    let uk = user_key(&p);
    if !MLS.has_user(&uk) {
        MLS.ensure_identity(&uk, "forge")?;
    }
    let kp = MLS.create_key_package(&uk)?;
    persist(&state, &p).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "key_package_tls_b64": base64::engine::general_purpose::STANDARD.encode(kp)
    }))))
}

#[derive(Deserialize)]
struct CreateGroupBody {
    #[serde(default)]
    name: String,
    repo_id: Option<Uuid>,
}

async fn create_group(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateGroupBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate(&state, &p).await?;
    let uk = user_key(&p);
    if !MLS.has_user(&uk) {
        MLS.ensure_identity(&uk, "forge")?;
    }
    let gid = Uuid::now_v7().to_string();
    let info = MLS.create_group(&uk, &gid)?;
    persist(&state, &p).await?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = CodeRepoStore::new(pool.clone());
        let name = if body.name.is_empty() {
            format!("mls-{gid}")
        } else {
            body.name
        };
        let _ = store
            .upsert_mls_group_meta(
                p.tenant_id,
                &gid,
                body.repo_id,
                &name,
                info.epoch as i64,
                info.member_count as i32,
            )
            .await;
    }
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "mls.group.create".into(),
            resource_type: "mls_group".into(),
            resource_id: gid.clone(),
            metadata: serde_json::json!({"epoch": info.epoch}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(info))))
}

#[derive(Deserialize)]
struct AddBody {
    key_package_tls_b64: String,
}

async fn add_member(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(group_id): Path<String>,
    Json(body): Json<AddBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate(&state, &p).await?;
    let kp = base64::engine::general_purpose::STANDARD
        .decode(body.key_package_tls_b64.trim())
        .map_err(|e| HelixError::validation(format!("kp b64: {e}")))?;
    let out = MLS.add_member(&user_key(&p), &group_id, &kp)?;
    persist(&state, &p).await?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = CodeRepoStore::new(pool.clone());
        let _ = store
            .upsert_mls_group_meta(
                p.tenant_id,
                &group_id,
                None,
                "",
                out.epoch as i64,
                out.members.len() as i32,
            )
            .await;
    }
    Ok(Json(ApiResponse::ok(serde_json::json!(out))))
}

#[derive(Deserialize)]
struct JoinBody {
    welcome_tls_b64: String,
}

async fn join_group(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(_group_id): Path<String>,
    Json(body): Json<JoinBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate(&state, &p).await?;
    let uk = user_key(&p);
    if !MLS.has_user(&uk) {
        MLS.ensure_identity(&uk, "forge")?;
    }
    let welcome = base64::engine::general_purpose::STANDARD
        .decode(body.welcome_tls_b64.trim())
        .map_err(|e| HelixError::validation(format!("welcome b64: {e}")))?;
    let info = MLS.join_with_welcome(&uk, &welcome, None)?;
    persist(&state, &p).await?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = CodeRepoStore::new(pool.clone());
        let _ = store
            .upsert_mls_group_meta(
                p.tenant_id,
                &info.group_id,
                None,
                "",
                info.epoch as i64,
                info.member_count as i32,
            )
            .await;
    }
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "mls.group.join".into(),
            resource_type: "mls_group".into(),
            resource_id: info.group_id.clone(),
            metadata: serde_json::json!({"members": info.member_count}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(info))))
}

async fn group_info(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(group_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    hydrate(&state, &p).await?;
    let info = MLS.group_info(&user_key(&p), &group_id)?;
    Ok(Json(ApiResponse::ok(serde_json::json!(info))))
}

#[derive(Deserialize)]
struct MsgBody {
    content: String,
    #[serde(default)]
    content_encoding: String,
}

async fn encrypt_msg(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(group_id): Path<String>,
    Json(body): Json<MsgBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate(&state, &p).await?;
    let plain = if body.content_encoding.eq_ignore_ascii_case("base64") {
        base64::engine::general_purpose::STANDARD
            .decode(body.content.trim())
            .map_err(|e| HelixError::validation(format!("b64: {e}")))?
    } else {
        body.content.into_bytes()
    };
    let msg = MLS.create_app_message(&user_key(&p), &group_id, &plain)?;
    persist(&state, &p).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "ciphertext_tls_b64": base64::engine::general_purpose::STANDARD.encode(msg)
    }))))
}

#[derive(Deserialize)]
struct DecryptBody {
    ciphertext_tls_b64: String,
}

async fn decrypt_msg(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(group_id): Path<String>,
    Json(body): Json<DecryptBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    hydrate(&state, &p).await?;
    let ct = base64::engine::general_purpose::STANDARD
        .decode(body.ciphertext_tls_b64.trim())
        .map_err(|e| HelixError::validation(format!("b64: {e}")))?;
    let plain = MLS.process_app_message(&user_key(&p), &group_id, &ct)?;
    persist(&state, &p).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "plaintext_b64": plain.as_ref().map(|b| base64::engine::general_purpose::STANDARD.encode(b)),
        "plaintext_utf8": plain.as_ref().and_then(|b| String::from_utf8(b.clone()).ok()),
        "application": plain.is_some(),
    }))))
}

/// Seal a forge secret using OpenMLS exported group DEK (application-layer pack).
#[derive(Deserialize)]
struct MlsSealedBody {
    group_id: String,
    content: String,
    #[serde(default = "default_name")]
    name: String,
}

fn default_name() -> String {
    "mls-pack".into()
}

async fn put_mls_sealed(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<MlsSealedBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate(&state, &p).await?;
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
    let dek = MLS.export_group_secret(&user_key(&p), &body.group_id)?;
    if dek.len() != 32 {
        return Err(HelixError::internal("mls dek size").into());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&dek);
    let plaintext = body.content.as_bytes();
    let ct = vault_seal_raw(&key, plaintext)?;
    let ct_hash = hex::encode(sha2::Sha256::digest(&ct));
    let pt_hash = hex::encode(sha2::Sha256::digest(plaintext));
    let quota = store.get_or_default_quota(p.tenant_id).await?;
    let used = store.sum_sealed_bytes(p.tenant_id).await?;
    let add = ct.len() as i64;
    if used.saturating_add(add) > quota.max_sealed_bytes {
        return Err(HelixError::forbidden(format!(
            "quota_exceeded: max_sealed_bytes={} used={used} need={add}",
            quota.max_sealed_bytes
        ))
        .into());
    }
    let storage_key = format!(
        "code/{}/mls-sealed/{}/{}",
        p.tenant_id.as_uuid(),
        repo.id,
        ct_hash
    );
    state
        .clients
        .objects
        .put_object(&storage_key, &ct, "application/octet-stream")
        .await?;
    let meta = store
        .insert_sealed_object(
            p.tenant_id,
            Some(repo.id),
            &ct_hash,
            &storage_key,
            "mls",
            ct.len() as i64,
            &body.name,
            "forge.openmls",
            "openmls-export-dek",
            "application/octet-stream",
            &pt_hash,
            &user_key(&p),
            None,
        )
        .await?;
    persist(&state, &p).await?;
    // Prove we can open with exported secret
    let opened = vault_open_raw(&key, &ct)?;
    if opened != plaintext {
        return Err(HelixError::internal("mls seal roundtrip failed").into());
    }
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "mls.sealed.put".into(),
            resource_type: "sealed_object".into(),
            resource_id: meta.id.to_string(),
            metadata: serde_json::json!({"group_id": body.group_id, "envelope": "openmls-export-dek"}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(meta))))
}
