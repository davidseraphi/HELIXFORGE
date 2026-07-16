//! Real-time hub + authenticated WebSocket for document collaboration.
//!
//! Auth: query `token` (Bearer/session) or `dev_user` (local only), plus optional headers.
//! Durable patches: authenticated WS `patch` applies optimistic Postgres write then fan-out.
//! Optional CRDT: `crdt_update` / `crdt_sync` when HELIX_COLLAB_CRDT=1.
//! Sealed CRDT (client e2ee): `crdt_sealed_update` / `crdt_sealed_sync` — opaque HC1 relay only.

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use helix_db::{AclPermission, DocumentPatch};
use nats_client::HelixBus;
use serde::{Deserialize, Serialize};
use service_kit::session_token_from_headers;
use shared_core::tenancy::Principal;
use shared_core::HelixError;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::crdt::{CrdtHub, SealedCrdtHub};
use super::CollabState;

/// NATS subject prefix for cross-replica WebSocket fan-out.
pub const WS_FANOUT_PREFIX: &str = "helix.collab.ws";
pub const WS_FANOUT_WILDCARD: &str = "helix.collab.ws.>";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollabMessage {
    Join {
        user_id: String,
        display_name: String,
    },
    Presence {
        user_id: String,
        display_name: String,
        cursor_pos: i32,
    },
    Patch {
        base_version: u32,
        content: String,
        author: String,
    },
    Snapshot {
        version: u32,
        content: String,
        title: String,
    },
    Ack {
        version: u32,
    },
    Error {
        message: String,
    },
    PeerLeft {
        user_id: String,
    },
    /// Yjs update (base64 v1), optional CRDT mode.
    CrdtUpdate {
        update_b64: String,
        #[serde(default)]
        author: String,
    },
    /// Full state sync response for joiners.
    CrdtSync {
        state_b64: String,
    },
    /// Client-e2ee sealed incremental Yjs update (HC1 envelope of update bytes).
    /// Server relays only — never decrypts or applies to yrs.
    CrdtSealedUpdate {
        sealed: String,
        #[serde(default)]
        author: String,
    },
    /// Client-e2ee sealed full-state bootstrap.
    /// Empty `sealed` = request; non-empty HC1 = full Yjs state envelope.
    CrdtSealedSync {
        #[serde(default)]
        sealed: String,
    },
    /// Ephemeral typing indicator (not durable).
    Typing {
        user_id: String,
        display_name: String,
        active: bool,
    },
    /// Sealed presence: display name/cursor encrypted (HC1) for client-e2ee rooms.
    SealedPresence {
        user_id: String,
        /// HC1 envelope of JSON `{display_name, cursor_pos}`.
        sealed: String,
    },
    /// Comment activity broadcast (anchor-aware clients refresh).
    CommentEvent {
        action: String,
        comment_id: String,
        #[serde(default)]
        anchor_start: Option<i32>,
        #[serde(default)]
        anchor_end: Option<i32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanoutEnvelope {
    pub origin: Uuid,
    pub document_id: Uuid,
    pub message: CollabMessage,
}

pub fn fanout_subject(doc_id: Uuid) -> String {
    format!("{WS_FANOUT_PREFIX}.{doc_id}")
}

pub struct RealtimeHub {
    rooms: DashMap<Uuid, broadcast::Sender<String>>,
    pub instance_id: Uuid,
    bus: Option<HelixBus>,
    pub crdt: CrdtHub,
    pub sealed: SealedCrdtHub,
}

impl RealtimeHub {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
            instance_id: Uuid::now_v7(),
            bus: None,
            crdt: CrdtHub::new(),
            sealed: SealedCrdtHub::new(),
        }
    }

    pub fn with_bus(bus: HelixBus) -> Self {
        Self {
            rooms: DashMap::new(),
            instance_id: Uuid::now_v7(),
            bus: Some(bus),
            crdt: CrdtHub::new(),
            sealed: SealedCrdtHub::new(),
        }
    }

    pub fn subscribe(&self, doc_id: Uuid) -> broadcast::Receiver<String> {
        self.rooms
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(256).0)
            .subscribe()
    }

    pub fn publish_local(&self, doc_id: Uuid, msg: &CollabMessage) {
        if let Ok(payload) = serde_json::to_string(msg) {
            if let Some(tx) = self.rooms.get(&doc_id) {
                let _ = tx.send(payload);
            } else {
                let (tx, _) = broadcast::channel(256);
                let _ = tx.send(payload);
                self.rooms.insert(doc_id, tx);
            }
        }
    }

    pub fn publish(&self, doc_id: Uuid, msg: &CollabMessage) {
        self.publish_local(doc_id, msg);
        self.bridge_publish(doc_id, msg);
    }

    fn bridge_publish(&self, doc_id: Uuid, msg: &CollabMessage) {
        let Some(bus) = self.bus.clone() else {
            return;
        };
        let envelope = FanoutEnvelope {
            origin: self.instance_id,
            document_id: doc_id,
            message: msg.clone(),
        };
        let subject = fanout_subject(doc_id);
        tokio::spawn(async move {
            if let Err(err) = bus.publish(&subject, &envelope).await {
                debug!(error = %err, %subject, "collab ws fan-out publish failed");
            }
        });
    }

    pub fn peer_count(&self, doc_id: Uuid) -> usize {
        self.rooms
            .get(&doc_id)
            .map(|tx| tx.receiver_count())
            .unwrap_or(0)
    }

    pub fn apply_remote_fanout(&self, envelope: &FanoutEnvelope) {
        if envelope.origin == self.instance_id {
            return;
        }
        // Apply remote plaintext CRDT updates into local yrs so late joiners see merged state.
        // Sealed updates stay opaque — only cache last sealed state / recent list.
        match &envelope.message {
            CollabMessage::CrdtUpdate { update_b64, .. } => {
                let _ = self.crdt.apply_update_b64(envelope.document_id, update_b64);
            }
            CollabMessage::CrdtSealedUpdate { sealed, .. } => {
                let _ = self.sealed.push_update(envelope.document_id, sealed);
            }
            CollabMessage::CrdtSealedSync { sealed } if !sealed.is_empty() => {
                let _ = self.sealed.put_state(envelope.document_id, sealed);
            }
            _ => {}
        }
        self.publish_local(envelope.document_id, &envelope.message);
    }
}

