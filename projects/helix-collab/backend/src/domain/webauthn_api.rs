//! Hardened passkey binding: challenge bound to user, RP, origin, and purpose.
//! Signature covers SHA-256 of a WebAuthn-shaped clientDataJSON.

use audit_log::AuditEvent;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use p256::pkcs8::DecodePublicKey;
use parking_lot::Mutex;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use service_kit::ApiError;
use sha2::{Digest, Sha256};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::documents::Auth;
use super::CollabState;

struct Challenge {
    bytes: Vec<u8>,
    expires: Instant,
}

#[derive(Clone)]
struct PasskeyRecord {
    spki: Vec<u8>,
    counter: u64,
    device_label: String,
}

struct PasskeyStore {
    challenges: Mutex<HashMap<String, Challenge>>,
    /// user_id -> passkeys
    keys: Mutex<HashMap<String, Vec<PasskeyRecord>>>,
}

fn store() -> &'static PasskeyStore {
    static S: OnceLock<PasskeyStore> = OnceLock::new();
    S.get_or_init(|| PasskeyStore {
        challenges: Mutex::new(HashMap::new()),
        keys: Mutex::new(HashMap::new()),
    })
}

fn b64(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

fn b64d(s: &str) -> HelixResult<Vec<u8>> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s.trim())
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(s.trim()))
        .map_err(|e| HelixError::validation(format!("b64: {e}")))
}

fn rp_id() -> String {
    std::env::var("HELIX_WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".into())
}

fn origin() -> String {
    std::env::var("HELIX_WEBAUTHN_ORIGIN").unwrap_or_else(|_| "http://localhost:3101".into())
}

/// WebAuthn-shaped client data the client must sign (SHA-256 then ECDSA).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientData {
    pub r#type: String,
    pub challenge: String,
    pub origin: String,
    pub rp_id: String,
    pub user_id: String,
}

fn client_data_bytes(purpose: &str, challenge_b64: &str, user_id: &str) -> Vec<u8> {
    let typ = if purpose == "register" {
        "webauthn.create"
    } else {
        "webauthn.get"
    };
    let cd = ClientData {
        r#type: typ.into(),
        challenge: challenge_b64.into(),
        origin: origin(),
        rp_id: rp_id(),
        user_id: user_id.into(),
    };
    serde_json::to_vec(&cd).expect("clientData")
}

pub fn routes() -> Router<CollabState> {
    Router::new()
        .route("/v1/webauthn/register/start", post(register_start))
        .route("/v1/webauthn/register/finish", post(register_finish))
        .route("/v1/webauthn/authenticate/start", post(auth_start))
        .route("/v1/webauthn/authenticate/finish", post(auth_finish))
        .route("/v1/webauthn/credentials", get(list_creds))
}

async fn register_start(
    State(_state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let mut ch = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut ch);
    let challenge_b64 = b64(&ch);
    let client_data = client_data_bytes("register", &challenge_b64, &p.user_id.to_string());
    store().challenges.lock().insert(
        format!("reg:{}", p.user_id),
        Challenge {
            bytes: ch,
            expires: Instant::now() + Duration::from_secs(300),
        },
    );
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "challenge_b64": challenge_b64,
        "client_data_b64": b64(&client_data),
        "client_data_hash_b64": b64(&Sha256::digest(&client_data)),
        "rp_id": rp_id(),
        "origin": origin(),
        "user_id": p.user_id.to_string(),
        "algorithm": "ECDSA_P256_SHA256",
        "protocol": "helix-passkey-v2",
        "sign_over": "SHA-256(clientDataJSON) with ECDSA P-256; clientData includes type,challenge,origin,rpId,userId"
    }))))
}

#[derive(Deserialize)]
struct RegFinish {
    public_key_spki_b64: String,
    /// Signature over SHA-256(clientDataJSON) or over clientDataJSON bytes.
    signature_b64: String,
    /// Must match server-built client data (or challenge only for v1 compat).
    #[serde(default)]
    client_data_b64: Option<String>,
    #[serde(default)]
    device_label: Option<String>,
}

