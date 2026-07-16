//! HTTP surface for OpenMLS groups bound to documents (with durable Postgres state).

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use helix_db::{AclPermission, SovereignCollabRepo};
use serde::Deserialize;
use service_kit::ApiError;
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError};
use uuid::Uuid;

use super::documents::{ensure_doc_access_pub, Auth};
use super::CollabState;

fn user_key(p: &shared_core::tenancy::Principal) -> String {
    p.user_id.to_string()
}

fn sov(state: &CollabState) -> Result<SovereignCollabRepo, ApiError> {
    let pool = state
        .core
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable MLS"))?;
    Ok(SovereignCollabRepo::new(pool.clone()))
}

/// Load MLS user from Postgres into memory engine if not already present.
async fn hydrate_user(
    state: &CollabState,
    p: &shared_core::tenancy::Principal,
) -> Result<(), ApiError> {
    let uk = user_key(p);
    if state.mls.has_user(&uk) {
        return Ok(());
    }
    if let Ok(repo) = sov(state) {
        if let Ok(Some(blob)) = repo
            .get_mls_user_blob(p.tenant_id, p.user_id, &p.user_id.to_string())
            .await
        {
            state.mls.import_user_blob(&uk, &blob)?;
        }
    }
    Ok(())
}

async fn persist_user(
    state: &CollabState,
    p: &shared_core::tenancy::Principal,
    group_id: Option<&str>,
) -> Result<(), ApiError> {
    let uk = user_key(p);
    let blob = state.mls.export_user_blob(&uk)?;
    let pub_b64 = state
        .mls
        .ensure_identity(&uk, &p.user_id.to_string())?
        .signature_public_b64;
    if let Ok(repo) = sov(state) {
        repo.upsert_mls_user_blob(
            p.tenant_id,
            p.user_id,
            &p.user_id.to_string(),
            &blob,
            &pub_b64,
        )
        .await?;
        if let Some(gid) = group_id {
            let json = String::from_utf8_lossy(&blob).into_owned();
            let _ = repo
                .upsert_mls_member_state(p.tenant_id, p.user_id, gid, &json, None)
                .await;
        }
    }
    Ok(())
}

pub fn routes() -> Router<CollabState> {
    Router::new()
        .route(
            "/v1/mls/identity",
            post(ensure_identity).get(identity_status),
        )
        .route("/v1/mls/key-packages", post(create_key_package))
        .route(
            "/v1/documents/{id}/mls/group",
            post(create_group).get(group_info),
        )
        .route("/v1/documents/{id}/mls/add", post(add_member))
        .route("/v1/documents/{id}/mls/join", post(join_group))
        .route("/v1/documents/{id}/mls/message", post(send_message))
        .route("/v1/documents/{id}/mls/process", post(process_message))
        .route("/v1/documents/{id}/mls/export-secret", get(export_secret))
        .route("/v1/mls/persist", post(force_persist))
}

async fn ensure_identity(
    State(state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate_user(&state, &p).await?;
    let out = state
        .mls
        .ensure_identity(&user_key(&p), &p.user_id.to_string())?;
    persist_user(&state, &p, None).await?;
    audit(
        &state,
        &p,
        "mls.identity",
        "user",
        &p.user_id.to_string(),
        serde_json::json!({"durable": true}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "identity": out,
        "engine": "openmls-0.8",
        "ciphersuite": "MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519",
        "rfc": 9420,
        "durable": true
    }))))
}

async fn identity_status(
    State(state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    hydrate_user(&state, &p).await?;
    let out = state
        .mls
        .ensure_identity(&user_key(&p), &p.user_id.to_string())?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "identity": out,
        "in_memory": state.mls.has_user(&user_key(&p)),
        "durable": true
    }))))
}

async fn create_key_package(
    State(state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate_user(&state, &p).await?;
    let _ = state
        .mls
        .ensure_identity(&user_key(&p), &p.user_id.to_string())?;
    let kp = state.mls.create_key_package(&user_key(&p))?;
    persist_user(&state, &p, None).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "key_package_tls_b64": base64::engine::general_purpose::STANDARD.encode(kp),
    }))))
}

async fn create_group(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Admin).await?;
    hydrate_user(&state, &p).await?;
    let _ = state
        .mls
        .ensure_identity(&user_key(&p), &p.user_id.to_string())?;
    let info = state.mls.create_group(&user_key(&p), &id.to_string())?;
    if let Some(pool) = state.core.clients.db.as_ref() {
        let _ = sqlx_insert_group(
            pool,
            p.tenant_id.as_uuid(),
            id,
            &info.group_id,
            info.epoch,
            p.user_id.as_uuid(),
        )
        .await;
    }
    persist_user(&state, &p, Some(&info.group_id)).await?;
    audit(
        &state,
        &p,
        "mls.group.create",
        "document",
        &id.to_string(),
        serde_json::json!({"epoch": info.epoch, "durable": true}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "group": info,
        "durable": true,
        "note": "exported_secret_b64 is epoch DEK material — never log in production"
    }))))
}

