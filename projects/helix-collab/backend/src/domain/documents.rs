//! Document REST API with durable Postgres persistence, ACL, and revisions.

use audit_log::AuditEvent;
use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::request::Parts;
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{AclPermission, CollabDocument, DocumentPatch, DocumentRevision};
use serde::{Deserialize, Serialize};
use service_kit::{session_token_from_headers, ApiError};
use shared_core::tenancy::{Actor, Principal};
use shared_core::{ApiResponse, HelixError};
use uuid::Uuid;

use super::realtime::CollabMessage;
use super::CollabState;

const RES_TYPE: &str = "document";

/// Principal extractor against [`CollabState`] (cookies + bearer + dev headers).
pub(crate) struct Auth(pub Principal);

impl FromRequestParts<CollabState> for Auth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &CollabState,
    ) -> Result<Self, Self::Rejection> {
        let session = session_token_from_headers(&parts.headers);
        let dev_user = parts
            .headers
            .get("x-helix-dev-user")
            .and_then(|v| v.to_str().ok());
        let principal = state
            .core
            .clients
            .auth
            .resolve(session.as_deref(), dev_user)
            .await
            .map_err(ApiError)?;
        Ok(Auth(principal))
    }
}

pub fn routes() -> Router<CollabState> {
    Router::new()
        .route("/v1/domain/status", get(domain_status))
        .route("/v1/documents", get(list_docs).post(create_doc))
        // query: ?workspace_id=&folder_id=&root=1
        .route(
            "/v1/documents/{id}",
            get(get_doc).patch(patch_doc).delete(delete_doc),
        )
        .route("/v1/documents/{id}/flags", post(set_flags))
        .route("/v1/documents/{id}/revisions", get(list_revisions))
        .route(
            "/v1/documents/{id}/revisions/{version}/restore",
            post(restore_revision),
        )
        .route("/v1/documents/{id}/share", post(share_doc).get(list_acl))
        .route(
            "/v1/documents/{id}/presence",
            get(list_presence).post(update_presence),
        )
        .route("/v1/presence", get(global_presence_hint))
        // Sealed CRDT late-joiner cache (opaque HC1 only; also available over WS).
        .route(
            "/v1/documents/{id}/sealed-crdt",
            get(get_sealed_crdt).post(put_sealed_crdt),
        )
}

#[derive(Deserialize)]
struct CreateDoc {
    title: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    workspace_id: Option<Uuid>,
    #[serde(default)]
    folder_id: Option<Uuid>,
    /// Server-side vault seal (HVA4). Mutually exclusive with client_e2ee.
    #[serde(default)]
    e2ee: bool,
    /// Client-held keys: body.content must already be HC1 envelope; server is blind.
    #[serde(default)]
    client_e2ee: bool,
    /// public|internal|restricted|sovereign — restricted/sovereign require client_e2ee.
    #[serde(default = "default_class")]
    classification: String,
}

fn default_class() -> String {
    "internal".into()
}

