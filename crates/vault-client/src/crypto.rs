//! AES-256-GCM envelope helpers for vault ciphertext.
//!
//! - **HVA5**: Argon2id per-tenant DEK + AAD with embedded key version — preferred
//! - **HVA4**: Argon2id per-tenant DEK + AAD (legacy unversioned)
//! - **HVA3**: random DEK wrapped by KMS
//! - **HVA2**: per-tenant DEK = SHA-256 (legacy; still readable)
//! - **HVA1**: global key (legacy)
//! - **XOR**: pre-AES demo rows (read-only migration)

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use getrandom::getrandom;
use sha2::{Digest, Sha256};
use shared_core::{HelixError, HelixResult};

const NONCE_LEN: usize = 12;
const MAGIC_V1: &[u8] = b"HVA1"; // global key
const MAGIC_V2: &[u8] = b"HVA2"; // per-tenant DEK (sha256)
const MAGIC_V3: &[u8] = b"HVA3"; // random DEK wrapped by KMS
const MAGIC_V4: &[u8] = b"HVA4"; // argon2id tenant DEK + AAD (legacy, key version = 1)
const MAGIC_V5: &[u8] = b"HVA5"; // argon2id tenant DEK + AAD with embedded key version

const DEFAULT_KEY_VERSION: u32 = 1;
const PURPOSE: &[u8] = b"helix-vault-secret-v1";

pub fn derive_key(master_key: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(master_key);
    hasher.update(b"helixforge-vault-aes-v1");
    let dig = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&dig);
    key
}

/// Legacy SHA-256 tenant DEK (HVA2). Prefer [`derive_tenant_key_argon2`].
pub fn derive_tenant_key(master_key: &[u8], tenant_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(master_key);
    hasher.update(b"helixforge-vault-tenant-dek-v1");
    hasher.update(tenant_id.as_bytes());
    let dig = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&dig);
    key
}

