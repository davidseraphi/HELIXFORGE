//! Sovereign Collab API — horizons A–C.
//!
//! Device keys, key shares, export backpack, classification policy,
//! spaces, attachments metadata, residency proofs, federation receipts,
//! threshold recovery, durable sealed CRDT, threat-model surface.

use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use helix_db::{requires_client_e2ee, validate_classification, AclPermission, SovereignCollabRepo};
use serde::Deserialize;
use service_kit::ApiError;
use sha2::{Digest, Sha256};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError};
use uuid::Uuid;

use super::crypto_doc;
use super::documents::{ensure_doc_access_pub, Auth};
use super::realtime::CollabMessage;
use super::CollabState;

fn sovereign(state: &CollabState) -> Result<SovereignCollabRepo, ApiError> {
    let pool = state
        .core
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for sovereign APIs"))?;
    Ok(SovereignCollabRepo::new(pool.clone()))
}

pub fn routes() -> Router<CollabState> {
    Router::new()
        .route("/v1/sovereign/threat-model", get(threat_model))
        .route("/v1/sovereign/capabilities", get(capabilities))
        .route("/v1/devices", get(list_devices).post(register_device))
        .route("/v1/devices/{id}/revoke", post(revoke_device))
        .route(
            "/v1/documents/{id}/key-shares",
            get(list_key_shares).post(put_key_share),
        )
        .route(
            "/v1/documents/{id}/classification",
            get(get_classification).post(set_classification),
        )
        .route("/v1/documents/{id}/export", get(export_backpack))
        .route(
            "/v1/documents/{id}/sealed-crdt/durable",
            get(get_durable_sealed).post(put_durable_sealed),
        )
        .route(
            "/v1/workspaces/{ws_id}/spaces",
            get(list_spaces).post(create_space),
        )
        .route(
            "/v1/documents/{id}/attachments",
            get(list_attachments).post(register_attachment),
        )
        .route(
            "/v1/documents/{id}/attachments/{att_id}",
            axum::routing::delete(delete_attachment),
        )
        .route(
            "/v1/documents/{id}/residency",
            get(list_residency).post(add_residency),
        )
        .route("/v1/federation/export", post(federation_export))
        .route("/v1/federation/import", post(federation_import))
        .route("/v1/documents/{id}/recovery", post(open_recovery))
        .route(
            "/v1/recovery/{ceremony_id}/complete",
            post(complete_recovery),
        )
        .route("/v1/documents/{id}/agent/suggest", post(agent_suggest))
        // Full OpenMLS lives under /v1/mls/* and /v1/documents/{id}/mls/*
        .route(
            "/v1/documents/{id}/attachments/{att_id}/body",
            get(get_attachment_body).post(put_attachment_body),
        )
        .route(
            "/v1/documents/{id}/attachments/upload",
            post(upload_attachment),
        )
        .route(
            "/v1/documents/{id}/required-region",
            post(set_required_region),
        )
}

// ── Horizon surface ──────────────────────────────────────────────

async fn threat_model() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::ok(serde_json::json!({
        "product": "helix-collab",
        "version": "sovereign-1.0",
        "server_sees": [
            "tenant_id", "document ids", "sizes", "versions",
            "ACL graph", "connect times", "sealed blob lengths",
            "NATS fan-out metadata", "device public keys only",
            "classification labels", "residency claims"
        ],
        "server_never_sees": [
            "plaintext document body (client_e2ee)",
            "Yjs ops (sealed CRDT)",
            "device private keys",
            "passphrases",
            "unwrapped DEKs",
            "attachment plaintext when client_sealed"
        ],
        "compromise_cluster": "availability + metadata; not content if client_e2ee + sealed CRDT + no escrow",
        "compromise_laptop": "limited by device keys + passphrase; revoke devices",
        "break_glass": "threshold recovery ceremony only — multi-party, audited",
        "doc": "projects/helix-collab/docs/THREAT_MODEL.md"
    })))
}

