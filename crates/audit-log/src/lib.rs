//! Immutable, hash-chained audit log.
//!
//! Each entry links to the previous entry's BLAKE3 hash. Tampering with any
//! historical record breaks the chain verification.

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use shared_core::hash::audit_link_hash;
use shared_core::ids::{AuditId, TenantId};
use shared_core::tenancy::Actor;
use shared_core::time::UtcTimestamp;
use shared_core::{HelixError, HelixResult};
use std::sync::{Arc, OnceLock};

/// Default genesis when HELIX_AUDIT_GENESIS is unset (override per deployment for sovereignty).
pub const GENESIS_HASH: &str = "helixforge-audit-genesis-v1";

static HMAC_SECRET: OnceLock<Option<String>> = OnceLock::new();

/// Set the global audit HMAC secret once per process (centralised via CoreConfig in Phase 6).
/// Falls back to HELIX_AUDIT_HMAC_SECRET if never called.
pub fn set_hmac_secret(secret: Option<String>) {
    let _ = HMAC_SECRET.set(secret);
}

fn hmac_secret() -> Option<String> {
    HMAC_SECRET.get().cloned().flatten().or_else(|| {
        std::env::var("HELIX_AUDIT_HMAC_SECRET")
            .ok()
            .filter(|s| !s.is_empty())
    })
}

pub fn genesis_hash() -> String {
    std::env::var("HELIX_AUDIT_GENESIS").unwrap_or_else(|_| GENESIS_HASH.to_string())
}

/// True when an audit HMAC secret is configured (sovereign mode).
pub fn hmac_enabled() -> bool {
    hmac_secret().is_some()
}