#[derive(Deserialize)]
struct DocFlagsBody {
    #[serde(default)]
    pinned: Option<bool>,
    #[serde(default)]
    archive: Option<bool>,
    /// Server-side vault seal enable/disable.
    #[serde(default)]
    e2ee: Option<bool>,
    /// Client-held E2EE enable/disable. When enabling, content must be HC1 ciphertext.
    #[serde(default)]
    client_e2ee: Option<bool>,
    /// Opaque content supplied by client when flipping client_e2ee.
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct ListDocsQuery {
    #[serde(default)]
    workspace_id: Option<Uuid>,
    #[serde(default)]
    folder_id: Option<Uuid>,
    /// When true and folder_id unset, only docs at workspace root (folder_id IS NULL).
    #[serde(default)]
    root: bool,
}

#[derive(Deserialize)]
struct PresenceBody {
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    cursor_pos: i32,
}

#[derive(Deserialize)]
struct RevQuery {
    #[serde(default = "default_rev_limit")]
    limit: i64,
}

fn default_rev_limit() -> i64 {
    50
}

#[derive(Serialize)]
struct MemoryDoc {
    id: Uuid,
    title: String,
    content: String,
    version: u32,
}

pub(crate) async fn ensure_doc_access_pub(
    state: &CollabState,
    p: &Principal,
    doc_id: Uuid,
    need: AclPermission,
) -> Result<(), ApiError> {
    // Tenant isolation first via get_document when durable.
    if let Some(repo) = state.core.clients.collab.as_ref() {
        let _ = repo.get_document(p.tenant_id, doc_id).await?;
    }
    if p.has_scope(&shared_core::tenancy::Scope::Platform) {
        return Ok(());
    }
    if let Some(acl) = state.core.clients.acl.as_ref() {
        // Legacy docs with empty ACL: tenant Read/Write scopes still work until shared.
        let entries = acl
            .list_for_resource(p.tenant_id, RES_TYPE, &doc_id.to_string())
            .await
            .map_err(ApiError)?;
        if entries.is_empty() {
            let ok = match need {
                AclPermission::Read => p.has_scope(&shared_core::tenancy::Scope::Read),
                AclPermission::Write | AclPermission::Delete | AclPermission::Share => {
                    p.has_scope(&shared_core::tenancy::Scope::Write)
                        || p.has_scope(&shared_core::tenancy::Scope::Admin)
                }
                AclPermission::Admin => p.has_scope(&shared_core::tenancy::Scope::Admin),
            };
            if !ok {
                return Err(HelixError::forbidden("document access denied").into());
            }
            return Ok(());
        }
        acl.require(p, RES_TYPE, &doc_id.to_string(), need)
            .await
            .map_err(ApiError)?;
    }
    Ok(())
}

async fn domain_status(
    State(state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "product": "helix-collab",
        "domain": "ready",
        "tenant": p.tenant_id.to_string(),
        "durable": state.core.clients.has_db(),
        "realtime": {
            "ws": "/v1/ws/documents/{id}",
            "fanout": "helix.collab.ws.>",
            "instance_id": state.hub.instance_id
        },
        "features": {
            "documents": true,
            "optimistic_versioning": true,
            "revisions": true,
            "presence": true,
            "acl": state.core.clients.acl.is_some(),
            "share": state.core.clients.acl.is_some(),
            "ws_auth": true,
            "durable_ws_patch": true,
            "crdt": super::crdt::CrdtHub::enabled(),
            "workspaces": state.core.clients.workspaces.is_some(),
            "folders": true,
            "comments": true,
            "mentions": true,
            "e2ee": true,
            "client_e2ee": true,
            "sealed_crdt": true,
            "prosemirror": true,
            "sovereign": true,
            "device_keys": true,
            "export_backpack": true,
            "spaces": true,
            "federation": true,
            "threshold_recovery": true,
            "residency_proofs": true,
            "anchored_comments": true,
            "activity": true,
            "typing": true,
            "nats_fanout": state.core.clients.bus.is_connected()
        }
    }))))
}

async fn list_docs(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Query(q): Query<ListDocsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(repo) = state.core.clients.collab.as_ref() {
        let docs = repo
            .list_documents_filtered(
                p.tenant_id,
                q.workspace_id,
                q.folder_id,
                q.root && q.folder_id.is_none(),
            )
            .await?;
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "durable": true,
            "items": docs
        }))));
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": false,
        "items": [MemoryDoc {
            id: Uuid::nil(),
            title: "Welcome".into(),
            content: "# HelixCollab\n\nStart Postgres for durable docs.".into(),
            version: 1,
        }]
    }))))
}