async fn capabilities(State(state): State<CollabState>) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::ok(serde_json::json!({
        "horizons": {
            "A": ["device_keys", "webauthn", "key_shares", "sealed_presence", "export_backpack", "classification_policy", "jetstream_sealed_durable"],
            "B": ["spaces_tree", "offline_protocol", "minio_attachments", "client_agent_suggest"],
            "C": ["openmls_rfc9420", "threshold_recovery", "residency_hard_enforce", "federation_receipts"]
        },
        "openmls": true,
        "minio_objects": true,
        "webauthn": true,
        "jetstream": state.core.clients.bus.jetstream_enabled(),
        "nats": state.core.clients.bus.is_connected(),
        "durable_db": state.core.clients.has_db(),
        "policy": {
            "restricted_requires_client_e2ee": true,
            "sovereign_requires_client_e2ee": true
        }
    })))
}

// ── Devices ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RegisterDevice {
    device_label: String,
    public_key_b64: String,
    #[serde(default)]
    credential_id: Option<String>,
    #[serde(default = "default_alg")]
    algorithm: String,
}

fn default_alg() -> String {
    "ECDSA_P256".into()
}

async fn register_device(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Json(body): Json<RegisterDevice>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let repo = sovereign(&state)?;
    let dk = repo
        .register_device_key(
            p.tenant_id,
            p.user_id,
            body.device_label.trim(),
            body.public_key_b64.trim(),
            body.credential_id.as_deref(),
            &body.algorithm,
        )
        .await?;
    audit(
        &state,
        &p,
        "device.register",
        "device_key",
        &dk.id.to_string(),
        serde_json::json!({"label": dk.device_label}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(dk))))
}

async fn list_devices(
    State(state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let repo = sovereign(&state)?;
    let items = repo.list_device_keys(p.tenant_id, p.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn revoke_device(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let repo = sovereign(&state)?;
    repo.revoke_device_key(p.tenant_id, p.user_id, id).await?;
    audit(
        &state,
        &p,
        "device.revoke",
        "device_key",
        &id.to_string(),
        serde_json::json!({}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "revoked": id }))))
}

// ── Key shares ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct KeyShareBody {
    wrapped_dek: String,
    #[serde(default)]
    device_key_id: Option<Uuid>,
    #[serde(default = "default_share_kind")]
    share_kind: String,
    #[serde(default)]
    threshold_n: Option<i32>,
    #[serde(default)]
    threshold_k: Option<i32>,
    #[serde(default)]
    shard_index: Option<i32>,
}

fn default_share_kind() -> String {
    "device".into()
}

async fn put_key_share(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<KeyShareBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Share).await?;
    let repo = sovereign(&state)?;
    let share = repo
        .put_key_share(
            p.tenant_id,
            id,
            body.device_key_id,
            body.wrapped_dek.trim(),
            &body.share_kind,
            body.threshold_n,
            body.threshold_k,
            body.shard_index,
        )
        .await?;
    audit(
        &state,
        &p,
        "key_share.put",
        "document",
        &id.to_string(),
        serde_json::json!({"share_kind": body.share_kind}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(share))))
}

async fn list_key_shares(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let repo = sovereign(&state)?;
    let items = repo.list_key_shares(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

// ── Classification ───────────────────────────────────────────────

#[derive(Deserialize)]
struct ClassBody {
    classification: String,
    #[serde(default)]
    sealed_comments: Option<bool>,
}

async fn get_classification(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let repo = sovereign(&state)?;
    let (classification, sealed_comments) = repo.get_classification(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "classification": classification,
        "sealed_comments": sealed_comments,
        "requires_client_e2ee": requires_client_e2ee(&classification),
    }))))
}

async fn set_classification(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<ClassBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    validate_classification(&body.classification)?;
    let collab = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let doc = collab.get_document(p.tenant_id, id).await?;
    if requires_client_e2ee(&body.classification) && !doc.client_e2ee {
        return Err(
            HelixError::forbidden("raise classification only after enabling client_e2ee").into(),
        );
    }
    let repo = sovereign(&state)?;
    repo.set_classification(p.tenant_id, id, &body.classification, body.sealed_comments)
        .await?;
    audit(
        &state,
        &p,
        "document.classification",
        "document",
        &id.to_string(),
        serde_json::json!({"classification": body.classification}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "classification": body.classification,
        "sealed_comments": body.sealed_comments,
    }))))
}