/// Optional HMAC-SHA256 of entry_hash. Empty when unset.
pub fn sign_entry_hash(entry_hash: &str) -> String {
    let Some(secret) = hmac_secret() else {
        return String::new();
    };
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    mac.update(entry_hash.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Fail-closed HMAC verification.
/// - Empty signature is only acceptable when HMAC is not configured.
/// - A signed row without an available secret fails verification (secret rotation must re-sign).
pub fn verify_entry_signature(entry_hash: &str, signature: &str) -> bool {
    if signature.is_empty() {
        return !hmac_enabled();
    }
    let expected = sign_entry_hash(entry_hash);
    if expected.is_empty() {
        // Signed row but no secret available -> fail closed.
        return false;
    }
    constant_time_eq(expected.as_bytes(), signature.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut d = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        d |= x ^ y;
    }
    d == 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: AuditId,
    pub tenant_id: Option<TenantId>,
    pub actor: Actor,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub metadata: serde_json::Value,
    pub created_at: UtcTimestamp,
    pub prev_hash: String,
    pub entry_hash: String,
    pub residency_region: String,
    /// HMAC-SHA256 hex of entry_hash when HELIX_AUDIT_HMAC_SECRET is set.
    #[serde(default)]
    pub hmac_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditBody {
    id: AuditId,
    tenant_id: Option<TenantId>,
    actor: Actor,
    action: String,
    resource_type: String,
    resource_id: String,
    metadata: serde_json::Value,
    created_at: UtcTimestamp,
    residency_region: String,
}

impl AuditEntry {
    /// Build a new chained entry from an event and the previous hash tip.
    pub fn from_event(event: AuditEvent, prev_hash: impl Into<String>) -> HelixResult<Self> {
        let prev_hash = prev_hash.into();
        let id = AuditId::new();
        let created_at = UtcTimestamp::now();
        let body = AuditBody {
            id,
            tenant_id: event.tenant_id,
            actor: event.actor.clone(),
            action: event.action.clone(),
            resource_type: event.resource_type.clone(),
            resource_id: event.resource_id.clone(),
            metadata: event.metadata.clone(),
            created_at,
            residency_region: event.residency_region.clone(),
        };
        let entry_hash = hash_body(&prev_hash, &body)?;
        let hmac_signature = sign_entry_hash(&entry_hash);
        Ok(Self {
            id,
            tenant_id: event.tenant_id,
            actor: event.actor,
            action: event.action,
            resource_type: event.resource_type,
            resource_id: event.resource_id,
            metadata: event.metadata,
            created_at,
            prev_hash,
            entry_hash,
            residency_region: event.residency_region,
            hmac_signature,
        })
    }

    /// Recompute this entry's content hash (for chain verification).
    pub fn recompute_hash(&self) -> HelixResult<String> {
        let body = AuditBody {
            id: self.id,
            tenant_id: self.tenant_id,
            actor: self.actor.clone(),
            action: self.action.clone(),
            resource_type: self.resource_type.clone(),
            resource_id: self.resource_id.clone(),
            metadata: self.metadata.clone(),
            created_at: self.created_at,
            residency_region: self.residency_region.clone(),
        };
        hash_body(&self.prev_hash, &body)
    }
}

fn hash_body(prev_hash: &str, body: &AuditBody) -> HelixResult<String> {
    let payload = canonical_body_json(body)?;
    Ok(audit_link_hash(prev_hash, &payload))
}

/// Deterministic, canonical JSON serialization for audit hashing.
/// Sorts object keys, preserves array order, and uses fixed microsecond RFC 3339 timestamps
/// so Postgres JSONB round-trips do not drift the content hash.
fn canonical_body_json(body: &AuditBody) -> HelixResult<Vec<u8>> {
    use chrono::SecondsFormat;
    use std::collections::BTreeMap;

    let mut map: BTreeMap<&str, serde_json::Value> = BTreeMap::new();
    map.insert("action", serde_json::Value::String(body.action.clone()));
    map.insert(
        "actor",
        serde_json::to_value(&body.actor)
            .map_err(|e| HelixError::internal(format!("audit actor encode: {e}")))?,
    );
    map.insert(
        "created_at",
        serde_json::Value::String(
            body.created_at
                .inner()
                .to_rfc3339_opts(SecondsFormat::Micros, true),
        ),
    );
    map.insert(
        "id",
        serde_json::to_value(body.id)
            .map_err(|e| HelixError::internal(format!("audit id encode: {e}")))?,
    );
    map.insert("metadata", canonical_value(&body.metadata));
    map.insert(
        "residency_region",
        serde_json::Value::String(body.residency_region.clone()),
    );
    map.insert(
        "resource_id",
        serde_json::Value::String(body.resource_id.clone()),
    );
    map.insert(
        "resource_type",
        serde_json::Value::String(body.resource_type.clone()),
    );
    map.insert(
        "tenant_id",
        serde_json::to_value(body.tenant_id)
            .map_err(|e| HelixError::internal(format!("audit tenant_id encode: {e}")))?,
    );

    serde_json::to_vec(&serde_json::Value::Object(
        map.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
    ))
    .map_err(|e| HelixError::internal(format!("audit canonical serialize: {e}")))
}

fn canonical_value(value: &serde_json::Value) -> serde_json::Value {
    use std::collections::BTreeMap;
    match value {
        serde_json::Value::Object(m) => {
            let sorted: BTreeMap<_, _> = m
                .iter()
                .map(|(k, v)| (k.clone(), canonical_value(v)))
                .collect();
            serde_json::Value::Object(sorted.into_iter().collect())
        }
        serde_json::Value::Array(a) => {
            serde_json::Value::Array(a.iter().map(canonical_value).collect())
        }
        other => other.clone(),
    }
}

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub tenant_id: Option<TenantId>,
    pub actor: Actor,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub metadata: serde_json::Value,
    pub residency_region: String,
}

#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn append(&self, event: AuditEvent) -> HelixResult<AuditEntry>;
    async fn verify_chain(&self) -> HelixResult<bool>;
    async fn list_recent(&self, limit: usize) -> HelixResult<Vec<AuditEntry>>;
    async fn count(&self) -> HelixResult<u64>;

    /// Tenant-scoped recent events (default: filter list_recent in memory).
    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
        limit: usize,
    ) -> HelixResult<Vec<AuditEntry>> {
        let all = self.list_recent(limit.saturating_mul(4).max(limit)).await?;
        Ok(all
            .into_iter()
            .filter(|e| e.tenant_id == Some(tenant_id))
            .take(limit)
            .collect())
    }
}

/// WORM / immutable archive sink for audit entries (e.g., object-store append).
#[async_trait]
pub trait ArchiveSink: Send + Sync {
    /// Persist one entry to archive storage. `seq` is the monotonic audit sequence.
    async fn append(&self, seq: i64, entry: &AuditEntry) -> HelixResult<()>;

    /// Highest sequence known to be archived, if any.
    async fn latest_archived_seq(&self) -> HelixResult<Option<i64>>;

    /// Read archive back and verify hashes + HMAC up to the given seq.
    async fn verify_archive(&self, up_to_seq: Option<i64>) -> HelixResult<bool>;
}

/// No-op archive sink for tests and deployments without WORM storage.
pub struct NullArchiveSink;

#[async_trait]
impl ArchiveSink for NullArchiveSink {
    async fn append(&self, _seq: i64, _entry: &AuditEntry) -> HelixResult<()> {
        Ok(())
    }

    async fn latest_archived_seq(&self) -> HelixResult<Option<i64>> {
        Ok(None)
    }

    async fn verify_archive(&self, _up_to_seq: Option<i64>) -> HelixResult<bool> {
        Ok(true)
    }
}

/// In-memory audit sink for tests and offline local boot.
#[derive(Clone, Default)]
pub struct MemoryAuditSink {
    inner: Arc<RwLock<Vec<AuditEntry>>>,
}