async fn create_doc(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Json(body): Json<CreateDoc>,
) -> Result<Json<ApiResponse<CollabDocument>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.title.trim().is_empty() {
        return Err(HelixError::validation("title required").into());
    }

    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable documents"))?;

    if let Some(ws) = state.core.clients.workspaces.as_ref() {
        let _ = ws
            .ensure_tenant(p.tenant_id, &p.user_id.to_string(), &p.residency_region)
            .await;
    }

    if body.e2ee && body.client_e2ee {
        return Err(HelixError::validation("e2ee and client_e2ee are mutually exclusive").into());
    }
    if body.client_e2ee && !super::crypto_doc::is_client_envelope(&body.content) {
        return Err(
            HelixError::validation("client_e2ee requires HC1 client envelope content").into(),
        );
    }
    super::policy::enforce_write_crypto(
        &body.classification,
        body.client_e2ee,
        body.e2ee && !body.client_e2ee,
        super::crypto_doc::is_client_envelope(&body.content),
    )?;

    let master = state.core.clients.config.vault_master_key.as_bytes();
    let tid = p.tenant_id.to_string();
    let content = if body.client_e2ee {
        // Server is blind — store client ciphertext as-is.
        body.content.clone()
    } else if body.e2ee {
        super::crypto_doc::encrypt_content(master, &tid, &body.content)?
    } else {
        body.content.clone()
    };
    let doc = repo
        .create_document_full_ex(
            p.tenant_id,
            p.user_id,
            body.title.trim(),
            &content,
            body.workspace_id,
            body.folder_id,
            body.e2ee || body.client_e2ee,
            body.client_e2ee,
        )
        .await?;

    // Persist classification when sovereign tables exist.
    if let Some(pool) = state.core.clients.db.as_ref() {
        let sov = helix_db::SovereignCollabRepo::new(pool.clone());
        let _ = sov
            .set_classification(p.tenant_id, doc.id, &body.classification, None)
            .await;
    }

    // Grant creator full document ACL.
    if let Some(acl) = state.core.clients.acl.as_ref() {
        let _ = acl
            .grant(
                p.tenant_id,
                RES_TYPE,
                &doc.id.to_string(),
                "user",
                &p.user_id.to_string(),
                &[
                    AclPermission::Read,
                    AclPermission::Write,
                    AclPermission::Delete,
                    AclPermission::Share,
                    AclPermission::Admin,
                ],
                Some(&p.user_id.to_string()),
            )
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
            action: "document.create".into(),
            resource_type: RES_TYPE.into(),
            resource_id: doc.id.to_string(),
            metadata: serde_json::json!({
                "title": doc.title,
                "version": doc.version,
                "classification": body.classification,
                "client_e2ee": body.client_e2ee,
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;

    state
        .core
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-collab",
            "documents.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;

    let _ = state
        .core
        .clients
        .bus
        .publish(
            "helix.collab.document.created",
            &serde_json::json!({"id": doc.id, "title": doc.title}),
        )
        .await;

    let mut out = doc.clone();
    // Client e2ee: never decrypt. Server vault e2ee: decrypt for API consumers.
    if out.encrypted && !out.client_e2ee {
        out.content = super::crypto_doc::decrypt_content(master, &tid, &out.content)?;
    }
    // Do not fan out client ciphertext as plaintext snapshots.
    if !out.client_e2ee {
        state.hub.publish(
            out.id,
            &CollabMessage::Snapshot {
                version: out.version,
                content: out.content.clone(),
                title: out.title.clone(),
            },
        );
    }

    Ok(Json(ApiResponse::ok(out)))
}

async fn get_doc(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<CollabDocument>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable documents"))?;
    let mut doc = repo.get_document(p.tenant_id, id).await?;
    if doc.encrypted && !doc.client_e2ee {
        let master = state.core.clients.config.vault_master_key.as_bytes();
        doc.content =
            super::crypto_doc::decrypt_content(master, &p.tenant_id.to_string(), &doc.content)?;
    }
    Ok(Json(ApiResponse::ok(doc)))
}

async fn patch_doc(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(patch): Json<DocumentPatch>,
) -> Result<Json<ApiResponse<CollabDocument>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable documents"))?;

    let existing = repo.get_document(p.tenant_id, id).await?;
    let master = state.core.clients.config.vault_master_key.as_bytes();
    let tid = p.tenant_id.to_string();
    let mut patch = patch;
    if existing.client_e2ee {
        if !super::crypto_doc::is_client_envelope(&patch.content) {
            return Err(HelixError::validation(
                "client_e2ee document requires HC1 envelope content",
            )
            .into());
        }
        // Store as-is; server never opens.
    } else if existing.encrypted {
        patch.content = super::crypto_doc::encrypt_content(master, &tid, &patch.content)?;
    }
    let mut doc = repo.apply_patch(p.tenant_id, id, p.user_id, patch).await?;
    let _ = repo
        .record_activity(
            p.tenant_id,
            id,
            Some(p.user_id),
            &p.user_id.to_string(),
            "document.patched",
            serde_json::json!({"version": doc.version, "client_e2ee": doc.client_e2ee}),
        )
        .await;

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
            action: "document.patch".into(),
            resource_type: RES_TYPE.into(),
            resource_id: doc.id.to_string(),
            metadata: serde_json::json!({
                "version": doc.version,
                "encrypted": doc.encrypted,
                "client_e2ee": doc.client_e2ee,
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    if doc.encrypted && !doc.client_e2ee {
        doc.content = super::crypto_doc::decrypt_content(master, &tid, &doc.content)?;
    }

    state
        .core
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-collab",
            "documents.patched",
            1.0,
            "count",
            serde_json::json!({"version": doc.version}),
        )
        .await?;

    // Client e2ee: only ack version; peers must unlock locally (no plaintext fan-out).
    if !doc.client_e2ee {
        state.hub.publish(
            doc.id,
            &CollabMessage::Snapshot {
                version: doc.version,
                content: doc.content.clone(),
                title: doc.title.clone(),
            },
        );
    }
    state.hub.publish(
        doc.id,
        &CollabMessage::Ack {
            version: doc.version,
        },
    );

    Ok(Json(ApiResponse::ok(doc)))
}

async fn set_flags(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<DocFlagsBody>,
) -> Result<Json<ApiResponse<CollabDocument>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let mut doc = repo
        .set_document_flags(p.tenant_id, id, body.pinned, body.archive)
        .await?;

    if let Some(want_client) = body.client_e2ee {
        if want_client != doc.client_e2ee {
            if want_client {
                let ct = body.content.as_deref().unwrap_or("");
                if !super::crypto_doc::is_client_envelope(ct) {
                    return Err(HelixError::validation(
                        "enabling client_e2ee requires HC1 content in flags body",
                    )
                    .into());
                }
                doc = repo
                    .set_encrypted_content_ex(
                        p.tenant_id,
                        id,
                        p.user_id,
                        ct,
                        true,
                        true,
                        doc.version,
                    )
                    .await?;
            } else {
                // Disable: client must send plaintext content after local decrypt.
                let plain = body.content.as_deref().unwrap_or("");
                if plain.is_empty() || super::crypto_doc::is_client_envelope(plain) {
                    return Err(HelixError::validation(
                        "disabling client_e2ee requires plaintext content in flags body",
                    )
                    .into());
                }
                doc = repo
                    .set_encrypted_content_ex(
                        p.tenant_id,
                        id,
                        p.user_id,
                        plain,
                        false,
                        false,
                        doc.version,
                    )
                    .await?;
            }
        }
    }

    if let Some(want_e2ee) = body.e2ee {
        if doc.client_e2ee && want_e2ee {
            return Err(HelixError::validation(
                "cannot enable server e2ee while client_e2ee is on",
            )
            .into());
        }
        if !doc.client_e2ee && want_e2ee != doc.encrypted {
            let master = state.core.clients.config.vault_master_key.as_bytes();
            let tid = p.tenant_id.to_string();
            let plain = if doc.encrypted {
                super::crypto_doc::decrypt_content(master, &tid, &doc.content)?
            } else {
                doc.content.clone()
            };
            let stored = if want_e2ee {
                super::crypto_doc::encrypt_content(master, &tid, &plain)?
            } else {
                plain.clone()
            };
            doc = repo
                .set_encrypted_content_ex(
                    p.tenant_id,
                    id,
                    p.user_id,
                    &stored,
                    want_e2ee,
                    false,
                    doc.version,
                )
                .await?;
            doc.content = if want_e2ee { plain } else { doc.content };
        }
    }
    if doc.encrypted && !doc.client_e2ee {
        let master = state.core.clients.config.vault_master_key.as_bytes();
        doc.content =
            super::crypto_doc::decrypt_content(master, &p.tenant_id.to_string(), &doc.content)?;
    }
    Ok(Json(ApiResponse::ok(doc)))
}

async fn delete_doc(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Delete).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable documents"))?;
    repo.delete_document(p.tenant_id, id).await?;
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
            action: "document.delete".into(),
            resource_type: RES_TYPE.into(),
            resource_id: id.to_string(),
            metadata: serde_json::json!({}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "deleted": id }))))
}