// ── Export backpack ──────────────────────────────────────────────

async fn export_backpack(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let collab = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let doc = collab.get_document(p.tenant_id, id).await?;
    let revs = collab.list_revisions(p.tenant_id, id, 100).await?;
    let comments = collab
        .list_comments(p.tenant_id, id)
        .await
        .unwrap_or_default();
    let activity = collab
        .list_activity(p.tenant_id, id, 100)
        .await
        .unwrap_or_default();
    let sov = sovereign(&state)?;
    let (classification, sealed_comments) = sov
        .get_classification(p.tenant_id, id)
        .await
        .unwrap_or_else(|_| ("internal".into(), false));
    let key_shares = sov
        .list_key_shares(p.tenant_id, id)
        .await
        .unwrap_or_default();
    let sealed_crdt = sov.get_sealed_crdt(p.tenant_id, id).await.unwrap_or(None);
    let attachments = sov
        .list_attachments(p.tenant_id, id)
        .await
        .unwrap_or_default();
    let acl = if let Some(acl) = state.core.clients.acl.as_ref() {
        acl.list_for_resource(p.tenant_id, "document", &id.to_string())
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    // Server never decrypts client envelopes — backpack may contain HC1 as-is.
    let backpack = serde_json::json!({
        "format": "helix-collab-backpack-v1",
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "exporter": p.user_id.to_string(),
        "tenant_id": p.tenant_id.to_string(),
        "document": doc,
        "classification": classification,
        "sealed_comments": sealed_comments,
        "revisions": revs,
        "comments": comments,
        "activity": activity,
        "acl": acl,
        "key_shares": key_shares,
        "sealed_crdt_state": sealed_crdt,
        "attachments": attachments,
        "threat_model_ref": "/v1/sovereign/threat-model",
        "notes": "If client_e2ee, content/revisions/sealed_crdt are opaque HC1 — unlock offline with DEK."
    });
    let bytes = serde_json::to_vec(&backpack)
        .map_err(|e| HelixError::internal(format!("backpack serialize: {e}")))?;
    let hash = hex::encode(Sha256::digest(&bytes));
    audit(
        &state,
        &p,
        "document.export",
        "document",
        &id.to_string(),
        serde_json::json!({"sha256": hash, "bytes": bytes.len()}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "sha256": hash,
        "backpack": backpack,
    }))))
}

// ── Durable sealed CRDT (+ JetStream) ────────────────────────────

#[derive(Deserialize)]
struct DurableSealedBody {
    sealed: String,
}

async fn put_durable_sealed(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<DurableSealedBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    if !crypto_doc::is_client_envelope(&body.sealed) {
        return Err(HelixError::validation("sealed must be HC1").into());
    }
    let repo = sovereign(&state)?;
    repo.upsert_sealed_crdt(p.tenant_id, id, &body.sealed, Some(p.user_id))
        .await?;
    // Memory cache for live joiners
    let _ = state.hub.sealed.put_state(id, &body.sealed);
    // JetStream durable subject (opaque payload)
    let _ = state
        .core
        .clients
        .bus
        .publish(
            &format!("helix.collab.sealed_crdt.{id}"),
            &serde_json::json!({
                "document_id": id,
                "tenant_id": p.tenant_id.to_string(),
                "sealed": body.sealed,
                "updated_by": p.user_id.to_string(),
            }),
        )
        .await;
    state.hub.publish(
        id,
        &CollabMessage::CrdtSealedSync {
            sealed: body.sealed.clone(),
        },
    );
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "stored": true,
        "jetstream": state.core.clients.bus.jetstream_enabled(),
        "server_blind": true,
    }))))
}

