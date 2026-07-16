//! Generic resource ACL for product resources (documents, matters, cases, …).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::tenancy::{Principal, Scope};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AclPermission {
    Read,
    Write,
    Delete,
    Share,
    Admin,
}

impl AclPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Share => "share",
            Self::Admin => "admin",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "delete" => Some(Self::Delete),
            "share" => Some(Self::Share),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclEntry {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub resource_type: String,
    pub resource_id: String,
    pub principal_kind: String,
    pub principal_id: String,
    pub permissions: Vec<String>,
    pub granted_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct ResourceAclRepo {
    pool: PgPool,
}

impl ResourceAclRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn grant(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
        principal_kind: &str,
        principal_id: &str,
        permissions: &[AclPermission],
        granted_by: Option<&str>,
    ) -> HelixResult<AclEntry> {
        let id = Uuid::now_v7();
        let perms: Vec<String> = permissions.iter().map(|p| p.as_str().to_string()).collect();
        let now = Utc::now();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("acl grant tx: {e}")))?;
        crate::set_tenant_context(&mut tx, tenant_id).await?;

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO helix_core.resource_acl
                (id, tenant_id, resource_type, resource_id, principal_kind, principal_id,
                 permissions, granted_by, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            ON CONFLICT (tenant_id, resource_type, resource_id, principal_kind, principal_id)
            DO UPDATE SET permissions = EXCLUDED.permissions, granted_by = EXCLUDED.granted_by
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .bind(principal_kind)
        .bind(principal_id)
        .bind(&perms)
        .bind(granted_by)
        .bind(now)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            return Err(HelixError::dependency(format!("acl grant: {e}")));
        }

        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            resource_type: String,
            resource_id: String,
            principal_kind: String,
            principal_id: String,
            permissions: Vec<String>,
            granted_by: Option<String>,
            created_at: DateTime<Utc>,
            expires_at: Option<DateTime<Utc>>,
        }
        let row: Option<Row> = match sqlx::query_as(
            r#"
            SELECT id, tenant_id, resource_type, resource_id, principal_kind, principal_id,
                   permissions, granted_by, created_at, expires_at
            FROM helix_core.resource_acl
            WHERE tenant_id = $1 AND resource_type = $2 AND resource_id = $3
              AND principal_kind = $4 AND principal_id = $5
              AND (expires_at IS NULL OR expires_at > now())
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .bind(principal_kind)
        .bind(principal_id)
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.rollback().await;
                return Err(HelixError::dependency(format!("acl grant fetch: {e}")));
            }
        };

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("acl grant commit: {e}")))?;

        row.map(|r| AclEntry {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            principal_kind: r.principal_kind,
            principal_id: r.principal_id,
            permissions: r.permissions,
            granted_by: r.granted_by,
            created_at: r.created_at,
            expires_at: r.expires_at,
        })
        .ok_or_else(|| HelixError::internal("acl grant race"))
    }

    pub async fn revoke(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
        principal_kind: &str,
        principal_id: &str,
    ) -> HelixResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("acl revoke tx: {e}")))?;
        crate::set_tenant_context(&mut tx, tenant_id).await?;

        let res = sqlx::query(
            r#"
            DELETE FROM helix_core.resource_acl
            WHERE tenant_id = $1 AND resource_type = $2 AND resource_id = $3
              AND principal_kind = $4 AND principal_id = $5
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .bind(principal_kind)
        .bind(principal_id)
        .execute(&mut *tx)
        .await;

        match res {
            Ok(r) if r.rows_affected() > 0 => {
                tx.commit()
                    .await
                    .map_err(|e| HelixError::dependency(format!("acl revoke commit: {e}")))?;
                Ok(())
            }
            Ok(_) => {
                let _ = tx.rollback().await;
                Err(HelixError::not_found("acl entry"))
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(HelixError::dependency(format!("acl revoke: {e}")))
            }
        }
    }

    pub async fn list_for_resource(
        &self,
        tenant_id: TenantId,
        resource_type: &str,
        resource_id: &str,
    ) -> HelixResult<Vec<AclEntry>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            resource_type: String,
            resource_id: String,
            principal_kind: String,
            principal_id: String,
            permissions: Vec<String>,
            granted_by: Option<String>,
            created_at: DateTime<Utc>,
            expires_at: Option<DateTime<Utc>>,
        }
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| HelixError::dependency(format!("acl list conn: {e}")))?;
        crate::set_tenant_context(&mut conn, tenant_id).await?;

        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, resource_type, resource_id, principal_kind, principal_id,
                   permissions, granted_by, created_at, expires_at
            FROM helix_core.resource_acl
            WHERE tenant_id = $1 AND resource_type = $2 AND resource_id = $3
              AND (expires_at IS NULL OR expires_at > now())
            ORDER BY created_at ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(resource_type)
        .bind(resource_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| HelixError::dependency(format!("acl list: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| AclEntry {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                resource_type: r.resource_type,
                resource_id: r.resource_id,
                principal_kind: r.principal_kind,
                principal_id: r.principal_id,
                permissions: r.permissions,
                granted_by: r.granted_by,
                created_at: r.created_at,
                expires_at: r.expires_at,
            })
            .collect())
    }

    /// Check access. Platform / Admin (tenant) bypass. Owners via ACL admin/write/read.
    pub async fn check(
        &self,
        principal: &Principal,
        resource_type: &str,
        resource_id: &str,
        need: AclPermission,
    ) -> HelixResult<bool> {
        if principal.has_scope(&Scope::Platform) {
            return Ok(true);
        }
        // No ACL rows => deny for non-admin (products should grant owner on create)
        let entries = self
            .list_for_resource(principal.tenant_id, resource_type, resource_id)
            .await?;
        if entries.is_empty() {
            // Empty ACL: allow tenant Admin (legacy bootstrap), else deny
            return Ok(principal.has_scope(&Scope::Admin));
        }

        let user_key = principal.user_id.to_string();
        let user_uuid = principal.user_id.as_uuid().to_string();
        let session = principal.session_id.clone().unwrap_or_default();

        for e in &entries {
            let matches = match e.principal_kind.as_str() {
                "user" => e.principal_id == user_key || e.principal_id == user_uuid,
                "api_key" => {
                    session.starts_with("api-key:")
                        && session.trim_start_matches("api-key:") == e.principal_id
                }
                "tenant" => {
                    e.principal_id == principal.tenant_id.to_string()
                        || e.principal_id == principal.tenant_id.as_uuid().to_string()
                }
                "role" => match e.principal_id.as_str() {
                    "admin" => principal.has_scope(&Scope::Admin),
                    "write" => principal.has_scope(&Scope::Write),
                    "read" => principal.has_scope(&Scope::Read),
                    _ => false,
                },
                _ => false,
            };
            if !matches {
                continue;
            }
            if e.permissions.iter().any(|p| {
                p == "admin" || p == need.as_str() || (need == AclPermission::Read && p == "write")
            }) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn require(
        &self,
        principal: &Principal,
        resource_type: &str,
        resource_id: &str,
        need: AclPermission,
    ) -> HelixResult<()> {
        if self
            .check(principal, resource_type, resource_id, need)
            .await?
        {
            Ok(())
        } else {
            Err(HelixError::forbidden(format!(
                "missing {need:?} on {resource_type}/{resource_id}"
            )))
        }
    }
}
