//! E5 sealed object plane — HVA4 tenant envelopes + forge crypto groups (MLS-like DEK wrap).
//!
//! Cleartext never lands in MinIO. Only ciphertext envelopes are stored.

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use helix_db::CodeRepoStore;
use serde::Deserialize;
use service_kit::{ApiError, AppState, RequireAuth};
use sha2::{Digest, Sha256};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError};
use uuid::Uuid;
use vault_client::{vault_open_raw, vault_open_tenant, vault_seal_raw, vault_seal_tenant};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/repos/{id}/sealed-objects",
            get(list_sealed).post(put_sealed),
        )
        .route(
            "/v1/repos/{id}/sealed-objects/{oid}",
            get(get_sealed).delete(delete_sealed),
        )
        .route("/v1/crypto-groups", get(list_groups).post(create_group))
        .route("/v1/crypto-groups/{gid}/members", post(add_member))
}

fn master(state: &AppState) -> &[u8] {
    state.clients.config.vault_master_key.as_bytes()
}

fn user_key(p: &shared_core::tenancy::Principal) -> String {
    p.user_id.to_string()
}

fn valid_classification(c: &str) -> bool {
    matches!(
        c,
        "internal" | "confidential" | "secret" | "top_secret" | "mls"
    )
}

#[derive(Deserialize)]
struct SealedBody {
    /// UTF-8 plaintext (or base64 if content_encoding=base64)
    content: String,
    #[serde(default = "default_class")]
    classification: String,
    #[serde(default)]
    name: String,
    #[serde(default = "default_purpose")]
    purpose: String,
    /// `tenant` (HVA4) or `group` (group DEK AES-GCM)
    #[serde(default = "default_mode")]
    seal_mode: String,
    group_id: Option<Uuid>,
    #[serde(default = "default_ct")]
    content_type: String,
    #[serde(default)]
    content_encoding: String,
}

fn default_class() -> String {
    "confidential".into()
}
fn default_purpose() -> String {
    "forge.secret".into()
}
fn default_mode() -> String {
    "tenant".into()
}
fn default_ct() -> String {
    "application/octet-stream".into()
}