impl Default for RealtimeHub {
    fn default() -> Self {
        Self::new()
    }
}

pub fn spawn_nats_bridge(hub: std::sync::Arc<RealtimeHub>, bus: HelixBus) {
    let instance_id = hub.instance_id;
    tokio::spawn(async move {
        let mut sub = match bus.subscribe(WS_FANOUT_WILDCARD).await {
            Ok(s) => s,
            Err(err) => {
                warn!(error = %err, "collab NATS fan-out subscribe failed — single-instance only");
                return;
            }
        };
        info!(
            %instance_id,
            subject = WS_FANOUT_WILDCARD,
            "collab multi-instance WS fan-out bridge started"
        );
        while let Some(msg) = sub.next().await {
            match msg.json::<FanoutEnvelope>() {
                Ok(envelope) => hub.apply_remote_fanout(&envelope),
                Err(err) => {
                    debug!(error = %err, subject = %msg.subject, "ignore non-fanout collab message");
                }
            }
        }
        warn!("collab NATS fan-out subscription ended");
    });
}

#[derive(Debug, Deserialize)]
pub struct WsAuthQuery {
    /// Session / OAuth bearer token (same as Authorization header).
    #[serde(default)]
    pub token: Option<String>,
    /// Local-only dev identity label (requires HELIX_ALLOW_DEV_HEADERS).
    #[serde(default)]
    pub dev_user: Option<String>,
}

pub fn routes() -> Router<CollabState> {
    Router::new().route("/v1/ws/documents/{id}", get(ws_upgrade))
}

