//! HelixCore Vault Service — AES-GCM envelope-encrypted secret store + object refs.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use helix_db::{VaultObjectRef, VaultObjectStore};
use serde::{Deserialize, Serialize};
use service_kit::{ApiError, AppState, RequireAuth, ServiceBuilder};
use shared_core::ids::TenantId;
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use vault_client::{KeyManagement, LocalSoftwareKms, SecretMeta, SecretRef};

#[tokio::main]
async fn main() -> HelixResult<()> {
    let builder = ServiceBuilder::new("vault-service", 8082).await?;
    let addr = builder.config().listen_addr;
    let state = builder.into_state();

    let app = ServiceBuilder::base_router(state.clone()).merge(
        Router::new()
            .route(
                "/v1/tenants/{tenant_id}/secrets",
                get(list_secrets).post(put_secret),
            )
            .route(
                "/v1/tenants/{tenant_id}/secrets/{name}",
                get(get_secret).delete(delete_secret),
            )
            .route(
                "/v1/tenants/{tenant_id}/objects",
                get(list_objects).post(put_object),
            )
            .route(
                "/v1/tenants/{tenant_id}/objects/{name}",
                get(get_object).delete(delete_object),
            )
            // Local software HSM/KMS endpoints (also used when HELIX_VAULT_KMS_MODE=http)
            .route("/v1/kms/wrap", post(kms_wrap))
            .route("/v1/kms/unwrap", post(kms_unwrap))
            .route("/v1/kms/health", get(kms_health))
            .route("/v1/keys/meta", get(key_meta))
            .route("/v1/keys/rotate-meta", post(rotate_key_meta))
            .with_state(state.clone()),
    );

    service_kit::serve_with_shutdown(addr, app, "vault-service", state.clone()).await
}

#[derive(Deserialize)]
struct PutBody {
    name: String,
    value_b64: String,
}

#[derive(Serialize)]
struct GetBody {
    name: String,
    value_b64: String,
}

fn ensure_tenant(
    principal: &shared_core::tenancy::Principal,
    tid: TenantId,
) -> Result<(), HelixError> {
    if tid != principal.tenant_id && !principal.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Err(HelixError::forbidden(
            "tenant isolation: secret access denied",
        ));
    }
    Ok(())
}

async fn put_secret(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
    Json(body): Json<PutBody>,
) -> Result<Json<ApiResponse<SecretRef>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body.value_b64)
        .map_err(|e| HelixError::validation(format!("value_b64: {e}")))?;
    let r = state.clients.vault.put(tid, &body.name, &bytes).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "secret.put".into(),
            resource_type: "secret".into(),
            resource_id: body.name.clone(),
            metadata: serde_json::json!({"version": r.version, "durable": state.clients.has_db()}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(r)))
}

async fn get_secret(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, name)): Path<(String, String)>,
) -> Result<Json<ApiResponse<GetBody>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let value = state.clients.vault.get(tid, &name).await?;
    Ok(Json(ApiResponse::ok(GetBody {
        name,
        value_b64: base64::engine::general_purpose::STANDARD.encode(&value[..]),
    })))
}

async fn list_secrets(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<SecretMeta>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let list = state.clients.vault.list(tid).await?;
    Ok(Json(ApiResponse::ok(list)))
}

async fn delete_secret(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, name)): Path<(String, String)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Admin)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    state.clients.vault.delete(tid, &name).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "secret.delete".into(),
            resource_type: "secret".into(),
            resource_id: name.clone(),
            metadata: serde_json::json!({}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted": name}))))
}

#[derive(Deserialize)]
struct PutObjectBody {
    name: String,
    #[serde(default)]
    object_key: Option<String>,
    #[serde(default = "default_ct")]
    content_type: String,
    /// Optional base64 payload — when set, vault-service PUTs bytes to MinIO.
    #[serde(default)]
    value_b64: Option<String>,
    #[serde(default)]
    size_bytes: i64,
}

fn default_ct() -> String {
    "application/octet-stream".into()
}

fn object_meta_store(state: &AppState) -> Result<VaultObjectStore, HelixError> {
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for vault object refs"))?;
    Ok(VaultObjectStore::new(pool.clone()))
}