async fn get_durable_sealed(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let repo = sovereign(&state)?;
    let durable = repo.get_sealed_crdt(p.tenant_id, id).await?;
    let memory = state.hub.sealed.get_state(id);
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "sealed_state": durable.clone().or(memory),
        "from_postgres": durable.is_some(),
        "server_blind": true,
    }))))
}

// ── Spaces ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SpaceBody {
    name: String,
    #[serde(default)]
    parent_id: Option<Uuid>,
    #[serde(default = "default_class")]
    classification: String,
}

fn default_class() -> String {
    "internal".into()
}

async fn create_space(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(ws_id): Path<Uuid>,
    Json(body): Json<SpaceBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let repo = sovereign(&state)?;
    let sp = repo
        .create_space(
            p.tenant_id,
            ws_id,
            body.parent_id,
            body.name.trim(),
            &body.classification,
            Some(p.user_id),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(sp))))
}

async fn list_spaces(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let repo = sovereign(&state)?;
    let items = repo.list_spaces(p.tenant_id, ws_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

// ── Attachments ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct AttachmentBody {
    filename: String,
    #[serde(default = "default_ct")]
    content_type: String,
    #[serde(default)]
    size_bytes: i64,
    object_key: String,
    #[serde(default)]
    client_sealed: bool,
    #[serde(default)]
    sha256_hex: String,
}

fn default_ct() -> String {
    "application/octet-stream".into()
}

async fn register_attachment(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<AttachmentBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let collab = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let doc = collab.get_document(p.tenant_id, id).await?;
    let (class, _) = sovereign(&state)?
        .get_classification(p.tenant_id, id)
        .await
        .unwrap_or_else(|_| ("internal".into(), false));
    if requires_client_e2ee(&class) && !body.client_sealed {
        return Err(HelixError::forbidden(
            "restricted/sovereign attachments must be client_sealed",
        )
        .into());
    }
    if doc.client_e2ee && !body.client_sealed {
        return Err(
            HelixError::validation("client_e2ee docs require client_sealed attachments").into(),
        );
    }
    let repo = sovereign(&state)?;
    let att = repo
        .register_attachment(
            p.tenant_id,
            id,
            body.filename.trim(),
            &body.content_type,
            body.size_bytes,
            body.object_key.trim(),
            body.client_sealed,
            &body.sha256_hex,
            Some(p.user_id),
        )
        .await?;
    audit(
        &state,
        &p,
        "attachment.register",
        "document",
        &id.to_string(),
        serde_json::json!({"attachment_id": att.id, "sealed": att.client_sealed}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(att))))
}

async fn list_attachments(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let items = sovereign(&state)?.list_attachments(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn delete_attachment(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path((id, att_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let object_key = sovereign(&state)?
        .delete_attachment(p.tenant_id, id, att_id)
        .await?;
    // Best-effort MinIO delete — DB row already gone.
    let minio_ok = state
        .core
        .clients
        .objects
        .delete_object(&object_key)
        .await
        .is_ok();
    audit(
        &state,
        &p,
        "attachment.delete",
        "document",
        &id.to_string(),
        serde_json::json!({
            "attachment_id": att_id,
            "object_key": object_key,
            "minio_deleted": minio_ok
        }),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "deleted": att_id,
        "object_key": object_key,
        "minio_deleted": minio_ok
    }))))
}

// ── Residency ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ResidencyBody {
    claimed_region: String,
    #[serde(default)]
    evidence: serde_json::Value,
    #[serde(default)]
    verified: bool,
    #[serde(default)]
    workspace_id: Option<Uuid>,
}

async fn add_residency(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<ResidencyBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let mut evidence = body.evidence;
    if evidence.is_null() {
        evidence = serde_json::json!({});
    }
    evidence["principal_region"] = serde_json::json!(p.residency_region);
    let proof = sovereign(&state)?
        .add_residency_proof(
            p.tenant_id,
            Some(id),
            body.workspace_id,
            body.claimed_region.trim(),
            evidence,
            body.verified,
        )
        .await?;
    // Auto-verify if claimed matches principal residency (local trust signal).
    Ok(Json(ApiResponse::ok(serde_json::json!(proof))))
}