async fn resolve_ws_principal(
    state: &CollabState,
    headers: &HeaderMap,
    q: &WsAuthQuery,
) -> Result<Principal, HelixError> {
    let token = q
        .token
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .or_else(|| session_token_from_headers(headers));
    let dev = q
        .dev_user
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get("x-helix-dev-user")
                .and_then(|v| v.to_str().ok())
        });
    state.core.clients.auth.resolve(token.as_deref(), dev).await
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    Path(id): Path<Uuid>,
    Query(q): Query<WsAuthQuery>,
    State(state): State<CollabState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let principal = match resolve_ws_principal(&state, &headers, &q).await {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                format!("ws auth failed: {e}"),
            )
                .into_response();
        }
    };

    // Tenant + optional ACL gate before upgrade.
    if let Some(repo) = state.core.clients.collab.as_ref() {
        match repo.get_document_by_id(id).await {
            Ok(doc) => {
                if doc.tenant_id != principal.tenant_id
                    && !principal.has_scope(&shared_core::tenancy::Scope::Platform)
                {
                    return (
                        axum::http::StatusCode::FORBIDDEN,
                        "tenant isolation".to_string(),
                    )
                        .into_response();
                }
                if let Some(acl) = state.core.clients.acl.as_ref() {
                    if let Err(e) = acl
                        .require(&principal, "document", &id.to_string(), AclPermission::Read)
                        .await
                    {
                        // Empty ACL legacy: allow Read scope within tenant.
                        let entries = acl
                            .list_for_resource(principal.tenant_id, "document", &id.to_string())
                            .await
                            .unwrap_or_default();
                        if !entries.is_empty() {
                            return (axum::http::StatusCode::FORBIDDEN, format!("acl: {e}"))
                                .into_response();
                        }
                        if !principal.has_scope(&shared_core::tenancy::Scope::Read) {
                            return (
                                axum::http::StatusCode::FORBIDDEN,
                                "acl: read required".to_string(),
                            )
                                .into_response();
                        }
                    }
                }
            }
            Err(e) => {
                return (axum::http::StatusCode::NOT_FOUND, format!("document: {e}"))
                    .into_response();
            }
        }
    }

    ws.on_upgrade(move |socket| handle_socket(socket, id, state, principal))
}

