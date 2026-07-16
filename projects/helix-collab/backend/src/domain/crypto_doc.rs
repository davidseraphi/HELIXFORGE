//! Document content encryption.
//!
//! - **Server vault e2ee**: HVA4 tenant seal (server can open with master key).
//! - **Client e2ee**: `HC1.` envelopes produced only on the client; server is blind.

use base64::Engine;
use shared_core::{HelixError, HelixResult};
use vault_client::{vault_open_tenant, vault_seal_tenant};

/// Client-held envelope prefix (HelixCollab client crypto v1).
pub const CLIENT_ENVELOPE_PREFIX: &str = "HC1.";

/// True when content is a client-held ciphertext envelope.
pub fn is_client_envelope(stored: &str) -> bool {
    stored.trim_start().starts_with(CLIENT_ENVELOPE_PREFIX)
}

/// Seal plaintext for a tenant; returns base64 envelope.
pub fn encrypt_content(master: &[u8], tenant_id: &str, plaintext: &str) -> HelixResult<String> {
    let sealed = vault_seal_tenant(master, tenant_id, plaintext.as_bytes())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(sealed))
}

/// Open sealed content; if not a vault envelope, return as plain (legacy).
/// Never attempts to open `HC1.` client envelopes.
pub fn decrypt_content(master: &[u8], tenant_id: &str, stored: &str) -> HelixResult<String> {
    if is_client_envelope(stored) {
        return Ok(stored.to_string());
    }
    let raw = match base64::engine::general_purpose::STANDARD.decode(stored.trim()) {
        Ok(b) if b.starts_with(b"HVA") => b,
        _ => return Ok(stored.to_string()),
    };
    let plain = vault_open_tenant(master, tenant_id, &raw)
        .map_err(|e| HelixError::internal(format!("doc decrypt: {e}")))?;
    String::from_utf8(plain).map_err(|e| HelixError::internal(format!("doc utf8: {e}")))
}
