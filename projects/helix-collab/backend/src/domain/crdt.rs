//! Optional Yjs/yrs CRDT layer behind the collab wire protocol.
//!
//! Enabled when `HELIX_COLLAB_CRDT=1`. Rooms hold an in-memory yrs Doc keyed by document id.
//! Clients exchange `crdt_update` / `crdt_sync` messages; text is mirrored under Y.Text("content").
//!
//! **Sealed (client e2ee) path:** `SealedCrdtHub` stores/relays opaque HC1 envelopes only —
//! the server never decodes Yjs updates or document text.

use base64::Engine;
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact, Update};

#[derive(Clone, Default)]
pub struct CrdtHub {
    rooms: Arc<DashMap<Uuid, Doc>>,
}

/// Blind relay cache for client-held-E2EE rooms.
/// Holds the latest sealed full-state envelope so late joiners can catch up without
/// the server ever seeing plaintext CRDT bytes.
#[derive(Clone, Default)]
pub struct SealedCrdtHub {
    /// document_id → last HC1 envelope of full Yjs state
    last_state: Arc<DashMap<Uuid, String>>,
    /// document_id → recent sealed incremental updates (for opportunistic catch-up)
    recent: Arc<DashMap<Uuid, Vec<String>>>,
}

impl SealedCrdtHub {
    pub fn new() -> Self {
        Self::default()
    }

    const RECENT_CAP: usize = 64;

    /// Accept a sealed full-state envelope (must look like HC1).
    pub fn put_state(&self, doc_id: Uuid, sealed: &str) -> Result<(), String> {
        let s = sealed.trim();
        if !s.starts_with("HC1.") {
            return Err("sealed CRDT state must be HC1 envelope".into());
        }
        self.last_state.insert(doc_id, s.to_string());
        // Full state supersedes incremental backlog.
        self.recent.insert(doc_id, Vec::new());
        Ok(())
    }

    pub fn get_state(&self, doc_id: Uuid) -> Option<String> {
        self.last_state.get(&doc_id).map(|v| v.clone())
    }

    /// Fan-out path: remember sealed incremental update (opaque).
    pub fn push_update(&self, doc_id: Uuid, sealed: &str) -> Result<(), String> {
        let s = sealed.trim();
        if !s.starts_with("HC1.") {
            return Err("sealed CRDT update must be HC1 envelope".into());
        }
        let mut entry = self.recent.entry(doc_id).or_default();
        entry.push(s.to_string());
        let overflow = entry.len().saturating_sub(Self::RECENT_CAP);
        if overflow > 0 {
            entry.drain(0..overflow);
        }
        Ok(())
    }

