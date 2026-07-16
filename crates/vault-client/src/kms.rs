//! Pluggable key-management for vault DEK wrapping (software KEK or remote HTTP KMS).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared_core::{HelixError, HelixResult};
use std::sync::Arc;

use crate::crypto::{derive_tenant_key_argon2, open_with_raw_key, seal_with_raw_key};

/// How DEKs are wrapped at rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KmsMode {
    /// Local software KEK derived from master (default).
    Local,
    /// Remote HTTP KMS: POST /v1/wrap and /v1/unwrap.
    Http,
}

impl KmsMode {
    pub fn from_env() -> Self {
        Self::parse(&std::env::var("HELIX_VAULT_KMS_MODE").unwrap_or_else(|_| "local".into()))
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "http" | "remote" | "hsm" => Self::Http,
            _ => Self::Local,
        }
    }
}

#[async_trait]
pub trait KeyManagement: Send + Sync {
    fn mode(&self) -> KmsMode;
    /// Wrap a raw 32-byte DEK → opaque blob for storage.
    async fn wrap_dek(&self, dek: &[u8; 32]) -> HelixResult<Vec<u8>>;
    /// Unwrap storage blob → raw DEK.
    async fn unwrap_dek(&self, wrapped: &[u8]) -> HelixResult<[u8; 32]>;
    /// Derive the per-tenant DEK used for HVA4/HVA5 envelopes.
    async fn derive_tenant_dek(&self, tenant_id: &str) -> HelixResult<[u8; 32]>;
}

/// Software KEK: AES-GCM wrap of DEK under SHA-256(master || "kek-v1").
pub struct LocalSoftwareKms {
    kek: [u8; 32],
    master: Vec<u8>,
}

impl LocalSoftwareKms {
    pub fn new(master_key: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(master_key);
        hasher.update(b"helixforge-vault-kek-v1");
        let dig = hasher.finalize();
        let mut kek = [0u8; 32];
        kek.copy_from_slice(&dig);
        Self {
            kek,
            master: master_key.to_vec(),
        }
    }

    pub fn from_explicit_kek(kek_material: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(kek_material);
        hasher.update(b"helixforge-vault-explicit-kek-v1");
        let dig = hasher.finalize();
        let mut kek = [0u8; 32];
        kek.copy_from_slice(&dig);
        Self {
            kek,
            master: kek_material.to_vec(),
        }
    }
}

#[async_trait]
impl KeyManagement for LocalSoftwareKms {
    fn mode(&self) -> KmsMode {
        KmsMode::Local
    }

    async fn wrap_dek(&self, dek: &[u8; 32]) -> HelixResult<Vec<u8>> {
        seal_with_raw_key(&self.kek, dek)
    }

    async fn unwrap_dek(&self, wrapped: &[u8]) -> HelixResult<[u8; 32]> {
        let plain = open_with_raw_key(&self.kek, wrapped)?;
        if plain.len() != 32 {
            return Err(HelixError::internal("unwrapped DEK length != 32"));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&plain);
        Ok(out)
    }

    async fn derive_tenant_dek(&self, tenant_id: &str) -> HelixResult<[u8; 32]> {
        derive_tenant_key_argon2(&self.master, tenant_id)
    }
}

/// Remote KMS over HTTP (HSM appliance, cloud KMS proxy, or self-hosted wrapper).
pub struct HttpKms {
    base_url: String,
    http: reqwest::Client,
    /// Fallback when remote is down (dev only).
    local: LocalSoftwareKms,
    allow_fallback: bool,
    master: Vec<u8>,
}

impl HttpKms {
    pub fn new(base_url: impl Into<String>, master_key: &[u8], allow_fallback: bool) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("kms http"),
            local: LocalSoftwareKms::new(master_key),
            allow_fallback,
            master: master_key.to_vec(),
        }
    }
}

#[derive(Serialize)]
struct WrapReq<'a> {
    plaintext_b64: &'a str,
}

#[derive(Deserialize)]
struct WrapResp {
    wrapped_b64: String,
}

#[derive(Serialize)]
struct UnwrapReq<'a> {
    wrapped_b64: &'a str,
}

#[derive(Deserialize)]
struct UnwrapResp {
    plaintext_b64: String,
}

#[async_trait]
impl KeyManagement for HttpKms {
    fn mode(&self) -> KmsMode {
        KmsMode::Http
    }

