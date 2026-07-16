//! Postgres-backed vault using AES-256-GCM + Argon2id tenant DEK (HVA4/HVA5).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use std::sync::Arc;
use vault_client::{
    vault_open_tenant_kms, vault_seal_tenant_kms, KeyManagement, LocalSoftwareKms, SecretMeta,
    SecretRef, Vault,
};
use zeroize::Zeroizing;

const DEFAULT_KEY_VERSION: u32 = 1;

fn lazy_reencrypt_enabled() -> bool {
    std::env::var("HELIX_VAULT_LAZY_REENCRYPT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true)
}

#[derive(Clone)]
pub struct PgVault {
    pool: PgPool,
    master: Vec<u8>,
    kms: Arc<dyn KeyManagement>,
}

impl PgVault {
    pub fn new(pool: PgPool, master_key: &[u8]) -> Self {
        Self {
            pool,
            master: master_key.to_vec(),
            kms: Arc::new(LocalSoftwareKms::new(master_key)),
        }
    }

    pub fn with_kms(pool: PgPool, master_key: &[u8], kms: Arc<dyn KeyManagement>) -> Self {
        Self {
            pool,
            master: master_key.to_vec(),
            kms,
        }
    }
}

#[async_trait]
impl Vault for PgVault {
    async fn put(&self, tenant_id: TenantId, name: &str, value: &[u8]) -> HelixResult<SecretRef> {
        if name.is_empty() || name.len() > 128 {
            return Err(HelixError::validation("secret name length 1..=128"));
        }
        let key_version = self.current_key_version().await?;
        let sealed = vault_seal_tenant_kms(
            self.kms.as_ref(),
            &tenant_id.to_string(),
            value,
            key_version,
        )
        .await?;
        let now = Utc::now();

        #[derive(sqlx::FromRow)]
        struct Row {
            version: i32,
        }
        let existing: Option<Row> = sqlx::query_as(
            "SELECT version FROM helix_core.secrets WHERE tenant_id = $1 AND name = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vault get version: {e}")))?;

        let version = existing.map(|r| r.version + 1).unwrap_or(1);

        sqlx::query(
            r#"
            INSERT INTO helix_core.secrets (tenant_id, name, version, key_version, ciphertext, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$6)
            ON CONFLICT (tenant_id, name) DO UPDATE SET
                version = EXCLUDED.version,
                key_version = EXCLUDED.key_version,
                ciphertext = EXCLUDED.ciphertext,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(version)
        .bind(key_version as i32)
        .bind(&sealed)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vault put: {e}")))?;

        Ok(SecretRef {
            tenant_id,
            name: name.into(),
            version: version as u32,
        })
    }

    async fn get(&self, tenant_id: TenantId, name: &str) -> HelixResult<Zeroizing<Vec<u8>>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            ciphertext: Vec<u8>,
            key_version: i32,
        }
        let row: Option<Row> = sqlx::query_as(
            "SELECT ciphertext, key_version FROM helix_core.secrets WHERE tenant_id = $1 AND name = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vault get: {e}")))?;

        let row = row.ok_or_else(|| HelixError::not_found(format!("secret {name}")))?;
        let tid = tenant_id.to_string();
        let (plain, envelope_version) =
            vault_open_tenant_kms(self.kms.as_ref(), &self.master, &tid, &row.ciphertext).await?;

        // Lazy re-encryption: if the stored key_version column or the envelope version
        // lags the current meta version, re-seal under the latest version in the same transaction.
        if lazy_reencrypt_enabled() {
            let current_version = self.current_key_version().await?;
            let stored_version = row.key_version.max(envelope_version as i32) as u32;
            if stored_version < current_version {
                tracing::info!(
                    tenant_id = %tid,
                    secret_name = %name,
                    stored_version,
                    current_version,
                    "lazy re-encrypting secret"
                );
                let resealed =
                    vault_seal_tenant_kms(self.kms.as_ref(), &tid, &plain, current_version).await?;
                if let Err(e) = sqlx::query(
                    "UPDATE helix_core.secrets SET ciphertext = $1, key_version = $2, updated_at = now() WHERE tenant_id = $3 AND name = $4",
                )
                .bind(&resealed)
                .bind(current_version as i32)
                .bind(tenant_id.as_uuid())
                .bind(name)
                .execute(&self.pool)
                .await
                {
                    tracing::warn!(error = %e, "lazy re-encrypt update failed; returning plaintext anyway");
                }
            }
        }

        Ok(Zeroizing::new(plain))
    }

    async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<SecretMeta>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            name: String,
            version: i32,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT name, version, created_at
            FROM helix_core.secrets
            WHERE tenant_id = $1
            ORDER BY name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vault list: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| SecretMeta {
                name: r.name,
                version: r.version as u32,
                created_at: r.created_at.to_rfc3339(),
            })
            .collect())
    }

    async fn delete(&self, tenant_id: TenantId, name: &str) -> HelixResult<()> {
        let res = sqlx::query("DELETE FROM helix_core.secrets WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id.as_uuid())
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("vault delete: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("secret {name}")));
        }
        Ok(())
    }
}

impl PgVault {
    /// Current key version from the rotation ledger. Defaults to 1 if no row exists.
    async fn current_key_version(&self) -> HelixResult<u32> {
        let row: Option<(i32,)> =
            sqlx::query_as("SELECT version FROM helix_core.vault_key_meta WHERE id = 'default'")
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("vault key meta: {e}")))?;
        Ok(row.map(|(v,)| v as u32).unwrap_or(DEFAULT_KEY_VERSION))
    }

    /// Re-encrypt all secrets under the current key version (bulk rotation).
    /// Opens each ciphertext and re-seals with HVA5 and the current `vault_key_meta.version`.
    pub async fn reencrypt_all(&self) -> HelixResult<u64> {
        let current_version = self.current_key_version().await?;
        #[derive(sqlx::FromRow)]
        struct Row {
            tenant_id: uuid::Uuid,
            name: String,
            ciphertext: Vec<u8>,
            version: i32,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT tenant_id, name, ciphertext, version FROM helix_core.secrets ORDER BY tenant_id, name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("vault reencrypt list: {e}")))?;

        let mut n = 0u64;
        for row in rows {
            let tid = TenantId::from_uuid(row.tenant_id);
            let (plain, _envelope_version) = vault_open_tenant_kms(
                self.kms.as_ref(),
                &self.master,
                &tid.to_string(),
                &row.ciphertext,
            )
            .await?;
            let sealed =
                vault_seal_tenant_kms(self.kms.as_ref(), &tid.to_string(), &plain, current_version)
                    .await?;
            sqlx::query(
                r#"
                UPDATE helix_core.secrets
                SET ciphertext = $1, version = $2, key_version = $3, updated_at = now()
                WHERE tenant_id = $4 AND name = $5
                "#,
            )
            .bind(&sealed)
            .bind(row.version + 1)
            .bind(current_version as i32)
            .bind(row.tenant_id)
            .bind(&row.name)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("vault reencrypt update: {e}")))?;
            n += 1;
        }
        Ok(n)
    }
}