async fn put_sealed(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<SealedBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.classification == "public" || !valid_classification(&body.classification) {
        return Err(HelixError::validation(
            "classification must be internal|confidential|secret|top_secret|mls (not public)",
        )
        .into());
    }
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

    let plaintext = if body.content_encoding.eq_ignore_ascii_case("base64") {
        base64::engine::general_purpose::STANDARD
            .decode(body.content.trim())
            .map_err(|e| HelixError::validation(format!("base64 content: {e}")))?
    } else {
        body.content.into_bytes()
    };
    if plaintext.is_empty() {
        return Err(HelixError::validation("content required").into());
    }
    let pt_hash = hex::encode(Sha256::digest(&plaintext));
    let tid = p.tenant_id.as_uuid().to_string();
    let mode = body.seal_mode.to_ascii_lowercase();

    let (ciphertext, envelope_kind, group_id) = match mode.as_str() {
        "tenant" | "hva4" => {
            let ct = vault_seal_tenant(master(&state), &tid, &plaintext)?;
            (ct, "hva4".to_string(), None)
        }
        "group" | "mls" => {
            let gid = body
                .group_id
                .ok_or_else(|| HelixError::validation("group_id required for seal_mode=group"))?;
            let _g = store
                .get_crypto_group(p.tenant_id, gid)
                .await?
                .ok_or_else(|| HelixError::not_found("crypto group not found"))?;
            let wrap_b64 = store
                .get_member_wrapped_dek(p.tenant_id, gid, &user_key(&p))
                .await?
                .ok_or_else(|| HelixError::validation("not a group member"))?;
            let wrapped = base64::engine::general_purpose::STANDARD
                .decode(wrap_b64)
                .map_err(|e| HelixError::internal(format!("wrap b64: {e}")))?;
            let dek_bytes = vault_open_tenant(master(&state), &tid, &wrapped)?;
            if dek_bytes.len() != 32 {
                return Err(HelixError::internal("group DEK length").into());
            }
            let mut dek = [0u8; 32];
            dek.copy_from_slice(&dek_bytes);
            let ct = vault_seal_raw(&dek, &plaintext)?;
            (ct, "group-aes-gcm".to_string(), Some(gid))
        }
        _ => {
            return Err(HelixError::validation("seal_mode must be tenant|group").into());
        }
    };

    let ct_hash = hex::encode(Sha256::digest(&ciphertext));
    // ES8: max_sealed_bytes quota
    let quota = store.get_or_default_quota(p.tenant_id).await?;
    let used = store.sum_sealed_bytes(p.tenant_id).await?;
    let add = ciphertext.len() as i64;
    if used.saturating_add(add) > quota.max_sealed_bytes {
        return Err(HelixError::forbidden(format!(
            "quota_exceeded: max_sealed_bytes={} used={used} need={add}",
            quota.max_sealed_bytes
        ))
        .into());
    }
    let key = format!(
        "code/{}/sealed/{}/{ct_hash}",
        p.tenant_id.as_uuid(),
        repo.id
    );
    // Store ciphertext only
    state
        .clients
        .objects
        .put_object(&key, &ciphertext, "application/octet-stream")
        .await?;

    let name = if body.name.is_empty() {
        format!("obj-{}", &ct_hash[..12])
    } else {
        body.name
    };
    let meta = store
        .insert_sealed_object(
            p.tenant_id,
            Some(repo.id),
            &ct_hash,
            &key,
            &body.classification,
            ciphertext.len() as i64,
            &name,
            &body.purpose,
            &envelope_kind,
            &body.content_type,
            &pt_hash,
            &user_key(&p),
            group_id,
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
            action: "sealed_object.put".into(),
            resource_type: "sealed_object".into(),
            resource_id: meta.id.to_string(),
            metadata: serde_json::json!({
                "ciphertext_sha256": ct_hash,
                "plaintext_sha256": pt_hash,
                "envelope_kind": envelope_kind,
                "classification": body.classification,
                "cleartext_in_minio": false,
                "group_id": group_id,
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;

    // Never return plaintext
    Ok(Json(ApiResponse::ok(serde_json::json!(meta))))
}

async fn list_sealed(
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
    let _repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let items = store.list_sealed(p.tenant_id, Some(id)).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct GetQuery {
    /// If true, return decrypted plaintext (base64). Default true for authorized read.
    #[serde(default = "default_true")]
    decrypt: bool,
}

fn default_true() -> bool {
    true
}

async fn get_sealed(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, oid)): Path<(Uuid, Uuid)>,
    Query(q): Query<GetQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let _repo = store
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("repo not found"))?;
    let meta = store
        .get_sealed(p.tenant_id, oid)
        .await?
        .ok_or_else(|| HelixError::not_found("sealed object not found"))?;
    if meta.repo_id != Some(id) {
        return Err(HelixError::not_found("sealed object not found").into());
    }

    if !q.decrypt {
        return Ok(Json(ApiResponse::ok(serde_json::json!({ "meta": meta }))));
    }

    let ciphertext = state.clients.objects.get_object(&meta.storage_key).await?;
    // Verify ciphertext hash
    let got = hex::encode(Sha256::digest(&ciphertext));
    if got != meta.content_sha256 {
        return Err(HelixError::internal("ciphertext integrity mismatch").into());
    }

    let tid = p.tenant_id.as_uuid().to_string();
    let plaintext = match meta.envelope_kind.as_str() {
        "hva4" | "tenant" => vault_open_tenant(master(&state), &tid, &ciphertext)?,
        "group-aes-gcm" => {
            let gid = meta
                .group_id
                .ok_or_else(|| HelixError::internal("group sealed missing group_id"))?;
            let wrap_b64 = store
                .get_member_wrapped_dek(p.tenant_id, gid, &user_key(&p))
                .await?
                .ok_or_else(|| HelixError::validation("not a group member"))?;
            let wrapped = base64::engine::general_purpose::STANDARD
                .decode(wrap_b64)
                .map_err(|e| HelixError::internal(format!("wrap b64: {e}")))?;
            let dek_bytes = vault_open_tenant(master(&state), &tid, &wrapped)?;
            let mut dek = [0u8; 32];
            dek.copy_from_slice(&dek_bytes[..32.min(dek_bytes.len())]);
            if dek_bytes.len() != 32 {
                return Err(HelixError::internal("group DEK length").into());
            }
            vault_open_raw(&dek, &ciphertext)?
        }
        other => {
            return Err(HelixError::internal(format!("unknown envelope {other}")).into());
        }
    };

    // Integrity of plaintext
    let pt_hash = hex::encode(Sha256::digest(&plaintext));
    if !meta.plaintext_sha256.is_empty() && pt_hash != meta.plaintext_sha256 {
        return Err(HelixError::internal("plaintext integrity mismatch").into());
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
            action: "sealed_object.get".into(),
            resource_type: "sealed_object".into(),
            resource_id: meta.id.to_string(),
            metadata: serde_json::json!({"decrypt": true}),
            residency_region: p.residency_region.clone(),
        })
        .await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "meta": meta,
        "content_b64": base64::engine::general_purpose::STANDARD.encode(&plaintext),
        "content_utf8": String::from_utf8(plaintext.clone()).ok(),
    }))))
}