async fn list_revisions(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Query(q): Query<RevQuery>,
) -> Result<Json<ApiResponse<Vec<DocumentRevision>>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    // Omit content for list? Keep full for restore UX simplicity; client can ignore.
    Ok(Json(ApiResponse::ok(
        repo.list_revisions(p.tenant_id, id, q.limit).await?,
    )))
}

async fn restore_revision(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path((id, version)): Path<(Uuid, u32)>,
) -> Result<Json<ApiResponse<CollabDocument>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let repo = state
        .core
        .clients
        .collab
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let doc = repo
        .restore_revision(p.tenant_id, id, p.user_id, version)
        .await?;
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
            action: "document.restore".into(),
            resource_type: RES_TYPE.into(),
            resource_id: id.to_string(),
            metadata: serde_json::json!({"restored_from": version, "new_version": doc.version}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state.hub.publish(
        doc.id,
        &CollabMessage::Snapshot {
            version: doc.version,
            content: doc.content.clone(),
            title: doc.title.clone(),
        },
    );
    Ok(Json(ApiResponse::ok(doc)))
}

async fn list_presence(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let peers = if let Some(repo) = state.core.clients.collab.as_ref() {
        repo.list_presence(id).await?
    } else {
        vec![]
    };
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "document_id": id,
        "ws_peers": state.hub.peer_count(id),
        "peers": peers,
    }))))
}

