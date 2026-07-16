//! Break-glass env usage — always logged (tracing) and retained process-local for ops.
//!
//! When an audit client is available, callers should also append durable audit events
//! via `record_with_audit`.

use parking_lot::Mutex;
use serde_json::json;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(1);
static EVENTS: once_cell::sync::Lazy<Mutex<VecDeque<serde_json::Value>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(VecDeque::with_capacity(64)));

/// Record a break-glass activation (never silent).
pub fn record(kind: &str, detail: &str) {
    let id = SEQ.fetch_add(1, Ordering::SeqCst);
    let ev = json!({
        "id": id,
        "kind": kind,
        "detail": detail,
        "ts_unix_ms": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
    });
    tracing::warn!(
        breakglass_kind = kind,
        breakglass_detail = detail,
        breakglass_id = id,
        "HELIX_CODE break-glass activated"
    );
    let mut g = EVENTS.lock();
    if g.len() >= 64 {
        g.pop_front();
    }
    g.push_back(ev);
}

/// Snapshot recent break-glass events for domain status / ops.
pub fn recent() -> Vec<serde_json::Value> {
    EVENTS.lock().iter().cloned().collect()
}

pub fn count() -> usize {
    EVENTS.lock().len()
}

/// Best-effort durable audit when clients are available.
#[allow(dead_code)]
pub async fn record_with_audit(
    audit: &dyn audit_log::AuditSink,
    tenant_id: Option<shared_core::ids::TenantId>,
    kind: &str,
    detail: &str,
) {
    record(kind, detail);
    let _ = audit
        .append(audit_log::AuditEvent {
            tenant_id,
            actor: shared_core::tenancy::Actor::System {
                reason: format!("breakglass:{kind}"),
            },
            action: format!("breakglass.{kind}"),
            resource_type: "breakglass".into(),
            resource_id: kind.into(),
            metadata: json!({ "detail": detail, "kind": kind }),
            residency_region: "local".into(),
        })
        .await;
}