/// Bind a large secret/asset to MinIO; optional in-process byte upload via `value_b64`.
async fn put_object(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
    Json(body): Json<PutObjectBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let store = object_meta_store(&state)?;
    let key = body
        .object_key
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| VaultObjectStore::suggest_key(tid, &body.name));

    let mut size_bytes = body.size_bytes;
    let mut uploaded = false;
    let key_version = current_key_version(&state).await?;
    if let Some(b64) = body.value_b64.as_ref() {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| HelixError::validation(format!("value_b64: {e}")))?;
        size_bytes = bytes.len() as i64;
        // Envelope-encrypt blob with tenant DEK (KMS-aware HVA5) before object store.
        let sealed = vault_client::vault_seal_tenant_kms(
            state.clients.kms.as_ref(),
            &tid.to_string(),
            &bytes,
            key_version,
        )
        .await?;
        state
            .clients
            .objects
            .put_object(&key, &sealed, &body.content_type)
            .await?;
        uploaded = true;
    }

    let rec = store
        .put_ref(tid, &body.name, &key, &body.content_type, size_bytes)
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "secret.object.put".into(),
            resource_type: "vault_object".into(),
            resource_id: body.name.clone(),
            metadata: serde_json::json!({
                "object_key": rec.object_key,
                "size_bytes": rec.size_bytes,
                "uploaded": uploaded,
                "envelope": if uploaded { "HVA5" } else { "metadata-only" },
                "key_version": if uploaded { serde_json::json!(key_version) } else { serde_json::Value::Null },
                "minio_endpoint": state.clients.config.minio_endpoint,
                "bucket": state.clients.config.minio_bucket
            }),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "ref": rec,
        "uploaded": uploaded,
        "envelope": if uploaded { "HVA5-tenant-dek" } else { "metadata-only" },
        "key_version": if uploaded { serde_json::json!(key_version) } else { serde_json::Value::Null },
        "upload_hint": if uploaded {
            "bytes sealed (HVA2) and stored in MinIO".into()
        } else {
            format!(
                "POST again with value_b64 or PUT bytes to MinIO bucket={} key={} endpoint={}",
                state.clients.config.minio_bucket,
                rec.object_key,
                state.clients.config.minio_endpoint
            )
        }
    }))))
}

async fn get_object(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, name)): Path<(String, String)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let store = object_meta_store(&state)?;
    let rec = store
        .get(tid, &name)
        .await?
        .ok_or_else(|| HelixError::not_found(format!("object {name}")))?;

    // Best-effort fetch + open sealed payload from MinIO.
    let (value_b64, bytes_ok) = match state.clients.objects.get_object(&rec.object_key).await {
        Ok(sealed) => {
            match vault_client::vault_open_tenant_kms(
                state.clients.kms.as_ref(),
                state.clients.config.vault_master_key.as_bytes(),
                &tid.to_string(),
                &sealed,
            )
            .await
            {
                Ok((plain, _version)) => (
                    Some(base64::engine::general_purpose::STANDARD.encode(plain)),
                    true,
                ),
                Err(_) => (None, false),
            }
        }
        Err(_) => (None, false),
    };

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "ref": rec,
        "value_b64": value_b64,
        "bytes_available": bytes_ok
    }))))
}

async fn list_objects(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<VaultObjectRef>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let store = object_meta_store(&state)?;
    Ok(Json(ApiResponse::ok(store.list(tid).await?)))
}

async fn delete_object(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, name)): Path<(String, String)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Admin)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    ensure_tenant(&principal, tid)?;
    let store = object_meta_store(&state)?;
    if let Some(rec) = store.get(tid, &name).await? {
        let _ = state.clients.objects.delete_object(&rec.object_key).await;
    }
    store.delete(tid, &name).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "secret.object.delete".into(),
            resource_type: "vault_object".into(),
            resource_id: name.clone(),
            metadata: serde_json::json!({}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted": name}))))
}

fn local_kms(state: &AppState) -> LocalSoftwareKms {
    let kek = state
        .clients
        .config
        .vault_kek
        .as_deref()
        .unwrap_or(state.clients.config.vault_master_key.as_str());
    LocalSoftwareKms::from_explicit_kek(kek.as_bytes())
}

#[derive(Deserialize)]
struct KmsWrapBody {
    plaintext_b64: String,
}

#[derive(Deserialize)]
struct KmsUnwrapBody {
    wrapped_b64: String,
}