async fn update_presence(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<PresenceBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    let name = if body.display_name.is_empty() {
        p.user_id.to_string()
    } else {
        body.display_name
    };

    if let Some(repo) = state.core.clients.collab.as_ref() {
        let peer = repo
            .upsert_presence(id, p.user_id, &name, body.cursor_pos)
            .await?;
        state.hub.publish(
            id,
            &CollabMessage::Presence {
                user_id: p.user_id.to_string(),
                display_name: name,
                cursor_pos: body.cursor_pos,
            },
        );
        return Ok(Json(ApiResponse::ok(serde_json::json!(peer))));
    }

    state.hub.publish(
        id,
        &CollabMessage::Presence {
            user_id: p.user_id.to_string(),
            display_name: name.clone(),
            cursor_pos: body.cursor_pos,
        },
    );
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "document_id": id,
        "user_id": p.user_id,
        "display_name": name,
        "cursor_pos": body.cursor_pos,
        "durable": false
    }))))
}

async fn global_presence_hint(
    State(state): State<CollabState>,
    Auth(p): Auth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "user": p.user_id.to_string(),
        "hint": "Use /v1/documents/{id}/presence and /v1/ws/documents/{id}?dev_user= or ?token=",
        "ws_auth": {
            "query_token": "token=<session_or_oauth>",
            "query_dev_user": "dev_user=<label> (local + HELIX_ALLOW_DEV_HEADERS)",
            "header": "Authorization: Bearer … or X-Helix-Dev-User"
        },
        "crdt": super::crdt::CrdtHub::enabled(),
        "sealed_crdt": true,
        "durable": state.core.clients.has_db(),
    }))))
}

