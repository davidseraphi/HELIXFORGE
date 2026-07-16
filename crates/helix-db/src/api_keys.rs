//! Service API keys (machine principals) for enterprise integrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared_core::ids::TenantId;
use shared_core::tenancy::{Principal, Scope};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedApiKey {
    pub record: ApiKeyRecord,
    /// Full secret shown once: `hk_live_<prefix>_<secret>`
    pub secret: String,
}

#[derive(Clone)]
pub struct ApiKeyStore {
    pool: PgPool,
}

impl ApiKeyStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn hash_key(raw: &str) -> String {
        let dig = Sha256::digest(raw.as_bytes());
        hex::encode(dig)
    }

    pub async fn issue(
        &self,
        tenant_id: TenantId,
        name: &str,
        scopes: &[Scope],
        expires_at: Option<DateTime<Utc>>,
    ) -> HelixResult<IssuedApiKey> {
        if name.trim().is_empty() {
            return Err(HelixError::validation("api key name required"));
        }
        let id = Uuid::now_v7();
        let prefix = &id.to_string().replace('-', "")[..12];
        let secret_tail = Uuid::now_v7().to_string().replace('-', "");
        let secret = format!("hk_live_{prefix}_{secret_tail}");
        let key_hash = Self::hash_key(&secret);
        let scope_strs: Vec<String> = scopes.iter().map(|s| s.as_str().to_string()).collect();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO helix_core.service_api_keys
                (id, tenant_id, name, key_prefix, key_hash, scopes, expires_at, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(prefix)
        .bind(&key_hash)
        .bind(&scope_strs)
        .bind(expires_at)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("api key issue: {e}")))?;

        Ok(IssuedApiKey {
            record: ApiKeyRecord {
                id,
                tenant_id,
                name: name.into(),
                key_prefix: prefix.into(),
                scopes: scope_strs,
                expires_at,
                revoked_at: None,
                created_at: now,
                last_used_at: None,
            },
            secret,
        })
    }

    pub async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<ApiKeyRecord>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            key_prefix: String,
            scopes: Vec<String>,
            expires_at: Option<DateTime<Utc>>,
            revoked_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
            last_used_at: Option<DateTime<Utc>>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, key_prefix, scopes, expires_at, revoked_at, created_at, last_used_at
            FROM helix_core.service_api_keys
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("api key list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| ApiKeyRecord {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                name: r.name,
                key_prefix: r.key_prefix,
                scopes: r.scopes,
                expires_at: r.expires_at,
                revoked_at: r.revoked_at,
                created_at: r.created_at,
                last_used_at: r.last_used_at,
            })
            .collect())
    }

    pub async fn revoke(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<()> {
        let res = sqlx::query(
            r#"
            UPDATE helix_core.service_api_keys
            SET revoked_at = now()
            WHERE tenant_id = $1 AND id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("api key revoke: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("api key {id}")));
        }
        Ok(())
    }

    /// Resolve a raw secret to a Principal (machine user).
    pub async fn resolve(&self, raw_key: &str) -> HelixResult<Option<Principal>> {
        let hash = Self::hash_key(raw_key);
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            scopes: Vec<String>,
            expires_at: Option<DateTime<Utc>>,
            revoked_at: Option<DateTime<Utc>>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, scopes, expires_at, revoked_at
            FROM helix_core.service_api_keys
            WHERE key_hash = $1
            "#,
        )
        .bind(&hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("api key resolve: {e}")))?;

        let Some(row) = row else {
            return Ok(None);
        };
        if row.revoked_at.is_some() {
            return Err(HelixError::unauthorized("api key revoked"));
        }
        if row.expires_at.is_some_and(|e| e < Utc::now()) {
            return Err(HelixError::unauthorized("api key expired"));
        }
        let _ = sqlx::query(
            "UPDATE helix_core.service_api_keys SET last_used_at = now() WHERE id = $1",
        )
        .bind(row.id)
        .execute(&self.pool)
        .await;

        let scopes: Vec<Scope> = row
            .scopes
            .iter()
            .filter_map(|s| Scope::parse_token(s))
            .collect();
        let scopes = if scopes.is_empty() {
            vec![Scope::Read]
        } else {
            scopes
        };

        Ok(Some(Principal {
            user_id: shared_core::ids::UserId::from_uuid(row.id),
            tenant_id: TenantId::from_uuid(row.tenant_id),
            org_id: None,
            scopes,
            session_id: Some(format!("api-key:{}", row.id)),
            residency_region: "local".into(), // refined by middleware from config if needed
        }))
    }
}