/// Software HSM: wrap a DEK (base64 32-byte key).
async fn kms_wrap(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<KmsWrapBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Admin)?;
    let plain = base64::engine::general_purpose::STANDARD
        .decode(body.plaintext_b64)
        .map_err(|e| HelixError::validation(format!("plaintext_b64: {e}")))?;
    if plain.len() != 32 {
        return Err(HelixError::validation("plaintext must be 32 bytes").into());
    }
    let mut dek = [0u8; 32];
    dek.copy_from_slice(&plain);
    let kms = local_kms(&state);
    let wrapped = kms.wrap_dek(&dek).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "wrapped_b64": base64::engine::general_purpose::STANDARD.encode(wrapped),
        "mode": "local_software_hsm"
    }))))
}

async fn kms_unwrap(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<KmsUnwrapBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Admin)?;
    let wrapped = base64::engine::general_purpose::STANDARD
        .decode(body.wrapped_b64)
        .map_err(|e| HelixError::validation(format!("wrapped_b64: {e}")))?;
    let kms = local_kms(&state);
    let dek = kms.unwrap_dek(&wrapped).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "plaintext_b64": base64::engine::general_purpose::STANDARD.encode(dek),
        "mode": "local_software_hsm"
    }))))
}

async fn kms_health(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    // Platform-only — do not leak KMS mode/URL to anonymous callers (Kimi P2).
    principal.require_scope(shared_core::tenancy::Scope::Platform)?;
    let cfg = &state.clients.config;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "mode": cfg.kms_mode,
        "software_hsm": true,
        "master_key_configured": !cfg.vault_master_key.is_empty(),
        "http_kms_url": cfg.kms_url,
        "fallback_allowed": cfg.kms_fallback,
        "note": "HVA3/HVA4/HVA5 envelopes; HELIX_VAULT_KMS_MODE=http for remote; no silent fallback"
    }))))
}

async fn key_meta(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Admin)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for key meta"))?;
    #[derive(sqlx::FromRow)]
    struct Row {
        version: i32,
        algorithm: String,
        rotated_at: chrono::DateTime<chrono::Utc>,
        note: String,
    }
    let row: Option<Row> = sqlx::query_as(
        "SELECT version, algorithm, rotated_at, note FROM helix_core.vault_key_meta WHERE id = 'default'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| HelixError::dependency(format!("key meta: {e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "meta": row.map(|r| serde_json::json!({
            "version": r.version,
            "algorithm": r.algorithm,
            "rotated_at": r.rotated_at,
            "note": r.note
        })),
        "kms_mode": state.clients.config.kms_mode,
        "envelope": "HVA5"
    }))))
}

#[derive(Deserialize)]
struct RotateMetaBody {
    #[serde(default)]
    note: Option<String>,
}

/// Record a key rotation event (operator rotates HELIX_VAULT_MASTER_KEY / KEK out-of-band).
async fn rotate_key_meta(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<RotateMetaBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Platform)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let note = body.note.unwrap_or_else(|| "operator rotation".into());
    sqlx::query(
        r#"
        INSERT INTO helix_core.vault_key_meta (id, version, algorithm, rotated_at, note)
        VALUES ('default', 1, 'HVA5', now(), $1)
        ON CONFLICT (id) DO UPDATE SET
            version = helix_core.vault_key_meta.version + 1,
            algorithm = 'HVA5',
            rotated_at = now(),
            note = EXCLUDED.note
        "#,
    )
    .bind(&note)
    .execute(pool)
    .await
    .map_err(|e| HelixError::dependency(format!("key rotate meta: {e}")))?;

    // Bulk re-encrypt secrets under current KMS (sovereign rotation depth).
    let reencrypted = {
        let vault = helix_db::PgVault::with_kms(
            pool.clone(),
            state.clients.config.vault_master_key.as_bytes(),
            state.clients.kms.clone(),
        );
        match vault.reencrypt_all().await {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(error = %e, "vault reencrypt_all failed after meta bump");
                0
            }
        }
    };

    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: None,
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "vault.key.rotate_meta".into(),
            resource_type: "vault_key".into(),
            resource_id: "default".into(),
            metadata: serde_json::json!({"note": note, "reencrypted": reencrypted}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "rotated": true,
        "note": note,
        "reencrypted_secrets": reencrypted,
        "envelope": "HVA5",
        "algorithm": "HVA5"
    }))))
}

async fn current_key_version(state: &AppState) -> HelixResult<u32> {
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for key versioning"))?;
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT version FROM helix_core.vault_key_meta WHERE id = 'default'")
            .fetch_optional(pool)
            .await
            .map_err(|e| HelixError::dependency(format!("key version: {e}")))?;
    Ok(row.map(|(v,)| v as u32).unwrap_or(1))
}