async fn group_info(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    hydrate_user(&state, &p).await?;
    let info = state.mls.group_info(&user_key(&p), &id.to_string())?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "group": info }))))
}

#[derive(Deserialize)]
struct AddBody {
    key_package_tls_b64: String,
}

async fn add_member(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<AddBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Share).await?;
    hydrate_user(&state, &p).await?;
    let kp = base64::engine::general_purpose::STANDARD
        .decode(body.key_package_tls_b64.trim())
        .map_err(|e| HelixError::validation(format!("b64: {e}")))?;
    let out = state.mls.add_member(&user_key(&p), &id.to_string(), &kp)?;
    if let Some(pool) = state.core.clients.db.as_ref() {
        let _ = sqlx_insert_group(
            pool,
            p.tenant_id.as_uuid(),
            id,
            &id.to_string(),
            out.epoch,
            p.user_id.as_uuid(),
        )
        .await;
    }
    persist_user(&state, &p, Some(&id.to_string())).await?;
    audit(
        &state,
        &p,
        "mls.member.add",
        "document",
        &id.to_string(),
        serde_json::json!({"epoch": out.epoch, "members": out.members.len()}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "add": out, "durable": true }),
    )))
}

#[derive(Deserialize)]
struct JoinBody {
    welcome_tls_b64: String,
}

async fn join_group(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<JoinBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    hydrate_user(&state, &p).await?;
    let _ = state
        .mls
        .ensure_identity(&user_key(&p), &p.user_id.to_string())?;
    let welcome = base64::engine::general_purpose::STANDARD
        .decode(body.welcome_tls_b64.trim())
        .map_err(|e| HelixError::validation(format!("b64: {e}")))?;
    let info = state.mls.join_with_welcome(&user_key(&p), &welcome, None)?;
    persist_user(&state, &p, Some(&id.to_string())).await?;
    audit(
        &state,
        &p,
        "mls.group.join",
        "document",
        &id.to_string(),
        serde_json::json!({"epoch": info.epoch, "durable": true}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "group": info, "durable": true }),
    )))
}

#[derive(Deserialize)]
struct MsgBody {
    plaintext_b64: String,
}

async fn send_message(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<MsgBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    hydrate_user(&state, &p).await?;
    let plain = base64::engine::general_purpose::STANDARD
        .decode(body.plaintext_b64.trim())
        .map_err(|e| HelixError::validation(format!("b64: {e}")))?;
    let msg = state
        .mls
        .create_app_message(&user_key(&p), &id.to_string(), &plain)?;
    persist_user(&state, &p, Some(&id.to_string())).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "message_tls_b64": base64::engine::general_purpose::STANDARD.encode(msg),
    }))))
}

#[derive(Deserialize)]
struct ProcessBody {
    message_tls_b64: String,
}

async fn process_message(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<ProcessBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    hydrate_user(&state, &p).await?;
    let msg = base64::engine::general_purpose::STANDARD
        .decode(body.message_tls_b64.trim())
        .map_err(|e| HelixError::validation(format!("b64: {e}")))?;
    let plain = state
        .mls
        .process_app_message(&user_key(&p), &id.to_string(), &msg)?;
    persist_user(&state, &p, Some(&id.to_string())).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "plaintext_b64": plain.map(|p| base64::engine::general_purpose::STANDARD.encode(p)),
        "durable": true
    }))))
}

async fn export_secret(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    hydrate_user(&state, &p).await?;
    let secret = state
        .mls
        .export_group_secret(&user_key(&p), &id.to_string())?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "exported_secret_b64": base64::engine::general_purpose::STANDARD.encode(secret),
        "label": "helix-collab-dek",
        "length": 32
    }))))
}

async fn force_persist(
    State(state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    hydrate_user(&state, &p).await?;
    let _ = state
        .mls
        .ensure_identity(&user_key(&p), &p.user_id.to_string())?;
    persist_user(&state, &p, None).await?;
    let blob = state.mls.export_user_blob(&user_key(&p))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "persisted": true,
        "blob_bytes": blob.len(),
    }))))
}

async fn sqlx_insert_group(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    document_id: Uuid,
    group_id: &str,
    epoch: u64,
    created_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO collab.mls_groups (group_id, tenant_id, document_id, epoch, created_by, created_at, updated_at)
        VALUES ($1,$2,$3,$4,$5,now(),now())
        ON CONFLICT (group_id) DO UPDATE SET epoch = EXCLUDED.epoch, updated_at = now()
        "#,
    )
    .bind(group_id)
    .bind(tenant_id)
    .bind(document_id)
    .bind(epoch as i64)
    .bind(created_by)
    .execute(pool)
    .await?;
    Ok(())
}

async fn audit(
    state: &CollabState,
    p: &shared_core::tenancy::Principal,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    metadata: serde_json::Value,
) -> Result<(), ApiError> {
    state
        .core
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            metadata,
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(())
}