#[derive(Deserialize)]
struct SealedCrdtBody {
    /// HC1 envelope of full Yjs state (or incremental for push_update path).
    sealed: String,
    #[serde(default)]
    kind: SealedKind,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum SealedKind {
    #[default]
    State,
    Update,
}

async fn get_sealed_crdt(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "document_id": id,
        "sealed_state": state.hub.sealed.get_state(id),
        "recent_updates": state.hub.sealed.recent_updates(id).len(),
        "server_blind": true,
    }))))
}

async fn put_sealed_crdt(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<SealedCrdtBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Write).await?;
    match body.kind {
        SealedKind::State => {
            state
                .hub
                .sealed
                .put_state(id, &body.sealed)
                .map_err(HelixError::validation)?;
            state.hub.publish(
                id,
                &CollabMessage::CrdtSealedSync {
                    sealed: body.sealed.clone(),
                },
            );
        }
        SealedKind::Update => {
            state
                .hub
                .sealed
                .push_update(id, &body.sealed)
                .map_err(HelixError::validation)?;
            state.hub.publish(
                id,
                &CollabMessage::CrdtSealedUpdate {
                    sealed: body.sealed.clone(),
                    author: p.user_id.to_string(),
                },
            );
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "document_id": id,
        "accepted": true,
        "server_blind": true,
        "has_state": state.hub.sealed.get_state(id).is_some(),
    }))))
}

#[derive(Deserialize)]
struct ShareBody {
    /// Target user id (UUID string) or email label for local dev (resolved as principal_id).
    principal_id: String,
    #[serde(default = "default_principal_kind")]
    principal_kind: String,
    /// Comma or list: read,write,share,delete,admin
    #[serde(default)]
    permissions: Option<Vec<String>>,
}

fn default_principal_kind() -> String {
    "user".into()
}

async fn share_doc(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
    Json(body): Json<ShareBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Share).await?;
    let acl = state
        .core
        .clients
        .acl
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("ACL store requires Postgres"))?;

    let raw = body
        .permissions
        .unwrap_or_else(|| vec!["read".into(), "write".into()]);
    let mut perms = Vec::new();
    for s in raw {
        if let Some(perm) = AclPermission::parse(&s) {
            perms.push(perm);
        }
    }
    if perms.is_empty() {
        perms.push(AclPermission::Read);
    }

    let principal_id = body.principal_id.trim().to_string();
    if principal_id.is_empty() {
        return Err(HelixError::validation("principal_id required").into());
    }

    let entry = acl
        .grant(
            p.tenant_id,
            RES_TYPE,
            &id.to_string(),
            body.principal_kind.trim(),
            &principal_id,
            &perms,
            Some(&p.user_id.to_string()),
        )
        .await?;

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
            action: "document.share".into(),
            resource_type: RES_TYPE.into(),
            resource_id: id.to_string(),
            metadata: serde_json::json!({
                "principal_kind": entry.principal_kind,
                "principal_id": entry.principal_id,
                "permissions": entry.permissions,
            }),
            residency_region: p.residency_region.clone(),
        })
        .await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({ "acl": entry }))))
}

