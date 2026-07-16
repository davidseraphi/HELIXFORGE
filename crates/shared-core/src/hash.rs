//! Content hashing helpers (SHA-256 + BLAKE3 for high-throughput paths).

use sha2::{Digest, Sha256};

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Canonical hash for immutable audit chain links.
pub fn audit_link_hash(prev: &str, payload: &[u8]) -> String {
    let mut buf = Vec::with_capacity(prev.len() + payload.len() + 1);
    buf.extend_from_slice(prev.as_bytes());
    buf.push(0xff);
    buf.extend_from_slice(payload);
    blake3_hex(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vector() {
        // empty string SHA-256
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn audit_chain_is_deterministic() {
        let h1 = audit_link_hash("genesis", b"event-1");
        let h2 = audit_link_hash("genesis", b"event-1");
        assert_eq!(h1, h2);
        let h3 = audit_link_hash(&h1, b"event-2");
        assert_ne!(h1, h3);
    }
}
