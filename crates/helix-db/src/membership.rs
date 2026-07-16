//! Tenant membership repository — records user-to-tenant bindings and roles.
//!
//! Every tenant-scoped operation runs inside a short transaction with the
//! `app.current_tenant` Postgres setting pinned to the target tenant. Row-level
//! security policies on `helix_core.memberships` then enforce that a connection
//! can only see rows for that tenant, even if the application filter is missing.

use chrono::{DateTime, Utc};
use shared_core::ids::{TenantId, UserId};
use shared_core::tenancy::{Membership, Role};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct MembershipRepo {
    pool: PgPool,
}

impl MembershipRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        role: Role,
        invited_by: Option<UserId>,
    ) -> HelixResult<Membership> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("membership tx begin: {e}")))?;
        crate::set_tenant_context(&mut tx, tenant_id).await?;

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO helix_core.memberships
                (tenant_id, user_id, role, invited_by, joined_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (tenant_id, user_id) DO UPDATE SET
                role = EXCLUDED.role,
                invited_by = COALESCE(EXCLUDED.invited_by, helix_core.memberships.invited_by)
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(role.as_str())
        .bind(invited_by.map(|u| u.as_uuid()))
        .bind(Utc::now())
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            return Err(HelixError::dependency(format!("membership create: {e}")));
        }

        let row = match sqlx::query_as::<_, MembershipRow>(
            r#"
            SELECT tenant_id, user_id, role, invited_by, joined_at
            FROM helix_core.memberships
            WHERE tenant_id = $1 AND user_id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.rollback().await;
                return Err(HelixError::dependency(format!(
                    "membership create fetch: {e}"
                )));
            }
        };

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("membership tx commit: {e}")))?;

        row.map(|r| r.into_membership())
            .ok_or_else(|| HelixError::internal("membership create race"))
    }

    pub async fn get(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> HelixResult<Option<Membership>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("membership tx begin: {e}")))?;
        crate::set_tenant_context(&mut tx, tenant_id).await?;

        let row = sqlx::query_as::<_, MembershipRow>(
            r#"
            SELECT tenant_id, user_id, role, invited_by, joined_at
            FROM helix_core.memberships
            WHERE tenant_id = $1 AND user_id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("membership get: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("membership tx commit: {e}")))?;

        Ok(row.map(|r| r.into_membership()))
    }

    pub async fn list_for_tenant(&self, tenant_id: TenantId) -> HelixResult<Vec<Membership>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("membership tx begin: {e}")))?;
        crate::set_tenant_context(&mut tx, tenant_id).await?;

        let rows: Vec<MembershipRow> = sqlx::query_as(
            r#"
            SELECT tenant_id, user_id, role, invited_by, joined_at
            FROM helix_core.memberships
            WHERE tenant_id = $1
            ORDER BY joined_at ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("membership list: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("membership tx commit: {e}")))?;

        Ok(rows.into_iter().map(|r| r.into_membership()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct MembershipRow {
    tenant_id: Uuid,
    user_id: Uuid,
    role: String,
    invited_by: Option<Uuid>,
    joined_at: DateTime<Utc>,
}

impl MembershipRow {
    fn into_membership(self) -> Membership {
        Membership {
            tenant_id: TenantId::from_uuid(self.tenant_id),
            user_id: UserId::from_uuid(self.user_id),
            role: Role::parse(&self.role).unwrap_or(Role::Guest),
            invited_by: self.invited_by.map(UserId::from_uuid),
            joined_at: self.joined_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenants::TenantRepo;

    fn db_url() -> Option<String> {
        std::env::var("DATABASE_URL").ok()
    }

    #[tokio::test]
    async fn membership_create_get_list_roundtrip() {
        let Some(url) = db_url() else {
            // Skip when no Postgres is available (CI without a DB service).
            return;
        };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let memberships = MembershipRepo::new(pool.clone());

        let tenant = tenants
            .create(TenantId::new(), "membership-test", "local", None)
            .await
            .unwrap();
        let user = UserId::new();
        let membership = memberships
            .create(tenant.id, user, Role::Owner, None)
            .await
            .unwrap();
        assert_eq!(membership.tenant_id, tenant.id);
        assert_eq!(membership.user_id, user);
        assert_eq!(membership.role, Role::Owner);

        let fetched = memberships.get(tenant.id, user).await.unwrap().unwrap();
        assert_eq!(fetched.user_id, user);
        assert_eq!(fetched.role, Role::Owner);

        let list = memberships.list_for_tenant(tenant.id).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].user_id, user);
    }

    #[tokio::test]
    async fn membership_isolation_across_tenants() {
        let Some(url) = db_url() else {
            return;
        };
        let pool = crate::pool::connect_and_migrate(&url).await.unwrap();
        let tenants = TenantRepo::new(pool.clone());
        let memberships = MembershipRepo::new(pool.clone());

        let tenant_a = tenants
            .create(TenantId::new(), "tenant-a", "local", None)
            .await
            .unwrap();
        let tenant_b = tenants
            .create(TenantId::new(), "tenant-b", "local", None)
            .await
            .unwrap();
        let user = UserId::new();
        memberships
            .create(tenant_a.id, user, Role::Member, None)
            .await
            .unwrap();

        assert!(memberships.get(tenant_a.id, user).await.unwrap().is_some());
        assert!(memberships.get(tenant_b.id, user).await.unwrap().is_none());

        assert_eq!(
            memberships
                .list_for_tenant(tenant_a.id)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(memberships
            .list_for_tenant(tenant_b.id)
            .await
            .unwrap()
            .is_empty());
    }
}