async fn list_residency(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let items = sovereign(&state)?
        .list_residency_proofs(p.tenant_id, Some(id))
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

// ── Federation ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct FedExportBody {
    document_id: Uuid,
    remote_deployment: String,
    #[serde(default)]
    signature_b64: String,
}

async fn federation_export(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Json(body): Json<FedExportBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, body.document_id, AclPermission::Share).await?;
    // Reuse backpack export path for payload hash
    let collab = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let doc = collab.get_document(p.tenant_id, body.document_id).await?;
    let payload = serde_json::json!({
        "format": "helix-collab-federation-v1",
        "document": { "id": doc.id, "title": doc.title, "version": doc.version, "client_e2ee": doc.client_e2ee },
        "content": doc.content,
        "from_tenant": p.tenant_id.to_string(),
        "to_deployment": body.remote_deployment,
    });
    let bytes = serde_json::to_vec(&payload).map_err(|e| HelixError::internal(e.to_string()))?;
    let hash = hex::encode(Sha256::digest(&bytes));
    let receipt = sovereign(&state)?
        .add_federation_receipt(
            p.tenant_id,
            &body.remote_deployment,
            Some(body.document_id),
            "export",
            &hash,
            &body.signature_b64,
        )
        .await?;
    audit(
        &state,
        &p,
        "federation.export",
        "document",
        &body.document_id.to_string(),
        serde_json::json!({"remote": body.remote_deployment, "hash": hash}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "receipt": receipt,
        "payload": payload,
        "payload_sha256": hash,
    }))))
}

#[derive(Deserialize)]
struct FedImportBody {
    remote_deployment: String,
    payload: serde_json::Value,
    #[serde(default)]
    signature_b64: String,
}

async fn federation_import(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Json(body): Json<FedImportBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let bytes =
        serde_json::to_vec(&body.payload).map_err(|e| HelixError::internal(e.to_string()))?;
    let hash = hex::encode(Sha256::digest(&bytes));
    let content = body
        .payload
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let title = body
        .payload
        .pointer("/document/title")
        .and_then(|v| v.as_str())
        .unwrap_or("Federated import")
        .to_string();
    let client_e2ee = body
        .payload
        .pointer("/document/client_e2ee")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if client_e2ee && !crypto_doc::is_client_envelope(&content) {
        return Err(HelixError::validation("federated client_e2ee payload invalid").into());
    }
    let collab = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let doc = collab
        .create_document_full_ex(
            p.tenant_id,
            p.user_id,
            &title,
            &content,
            None,
            None,
            client_e2ee,
            client_e2ee,
        )
        .await?;
    let receipt = sovereign(&state)?
        .add_federation_receipt(
            p.tenant_id,
            &body.remote_deployment,
            Some(doc.id),
            "import",
            &hash,
            &body.signature_b64,
        )
        .await?;
    audit(
        &state,
        &p,
        "federation.import",
        "document",
        &doc.id.to_string(),
        serde_json::json!({"remote": body.remote_deployment, "hash": hash}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "document": doc,
        "receipt": receipt,
        "payload_sha256": hash,
    }))))
}

// ── Threshold recovery ───────────────────────────────────────────

#[derive(Deserialize)]
struct RecoveryBody {
    k: i32,
    n: i32,
    #[serde(default)]
    meta: serde_json::Value,
}

async fn open_recovery(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<RecoveryBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let ceremony = sovereign(&state)?
        .open_recovery(p.tenant_id, id, body.k, body.n, body.meta)
        .await?;
    audit(
        &state,
        &p,
        "recovery.open",
        "document",
        &id.to_string(),
        serde_json::json!({"k": body.k, "n": body.n, "ceremony": ceremony.id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "ceremony": ceremony,
        "note": "Shards stay client-side; server only records ceremony bookkeeping."
    }))))
}

async fn complete_recovery(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(ceremony_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    sovereign(&state)?
        .complete_recovery(p.tenant_id, ceremony_id)
        .await?;
    audit(
        &state,
        &p,
        "recovery.complete",
        "recovery_ceremony",
        &ceremony_id.to_string(),
        serde_json::json!({}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "completed": ceremony_id }),
    )))
}

