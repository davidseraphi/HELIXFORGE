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
    pub deactivated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
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
    pub responded_at: Option<DateTime<Utc>>,
    pub blocked_by: Option<Uuid>,
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
    pub closed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NetworkSummaryRow {
    pub id: Uuid,
    pub display_name: String,
    pub status: String,
    pub accepted_count: i64,
    pub pending_in: i64,
    pub pending_out: i64,
    pub open_opportunities: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileUpdate {
    pub display_name: Option<String>,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub skills: Option<serde_json::Value>,
    pub location: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct OpportunityUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a profile lifecycle transition and return the resulting status.
pub fn next_profile_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("active", "deactivate") => Ok("deactivated"),
        ("deactivated", "reactivate") => Ok("active"),
        (_, "deactivate") => Err(HelixError::validation(format!(
            "cannot deactivate a {current} profile"
        ))),
        (_, "reactivate") => Err(HelixError::validation(format!(
            "cannot reactivate a {current} profile"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown profile action {action}"
        ))),
    }
}

/// Validate an opportunity lifecycle transition and return the resulting status.
pub fn next_opportunity_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("open", "close") => Ok("closed"),
        ("closed", "reopen") => Ok("open"),
        (_, "close") => Err(HelixError::validation(format!(
            "cannot close a {current} opportunity"
        ))),
        (_, "reopen") => Err(HelixError::validation(format!(
            "cannot reopen a {current} opportunity"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown opportunity action {action}"
        ))),
    }
}

