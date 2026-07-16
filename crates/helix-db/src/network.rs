//! HelixNetwork profiles, connections, opportunities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::{TenantId, UserId};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub display_name: String,
    pub headline: String,
    pub bio: String,
    pub skills: serde_json::Value,
    pub location: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub from_profile_id: Uuid,
    pub to_profile_id: Uuid,
    pub status: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub owner_profile_id: Uuid,
    pub title: String,
    pub description: String,
    pub kind: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct ProfileRow {
    id: Uuid,
    tenant_id: Uuid,
    user_id: Uuid,
    display_name: String,
    headline: String,
    bio: String,
    skills: serde_json::Value,
    location: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl ProfileRow {
    fn into_profile(self) -> Profile {
        Profile {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            user_id: UserId::from_uuid(self.user_id),
            display_name: self.display_name,
            headline: self.headline,
            bio: self.bio,
            skills: self.skills,
            location: self.location,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
        }
    }
}

const PROFILE_SELECT: &str = r#"
    SELECT id, tenant_id, user_id, display_name, headline, bio, skills, location, status, metadata, created_at
    FROM network.profiles
"#;

#[derive(Clone)]
pub struct NetworkRepo {
    pool: PgPool,
}

impl NetworkRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_profiles(&self, tenant_id: TenantId) -> HelixResult<Vec<Profile>> {
        let rows: Vec<ProfileRow> = sqlx::query_as(&format!(
            "{PROFILE_SELECT} WHERE tenant_id = $1 ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network list profiles: {e}")))?;
        Ok(rows.into_iter().map(ProfileRow::into_profile).collect())
    }

    pub async fn create_profile(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        display_name: &str,
        headline: &str,
        bio: &str,
        skills: serde_json::Value,
        location: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Profile> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let skills = if skills.is_null() {
            serde_json::json!([])
        } else {
            skills
        };
        sqlx::query(
            r#"
            INSERT INTO network.profiles
                (id, tenant_id, user_id, display_name, headline, bio, skills, location,
                 status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'active',$9,$10,$10)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(display_name)
        .bind(headline)
        .bind(bio)
        .bind(&skills)
        .bind(location)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                HelixError::conflict("profile already exists for this user")
            } else {
                HelixError::dependency(format!("network create profile: {e}"))
            }
        })?;
        Ok(Profile {
            id,
            tenant_id,
            user_id,
            display_name: display_name.into(),
            headline: headline.into(),
            bio: bio.into(),
            skills,
            location: location.into(),
            status: "active".into(),
            metadata,
            created_at,
        })
    }

    pub async fn get_profile(
        &self,
        tenant_id: TenantId,
        profile_id: Uuid,
    ) -> HelixResult<Option<Profile>> {
        let row: Option<ProfileRow> = sqlx::query_as(&format!(
            "{PROFILE_SELECT} WHERE tenant_id = $1 AND id = $2"
        ))
        .bind(tenant_id.as_uuid())
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network get profile: {e}")))?;
        Ok(row.map(ProfileRow::into_profile))
    }

    pub async fn get_profile_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> HelixResult<Option<Profile>> {
        let row: Option<ProfileRow> = sqlx::query_as(&format!(
            "{PROFILE_SELECT} WHERE tenant_id = $1 AND user_id = $2"
        ))
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network get profile by user: {e}")))?;
        Ok(row.map(ProfileRow::into_profile))
    }

    pub async fn request_connection(
        &self,
        tenant_id: TenantId,
        from_profile_id: Uuid,
        to_profile_id: Uuid,
        message: &str,
    ) -> HelixResult<Connection> {
        if from_profile_id == to_profile_id {
            return Err(HelixError::validation("cannot connect to yourself"));
        }
        let from = self
            .get_profile(tenant_id, from_profile_id)
            .await?
            .ok_or_else(|| HelixError::not_found("from profile not found"))?;
        let _to = self
            .get_profile(tenant_id, to_profile_id)
            .await?
            .ok_or_else(|| HelixError::not_found("to profile not found"))?;
        let _ = from;

        let id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO network.connections
                (id, tenant_id, from_profile_id, to_profile_id, status, message, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'pending',$5,$6,$6)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(from_profile_id)
        .bind(to_profile_id)
        .bind(message)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                HelixError::conflict("connection already requested")
            } else {
                HelixError::dependency(format!("network request connection: {e}"))
            }
        })?;
        Ok(Connection {
            id,
            tenant_id,
            from_profile_id,
            to_profile_id,
            status: "pending".into(),
            message: message.into(),
            created_at,
        })
    }

    pub async fn accept_connection(
        &self,
        tenant_id: TenantId,
        connection_id: Uuid,
        acceptor_profile_id: Uuid,
    ) -> HelixResult<Connection> {
        let updated_at = Utc::now();
        let res = sqlx::query(
            r#"
            UPDATE network.connections
            SET status = 'accepted', updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND to_profile_id = $4 AND status = 'pending'
            "#,
        )
        .bind(updated_at)
        .bind(tenant_id.as_uuid())
        .bind(connection_id)
        .bind(acceptor_profile_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network accept connection: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(
                "pending connection not found for this profile",
            ));
        }
        self.get_connection(tenant_id, connection_id)
            .await?
            .ok_or_else(|| HelixError::not_found("connection not found"))
    }

    pub async fn get_connection(
        &self,
        tenant_id: TenantId,
        connection_id: Uuid,
    ) -> HelixResult<Option<Connection>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            from_profile_id: Uuid,
            to_profile_id: Uuid,
            status: String,
            message: String,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, from_profile_id, to_profile_id, status, message, created_at
            FROM network.connections
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network get connection: {e}")))?;
        Ok(row.map(|r| Connection {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            from_profile_id: r.from_profile_id,
            to_profile_id: r.to_profile_id,
            status: r.status,
            message: r.message,
            created_at: r.created_at,
        }))
    }

    pub async fn list_connections(
        &self,
        tenant_id: TenantId,
        profile_id: Option<Uuid>,
    ) -> HelixResult<Vec<Connection>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            from_profile_id: Uuid,
            to_profile_id: Uuid,
            status: String,
            message: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = if let Some(pid) = profile_id {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, from_profile_id, to_profile_id, status, message, created_at
                FROM network.connections
                WHERE tenant_id = $1 AND (from_profile_id = $2 OR to_profile_id = $2)
                ORDER BY created_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(pid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, from_profile_id, to_profile_id, status, message, created_at
                FROM network.connections
                WHERE tenant_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("network list connections: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Connection {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                from_profile_id: r.from_profile_id,
                to_profile_id: r.to_profile_id,
                status: r.status,
                message: r.message,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn list_opportunities(&self, tenant_id: TenantId) -> HelixResult<Vec<Opportunity>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            owner_profile_id: Uuid,
            title: String,
            description: String,
            kind: String,
            status: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, owner_profile_id, title, description, kind, status, metadata, created_at
            FROM network.opportunities
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network list opportunities: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Opportunity {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                owner_profile_id: r.owner_profile_id,
                title: r.title,
                description: r.description,
                kind: r.kind,
                status: r.status,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create_opportunity(
        &self,
        tenant_id: TenantId,
        owner_profile_id: Uuid,
        title: &str,
        description: &str,
        kind: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Opportunity> {
        let _owner = self
            .get_profile(tenant_id, owner_profile_id)
            .await?
            .ok_or_else(|| HelixError::not_found("owner profile not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let kind = if kind.trim().is_empty() {
            "role"
        } else {
            kind.trim()
        };
        sqlx::query(
            r#"
            INSERT INTO network.opportunities
                (id, tenant_id, owner_profile_id, title, description, kind, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,'open',$7,$8,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(owner_profile_id)
        .bind(title)
        .bind(description)
        .bind(kind)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network create opportunity: {e}")))?;
        Ok(Opportunity {
            id,
            tenant_id,
            owner_profile_id,
            title: title.into(),
            description: description.into(),
            kind: kind.into(),
            status: "open".into(),
            metadata,
            created_at,
        })
    }

    pub async fn get_opportunity(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
    ) -> HelixResult<Option<Opportunity>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            owner_profile_id: Uuid,
            title: String,
            description: String,
            kind: String,
            status: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, owner_profile_id, title, description, kind, status, metadata, created_at
            FROM network.opportunities
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(opportunity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network get opportunity: {e}")))?;
        Ok(row.map(|r| Opportunity {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            owner_profile_id: r.owner_profile_id,
            title: r.title,
            description: r.description,
            kind: r.kind,
            status: r.status,
            metadata: r.metadata,
            created_at: r.created_at,
        }))
    }
}