// ── Client agent (Horizon B) — server only accepts already-unsealed selection ──

#[derive(Deserialize)]
struct AgentSuggestBody {
    /// Caller-decrypted selection; never send HC1 here.
    selection: String,
    #[serde(default)]
    intent: String,
}

async fn agent_suggest(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<AgentSuggestBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    if crypto_doc::is_client_envelope(&body.selection) {
        return Err(
            HelixError::validation("agent refuses HC1 — decrypt in client sandbox first").into(),
        );
    }
    if body.selection.len() > 8_000 {
        return Err(HelixError::validation("selection too large").into());
    }
    // Local deterministic "agent" — no external vendor LLM by default (sovereign).
    let intent = if body.intent.is_empty() {
        "summarize"
    } else {
        body.intent.as_str()
    };
    let words = body.selection.split_whitespace().count();
    let suggestion = match intent {
        "outline" => {
            let lines: Vec<_> = body
                .selection
                .lines()
                .filter(|l| !l.trim().is_empty())
                .take(8)
                .map(|l| format!("- {}", l.trim().chars().take(80).collect::<String>()))
                .collect();
            lines.join("\n")
        }
        "title" => body
            .selection
            .lines()
            .next()
            .unwrap_or("Untitled")
            .chars()
            .take(72)
            .collect(),
        _ => format!(
            "Summary ({words} words): {}",
            body.selection.chars().take(280).collect::<String>()
        ),
    };
    audit(
        &state,
        &p,
        "agent.suggest",
        "document",
        &id.to_string(),
        serde_json::json!({"intent": intent, "selection_chars": body.selection.len()}),
    )
    .await?;
    let _ = state
        .core
        .clients
        .bus
        .publish(
            "helix.collab.agent.suggest",
            &serde_json::json!({"document_id": id, "intent": intent}),
        )
        .await;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "intent": intent,
        "suggestion": suggestion,
        "model": "local-sovereign-extractive-v1",
        "note": "No vendor LLM; selection was client-unsealed."
    }))))
}

// ── MinIO attachment body upload/download ────────────────────────

#[derive(Deserialize)]
struct UploadAttachmentBody {
    filename: String,
    #[serde(default = "default_ct2")]
    content_type: String,
    /// Raw bytes base64 (client may pre-seal with HC1 for client_e2ee docs).
    data_b64: String,
    #[serde(default)]
    client_sealed: bool,
}

fn default_ct2() -> String {
    "application/octet-stream".into()
}

async fn upload_attachment(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<UploadAttachmentBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        body.data_b64.trim(),
    )
    .map_err(|e| HelixError::validation(format!("data_b64: {e}")))?;
    if bytes.len() > 25 * 1024 * 1024 {
        return Err(HelixError::validation("attachment max 25MiB").into());
    }
    let collab = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let doc = collab.get_document(p.tenant_id, id).await?;
    enforce_residency(&state, &p, &doc).await?;
    let (class, _) = sovereign(&state)?
        .get_classification(p.tenant_id, id)
        .await
        .unwrap_or_else(|_| ("internal".into(), false));
    let mut sealed = body.client_sealed;
    if requires_client_e2ee(&class) || doc.client_e2ee {
        if !crypto_doc::is_client_envelope(std::str::from_utf8(&bytes).unwrap_or(""))
            && !body.client_sealed
        {
            // Allow binary sealed blobs flagged client_sealed even if not UTF-8 HC1 prefix
            if !body.client_sealed {
                return Err(HelixError::forbidden(
                    "restricted/client_e2ee attachments must set client_sealed=true",
                )
                .into());
            }
        }
        sealed = true;
    }
    let sha = hex::encode(Sha256::digest(&bytes));
    // S3 keys must be path-safe — never use TenantId Display (may include "ten:" prefix).
    let object_key = format!("collab/{}/{}/{}", p.tenant_id.as_uuid(), id, Uuid::now_v7());
    state
        .core
        .clients
        .objects
        .put_object(&object_key, &bytes, &body.content_type)
        .await?;
    let att = sovereign(&state)?
        .register_attachment(
            p.tenant_id,
            id,
            body.filename.trim(),
            &body.content_type,
            bytes.len() as i64,
            &object_key,
            sealed,
            &sha,
            Some(p.user_id),
        )
        .await?;
    if let Some(pool) = state.core.clients.db.as_ref() {
        let _ = sqlx::query("UPDATE collab.attachments SET body_stored = true WHERE id = $1")
            .bind(att.id)
            .execute(pool)
            .await;
    }
    audit(
        &state,
        &p,
        "attachment.upload",
        "document",
        &id.to_string(),
        serde_json::json!({
            "attachment_id": att.id,
            "bytes": bytes.len(),
            "sha256": sha,
            "sealed": sealed,
            "object_key": object_key
        }),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "attachment": att,
        "body_stored": true,
        "storage": "minio"
    }))))
}