impl MemoryAuditSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl AuditSink for MemoryAuditSink {
    async fn append(&self, event: AuditEvent) -> HelixResult<AuditEntry> {
        let mut guard = self.inner.write();
        let prev_hash = guard
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(genesis_hash);
        let entry = AuditEntry::from_event(event, prev_hash)?;
        guard.push(entry.clone());
        Ok(entry)
    }

    async fn verify_chain(&self) -> HelixResult<bool> {
        let guard = self.inner.read();
        let mut expected_prev = genesis_hash();
        for entry in guard.iter() {
            if entry.prev_hash != expected_prev {
                return Ok(false);
            }
            // Memory path keeps original objects — full content hash check.
            let recomputed = entry.recompute_hash()?;
            if recomputed != entry.entry_hash {
                return Ok(false);
            }
            if !verify_entry_signature(&entry.entry_hash, &entry.hmac_signature) {
                return Ok(false);
            }
            expected_prev = entry.entry_hash.clone();
        }
        Ok(true)
    }

    async fn list_recent(&self, limit: usize) -> HelixResult<Vec<AuditEntry>> {
        let guard = self.inner.read();
        let start = guard.len().saturating_sub(limit);
        Ok(guard[start..].to_vec())
    }

    async fn count(&self) -> HelixResult<u64> {
        Ok(self.len() as u64)
    }

    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
        limit: usize,
    ) -> HelixResult<Vec<AuditEntry>> {
        let guard = self.inner.read();
        Ok(guard
            .iter()
            .rev()
            .filter(|e| e.tenant_id == Some(tenant_id))
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_core::ids::{TenantId, UserId};

    #[tokio::test]
    async fn chain_verifies_after_appends() {
        let sink = MemoryAuditSink::new();
        for i in 0..5 {
            sink.append(AuditEvent {
                tenant_id: Some(TenantId::new()),
                actor: Actor::User {
                    user_id: UserId::new(),
                    tenant_id: TenantId::new(),
                },
                action: format!("test.action.{i}"),
                resource_type: "unit".into(),
                resource_id: format!("{i}"),
                metadata: serde_json::json!({"i": i}),
                residency_region: "local".into(),
            })
            .await
            .unwrap();
        }
        assert!(sink.verify_chain().await.unwrap());
        assert_eq!(sink.count().await.unwrap(), 5);
    }

    #[test]
    fn from_event_is_deterministic_for_same_created_fields() {
        // Hash depends on created_at; ensure recompute matches stored hash.
        let event = AuditEvent {
            tenant_id: None,
            actor: Actor::System {
                reason: "test".into(),
            },
            action: "a".into(),
            resource_type: "r".into(),
            resource_id: "1".into(),
            metadata: serde_json::json!({}),
            residency_region: "local".into(),
        };
        let entry = AuditEntry::from_event(event, GENESIS_HASH).unwrap();
        assert_eq!(entry.recompute_hash().unwrap(), entry.entry_hash);
    }

    #[test]
    fn canonical_hash_is_key_order_independent() {
        // Two metadata objects with different insertion order but same content
        // must produce the same entry hash when all other fields are equal.
        let ts = shared_core::time::UtcTimestamp::now();
        let id = shared_core::ids::AuditId::new();
        let body1 = AuditBody {
            id,
            tenant_id: None,
            actor: Actor::System {
                reason: "test".into(),
            },
            action: "a".into(),
            resource_type: "r".into(),
            resource_id: "1".into(),
            metadata: serde_json::json!({"z": 1, "a": 2}),
            created_at: ts,
            residency_region: "local".into(),
        };
        let mut body2 = body1.clone();
        body2.metadata = serde_json::json!({"a": 2, "z": 1});

        let hash1 = hash_body(GENESIS_HASH, &body1).unwrap();
        let hash2 = hash_body(GENESIS_HASH, &body2).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hmac_signature_fail_closed() {
        // Without a secret configured, empty signatures are accepted.
        std::env::remove_var("HELIX_AUDIT_HMAC_SECRET");
        assert!(verify_entry_signature("any", ""));

        // With a secret configured, empty signatures are rejected.
        std::env::set_var("HELIX_AUDIT_HMAC_SECRET", "test-secret");
        assert!(!verify_entry_signature("any", ""));

        // Valid signature verifies.
        let sig = sign_entry_hash("entry-hash");
        assert!(!sig.is_empty());
        assert!(verify_entry_signature("entry-hash", &sig));

        // Tampered hash fails.
        assert!(!verify_entry_signature("different-hash", &sig));

        // Signed row with missing secret fails.
        std::env::remove_var("HELIX_AUDIT_HMAC_SECRET");
        assert!(!verify_entry_signature("entry-hash", &sig));
    }
}