async fn delete_sealed(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path((id, oid)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let meta = store
        .get_sealed(p.tenant_id, oid)
        .await?
        .ok_or_else(|| HelixError::not_found("sealed object not found"))?;
    if meta.repo_id != Some(id) {
        return Err(HelixError::not_found("sealed object not found").into());
    }
    let _ = state.clients.objects.delete_object(&meta.storage_key).await;
    let ok = store.delete_sealed(p.tenant_id, oid).await?;
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "sealed_object.delete".into(),
            resource_type: "sealed_object".into(),
            resource_id: oid.to_string(),
            metadata: serde_json::json!({"storage_key": meta.storage_key}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "deleted": ok }))))
}

#[derive(Deserialize)]
struct CreateGroupBody {
    name: String,
    #[serde(default = "default_group_purpose")]
    purpose: String,
}

fn default_group_purpose() -> String {
    "forge.mls-like".into()
}

async fn create_group(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateGroupBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let name = body.name.trim();
    if name.is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    // Random group DEK
    let mut dek = [0u8; 32];
    getrandom::getrandom(&mut dek).map_err(|e| HelixError::internal(format!("rng: {e}")))?;
    let tid = p.tenant_id.as_uuid().to_string();
    let wrapped = vault_seal_tenant(master(&state), &tid, &dek)?;
    let wrap_b64 = base64::engine::general_purpose::STANDARD.encode(&wrapped);
    let g = store
        .create_crypto_group(p.tenant_id, name, &body.purpose, &user_key(&p), &wrap_b64)
        .await?;
    dek.fill(0);
    state
        .clients
        .audit
        .append(audit_log::AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "crypto_group.create".into(),
            resource_type: "crypto_group".into(),
            resource_id: g.id.to_string(),
            metadata: serde_json::json!({"name": name, "purpose": body.purpose}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(g))))
}

async fn list_groups(
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
    let items = store.list_crypto_groups(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct AddMemberBody {
    user_key: String,
}

async fn add_member(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(gid): Path<Uuid>,
    Json(body): Json<AddMemberBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .clone()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let store = CodeRepoStore::new(pool);
    let g = store
        .get_crypto_group(p.tenant_id, gid)
        .await?
        .ok_or_else(|| HelixError::not_found("crypto group not found"))?;
    if g.owner_user != user_key(&p) {
        // For E5 allow any tenant member with write to add if they have the DEK
        // (still must unwrap their wrap first)
    }
    let my_wrap = store
        .get_member_wrapped_dek(p.tenant_id, gid, &user_key(&p))
        .await?
        .ok_or_else(|| HelixError::validation("caller is not a group member"))?;
    let tid = p.tenant_id.as_uuid().to_string();
    let wrapped = base64::engine::general_purpose::STANDARD
        .decode(my_wrap)
        .map_err(|e| HelixError::internal(format!("wrap b64: {e}")))?;
    let dek = vault_open_tenant(master(&state), &tid, &wrapped)?;
    // Re-wrap for new member under same tenant HVA4 (multi-user within tenant).
    // Cross-tenant MLS would re-wrap under their tenant key in a later phase.
    let member_wrap = vault_seal_tenant(master(&state), &tid, &dek)?;
    let member_b64 = base64::engine::general_purpose::STANDARD.encode(&member_wrap);
    store
        .add_crypto_group_member(p.tenant_id, gid, body.user_key.trim(), &member_b64)
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
            action: "crypto_group.member_add".into(),
            resource_type: "crypto_group".into(),
            resource_id: gid.to_string(),
            metadata: serde_json::json!({"user_key": body.user_key}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "group_id": gid,
        "user_key": body.user_key,
        "added": true
    }))))
}