async fn list_acl(
    State(state): State<CollabState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    ensure_doc_access_pub(&state, &p, id, AclPermission::Read).await?;
    let acl = state
        .core
        .clients
        .acl
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("ACL store requires Postgres"))?;
    let entries = acl
        .list_for_resource(p.tenant_id, RES_TYPE, &id.to_string())
        .await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "items": entries }),
    )))
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use service_kit::ApiError;
    use shared_core::{ApiResponse, ErrorCode};

    use crate::domain::test_support::{create_test_doc, dev_principal, locked_state};

    use super::{create_doc, patch_doc, Auth, CreateDoc, DocumentPatch};

    fn unwrap_data<T>(resp: ApiResponse<T>) -> T {
        match resp {
            ApiResponse::Ok { data, .. } => data,
            ApiResponse::Err { error } => panic!("unexpected API error: {error:?}"),
        }
    }

    fn unwrap_ok<T>(res: Result<T, ApiError>) -> T {
        match res {
            Ok(v) => v,
            Err(e) => panic!("API error: code={:?} message={}", e.0.code, e.0.message),
        }
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres/NATS/MinIO)"]
    async fn optimistic_concurrency_rejects_stale_base() {
        let (state, _guard) = locked_state().await;
        let p = dev_principal("offline-alice");

        let Json(resp) = unwrap_ok(
            create_doc(
                axum::extract::State(state.clone()),
                Auth(p.clone()),
                axum::extract::Json(CreateDoc {
                    title: "offline-merge-doc".into(),
                    content: "base".into(),
                    workspace_id: None,
                    folder_id: None,
                    e2ee: false,
                    client_e2ee: false,
                    classification: "internal".into(),
                }),
            )
            .await,
        );
        let doc = unwrap_data(resp);
        assert_eq!(doc.version, 1);

        // First patch succeeds: v1 -> v2.
        let Json(resp) = unwrap_ok(
            patch_doc(
                axum::extract::State(state.clone()),
                Auth(p.clone()),
                axum::extract::Path(doc.id),
                axum::extract::Json(DocumentPatch {
                    base_version: 1,
                    content: "first edit".into(),
                    title: None,
                }),
            )
            .await,
        );
        let doc = unwrap_data(resp);
        assert_eq!(doc.version, 2);

        // Second patch still bases itself on v1 -> must be rejected with Conflict.
        let err = patch_doc(
            axum::extract::State(state.clone()),
            Auth(p.clone()),
            axum::extract::Path(doc.id),
            axum::extract::Json(DocumentPatch {
                base_version: 1,
                content: "conflicting edit".into(),
                title: None,
            }),
        )
        .await
        .expect_err("stale base should conflict");
        assert_eq!(err.0.code, ErrorCode::Conflict);
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres/NATS/MinIO)"]
    async fn sequential_patches_create_durable_revisions() {
        let (state, _guard) = locked_state().await;
        let p = dev_principal("offline-bob");

        let doc = create_test_doc(&state, &p, "revision-chain", "v1").await;

        for i in 2..=4u32 {
            let Json(resp) = unwrap_ok(
                patch_doc(
                    axum::extract::State(state.clone()),
                    Auth(p.clone()),
                    axum::extract::Path(doc.id),
                    axum::extract::Json(DocumentPatch {
                        base_version: i - 1,
                        content: format!("v{i}"),
                        title: None,
                    }),
                )
                .await,
            );
            let patched = unwrap_data(resp);
            assert_eq!(patched.version, i);
        }

        let repo = state.core.clients.collab.as_ref().expect("collab repo");
        let revisions = repo
            .list_revisions(p.tenant_id, doc.id, 10)
            .await
            .expect("list revisions");
        // v1 initial + 3 patches = at least 4 revisions.
        assert!(
            revisions.len() >= 4,
            "expected at least 4 revisions, got {}",
            revisions.len()
        );
        let versions: Vec<u32> = revisions.iter().map(|r| r.version).collect();
        assert!(versions.contains(&4));
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres/NATS/MinIO)"]
    async fn client_e2ee_doc_rejects_plaintext_patch() {
        let (state, _guard) = locked_state().await;
        let p = dev_principal("offline-carol");

        let Json(resp) = unwrap_ok(
            create_doc(
                axum::extract::State(state.clone()),
                Auth(p.clone()),
                axum::extract::Json(CreateDoc {
                    title: "sealed-doc".into(),
                    content: "HC1.v1.payload".into(),
                    workspace_id: None,
                    folder_id: None,
                    e2ee: false,
                    client_e2ee: true,
                    classification: "internal".into(),
                }),
            )
            .await,
        );
        let doc = unwrap_data(resp);
        assert!(doc.client_e2ee);

        let err = patch_doc(
            axum::extract::State(state.clone()),
            Auth(p.clone()),
            axum::extract::Path(doc.id),
            axum::extract::Json(DocumentPatch {
                base_version: 1,
                content: "plaintext".into(),
                title: None,
            }),
        )
        .await
        .expect_err("plaintext patch on HC1 doc");
        assert_eq!(err.0.code, ErrorCode::Validation);
    }
}