    async fn derive_tenant_dek(&self, tenant_id: &str) -> HelixResult<[u8; 32]> {
        // TODO: when the remote KMS exposes `/v1/derive`, call it and fall back only if allowed.
        // Until then, local Argon2id derivation from the configured master key material is used
        // so HVA4/HVA5 envelopes remain available regardless of HTTP KMS reachability.
        derive_tenant_key_argon2(&self.master, tenant_id)
    }

    async fn wrap_dek(&self, dek: &[u8; 32]) -> HelixResult<Vec<u8>> {
        let url = format!("{}/v1/wrap", self.base_url.trim_end_matches('/'));
        let plaintext_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, dek);
        match self
            .http
            .post(&url)
            .json(&WrapReq {
                plaintext_b64: &plaintext_b64,
            })
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let body: WrapResp = resp
                    .json()
                    .await
                    .map_err(|e| HelixError::dependency(format!("kms wrap decode: {e}")))?;
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, body.wrapped_b64)
                    .map_err(|e| HelixError::dependency(format!("kms wrap b64: {e}")))
            }
            Ok(resp) if self.allow_fallback => {
                tracing::warn!(status = %resp.status(), "kms wrap failed — local KEK fallback");
                self.local.wrap_dek(dek).await
            }
            Ok(resp) => Err(HelixError::dependency(format!(
                "kms wrap status {}",
                resp.status()
            ))),
            Err(e) if self.allow_fallback => {
                tracing::warn!(error = %e, "kms wrap error — local KEK fallback");
                self.local.wrap_dek(dek).await
            }
            Err(e) => Err(HelixError::dependency(format!("kms wrap: {e}"))),
        }
    }

    async fn unwrap_dek(&self, wrapped: &[u8]) -> HelixResult<[u8; 32]> {
        let url = format!("{}/v1/unwrap", self.base_url.trim_end_matches('/'));
        let wrapped_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, wrapped);
        match self
            .http
            .post(&url)
            .json(&UnwrapReq {
                wrapped_b64: &wrapped_b64,
            })
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let body: UnwrapResp = resp
                    .json()
                    .await
                    .map_err(|e| HelixError::dependency(format!("kms unwrap decode: {e}")))?;
                let plain = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    body.plaintext_b64,
                )
                .map_err(|e| HelixError::dependency(format!("kms unwrap b64: {e}")))?;
                if plain.len() != 32 {
                    return Err(HelixError::internal("remote unwrapped DEK length != 32"));
                }
                let mut out = [0u8; 32];
                out.copy_from_slice(&plain);
                Ok(out)
            }
            Ok(resp) if self.allow_fallback => {
                tracing::warn!(status = %resp.status(), "kms unwrap failed — local KEK fallback");
                self.local.unwrap_dek(wrapped).await
            }
            Ok(resp) => Err(HelixError::dependency(format!(
                "kms unwrap status {}",
                resp.status()
            ))),
            Err(e) if self.allow_fallback => {
                tracing::warn!(error = %e, "kms unwrap error — local KEK fallback");
                self.local.unwrap_dek(wrapped).await
            }
            Err(e) => Err(HelixError::dependency(format!("kms unwrap: {e}"))),
        }
    }
}

pub fn build_kms(
    master_key: &[u8],
    cfg: &shared_core::config::CoreConfig,
) -> Arc<dyn KeyManagement> {
    let mode = KmsMode::parse(&cfg.kms_mode);
    match mode {
        KmsMode::Http => {
            // Fail closed by default. Explicit opt-in only in local (Kimi P1).
            let allow_fallback = cfg.environment == "local" && cfg.kms_fallback;
            Arc::new(HttpKms::new(&cfg.kms_url, master_key, allow_fallback))
        }
        KmsMode::Local => Arc::new(LocalSoftwareKms::new(master_key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_wrap_roundtrip() {
        let kms = LocalSoftwareKms::new(b"master");
        let dek = [7u8; 32];
        let wrapped = kms.wrap_dek(&dek).await.unwrap();
        let opened = kms.unwrap_dek(&wrapped).await.unwrap();
        assert_eq!(opened, dek);
    }

    #[tokio::test]
    async fn local_derive_tenant_dek() {
        let kms = LocalSoftwareKms::new(b"master");
        let a = kms.derive_tenant_dek("tenant-a").await.unwrap();
        let b = kms.derive_tenant_dek("tenant-b").await.unwrap();
        let a2 = kms.derive_tenant_dek("tenant-a").await.unwrap();
        assert_ne!(a, b);
        assert_eq!(a, a2);
    }
}
