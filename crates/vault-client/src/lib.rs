//! Vault client — AES-256-GCM envelope-encrypted secret storage.
//!
//! Memory backend for offline boot; Postgres backend via `helix_db::PgVault`.
//! New secrets use **per-tenant DEK** (HVA2). HVA1 + XOR remain readable.

mod crypto;
mod kms;
mod minio;

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use std::collections::HashMap;
use std::sync::Arc;
use zeroize::Zeroizing;

pub use crypto::{
    open as vault_open, open_kms as vault_open_kms, open_tenant as vault_open_tenant,
    open_tenant_kms as vault_open_tenant_kms,
    open_tenant_with_version as vault_open_tenant_version, open_with_raw_key as vault_open_raw,
    seal as vault_seal, seal_kms as vault_seal_kms, seal_tenant as vault_seal_tenant,
    seal_tenant_kms as vault_seal_tenant_kms,
    seal_tenant_with_version as vault_seal_tenant_version, seal_with_raw_key as vault_seal_raw,
};
pub use kms::{build_kms, KeyManagement, KmsMode, LocalSoftwareKms};
pub use minio::{ObjectStore, ObjectStoreConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRef {
    pub tenant_id: TenantId,
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMeta {
    pub name: String,
    pub version: u32,
    pub created_at: String,
}

#[async_trait]
pub trait Vault: Send + Sync {
    async fn put(&self, tenant_id: TenantId, name: &str, value: &[u8]) -> HelixResult<SecretRef>;
    async fn get(&self, tenant_id: TenantId, name: &str) -> HelixResult<Zeroizing<Vec<u8>>>;
    async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<SecretMeta>>;
    async fn delete(&self, tenant_id: TenantId, name: &str) -> HelixResult<()>;
}

#[derive(Clone)]
pub struct VaultClient {
    inner: Arc<dyn Vault>,
}

impl VaultClient {
    pub fn new(inner: Arc<dyn Vault>) -> Self {
        Self { inner }
    }

    pub fn memory(master_key: &[u8]) -> Self {
        Self::new(Arc::new(MemoryVault::new(master_key)))
    }

    pub fn remote(base_url: impl Into<String>) -> Self {
        Self::new(Arc::new(HttpVault::new(base_url.into())))
    }

    pub async fn put(
        &self,
        tenant_id: TenantId,
        name: &str,
        value: &[u8],
    ) -> HelixResult<SecretRef> {
        self.inner.put(tenant_id, name, value).await
    }

    pub async fn get(&self, tenant_id: TenantId, name: &str) -> HelixResult<Zeroizing<Vec<u8>>> {
        self.inner.get(tenant_id, name).await
    }

    pub async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<SecretMeta>> {
        self.inner.list(tenant_id).await
    }

    pub async fn delete(&self, tenant_id: TenantId, name: &str) -> HelixResult<()> {
        self.inner.delete(tenant_id, name).await
    }
}

type MemoryVaultStore = HashMap<(String, String), (u32, Vec<u8>)>;

struct MemoryVault {
    master: Vec<u8>,
    kms: Arc<dyn KeyManagement>,
    store: RwLock<MemoryVaultStore>,
}

impl MemoryVault {
    fn new(master_key: &[u8]) -> Self {
        Self {
            master: master_key.to_vec(),
            kms: Arc::new(LocalSoftwareKms::new(master_key)),
            store: RwLock::new(HashMap::new()),
        }
    }

    fn with_kms(master_key: &[u8], kms: Arc<dyn KeyManagement>) -> Self {
        Self {
            master: master_key.to_vec(),
            kms,
            store: RwLock::new(HashMap::new()),
        }
    }
}

impl VaultClient {
    pub fn memory_with_kms(master_key: &[u8], kms: Arc<dyn KeyManagement>) -> Self {
        Self::new(Arc::new(MemoryVault::with_kms(master_key, kms)))
    }
}

#[async_trait]
impl Vault for MemoryVault {
    async fn put(&self, tenant_id: TenantId, name: &str, value: &[u8]) -> HelixResult<SecretRef> {
        if name.is_empty() || name.len() > 128 {
            return Err(HelixError::validation("secret name length 1..=128"));
        }
        let tid = tenant_id.to_string();
        let key = (tid.clone(), name.to_string());
        let sealed = crypto::seal_tenant_kms(self.kms.as_ref(), &tid, value, 1).await?;
        let mut guard = self.store.write();
        let version = guard.get(&key).map(|(v, _)| v + 1).unwrap_or(1);
        guard.insert(key, (version, sealed));
        Ok(SecretRef {
            tenant_id,
            name: name.into(),
            version,
        })
    }

    async fn get(&self, tenant_id: TenantId, name: &str) -> HelixResult<Zeroizing<Vec<u8>>> {
        let tid = tenant_id.to_string();
        let key = (tid.clone(), name.to_string());
        let sealed = {
            let guard = self.store.read();
            guard
                .get(&key)
                .map(|(_, s)| s.clone())
                .ok_or_else(|| HelixError::not_found(format!("secret {name}")))?
        };
        let (plain, _key_version) =
            crypto::open_tenant_kms(self.kms.as_ref(), &self.master, &tid, &sealed).await?;
        Ok(Zeroizing::new(plain))
    }

    async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<SecretMeta>> {
        let tid = tenant_id.to_string();
        let guard = self.store.read();
        Ok(guard
            .iter()
            .filter(|((t, _), _)| t == &tid)
            .map(|((_, name), (version, _))| SecretMeta {
                name: name.clone(),
                version: *version,
                created_at: chrono_lite_now(),
            })
            .collect())
    }

    async fn delete(&self, tenant_id: TenantId, name: &str) -> HelixResult<()> {
        let key = (tenant_id.to_string(), name.to_string());
        let mut guard = self.store.write();
        guard
            .remove(&key)
            .ok_or_else(|| HelixError::not_found(format!("secret {name}")))?;
        Ok(())
    }
}

fn chrono_lite_now() -> String {
    // Avoid chrono dep here — ISO-ish timestamp via system time is fine for meta.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

struct HttpVault {
    base_url: String,
    http: reqwest::Client,
}

impl HttpVault {
    fn new(base_url: String) -> Self {
        Self {
            base_url,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct PutBody<'a> {
    name: &'a str,
    value_b64: String,
}

#[async_trait]
impl Vault for HttpVault {
    async fn put(&self, tenant_id: TenantId, name: &str, value: &[u8]) -> HelixResult<SecretRef> {
        let url = format!(
            "{}/v1/tenants/{tenant_id}/secrets",
            self.base_url.trim_end_matches('/')
        );
        let body = PutBody {
            name,
            value_b64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, value),
        };
        let resp = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("vault put: {e}")))?;
        if !resp.status().is_success() {
            return Err(HelixError::dependency(format!(
                "vault put status {}",
                resp.status()
            )));
        }
        resp.json()
            .await
            .map_err(|e| HelixError::dependency(format!("vault put decode: {e}")))
    }

    async fn get(&self, tenant_id: TenantId, name: &str) -> HelixResult<Zeroizing<Vec<u8>>> {
        let url = format!(
            "{}/v1/tenants/{tenant_id}/secrets/{name}",
            self.base_url.trim_end_matches('/')
        );
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("vault get: {e}")))?;
        if resp.status().as_u16() == 404 {
            return Err(HelixError::not_found(format!("secret {name}")));
        }
        #[derive(Deserialize)]
        struct GetResp {
            value_b64: String,
        }
        let body: GetResp = resp
            .json()
            .await
            .map_err(|e| HelixError::dependency(format!("vault get decode: {e}")))?;
        let bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, body.value_b64)
                .map_err(|e| HelixError::internal(format!("b64: {e}")))?;
        Ok(Zeroizing::new(bytes))
    }

    async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<SecretMeta>> {
        let url = format!(
            "{}/v1/tenants/{tenant_id}/secrets",
            self.base_url.trim_end_matches('/')
        );
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("vault list: {e}")))?;
        resp.json()
            .await
            .map_err(|e| HelixError::dependency(format!("vault list decode: {e}")))
    }

    async fn delete(&self, tenant_id: TenantId, name: &str) -> HelixResult<()> {
        let url = format!(
            "{}/v1/tenants/{tenant_id}/secrets/{name}",
            self.base_url.trim_end_matches('/')
        );
        let resp = self
            .http
            .delete(url)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("vault delete: {e}")))?;
        if resp.status().as_u16() == 404 {
            return Err(HelixError::not_found(format!("secret {name}")));
        }
        if !resp.status().is_success() {
            return Err(HelixError::dependency(format!(
                "vault delete status {}",
                resp.status()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_put_get_roundtrip() {
        let vault = VaultClient::memory(b"test-master-key");
        let tid = TenantId::new();
        vault.put(tid, "api-key", b"super-secret").await.unwrap();
        let got = vault.get(tid, "api-key").await.unwrap();
        assert_eq!(&got[..], b"super-secret");
    }

    #[tokio::test]
    async fn tenant_isolation_on_dek() {
        let vault = VaultClient::memory(b"test-master-key");
        let a = TenantId::new();
        let b = TenantId::new();
        vault.put(a, "k", b"secret-a").await.unwrap();
        vault.put(b, "k", b"secret-b").await.unwrap();
        assert_eq!(&vault.get(a, "k").await.unwrap()[..], b"secret-a");
        assert_eq!(&vault.get(b, "k").await.unwrap()[..], b"secret-b");
    }
}