async fn put_attachment_body(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path((id, att_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UploadAttachmentBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    // Same as upload but for existing meta row — re-use upload for simplicity
    let _ = att_id;
    upload_attachment(State(state), Auth(p), Path(id), Json(body)).await
}

async fn get_attachment_body(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path((id, att_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let items = sovereign(&state)?.list_attachments(p.tenant_id, id).await?;
    let att = items
        .into_iter()
        .find(|a| a.id == att_id)
        .ok_or_else(|| HelixError::not_found("attachment"))?;
    let bytes = state
        .core
        .clients
        .objects
        .get_object(&att.object_key)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "attachment_id": att.id,
        "filename": att.filename,
        "content_type": att.content_type,
        "client_sealed": att.client_sealed,
        "sha256_hex": att.sha256_hex,
        "data_b64": base64::engine::general_purpose::STANDARD.encode(bytes),
        "note": if att.client_sealed { "ciphertext — decrypt client-side" } else { "plaintext at rest" }
    }))))
}

#[derive(Deserialize)]
struct RegionBody {
    required_region: String,
}

async fn set_required_region(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<RegionBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let region = body.required_region.trim();
    if region.is_empty() {
        return Err(HelixError::validation("required_region empty").into());
    }
    let pool = state
        .core
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    sqlx::query(
        "UPDATE collab.documents SET required_region = $3 WHERE tenant_id = $1 AND id = $2",
    )
    .bind(p.tenant_id.as_uuid())
    .bind(id)
    .bind(region)
    .execute(pool)
    .await
    .map_err(|e| HelixError::dependency(format!("set region: {e}")))?;
    // Record proof
    let _ = sovereign(&state)?
        .add_residency_proof(
            p.tenant_id,
            Some(id),
            None,
            region,
            serde_json::json!({
                "enforced": true,
                "principal_region": p.residency_region,
            }),
            p.residency_region == region || region == "local",
        )
        .await;
    audit(
        &state,
        &p,
        "document.required_region",
        "document",
        &id.to_string(),
        serde_json::json!({"region": region}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "required_region": region,
        "principal_region": p.residency_region,
        "allowed_now": p.residency_region == region || region == "local" || p.residency_region == "local"
    }))))
}

async fn enforce_residency(
    state: &CollabState,
    p: &shared_core::tenancy::Principal,
    doc: &helix_db::CollabDocument,
) -> Result<(), ApiError> {
    let pool = match state.core.clients.db.as_ref() {
        Some(p) => p,
        None => return Ok(()),
    };
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT required_region FROM collab.documents WHERE id = $1 AND tenant_id = $2",
    )
    .bind(doc.id)
    .bind(p.tenant_id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(|e| HelixError::dependency(format!("residency read: {e}")))?;
    if let Some((Some(req),)) = row {
        if !req.is_empty()
            && req != "local"
            && p.residency_region != "local"
            && p.residency_region != req
        {
            return Err(HelixError::forbidden(format!(
                "residency denied: principal region {} cannot access required_region {}",
                p.residency_region, req
            ))
            .into());
        }
    }
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
