//! Operator-only break-glass audit chain rehash.
//!
//! Usage:
//!   HELIX_AUDIT_REHASH_APPROVED=1 helix-audit-rehash --approve
//!
//! This rewrites prev_hash/entry_hash for every row in helix_core.events from
//! stored bodies. It must only be used after verifying the cause of chain drift.

use audit_log::{AuditEvent, AuditSink};
use helix_db::PgAuditSink;
use shared_core::tenancy::Actor;
use shared_core::DbPoolConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if !args.iter().any(|a| a == "--approve") {
        eprintln!("Usage: HELIX_AUDIT_REHASH_APPROVED=1 helix-audit-rehash --approve");
        std::process::exit(1);
    }
    let approved = std::env::var("HELIX_AUDIT_REHASH_APPROVED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !approved {
        eprintln!("HELIX_AUDIT_REHASH_APPROVED=1 is required");
        std::process::exit(1);
    }
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("DATABASE_URL required");
        std::process::exit(1);
    });
    let pool = helix_db::connect_and_migrate_with_config(
        &database_url,
        &DbPoolConfig {
            max_connections: 2,
            ..Default::default()
        },
    )
    .await?;
    let sink = PgAuditSink::new(pool.clone());
    let n = sink.rehash_chain().await?;
    let verified = sink.verify_chain().await?;
    let _ = sink
        .append(AuditEvent {
            tenant_id: None,
            actor: Actor::System {
                reason: "operator_break_glass".into(),
            },
            action: "audit.rehash".into(),
            resource_type: "audit_chain".into(),
            resource_id: "global".into(),
            metadata: serde_json::json!({"rows_rehashed": n, "verified": verified}),
            residency_region: "global".into(),
        })
        .await;
    println!("rehashed={n} verified={verified}");
    Ok(())
}
