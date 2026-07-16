//! WORM archive sink for audit entries using the object store.
//!
//! Writes one NDJSON object per entry to a date-keyed prefix:
//!   `audit/{region}/{yyyy}/{mm}/{dd}/{seq}-{entry_hash}.ndjson`
//!
//! The destination bucket must be configured with Object Lock / WORM retention;
//! this implementation intentionally exposes no delete path.

use async_trait::async_trait;
use audit_log::{ArchiveSink, AuditEntry};
use chrono::{Datelike, Utc};
use shared_core::{HelixError, HelixResult};
use vault_client::ObjectStore;

pub struct ObjectStoreArchiveSink {
    store: ObjectStore,
    region: String,
}

impl ObjectStoreArchiveSink {
    pub fn new(store: ObjectStore, region: impl Into<String>) -> Self {
        Self {
            store,
            region: region.into(),
        }
    }

    fn key(&self, seq: i64, entry: &AuditEntry) -> String {
        let now = Utc::now();
        format!(
            "audit/{region}/{year:04}/{month:02}/{day:02}/{seq:012}-{hash}.ndjson",
            region = self.region,
            year = now.year(),
            month = now.month(),
            day = now.day(),
            seq = seq,
            hash = entry.entry_hash
        )
    }
}

#[async_trait]
impl ArchiveSink for ObjectStoreArchiveSink {
    async fn append(&self, seq: i64, entry: &AuditEntry) -> HelixResult<()> {
        let bytes = serde_json::to_vec(entry)
            .map_err(|e| HelixError::internal(format!("audit archive serialize: {e}")))?;
        self.store
            .put_object(&self.key(seq, entry), &bytes, "application/x-ndjson")
            .await
    }

    async fn latest_archived_seq(&self) -> HelixResult<Option<i64>> {
        let prefix = format!("audit/{}/", self.region);
        let keys = self.store.list_keys(&prefix).await?;
        Ok(keys.iter().filter_map(|k| seq_from_key(k)).max())
    }

    async fn verify_archive(&self, up_to_seq: Option<i64>) -> HelixResult<bool> {
        let Some(up_to) = up_to_seq else {
            return Ok(true);
        };
        if up_to <= 0 {
            return Ok(true);
        }
        let prefix = format!("audit/{}/", self.region);
        let keys = self.store.list_keys(&prefix).await?;
        let present: std::collections::HashSet<i64> =
            keys.iter().filter_map(|k| seq_from_key(k)).collect();
        for seq in 1..=up_to {
            if !present.contains(&seq) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn seq_from_key(key: &str) -> Option<i64> {
    // Keys look like: audit/{region}/{yyyy}/{mm}/{dd}/{seq:012}-{hash}.ndjson
    let file = key.rsplit('/').next()?;
    let seq_part = file.split('-').next()?;
    seq_part.parse().ok()
}