/// A declined or removed connection can be revived by a fresh request.
pub fn can_revive_connection(status: &str) -> bool {
    matches!(status, "declined" | "removed")
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
    deactivated_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
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
            deactivated_at: self.deactivated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ConnectionRow {
    id: Uuid,
    tenant_id: Uuid,
    from_profile_id: Uuid,
    to_profile_id: Uuid,
    status: String,
    message: String,
    created_at: DateTime<Utc>,
    responded_at: Option<DateTime<Utc>>,
    blocked_by: Option<Uuid>,
}

impl ConnectionRow {
    fn into_connection(self) -> Connection {
        Connection {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            from_profile_id: self.from_profile_id,
            to_profile_id: self.to_profile_id,
            status: self.status,
            message: self.message,
            created_at: self.created_at,
            responded_at: self.responded_at,
            blocked_by: self.blocked_by,
        }
    }
}

#[derive(sqlx::FromRow)]
struct OpportunityRow {
    id: Uuid,
    tenant_id: Uuid,
    owner_profile_id: Uuid,
    title: String,
    description: String,
    kind: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl OpportunityRow {
    fn into_opportunity(self) -> Opportunity {
        Opportunity {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            owner_profile_id: self.owner_profile_id,
            title: self.title,
            description: self.description,
            kind: self.kind,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            closed_at: self.closed_at,
            deleted_at: self.deleted_at,
        }
    }
}

const PROFILE_SELECT: &str = r#"
    SELECT id, tenant_id, user_id, display_name, headline, bio, skills, location,
           status, metadata, created_at, deactivated_at, deleted_at
    FROM network.profiles
"#;

const PROFILE_RETURNING: &str = r#"
    RETURNING id, tenant_id, user_id, display_name, headline, bio, skills, location,
              status, metadata, created_at, deactivated_at, deleted_at
"#;

const CONNECTION_SELECT: &str = r#"
    SELECT id, tenant_id, from_profile_id, to_profile_id, status, message,
           created_at, responded_at, blocked_by
    FROM network.connections
"#;

const CONNECTION_RETURNING: &str = r#"
    RETURNING id, tenant_id, from_profile_id, to_profile_id, status, message,
              created_at, responded_at, blocked_by
"#;

const OPPORTUNITY_SELECT: &str = r#"
    SELECT id, tenant_id, owner_profile_id, title, description, kind, status,
           metadata, created_at, closed_at, deleted_at
    FROM network.opportunities
"#;

const OPPORTUNITY_RETURNING: &str = r#"
    RETURNING id, tenant_id, owner_profile_id, title, description, kind, status,
              metadata, created_at, closed_at, deleted_at
"#;

#[derive(Clone)]
pub struct NetworkRepo {
    pool: PgPool,
}

impl NetworkRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Profiles ---

    pub async fn list_profiles(&self, tenant_id: TenantId) -> HelixResult<Vec<Profile>> {
        let rows: Vec<ProfileRow> = sqlx::query_as(&format!(
            "{PROFILE_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
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
        let row: ProfileRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO network.profiles
                (id, tenant_id, user_id, display_name, headline, bio, skills, location,
                 status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'active',$9,$10,$10)
            {PROFILE_RETURNING}
            "#
        ))
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
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                HelixError::conflict("profile already exists for this user")
            } else {
                HelixError::dependency(format!("network create profile: {e}"))
            }
        })?;
        Ok(row.into_profile())
    }

    pub async fn get_profile(
        &self,
        tenant_id: TenantId,
        profile_id: Uuid,
    ) -> HelixResult<Option<Profile>> {
        let row: Option<ProfileRow> = sqlx::query_as(&format!(
            "{PROFILE_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
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
            "{PROFILE_SELECT} WHERE tenant_id = $1 AND user_id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network get profile by user: {e}")))?;
        Ok(row.map(ProfileRow::into_profile))
    }

    async fn fetch_profile_any(
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
        .map_err(|e| HelixError::dependency(format!("network fetch profile: {e}")))?;
        Ok(row.map(ProfileRow::into_profile))
    }

    pub async fn update_profile(
        &self,
        tenant_id: TenantId,
        profile_id: Uuid,
        owner_user_id: UserId,
        update: ProfileUpdate,
    ) -> HelixResult<Profile> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE network.profiles SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(n) = update.display_name {
            builder.push(", display_name = ");
            builder.push_bind(n);
        }
        if let Some(h) = update.headline {
            builder.push(", headline = ");
            builder.push_bind(h);
        }
        if let Some(b) = update.bio {
            builder.push(", bio = ");
            builder.push_bind(b);
        }
        if let Some(s) = update.skills {
            builder.push(", skills = ");
            builder.push_bind(s);
        }
        if let Some(l) = update.location {
            builder.push(", location = ");
            builder.push_bind(l);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(profile_id);
        builder.push(" AND user_id = ");
        builder.push_bind(owner_user_id.as_uuid());
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {PROFILE_RETURNING}"));

        let row: Option<ProfileRow> = builder
            .build_query_as::<ProfileRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("network update profile: {e}")))?;

        row.map(ProfileRow::into_profile)
            .ok_or_else(|| HelixError::not_found("profile not found"))
    }

    async fn transition_profile(
        &self,
        tenant_id: TenantId,
        profile_id: Uuid,
        owner_user_id: UserId,
        action: &str,
    ) -> HelixResult<Profile> {
        let profile = self
            .get_profile(tenant_id, profile_id)
            .await?
            .ok_or_else(|| HelixError::not_found("profile not found"))?;
        if profile.user_id != owner_user_id {
            return Err(HelixError::not_found("profile not found"));
        }
        let next = next_profile_status(&profile.status, action)?;
        let now = Utc::now();
        let deactivated_at = if next == "deactivated" {
            Some(now)
        } else {
            None
        };
        let row: Option<ProfileRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.profiles
            SET status = $1, deactivated_at = $2, updated_at = $3
            WHERE tenant_id = $4 AND id = $5 AND deleted_at IS NULL
            {PROFILE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(deactivated_at)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network {action} profile: {e}")))?;

        row.map(ProfileRow::into_profile)
            .ok_or_else(|| HelixError::not_found("profile not found"))
    }

    pub async fn deactivate_profile(
        &self,
        tenant_id: TenantId,
        profile_id: Uuid,
        owner_user_id: UserId,
    ) -> HelixResult<Profile> {
        self.transition_profile(tenant_id, profile_id, owner_user_id, "deactivate")
            .await
    }

    pub async fn reactivate_profile(
        &self,
        tenant_id: TenantId,
        profile_id: Uuid,
        owner_user_id: UserId,
    ) -> HelixResult<Profile> {
        self.transition_profile(tenant_id, profile_id, owner_user_id, "reactivate")
            .await
    }

    pub async fn soft_delete_profile(
        &self,
        tenant_id: TenantId,
        profile_id: Uuid,
        owner_user_id: UserId,
    ) -> HelixResult<Profile> {
        let profile = self
            .get_profile(tenant_id, profile_id)
            .await?
            .ok_or_else(|| HelixError::not_found("profile not found"))?;
        if profile.user_id != owner_user_id {
            return Err(HelixError::not_found("profile not found"));
        }
        if profile.status == "deleted" {
            return Err(HelixError::validation("profile is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<ProfileRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.profiles
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {PROFILE_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network soft-delete profile: {e}")))?;

        row.map(ProfileRow::into_profile)
            .ok_or_else(|| HelixError::not_found("profile not found"))
    }

    /// Restore a soft-deleted profile, returning it to its pre-delete status.
    pub async fn restore_profile(
        &self,
        tenant_id: TenantId,
        profile_id: Uuid,
        owner_user_id: UserId,
    ) -> HelixResult<Profile> {
        let profile = self
            .fetch_profile_any(tenant_id, profile_id)
            .await?
            .ok_or_else(|| HelixError::not_found("profile not found"))?;
        if profile.user_id != owner_user_id {
            return Err(HelixError::not_found("profile not found"));
        }
        if profile.deleted_at.is_none() {
            return Err(HelixError::validation("profile is not deleted"));
        }
        let restored = if profile.deactivated_at.is_some() {
            "deactivated"
        } else {
            "active"
        };
        let now = Utc::now();
        let row: Option<ProfileRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.profiles
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {PROFILE_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network restore profile: {e}")))?;

        row.map(ProfileRow::into_profile)
            .ok_or_else(|| HelixError::not_found("profile not found or not deleted"))
    }

    // --- Connections ---

    pub async fn get_connection(
        &self,
        tenant_id: TenantId,
        connection_id: Uuid,
    ) -> HelixResult<Option<Connection>> {
        let row: Option<ConnectionRow> = sqlx::query_as(&format!(
            "{CONNECTION_SELECT} WHERE tenant_id = $1 AND id = $2"
        ))
        .bind(tenant_id.as_uuid())
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network get connection: {e}")))?;
        Ok(row.map(ConnectionRow::into_connection))
    }

    pub async fn list_connections(
        &self,
        tenant_id: TenantId,
        profile_id: Option<Uuid>,
    ) -> HelixResult<Vec<Connection>> {
        let rows: Vec<ConnectionRow> = if let Some(pid) = profile_id {
            sqlx::query_as(&format!(
                "{CONNECTION_SELECT} WHERE tenant_id = $1 AND (from_profile_id = $2 OR to_profile_id = $2) ORDER BY created_at DESC"
            ))
            .bind(tenant_id.as_uuid())
            .bind(pid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(&format!(
                "{CONNECTION_SELECT} WHERE tenant_id = $1 ORDER BY created_at DESC"
            ))
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("network list connections: {e}")))?;
        Ok(rows
            .into_iter()
            .map(ConnectionRow::into_connection)
            .collect())
    }

    /// Request a connection. A declined or removed ordered pair is revived back
    /// to pending; a pair with a `blocked` row in either direction is rejected.
    /// Profile checks, blocked-pair check, and the insert/revive run in one
    /// transaction with the profile rows locked — no check-then-act window.
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

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("network request tx: {e}")))?;

        let from: Option<ProfileRow> = sqlx::query_as(&format!(
            "{PROFILE_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE"
        ))
        .bind(tenant_id.as_uuid())
        .bind(from_profile_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("network lock from profile: {e}")))?;
        let from = from
            .map(ProfileRow::into_profile)
            .ok_or_else(|| HelixError::not_found("from profile not found"))?;

        let to: Option<ProfileRow> = sqlx::query_as(&format!(
            "{PROFILE_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE"
        ))
        .bind(tenant_id.as_uuid())
        .bind(to_profile_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("network lock to profile: {e}")))?;
        let to = to
            .map(ProfileRow::into_profile)
            .ok_or_else(|| HelixError::not_found("to profile not found"))?;

        if from.status != "active" || to.status != "active" {
            return Err(HelixError::validation(
                "both profiles must be active to connect",
            ));
        }

        let blocked: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM network.connections
            WHERE tenant_id = $1 AND status = 'blocked'
              AND ((from_profile_id = $2 AND to_profile_id = $3)
                OR (from_profile_id = $3 AND to_profile_id = $2))
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(from_profile_id)
        .bind(to_profile_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("network blocked-pair check: {e}")))?;
        if blocked > 0 {
            return Err(HelixError::validation(
                "connection is blocked between these profiles",
            ));
        }

        let existing: Option<ConnectionRow> = sqlx::query_as(&format!(
            "{CONNECTION_SELECT} WHERE tenant_id = $1 AND from_profile_id = $2 AND to_profile_id = $3 FOR UPDATE"
        ))
        .bind(tenant_id.as_uuid())
        .bind(from_profile_id)
        .bind(to_profile_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("network connection lookup: {e}")))?;

        if let Some(row) = existing {
            let status = row.status.clone();
            let conn = row.into_connection();
            if can_revive_connection(&status) {
                let now = Utc::now();
                let revived: Option<ConnectionRow> = sqlx::query_as(&format!(
                    r#"
                    UPDATE network.connections
                    SET status = 'pending', message = $1, responded_at = NULL,
                        blocked_by = NULL, updated_at = $2
                    WHERE tenant_id = $3 AND id = $4
                    {CONNECTION_RETURNING}
                    "#
                ))
                .bind(message)
                .bind(now)
                .bind(tenant_id.as_uuid())
                .bind(conn.id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| HelixError::dependency(format!("network revive connection: {e}")))?;
                let out = revived
                    .map(ConnectionRow::into_connection)
                    .ok_or_else(|| HelixError::not_found("connection not found"))?;
                tx.commit()
                    .await
                    .map_err(|e| HelixError::dependency(format!("network revive commit: {e}")))?;
                return Ok(out);
            }
            return Err(match status.as_str() {
                "pending" => HelixError::conflict("connection already requested"),
                "accepted" => HelixError::conflict("profiles already connected"),
                other => HelixError::validation(format!("cannot re-request a {other} connection")),
            });
        }

        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: ConnectionRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO network.connections
                (id, tenant_id, from_profile_id, to_profile_id, status, message, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'pending',$5,$6,$6)
            {CONNECTION_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(from_profile_id)
        .bind(to_profile_id)
        .bind(message)
        .bind(created_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                HelixError::conflict("connection already requested")
            } else {
                HelixError::dependency(format!("network request connection: {e}"))
            }
        })?;
        let out = row.into_connection();
        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("network request commit: {e}")))?;
        Ok(out)
    }

    pub async fn accept_connection(
        &self,
        tenant_id: TenantId,
        connection_id: Uuid,
        acceptor_profile_id: Uuid,
    ) -> HelixResult<Connection> {
        let now = Utc::now();
        let row: Option<ConnectionRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.connections
            SET status = 'accepted', responded_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND to_profile_id = $4 AND status = 'pending'
            {CONNECTION_RETURNING}
            "#
        ))
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(connection_id)
        .bind(acceptor_profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network accept connection: {e}")))?;

        row.map(ConnectionRow::into_connection)
            .ok_or_else(|| HelixError::not_found("pending connection not found for this profile"))
    }

    pub async fn decline_connection(
        &self,
        tenant_id: TenantId,
        connection_id: Uuid,
        decliner_profile_id: Uuid,
    ) -> HelixResult<Connection> {
        let now = Utc::now();
        let row: Option<ConnectionRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.connections
            SET status = 'declined', responded_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND to_profile_id = $4 AND status = 'pending'
            {CONNECTION_RETURNING}
            "#
        ))
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(connection_id)
        .bind(decliner_profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network decline connection: {e}")))?;

        row.map(ConnectionRow::into_connection)
            .ok_or_else(|| HelixError::not_found("pending connection not found for this profile"))
    }

    /// Remove an accepted connection; either party may remove it.
    pub async fn remove_connection(
        &self,
        tenant_id: TenantId,
        connection_id: Uuid,
        actor_profile_id: Uuid,
    ) -> HelixResult<Connection> {
        let now = Utc::now();
        let row: Option<ConnectionRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.connections
            SET status = 'removed', updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND status = 'accepted'
              AND (from_profile_id = $4 OR to_profile_id = $4)
            {CONNECTION_RETURNING}
            "#
        ))
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(connection_id)
        .bind(actor_profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network remove connection: {e}")))?;

        row.map(ConnectionRow::into_connection)
            .ok_or_else(|| HelixError::not_found("accepted connection not found for this profile"))
    }

    /// Block a connection pair; either party may block. A blocked pair cannot
    /// request again in either direction.
    pub async fn block_connection(
        &self,
        tenant_id: TenantId,
        connection_id: Uuid,
        actor_profile_id: Uuid,
    ) -> HelixResult<Connection> {
        let now = Utc::now();
        let row: Option<ConnectionRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.connections
            SET status = 'blocked', blocked_by = $1, responded_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status <> 'blocked'
              AND (from_profile_id = $5 OR to_profile_id = $5)
            {CONNECTION_RETURNING}
            "#
        ))
        .bind(actor_profile_id)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(connection_id)
        .bind(actor_profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network block connection: {e}")))?;

        row.map(ConnectionRow::into_connection)
            .ok_or_else(|| HelixError::not_found("connection not found for this profile"))
    }

    // --- Opportunities ---

    pub async fn list_opportunities(&self, tenant_id: TenantId) -> HelixResult<Vec<Opportunity>> {
        let rows: Vec<OpportunityRow> = sqlx::query_as(&format!(
            "{OPPORTUNITY_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network list opportunities: {e}")))?;
        Ok(rows
            .into_iter()
            .map(OpportunityRow::into_opportunity)
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
        let row: OpportunityRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO network.opportunities
                (id, tenant_id, owner_profile_id, title, description, kind, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,'open',$7,$8,$8)
            {OPPORTUNITY_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(owner_profile_id)
        .bind(title)
        .bind(description)
        .bind(kind)
        .bind(&metadata)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network create opportunity: {e}")))?;
        Ok(row.into_opportunity())
    }

    pub async fn get_opportunity(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
    ) -> HelixResult<Option<Opportunity>> {
        let row: Option<OpportunityRow> = sqlx::query_as(&format!(
            "{OPPORTUNITY_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(opportunity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network get opportunity: {e}")))?;
        Ok(row.map(OpportunityRow::into_opportunity))
    }

    async fn fetch_opportunity_any(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
    ) -> HelixResult<Option<Opportunity>> {
        let row: Option<OpportunityRow> = sqlx::query_as(&format!(
            "{OPPORTUNITY_SELECT} WHERE tenant_id = $1 AND id = $2"
        ))
        .bind(tenant_id.as_uuid())
        .bind(opportunity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network fetch opportunity: {e}")))?;
        Ok(row.map(OpportunityRow::into_opportunity))
    }

    pub async fn update_opportunity(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
        owner_profile_id: Uuid,
        update: OpportunityUpdate,
    ) -> HelixResult<Opportunity> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE network.opportunities SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(t) = update.title {
            builder.push(", title = ");
            builder.push_bind(t);
        }
        if let Some(d) = update.description {
            builder.push(", description = ");
            builder.push_bind(d);
        }
        if let Some(k) = update.kind {
            let k = if k.trim().is_empty() {
                "role".to_string()
            } else {
                k.trim().to_string()
            };
            builder.push(", kind = ");
            builder.push_bind(k);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(opportunity_id);
        builder.push(" AND owner_profile_id = ");
        builder.push_bind(owner_profile_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {OPPORTUNITY_RETURNING}"));

        let row: Option<OpportunityRow> = builder
            .build_query_as::<OpportunityRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("network update opportunity: {e}")))?;

        row.map(OpportunityRow::into_opportunity)
            .ok_or_else(|| HelixError::not_found("opportunity not found"))
    }

    async fn transition_opportunity(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
        owner_profile_id: Uuid,
        action: &str,
    ) -> HelixResult<Opportunity> {
        let opp = self
            .get_opportunity(tenant_id, opportunity_id)
            .await?
            .ok_or_else(|| HelixError::not_found("opportunity not found"))?;
        if opp.owner_profile_id != owner_profile_id {
            return Err(HelixError::not_found("opportunity not found"));
        }
        let next = next_opportunity_status(&opp.status, action)?;
        let now = Utc::now();
        let closed_at = if next == "closed" { Some(now) } else { None };
        let row: Option<OpportunityRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.opportunities
            SET status = $1, closed_at = $2, updated_at = $3
            WHERE tenant_id = $4 AND id = $5 AND deleted_at IS NULL
            {OPPORTUNITY_RETURNING}
            "#
        ))
        .bind(next)
        .bind(closed_at)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(opportunity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network {action} opportunity: {e}")))?;

        row.map(OpportunityRow::into_opportunity)
            .ok_or_else(|| HelixError::not_found("opportunity not found"))
    }

    pub async fn close_opportunity(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
        owner_profile_id: Uuid,
    ) -> HelixResult<Opportunity> {
        self.transition_opportunity(tenant_id, opportunity_id, owner_profile_id, "close")
            .await
    }

    pub async fn reopen_opportunity(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
        owner_profile_id: Uuid,
    ) -> HelixResult<Opportunity> {
        self.transition_opportunity(tenant_id, opportunity_id, owner_profile_id, "reopen")
            .await
    }

    pub async fn soft_delete_opportunity(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
        owner_profile_id: Uuid,
    ) -> HelixResult<Opportunity> {
        let opp = self
            .get_opportunity(tenant_id, opportunity_id)
            .await?
            .ok_or_else(|| HelixError::not_found("opportunity not found"))?;
        if opp.owner_profile_id != owner_profile_id {
            return Err(HelixError::not_found("opportunity not found"));
        }
        if opp.status == "deleted" {
            return Err(HelixError::validation("opportunity is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<OpportunityRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.opportunities
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {OPPORTUNITY_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(opportunity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network soft-delete opportunity: {e}")))?;

        row.map(OpportunityRow::into_opportunity)
            .ok_or_else(|| HelixError::not_found("opportunity not found"))
    }

    /// Restore a soft-deleted opportunity, returning it to its pre-delete status.
    pub async fn restore_opportunity(
        &self,
        tenant_id: TenantId,
        opportunity_id: Uuid,
        owner_profile_id: Uuid,
    ) -> HelixResult<Opportunity> {
        let opp = self
            .fetch_opportunity_any(tenant_id, opportunity_id)
            .await?
            .ok_or_else(|| HelixError::not_found("opportunity not found"))?;
        if opp.owner_profile_id != owner_profile_id {
            return Err(HelixError::not_found("opportunity not found"));
        }
        if opp.deleted_at.is_none() {
            return Err(HelixError::validation("opportunity is not deleted"));
        }
        let restored = if opp.closed_at.is_some() {
            "closed"
        } else {
            "open"
        };
        let now = Utc::now();
        let row: Option<OpportunityRow> = sqlx::query_as(&format!(
            r#"
            UPDATE network.opportunities
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {OPPORTUNITY_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(opportunity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network restore opportunity: {e}")))?;

        row.map(OpportunityRow::into_opportunity)
            .ok_or_else(|| HelixError::not_found("opportunity not found or not deleted"))
    }

    // --- Reports ---

    /// Per-profile connection and opportunity counts for non-deleted profiles.
    pub async fn get_network_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<NetworkSummaryRow>> {
        let rows: Vec<NetworkSummaryRow> = sqlx::query_as(
            r#"
            SELECT p.id, p.display_name, p.status,
                   COUNT(DISTINCT c_acc.id) AS accepted_count,
                   COUNT(DISTINCT c_in.id) AS pending_in,
                   COUNT(DISTINCT c_out.id) AS pending_out,
                   COUNT(DISTINCT o.id) AS open_opportunities
            FROM network.profiles p
            LEFT JOIN network.connections c_acc
                   ON c_acc.tenant_id = p.tenant_id AND c_acc.status = 'accepted'
                  AND (c_acc.from_profile_id = p.id OR c_acc.to_profile_id = p.id)
            LEFT JOIN network.connections c_in
                   ON c_in.tenant_id = p.tenant_id AND c_in.status = 'pending'
                  AND c_in.to_profile_id = p.id
            LEFT JOIN network.connections c_out
                   ON c_out.tenant_id = p.tenant_id AND c_out.status = 'pending'
                  AND c_out.from_profile_id = p.id
            LEFT JOIN network.opportunities o
                   ON o.tenant_id = p.tenant_id AND o.owner_profile_id = p.id
                  AND o.status = 'open' AND o.deleted_at IS NULL
            WHERE p.tenant_id = $1 AND p.deleted_at IS NULL
            GROUP BY p.id, p.display_name, p.status, p.created_at
            ORDER BY p.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("network summary: {e}")))?;
        Ok(rows)
    }
}