/// Argon2id per-tenant DEK (Kimi P1). Salt is deterministic from tenant_id so DEKs are stable.
pub fn derive_tenant_key_argon2(master_key: &[u8], tenant_id: &str) -> HelixResult<[u8; 32]> {
    // Salt = first 16 bytes of SHA-256("helix-salt-v1" || tenant_id) — unique per tenant.
    let mut salt_hasher = Sha256::new();
    salt_hasher.update(b"helixforge-vault-salt-v1");
    salt_hasher.update(tenant_id.as_bytes());
    let salt_full = salt_hasher.finalize();
    let salt = &salt_full[..16];

    // Interactive-ish params: m=19 MiB, t=2, p=1 — fine for local/dev; raise for HSM-bound paths.
    let params = Params::new(19 * 1024, 2, 1, Some(32))
        .map_err(|e| HelixError::internal(format!("argon2 params: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon
        .hash_password_into(master_key, salt, &mut out)
        .map_err(|e| HelixError::internal(format!("argon2 dek: {e}")))?;
    Ok(out)
}

fn aad_for(tenant_id: &str, key_version: u32) -> Vec<u8> {
    let mut aad = Vec::with_capacity(tenant_id.len() + 32);
    aad.extend_from_slice(tenant_id.as_bytes());
    aad.push(0);
    aad.extend_from_slice(&key_version.to_be_bytes());
    aad.push(0);
    aad.extend_from_slice(PURPOSE);
    aad
}

fn seal_with_key(key: &[u8; 32], plaintext: &[u8], magic: &[u8]) -> HelixResult<Vec<u8>> {
    let mut body = seal_with_raw_key(key, plaintext)?;
    let mut out = Vec::with_capacity(magic.len() + body.len());
    out.extend_from_slice(magic);
    out.append(&mut body);
    Ok(out)
}

fn open_with_key(key: &[u8; 32], payload: &[u8], magic: &[u8]) -> HelixResult<Vec<u8>> {
    if !payload.starts_with(magic) || payload.len() <= magic.len() + NONCE_LEN {
        return Err(HelixError::internal("invalid vault envelope"));
    }
    open_with_raw_key(key, &payload[magic.len()..])
}

/// Raw AES-GCM seal without magic (used for KMS wrap of DEKs).
/// Output = nonce(12) || ciphertext+tag.
pub fn seal_with_raw_key(key: &[u8; 32], plaintext: &[u8]) -> HelixResult<Vec<u8>> {
    seal_with_raw_key_aad(key, plaintext, b"")
}

/// AES-GCM seal with AAD binding (tenant / version / purpose).
pub fn seal_with_raw_key_aad(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> HelixResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| HelixError::internal(format!("aes key: {e}")))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom(&mut nonce_bytes).map_err(|e| HelixError::internal(format!("nonce: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|e| HelixError::internal(format!("aes encrypt: {e}")))?;
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Raw AES-GCM open (nonce || ct).
pub fn open_with_raw_key(key: &[u8; 32], payload: &[u8]) -> HelixResult<Vec<u8>> {
    open_with_raw_key_aad(key, payload, b"")
}

pub fn open_with_raw_key_aad(key: &[u8; 32], payload: &[u8], aad: &[u8]) -> HelixResult<Vec<u8>> {
    if payload.len() <= NONCE_LEN {
        return Err(HelixError::internal("ciphertext too short"));
    }
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| HelixError::internal(format!("aes key: {e}")))?;
    let nonce = Nonce::from_slice(&payload[..NONCE_LEN]);
    let ct = &payload[NONCE_LEN..];
    cipher
        .decrypt(nonce, Payload { msg: ct, aad })
        .map_err(|_| HelixError::internal("aes decrypt failed"))
}

/// Seal with random DEK, wrap DEK via KMS, store HVA3 envelope.
/// Format: HVA3 || u16_be(wrap_len) || wrapped_dek || nonce||ct (of plaintext under DEK)
pub async fn seal_kms(
    kms: &dyn crate::kms::KeyManagement,
    plaintext: &[u8],
) -> HelixResult<Vec<u8>> {
    let mut dek = [0u8; 32];
    getrandom(&mut dek).map_err(|e| HelixError::internal(format!("dek rng: {e}")))?;
    let body = seal_with_raw_key(&dek, plaintext)?;
    let wrapped = kms.wrap_dek(&dek).await?;
    if wrapped.len() > u16::MAX as usize {
        return Err(HelixError::internal("wrapped DEK too large"));
    }
    let wrap_len = (wrapped.len() as u16).to_be_bytes();
    let mut out = Vec::with_capacity(MAGIC_V3.len() + 2 + wrapped.len() + body.len());
    out.extend_from_slice(MAGIC_V3);
    out.extend_from_slice(&wrap_len);
    out.extend_from_slice(&wrapped);
    out.extend_from_slice(&body);
    dek.fill(0);
    Ok(out)
}

/// Open HVA3 (KMS-wrapped DEK) or fall back to HVA4/HVA2/HVA1/XOR via [`open_tenant`].
pub async fn open_kms(
    kms: &dyn crate::kms::KeyManagement,
    master_key: &[u8],
    tenant_id: &str,
    payload: &[u8],
) -> HelixResult<Vec<u8>> {
    if payload.starts_with(MAGIC_V3) && payload.len() > MAGIC_V3.len() + 2 {
        let wrap_len =
            u16::from_be_bytes([payload[MAGIC_V3.len()], payload[MAGIC_V3.len() + 1]]) as usize;
        let start = MAGIC_V3.len() + 2;
        let end = start + wrap_len;
        if end > payload.len() {
            return Err(HelixError::internal("HVA3 wrap length out of bounds"));
        }
        let wrapped = &payload[start..end];
        let body = &payload[end..];
        let dek = kms.unwrap_dek(wrapped).await?;
        return open_with_raw_key(&dek, body);
    }
    open_tenant(master_key, tenant_id, payload)
}

/// Encrypt with legacy **Argon2id tenant DEK + AAD** (HVA4, unversioned).
/// Kept for backwards compatibility; new code should prefer [`seal_tenant_with_version`].
pub fn seal_tenant(master_key: &[u8], tenant_id: &str, plaintext: &[u8]) -> HelixResult<Vec<u8>> {
    let key = derive_tenant_key_argon2(master_key, tenant_id)?;
    let aad = aad_for(tenant_id, DEFAULT_KEY_VERSION);
    let body = seal_with_raw_key_aad(&key, plaintext, &aad)?;
    let mut out = Vec::with_capacity(MAGIC_V4.len() + body.len());
    out.extend_from_slice(MAGIC_V4);
    out.extend_from_slice(&body);
    Ok(out)
}

/// Encrypt with **Argon2id tenant DEK + AAD** and an embedded key version (HVA5).
pub fn seal_tenant_with_version(
    master_key: &[u8],
    tenant_id: &str,
    plaintext: &[u8],
    key_version: u32,
) -> HelixResult<Vec<u8>> {
    let key = derive_tenant_key_argon2(master_key, tenant_id)?;
    let aad = aad_for(tenant_id, key_version);
    let body = seal_with_raw_key_aad(&key, plaintext, &aad)?;
    let mut out = Vec::with_capacity(MAGIC_V5.len() + 4 + body.len());
    out.extend_from_slice(MAGIC_V5);
    out.extend_from_slice(&key_version.to_be_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// Open a tenant envelope (HVA5 versioned, HVA4 legacy, HVA2, HVA1, or XOR).
/// Returns the plaintext and the key version that was used to seal it.
pub fn open_tenant(master_key: &[u8], tenant_id: &str, payload: &[u8]) -> HelixResult<Vec<u8>> {
    open_tenant_with_version(master_key, tenant_id, payload).map(|(plain, _)| plain)
}

/// Open a tenant envelope and return the key version.
pub fn open_tenant_with_version(
    master_key: &[u8],
    tenant_id: &str,
    payload: &[u8],
) -> HelixResult<(Vec<u8>, u32)> {
    if payload.starts_with(MAGIC_V5) {
        if payload.len() <= MAGIC_V5.len() + 4 + NONCE_LEN {
            return Err(HelixError::internal("invalid HVA5 envelope"));
        }
        let key_version = u32::from_be_bytes([
            payload[MAGIC_V5.len()],
            payload[MAGIC_V5.len() + 1],
            payload[MAGIC_V5.len() + 2],
            payload[MAGIC_V5.len() + 3],
        ]);
        let key = derive_tenant_key_argon2(master_key, tenant_id)?;
        let aad = aad_for(tenant_id, key_version);
        let plain = open_with_raw_key_aad(&key, &payload[MAGIC_V5.len() + 4..], &aad)?;
        return Ok((plain, key_version));
    }
    if payload.starts_with(MAGIC_V4) {
        if payload.len() <= MAGIC_V4.len() + NONCE_LEN {
            return Err(HelixError::internal("invalid HVA4 envelope"));
        }
        let key = derive_tenant_key_argon2(master_key, tenant_id)?;
        let aad = aad_for(tenant_id, DEFAULT_KEY_VERSION);
        let plain = open_with_raw_key_aad(&key, &payload[MAGIC_V4.len()..], &aad)?;
        return Ok((plain, DEFAULT_KEY_VERSION));
    }
    if payload.starts_with(MAGIC_V2) {
        let key = derive_tenant_key(master_key, tenant_id);
        let plain = open_with_key(&key, payload, MAGIC_V2)?;
        return Ok((plain, DEFAULT_KEY_VERSION));
    }
    let plain = open(master_key, payload)?;
    Ok((plain, DEFAULT_KEY_VERSION))
}

/// KMS-aware seal with Argon2id tenant DEK + embedded key version (HVA5).
/// This is the preferred path for production code so object encryption also
/// participates in the configured `KeyManagement` provider.
pub async fn seal_tenant_kms(
    kms: &dyn crate::kms::KeyManagement,
    tenant_id: &str,
    plaintext: &[u8],
    key_version: u32,
) -> HelixResult<Vec<u8>> {
    let key = kms.derive_tenant_dek(tenant_id).await?;
    let aad = aad_for(tenant_id, key_version);
    let body = seal_with_raw_key_aad(&key, plaintext, &aad)?;
    let mut out = Vec::with_capacity(MAGIC_V5.len() + 4 + body.len());
    out.extend_from_slice(MAGIC_V5);
    out.extend_from_slice(&key_version.to_be_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// KMS-aware open of HVA5/HVA4/HVA2/HVA1/XOR envelopes. Returns plaintext and key version.
pub async fn open_tenant_kms(
    kms: &dyn crate::kms::KeyManagement,
    master_key: &[u8],
    tenant_id: &str,
    payload: &[u8],
) -> HelixResult<(Vec<u8>, u32)> {
    if payload.starts_with(MAGIC_V5) {
        if payload.len() <= MAGIC_V5.len() + 4 + NONCE_LEN {
            return Err(HelixError::internal("invalid HVA5 envelope"));
        }
        let key_version = u32::from_be_bytes([
            payload[MAGIC_V5.len()],
            payload[MAGIC_V5.len() + 1],
            payload[MAGIC_V5.len() + 2],
            payload[MAGIC_V5.len() + 3],
        ]);
        let key = kms.derive_tenant_dek(tenant_id).await?;
        let aad = aad_for(tenant_id, key_version);
        let plain = open_with_raw_key_aad(&key, &payload[MAGIC_V5.len() + 4..], &aad)?;
        return Ok((plain, key_version));
    }
    // For legacy envelopes the master key path is sufficient.
    open_tenant_with_version(master_key, tenant_id, payload)
}

/// Encrypt plaintext with global key (HVA1). Prefer [`seal_tenant`] for new data.
pub fn seal(master_key: &[u8], plaintext: &[u8]) -> HelixResult<Vec<u8>> {
    let key = derive_key(master_key);
    seal_with_key(&key, plaintext, MAGIC_V1)
}

/// Decrypt payload produced by [`seal`] / [`seal_tenant`]. Also accepts legacy XOR.
pub fn open(master_key: &[u8], payload: &[u8]) -> HelixResult<Vec<u8>> {
    if payload.starts_with(MAGIC_V4) || payload.starts_with(MAGIC_V2) {
        return Err(HelixError::internal(
            "tenant envelope requires open_tenant with tenant_id",
        ));
    }
    if payload.starts_with(MAGIC_V1) && payload.len() > MAGIC_V1.len() + NONCE_LEN {
        let key = derive_key(master_key);
        return open_with_key(&key, payload, MAGIC_V1);
    }
    // Legacy XOR demo envelope (pre AES) for reading old rows.
    let mut hasher = Sha256::new();
    hasher.update(master_key);
    let master = hasher.finalize();
    Ok(payload
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ master[i % master.len()])
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_aes_global() {
        let pt = b"super-secret-value";
        let sealed = seal(b"master-key", pt).unwrap();
        assert!(sealed.starts_with(MAGIC_V1));
        let opened = open(b"master-key", &sealed).unwrap();
        assert_eq!(opened, pt);
    }

    #[test]
    fn roundtrip_tenant_dek_hva4() {
        let pt = b"tenant-secret";
        let sealed = seal_tenant(b"master-key", "ten:aaa", pt).unwrap();
        assert!(sealed.starts_with(MAGIC_V4));
        let opened = open_tenant(b"master-key", "ten:aaa", &sealed).unwrap();
        assert_eq!(opened, pt);
        // wrong tenant must fail (AAD + different DEK)
        assert!(open_tenant(b"master-key", "ten:bbb", &sealed).is_err());
    }

    #[test]
    fn roundtrip_tenant_dek_hva5_versioned() {
        let pt = b"tenant-secret-versioned";
        let sealed = seal_tenant_with_version(b"master-key", "ten:aaa", pt, 7).unwrap();
        assert!(sealed.starts_with(MAGIC_V5));
        let (opened, version) =
            open_tenant_with_version(b"master-key", "ten:aaa", &sealed).unwrap();
        assert_eq!(opened, pt);
        assert_eq!(version, 7);
        // wrong tenant must fail
        assert!(open_tenant_with_version(b"master-key", "ten:bbb", &sealed).is_err());
    }

    #[test]
    fn hva5_aad_binds_key_version() {
        let pt = b"tenant-secret";
        let sealed_v1 = seal_tenant_with_version(b"master-key", "ten:aaa", pt, 1).unwrap();
        let sealed_v2 = seal_tenant_with_version(b"master-key", "ten:aaa", pt, 2).unwrap();
        // Same plaintext but different AAD => different ciphertext (with overwhelming probability).
        assert_ne!(sealed_v1, sealed_v2);
    }

    #[test]
    fn reads_legacy_hva2() {
        let pt = b"legacy";
        let key = derive_tenant_key(b"master-key", "ten:aaa");
        let sealed = seal_with_key(&key, pt, MAGIC_V2).unwrap();
        let opened = open_tenant(b"master-key", "ten:aaa", &sealed).unwrap();
        assert_eq!(opened, pt);
    }

    #[test]
    fn tenant_keys_differ() {
        let a = derive_tenant_key_argon2(b"m", "t1").unwrap();
        let b = derive_tenant_key_argon2(b"m", "t2").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn argon2_stable() {
        let a = derive_tenant_key_argon2(b"m", "t1").unwrap();
        let b = derive_tenant_key_argon2(b"m", "t1").unwrap();
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn hva3_kms_roundtrip() {
        let kms = crate::kms::LocalSoftwareKms::new(b"master");
        let sealed = seal_kms(&kms, b"hva3-secret").await.unwrap();
        assert!(sealed.starts_with(MAGIC_V3));
        let opened = open_kms(&kms, b"master", "ten:x", &sealed).await.unwrap();
        assert_eq!(opened, b"hva3-secret");
    }

    #[test]
    fn hva4_to_hva5_read_backwards_compatible() {
        let pt = b"legacy-hva4";
        let sealed = seal_tenant(b"master-key", "ten:aaa", pt).unwrap();
        // New reader returns version 1 for legacy HVA4.
        let (opened, version) =
            open_tenant_with_version(b"master-key", "ten:aaa", &sealed).unwrap();
        assert_eq!(opened, pt);
        assert_eq!(version, 1);
    }

    #[tokio::test]
    async fn hva5_kms_roundtrip() {
        let kms = crate::kms::LocalSoftwareKms::new(b"master");
        let sealed = seal_tenant_kms(&kms, "ten:x", b"hva5-secret", 3)
            .await
            .unwrap();
        assert!(sealed.starts_with(MAGIC_V5));
        let (opened, version) = open_tenant_kms(&kms, b"master", "ten:x", &sealed)
            .await
            .unwrap();
        assert_eq!(opened, b"hva5-secret");
        assert_eq!(version, 3);
    }
}