async fn handle_socket(socket: WebSocket, doc_id: Uuid, state: CollabState, principal: Principal) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.hub.subscribe(doc_id);

    // Durable snapshot + optional CRDT seed for joiner.
    // Client-e2ee: never seed yrs from ciphertext; push sealed state if cached.
    if let Some(repo) = state.core.clients.collab.as_ref() {
        if let Ok(doc) = repo.get_document(principal.tenant_id, doc_id).await {
            if doc.client_e2ee {
                if let Some(sealed) = state.hub.sealed.get_state(doc_id) {
                    state
                        .hub
                        .publish_local(doc_id, &CollabMessage::CrdtSealedSync { sealed });
                }
                // Snapshot carries opaque HC1 content (client decrypts).
                let snap = CollabMessage::Snapshot {
                    version: doc.version,
                    content: doc.content,
                    title: doc.title,
                };
                if let Ok(payload) = serde_json::to_string(&snap) {
                    let _ = sender.send(Message::Text(payload.into())).await;
                }
            } else {
                if CrdtHub::enabled() {
                    // Only seed yrs when content is not a client envelope.
                    if !super::crypto_doc::is_client_envelope(&doc.content) {
                        state.hub.crdt.seed_if_empty(doc_id, &doc.content);
                    }
                    if let Ok(state_b64) = state.hub.crdt.encode_full_state_b64(doc_id) {
                        state
                            .hub
                            .publish_local(doc_id, &CollabMessage::CrdtSync { state_b64 });
                    }
                }
                let mut content = doc.content.clone();
                if doc.encrypted && !doc.client_e2ee {
                    let master = state.core.clients.config.vault_master_key.as_bytes();
                    if let Ok(p) = super::crypto_doc::decrypt_content(
                        master,
                        &principal.tenant_id.to_string(),
                        &content,
                    ) {
                        content = p;
                    }
                }
                let snap = CollabMessage::Snapshot {
                    version: doc.version,
                    content,
                    title: doc.title,
                };
                if let Ok(payload) = serde_json::to_string(&snap) {
                    let _ = sender.send(Message::Text(payload.into())).await;
                }
            }
        }
    }

    let hub = state.hub.clone();
    let forward = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(payload) => {
                    if sender.send(Message::Text(payload.into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let mut user_id: Option<String> = Some(principal.user_id.to_string());
    let can_write = principal.has_scope(&shared_core::tenancy::Scope::Write)
        || principal.has_scope(&shared_core::tenancy::Scope::Admin)
        || principal.has_scope(&shared_core::tenancy::Scope::Platform);

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                let parsed: Result<CollabMessage, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(CollabMessage::Join {
                        user_id: _client_uid,
                        display_name,
                    }) => {
                        // Bind presence to authenticated principal (ignore spoofed user_id).
                        let uid = principal.user_id.to_string();
                        user_id = Some(uid.clone());
                        let name = if display_name.is_empty() {
                            uid.clone()
                        } else {
                            display_name
                        };
                        hub.publish(
                            doc_id,
                            &CollabMessage::Presence {
                                user_id: uid,
                                display_name: name,
                                cursor_pos: 0,
                            },
                        );
                    }
                    Ok(CollabMessage::Presence {
                        display_name,
                        cursor_pos,
                        ..
                    }) => {
                        let uid = principal.user_id.to_string();
                        if let Some(repo) = state.core.clients.collab.as_ref() {
                            let _ = repo
                                .upsert_presence(
                                    doc_id,
                                    principal.user_id,
                                    &display_name,
                                    cursor_pos,
                                )
                                .await;
                        }
                        hub.publish(
                            doc_id,
                            &CollabMessage::Presence {
                                user_id: uid,
                                display_name,
                                cursor_pos,
                            },
                        );
                    }
                    Ok(CollabMessage::Patch {
                        base_version,
                        content,
                        ..
                    }) => {
                        if !can_write {
                            hub.publish(
                                doc_id,
                                &CollabMessage::Error {
                                    message: "write scope required for durable patch".into(),
                                },
                            );
                            continue;
                        }
                        if let Some(repo) = state.core.clients.collab.as_ref() {
                            let existing =
                                match repo.get_document(principal.tenant_id, doc_id).await {
                                    Ok(d) => d,
                                    Err(e) => {
                                        hub.publish(
                                            doc_id,
                                            &CollabMessage::Error {
                                                message: e.to_string(),
                                            },
                                        );
                                        continue;
                                    }
                                };
                            let mut stored = content.clone();
                            if existing.client_e2ee {
                                if !super::crypto_doc::is_client_envelope(&stored) {
                                    hub.publish(
                                        doc_id,
                                        &CollabMessage::Error {
                                            message: "client_e2ee requires HC1 envelope on patch"
                                                .into(),
                                        },
                                    );
                                    continue;
                                }
                            } else if existing.encrypted {
                                let master = state.core.clients.config.vault_master_key.as_bytes();
                                match super::crypto_doc::encrypt_content(
                                    master,
                                    &principal.tenant_id.to_string(),
                                    &stored,
                                ) {
                                    Ok(c) => stored = c,
                                    Err(e) => {
                                        hub.publish(
                                            doc_id,
                                            &CollabMessage::Error {
                                                message: e.to_string(),
                                            },
                                        );
                                        continue;
                                    }
                                }
                            }
                            match repo
                                .apply_patch(
                                    principal.tenant_id,
                                    doc_id,
                                    principal.user_id,
                                    DocumentPatch {
                                        base_version,
                                        content: stored,
                                        title: None,
                                    },
                                )
                                .await
                            {
                                Ok(doc) => {
                                    // Client e2ee: never fan out ciphertext as "content" peers can't use
                                    // without local decrypt — ack only.
                                    if !doc.client_e2ee {
                                        let mut fanout_content = doc.content.clone();
                                        if doc.encrypted {
                                            let master = state
                                                .core
                                                .clients
                                                .config
                                                .vault_master_key
                                                .as_bytes();
                                            if let Ok(p) = super::crypto_doc::decrypt_content(
                                                master,
                                                &principal.tenant_id.to_string(),
                                                &fanout_content,
                                            ) {
                                                fanout_content = p;
                                            }
                                        }
                                        hub.publish(
                                            doc_id,
                                            &CollabMessage::Snapshot {
                                                version: doc.version,
                                                content: fanout_content,
                                                title: doc.title,
                                            },
                                        );
                                    }
                                    hub.publish(
                                        doc_id,
                                        &CollabMessage::Ack {
                                            version: doc.version,
                                        },
                                    );
                                }
                                Err(e) => {
                                    hub.publish(
                                        doc_id,
                                        &CollabMessage::Error {
                                            message: e.to_string(),
                                        },
                                    );
                                }
                            }
                        } else {
                            // No durable store: live fan-out only.
                            hub.publish(
                                doc_id,
                                &CollabMessage::Patch {
                                    base_version,
                                    content,
                                    author: principal.user_id.to_string(),
                                },
                            );
                        }
                    }
                    Ok(CollabMessage::CrdtUpdate { update_b64, .. }) => {
                        if !CrdtHub::enabled() {
                            hub.publish(
                                doc_id,
                                &CollabMessage::Error {
                                    message: "CRDT disabled (set HELIX_COLLAB_CRDT=1)".into(),
                                },
                            );
                            continue;
                        }
                        if !can_write {
                            hub.publish(
                                doc_id,
                                &CollabMessage::Error {
                                    message: "write scope required for crdt_update".into(),
                                },
                            );
                            continue;
                        }
                        // Reject plaintext CRDT on client-e2ee docs — must use sealed path.
                        if let Some(repo) = state.core.clients.collab.as_ref() {
                            if let Ok(d) = repo.get_document(principal.tenant_id, doc_id).await {
                                if d.client_e2ee {
                                    hub.publish(
                                        doc_id,
                                        &CollabMessage::Error {
                                            message: "client_e2ee requires crdt_sealed_update"
                                                .into(),
                                        },
                                    );
                                    continue;
                                }
                            }
                        }
                        match hub.crdt.apply_update_b64(doc_id, &update_b64) {
                            Ok(_) => {
                                hub.publish(
                                    doc_id,
                                    &CollabMessage::CrdtUpdate {
                                        update_b64,
                                        author: principal.user_id.to_string(),
                                    },
                                );
                            }
                            Err(e) => {
                                hub.publish(doc_id, &CollabMessage::Error { message: e });
                            }
                        }
                    }
                    Ok(CollabMessage::CrdtSync { .. }) => {
                        // Clients shouldn't push full sync; re-send server state.
                        if CrdtHub::enabled() {
                            if let Ok(state_b64) = hub.crdt.encode_full_state_b64(doc_id) {
                                hub.publish_local(doc_id, &CollabMessage::CrdtSync { state_b64 });
                            }
                        }
                    }
                    Ok(CollabMessage::CrdtSealedUpdate { sealed, .. }) => {
                        if !can_write {
                            hub.publish(
                                doc_id,
                                &CollabMessage::Error {
                                    message: "write scope required for crdt_sealed_update".into(),
                                },
                            );
                            continue;
                        }
                        match hub.sealed.push_update(doc_id, &sealed) {
                            Ok(()) => {
                                hub.publish(
                                    doc_id,
                                    &CollabMessage::CrdtSealedUpdate {
                                        sealed,
                                        author: principal.user_id.to_string(),
                                    },
                                );
                            }
                            Err(e) => {
                                hub.publish(doc_id, &CollabMessage::Error { message: e });
                            }
                        }
                    }
                    Ok(CollabMessage::CrdtSealedSync { sealed }) => {
                        if sealed.trim().is_empty() {
                            // Request: reply with last sealed full state if any.
                            if let Some(s) = hub.sealed.get_state(doc_id) {
                                hub.publish_local(
                                    doc_id,
                                    &CollabMessage::CrdtSealedSync { sealed: s },
                                );
                            } else {
                                // No cached state — empty ack so client can seed from durable HC1.
                                hub.publish_local(
                                    doc_id,
                                    &CollabMessage::CrdtSealedSync {
                                        sealed: String::new(),
                                    },
                                );
                            }
                        } else if can_write {
                            match hub.sealed.put_state(doc_id, &sealed) {
                                Ok(()) => {
                                    hub.publish(doc_id, &CollabMessage::CrdtSealedSync { sealed });
                                }
                                Err(e) => {
                                    hub.publish(doc_id, &CollabMessage::Error { message: e });
                                }
                            }
                        } else {
                            hub.publish(
                                doc_id,
                                &CollabMessage::Error {
                                    message: "write scope required to publish sealed sync".into(),
                                },
                            );
                        }
                    }
                    Ok(CollabMessage::Typing {
                        display_name,
                        active,
                        ..
                    }) => {
                        hub.publish(
                            doc_id,
                            &CollabMessage::Typing {
                                user_id: principal.user_id.to_string(),
                                display_name: if display_name.is_empty() {
                                    principal.user_id.to_string()
                                } else {
                                    display_name
                                },
                                active,
                            },
                        );
                    }
                    Ok(CollabMessage::SealedPresence { sealed, .. }) => {
                        if !super::crypto_doc::is_client_envelope(&sealed) {
                            hub.publish(
                                doc_id,
                                &CollabMessage::Error {
                                    message: "sealed_presence requires HC1 envelope".into(),
                                },
                            );
                            continue;
                        }
                        hub.publish(
                            doc_id,
                            &CollabMessage::SealedPresence {
                                user_id: principal.user_id.to_string(),
                                sealed,
                            },
                        );
                    }
                    Ok(other) => {
                        hub.publish(doc_id, &other);
                    }
                    Err(e) => {
                        hub.publish(
                            doc_id,
                            &CollabMessage::Error {
                                message: format!("invalid message: {e}"),
                            },
                        );
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    if let Some(uid) = user_id {
        hub.publish(doc_id, &CollabMessage::PeerLeft { user_id: uid });
    }
    forward.abort();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn fanout_subject_format() {
        let id = Uuid::nil();
        assert_eq!(
            fanout_subject(id),
            "helix.collab.ws.00000000-0000-0000-0000-000000000000"
        );
    }

    #[tokio::test]
    async fn apply_remote_skips_own_origin() {
        let hub = Arc::new(RealtimeHub::new());
        let mut rx = hub.subscribe(Uuid::nil());
        let envelope = FanoutEnvelope {
            origin: hub.instance_id,
            document_id: Uuid::nil(),
            message: CollabMessage::Ack { version: 3 },
        };
        hub.apply_remote_fanout(&envelope);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn apply_remote_delivers_other_origin() {
        let hub = Arc::new(RealtimeHub::new());
        let mut rx = hub.subscribe(Uuid::nil());
        let envelope = FanoutEnvelope {
            origin: Uuid::now_v7(),
            document_id: Uuid::nil(),
            message: CollabMessage::Ack { version: 9 },
        };
        hub.apply_remote_fanout(&envelope);
        let payload = rx.try_recv().expect("remote message");
        let msg: CollabMessage = serde_json::from_str(&payload).unwrap();
        match msg {
            CollabMessage::Ack { version } => assert_eq!(version, 9),
            other => panic!("unexpected {other:?}"),
        }
    }
}