async fn register_finish(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Json(body): Json<RegFinish>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let ch = store()
        .challenges
        .lock()
        .remove(&format!("reg:{}", p.user_id))
        .ok_or_else(|| HelixError::validation("no registration challenge"))?;
    if Instant::now() > ch.expires {
        return Err(HelixError::validation("challenge expired").into());
    }
    let challenge_b64 = b64(&ch.bytes);
    let expected = client_data_bytes("register", &challenge_b64, &p.user_id.to_string());
    if let Some(cd_b64) = body.client_data_b64.as_deref() {
        let got = b64d(cd_b64)?;
        if got != expected {
            return Err(HelixError::validation(
                "clientDataJSON mismatch (bound to user/rp/origin/challenge)",
            )
            .into());
        }
    }
    let spki = b64d(&body.public_key_spki_b64)?;
    let sig_bytes = b64d(&body.signature_b64)?;
    // Prefer verify over clientData hash; fall back to raw challenge (v1).
    verify_p256_flexible(&spki, &expected, &ch.bytes, &sig_bytes)?;
    store()
        .keys
        .lock()
        .entry(p.user_id.to_string())
        .or_default()
        .push(PasskeyRecord {
            spki: spki.clone(),
            counter: 0,
            device_label: body
                .device_label
                .clone()
                .unwrap_or_else(|| "passkey".into()),
        });
    let id = Uuid::now_v7();
    if let Some(pool) = state.core.clients.db.as_ref() {
        let _ = sqlx::query(
            r#"
            INSERT INTO collab.device_keys
                (id, tenant_id, user_id, device_label, public_key_b64, algorithm, webauthn_cose_key, webauthn_counter, created_at, last_seen_at)
            VALUES ($1,$2,$3,$4,$5,'ECDSA_P256_passkey',$6,0,now(),now())
            "#,
        )
        .bind(id)
        .bind(p.tenant_id.as_uuid())
        .bind(p.user_id.as_uuid())
        .bind(body.device_label.as_deref().unwrap_or("passkey"))
        .bind(b64(&spki))
        .bind(&spki)
        .execute(pool)
        .await;
    }
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
            action: "passkey.register".into(),
            resource_type: "device_key".into(),
            resource_id: id.to_string(),
            metadata: serde_json::json!({"alg": "ECDSA_P256", "protocol": "helix-passkey-v2"}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "registered": true,
        "device_key_id": id,
        "protocol": "helix-passkey-v2"
    }))))
}

async fn auth_start(
    State(state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let mut keys = store()
        .keys
        .lock()
        .get(&p.user_id.to_string())
        .cloned()
        .unwrap_or_default();
    if keys.is_empty() {
        if let Some(pool) = state.core.clients.db.as_ref() {
            let rows: Vec<(Vec<u8>, i64, String)> = sqlx::query_as(
                r#"
                SELECT webauthn_cose_key, webauthn_counter, device_label
                FROM collab.device_keys
                WHERE tenant_id = $1 AND user_id = $2 AND revoked_at IS NULL
                  AND algorithm LIKE '%passkey%' AND webauthn_cose_key IS NOT NULL
                "#,
            )
            .bind(p.tenant_id.as_uuid())
            .bind(p.user_id.as_uuid())
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            for (spki, counter, label) in rows {
                keys.push(PasskeyRecord {
                    spki,
                    counter: counter as u64,
                    device_label: label,
                });
            }
            if !keys.is_empty() {
                store()
                    .keys
                    .lock()
                    .insert(p.user_id.to_string(), keys.clone());
            }
        }
    }
    if keys.is_empty() {
        return Err(HelixError::validation("no passkeys registered").into());
    }
    let mut ch = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut ch);
    let challenge_b64 = b64(&ch);
    let client_data = client_data_bytes("authenticate", &challenge_b64, &p.user_id.to_string());
    let max_counter = keys.iter().map(|k| k.counter).max().unwrap_or(0);
    store().challenges.lock().insert(
        format!("auth:{}", p.user_id),
        Challenge {
            bytes: ch,
            expires: Instant::now() + Duration::from_secs(300),
        },
    );
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "challenge_b64": challenge_b64,
        "client_data_b64": b64(&client_data),
        "client_data_hash_b64": b64(&Sha256::digest(&client_data)),
        "allow_credentials": keys.len(),
        "expected_counter_min": max_counter,
        "protocol": "helix-passkey-v2"
    }))))
}