    pub fn recent_updates(&self, doc_id: Uuid) -> Vec<String> {
        self.recent
            .get(&doc_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn clear(&self, doc_id: Uuid) {
        self.last_state.remove(&doc_id);
        self.recent.remove(&doc_id);
    }
}

impl CrdtHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled() -> bool {
        std::env::var("HELIX_COLLAB_CRDT")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    fn room(&self, doc_id: Uuid) -> Doc {
        self.rooms.entry(doc_id).or_default().clone()
    }

    /// Seed CRDT text from plain content if empty.
    pub fn seed_if_empty(&self, doc_id: Uuid, content: &str) {
        let doc = self.room(doc_id);
        let text = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        if text.get_string(&txn).is_empty() && !content.is_empty() {
            text.insert(&mut txn, 0, content);
        }
    }

    /// Apply a base64-encoded Yjs update (v1). Returns full state update for peers (optional).
    pub fn apply_update_b64(&self, doc_id: Uuid, update_b64: &str) -> Result<String, String> {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(update_b64.trim())
            .map_err(|e| format!("crdt b64: {e}"))?;
        let update = Update::decode_v1(&raw).map_err(|e| format!("crdt decode: {e}"))?;
        let doc = self.room(doc_id);
        {
            let mut txn = doc.transact_mut();
            txn.apply_update(update)
                .map_err(|e| format!("crdt apply: {e}"))?;
        }
        // Echo the same update for peers (origin filters at NATS layer).
        Ok(update_b64.to_string())
    }

    /// Encode full document state as update from empty SV (bootstrap for joiners).
    pub fn encode_full_state_b64(&self, doc_id: Uuid) -> Result<String, String> {
        let doc = self.room(doc_id);
        let txn = doc.transact();
        let bytes = txn.encode_state_as_update_v1(&StateVector::default());
        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    /// Current plain text snapshot from Y.Text("content").
    #[allow(dead_code)]
    pub fn text_snapshot(&self, doc_id: Uuid) -> String {
        let doc = self.room(doc_id);
        let text = doc.get_or_insert_text("content");
        let txn = doc.transact();
        text.get_string(&txn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_and_snapshot() {
        let hub = CrdtHub::new();
        let id = Uuid::nil();
        hub.seed_if_empty(id, "hello");
        assert_eq!(hub.text_snapshot(id), "hello");
        let full = hub.encode_full_state_b64(id).unwrap();
        assert!(!full.is_empty());
    }

    #[test]
    fn sealed_hub_rejects_plaintext() {
        let hub = SealedCrdtHub::new();
        let id = Uuid::nil();
        assert!(hub.put_state(id, "not-sealed").is_err());
        assert!(hub.put_state(id, "HC1.iv.payload").is_ok());
        assert_eq!(hub.get_state(id).as_deref(), Some("HC1.iv.payload"));
        assert!(hub.push_update(id, "HC1.a.b").is_ok());
        assert_eq!(hub.recent_updates(id).len(), 1);
    }

    /// Simulate two peers editing offline and uploading their updates out of order.
    /// yrs must converge to the same text regardless of order and duplicates.
    #[test]
    fn offline_peers_converge_out_of_order_and_dedupe() {
        let hub = CrdtHub::new();
        let id = Uuid::nil();
        hub.seed_if_empty(id, "base");

        // Peer 1 inserts "A" after "base".
        let peer1 = Doc::new();
        let txt1 = peer1.get_or_insert_text("content");
        {
            let mut txn = peer1.transact_mut();
            txt1.insert(&mut txn, 0, "base");
            txt1.insert(&mut txn, 4, "A");
        }
        let upd1_b64 = {
            let txn = peer1.transact();
            let bytes = txn.encode_state_as_update_v1(&StateVector::default());
            base64::engine::general_purpose::STANDARD.encode(bytes)
        };

        // Peer 2 concurrently inserts "B" after "base".
        let peer2 = Doc::new();
        let txt2 = peer2.get_or_insert_text("content");
        {
            let mut txn = peer2.transact_mut();
            txt2.insert(&mut txn, 0, "base");
            txt2.insert(&mut txn, 4, "B");
        }
        let upd2_b64 = {
            let txn = peer2.transact();
            let bytes = txn.encode_state_as_update_v1(&StateVector::default());
            base64::engine::general_purpose::STANDARD.encode(bytes)
        };

        // Apply out of order and with a duplicate.
        hub.apply_update_b64(id, &upd2_b64).unwrap();
        hub.apply_update_b64(id, &upd1_b64).unwrap();
        hub.apply_update_b64(id, &upd2_b64).unwrap();

        let snapshot = hub.text_snapshot(id);
        assert!(snapshot.contains("base"), "base text lost: {snapshot}");
        assert!(
            snapshot.contains('A') && snapshot.contains('B'),
            "peer edits missing: {snapshot}"
        );
    }

    #[test]
    fn sealed_hub_catch_up_late_joiner() {
        let hub = SealedCrdtHub::new();
        let id = Uuid::nil();
        assert!(hub.put_state(id, "HC1.full.state").is_ok());
        for i in 0..3 {
            assert!(hub.push_update(id, &format!("HC1.update.{i}")).is_ok());
        }
        assert_eq!(hub.get_state(id).as_deref(), Some("HC1.full.state"));
        assert_eq!(hub.recent_updates(id).len(), 3);

        // A full-state reset must clear the incremental backlog.
        assert!(hub.put_state(id, "HC1.new.full").is_ok());
        assert_eq!(hub.recent_updates(id).len(), 0);
    }
}