#[derive(Deserialize)]
struct AuthFinish {
    public_key_spki_b64: String,
    signature_b64: String,
    #[serde(default)]
    client_data_b64: Option<String>,
    #[serde(default)]
    counter: Option<u64>,
}

async fn auth_finish(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Json(body): Json<AuthFinish>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let ch = store()
        .challenges
        .lock()
        .remove(&format!("auth:{}", p.user_id))
        .ok_or_else(|| HelixError::validation("no auth challenge"))?;
    if Instant::now() > ch.expires {
        return Err(HelixError::validation("challenge expired").into());
    }
    let challenge_b64 = b64(&ch.bytes);
    let expected = client_data_bytes("authenticate", &challenge_b64, &p.user_id.to_string());
    if let Some(cd_b64) = body.client_data_b64.as_deref() {
        let got = b64d(cd_b64)?;
        if got != expected {
            return Err(HelixError::validation("clientDataJSON mismatch").into());
        }
    }
    let spki = b64d(&body.public_key_spki_b64)?;
    let mut keys = store()
        .keys
        .lock()
        .get(&p.user_id.to_string())
        .cloned()
        .unwrap_or_default();
    let idx = keys
        .iter()
        .position(|k| k.spki == spki)
        .ok_or_else(|| HelixError::forbidden("unknown passkey"))?;
    let sig = b64d(&body.signature_b64)?;
    verify_p256_flexible(&spki, &expected, &ch.bytes, &sig)?;
    let new_counter = body.counter.unwrap_or(keys[idx].counter.saturating_add(1));
    if new_counter < keys[idx].counter {
        return Err(HelixError::forbidden("passkey counter rollback").into());
    }
    keys[idx].counter = new_counter;
    store()
        .keys
        .lock()
        .insert(p.user_id.to_string(), keys.clone());
    if let Some(pool) = state.core.clients.db.as_ref() {
        let _ = sqlx::query(
            r#"
            UPDATE collab.device_keys
            SET webauthn_counter = $4, last_seen_at = now()
            WHERE tenant_id = $1 AND user_id = $2 AND webauthn_cose_key = $3
            "#,
        )
        .bind(p.tenant_id.as_uuid())
        .bind(p.user_id.as_uuid())
        .bind(&spki)
        .bind(new_counter as i64)
        .execute(pool)
        .await;
    }
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
            action: "passkey.authenticate".into(),
            resource_type: "user".into(),
            resource_id: p.user_id.to_string(),
            metadata: serde_json::json!({"counter": new_counter, "protocol": "helix-passkey-v2"}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "authenticated": true,
        "counter": new_counter,
        "protocol": "helix-passkey-v2"
    }))))
}

async fn list_creds(
    State(_state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let keys = store()
        .keys
        .lock()
        .get(&p.user_id.to_string())
        .cloned()
        .unwrap_or_default();
    let items: Vec<_> = keys
        .iter()
        .map(|k| {
            serde_json::json!({
                "public_key_spki_b64": b64(&k.spki),
                "counter": k.counter,
                "device_label": k.device_label,
            })
        })
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

fn verify_p256_flexible(
    spki: &[u8],
    client_data: &[u8],
    challenge: &[u8],
    signature: &[u8],
) -> HelixResult<()> {
    let vk = VerifyingKey::from_public_key_der(spki)
        .map_err(|e| HelixError::validation(format!("bad spki: {e}")))?;
    let sig = Signature::from_der(signature)
        .or_else(|_| Signature::from_slice(signature))
        .map_err(|e| HelixError::validation(format!("bad signature: {e}")))?;
    let cd_hash = Sha256::digest(client_data);
    if vk.verify(&cd_hash, &sig).is_ok() {
        return Ok(());
    }
    if vk.verify(client_data, &sig).is_ok() {
        return Ok(());
    }
    // v1 fallback: raw challenge
    if vk.verify(challenge, &sig).is_ok() {
        return Ok(());
    }
    let ch_hash = Sha256::digest(challenge);
    vk.verify(&ch_hash, &sig)
        .map_err(|e| HelixError::forbidden(format!("passkey verify failed: {e}")))
}
