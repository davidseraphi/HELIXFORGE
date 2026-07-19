//! HelixGrid Prime durable store — `grid` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSite {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub energized_at: Option<DateTime<Utc>>,
    pub offline_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reading {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GridSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_readings: i64,
    pub draft_readings: i64,
    pub verified_readings: i64,
    pub rejected_readings: i64,
}

#[derive(Debug, Clone, Default)]
pub struct SiteUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct ReadingUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a site lifecycle transition and return the resulting status.
pub fn next_site_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "energize") => Ok("active"),
        ("active", "offline") => Ok("offline"),
        ("offline", "online") => Ok("active"),
        (_, "energize") => Err(HelixError::validation(format!(
            "cannot energize a {current} site"
        ))),
        (_, "offline") => Err(HelixError::validation(format!(
            "cannot take a {current} site offline"
        ))),
        (_, "online") => Err(HelixError::validation(format!(
            "cannot bring a {current} site online"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown site action {action}"
        ))),
    }
}

/// Validate a reading lifecycle transition and return the resulting status.
pub fn next_reading_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "verify") => Ok("verified"),
        ("draft", "reject") | ("verified", "reject") => Ok("rejected"),
        (_, "verify") => Err(HelixError::validation(format!(
            "cannot verify a {current} reading"
        ))),
        (_, "reject") => Err(HelixError::validation(format!(
            "cannot reject a {current} reading"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown reading action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct SiteRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    energized_at: Option<DateTime<Utc>>,
    offline_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl SiteRow {
    fn into_site(self) -> GridSite {
        GridSite {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            energized_at: self.energized_at,
            offline_at: self.offline_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ReadingRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    verified_at: Option<DateTime<Utc>>,
    rejected_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ReadingRow {
    fn into_reading(self) -> Reading {
        Reading {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            verified_at: self.verified_at,
            rejected_at: self.rejected_at,
            deleted_at: self.deleted_at,
        }
    }
}

const SITE_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           energized_at, offline_at, deleted_at
    FROM grid.sites
"#;

const SITE_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              energized_at, offline_at, deleted_at
"#;

const READING_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, verified_at, rejected_at, deleted_at
    FROM grid.readings
"#;

const READING_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, verified_at, rejected_at, deleted_at
"#;

#[derive(Clone)]
pub struct GridRepo {
    pool: PgPool,
}

impl GridRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Sites ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<GridSite>> {
        let rows: Vec<SiteRow> = sqlx::query_as(&format!(
            "{SITE_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid list: {e}")))?;
        Ok(rows.into_iter().map(SiteRow::into_site).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<GridSite> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: SiteRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO grid.sites
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {SITE_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(&metadata)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid create: {e}")))?;
        Ok(row.into_site())
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<GridSite>> {
        let row: Option<SiteRow> = sqlx::query_as(&format!(
            "{SITE_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid get: {e}")))?;
        Ok(row.map(SiteRow::into_site))
    }

    async fn fetch_site_any(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<GridSite>> {
        let row: Option<SiteRow> =
            sqlx::query_as(&format!("{SITE_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("grid fetch site: {e}")))?;
        Ok(row.map(SiteRow::into_site))
    }

    pub async fn update_site(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
        update: SiteUpdate,
    ) -> HelixResult<GridSite> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE grid.sites SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(n) = update.name {
            builder.push(", name = ");
            builder.push_bind(n);
        }
        if let Some(d) = update.description {
            builder.push(", description = ");
            builder.push_bind(d);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(site_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {SITE_RETURNING}"));

        let row: Option<SiteRow> = builder
            .build_query_as::<SiteRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("grid update site: {e}")))?;

        row.map(SiteRow::into_site)
            .ok_or_else(|| HelixError::not_found("site not found"))
    }

    pub async fn energize_site(&self, tenant_id: TenantId, site_id: Uuid) -> HelixResult<GridSite> {
        let site = self
            .get_parent(tenant_id, site_id)
            .await?
            .ok_or_else(|| HelixError::not_found("site not found"))?;
        let next = next_site_status(&site.status, "energize")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<SiteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.sites
            SET status = $1, energized_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {SITE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .bind(&site.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid energize site: {e}")))?;

        row.map(SiteRow::into_site)
            .ok_or_else(|| HelixError::conflict("site changed during energize; retry"))
    }

    /// Take an active site offline. Rejected while draft readings remain.
    /// The active-status and no-draft-readings guards are part of the
    /// UPDATE itself, so a concurrent offline/energize or a reading
    /// created mid-flight cannot slip through a check-then-act window; the
    /// earlier reads only shape the error returned for the steady-state
    /// cases.
    pub async fn take_offline(&self, tenant_id: TenantId, site_id: Uuid) -> HelixResult<GridSite> {
        let site = self
            .get_parent(tenant_id, site_id)
            .await?
            .ok_or_else(|| HelixError::not_found("site not found"))?;
        let next = next_site_status(&site.status, "offline")?;

        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM grid.readings WHERE tenant_id = $1 AND parent_id = $2 AND status = 'draft' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid offline reading count: {e}")))?;
        if drafts > 0 {
            return Err(HelixError::validation(format!(
                "site has {drafts} draft reading(s); verify or reject them first"
            )));
        }

        let now = Utc::now();
        let row: Option<SiteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.sites
            SET status = $1, offline_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = 'active' AND deleted_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM grid.readings r
                  WHERE r.tenant_id = $3 AND r.parent_id = $4
                    AND r.status = 'draft' AND r.deleted_at IS NULL
              )
            {SITE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid take site offline: {e}")))?;

        row.map(SiteRow::into_site).ok_or_else(|| {
            HelixError::conflict("site changed during offline or gained a draft reading; retry")
        })
    }

    pub async fn bring_online(&self, tenant_id: TenantId, site_id: Uuid) -> HelixResult<GridSite> {
        let site = self
            .get_parent(tenant_id, site_id)
            .await?
            .ok_or_else(|| HelixError::not_found("site not found"))?;
        let next = next_site_status(&site.status, "online")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<SiteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.sites
            SET status = $1, offline_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {SITE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .bind(&site.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid bring site online: {e}")))?;

        row.map(SiteRow::into_site)
            .ok_or_else(|| HelixError::conflict("site changed during online; retry"))
    }

    pub async fn soft_delete_site(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
    ) -> HelixResult<GridSite> {
        let site = self
            .get_parent(tenant_id, site_id)
            .await?
            .ok_or_else(|| HelixError::not_found("site not found"))?;
        if site.status == "deleted" {
            return Err(HelixError::validation("site is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<SiteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.sites
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {SITE_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid soft-delete site: {e}")))?;

        row.map(SiteRow::into_site)
            .ok_or_else(|| HelixError::not_found("site not found"))
    }

    /// Restore a soft-deleted site, returning it to its pre-delete status.
    pub async fn restore_site(&self, tenant_id: TenantId, site_id: Uuid) -> HelixResult<GridSite> {
        let site = self
            .fetch_site_any(tenant_id, site_id)
            .await?
            .ok_or_else(|| HelixError::not_found("site not found"))?;
        if site.deleted_at.is_none() {
            return Err(HelixError::validation("site is not deleted"));
        }
        let restored = if site.offline_at.is_some() {
            "offline"
        } else if site.energized_at.is_some() {
            "active"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<SiteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.sites
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {SITE_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid restore site: {e}")))?;

        row.map(SiteRow::into_site)
            .ok_or_else(|| HelixError::not_found("site not found or not deleted"))
    }

    // --- Readings ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Reading>> {
        let rows: Vec<ReadingRow> = sqlx::query_as(&format!(
            "{READING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid list children: {e}")))?;
        Ok(rows.into_iter().map(ReadingRow::into_reading).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Reading> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        // The non-deleted-parent guard is part of the INSERT itself: a site
        // soft-deleted between a separate check and insert cannot leak readings.
        let row: Option<ReadingRow> = sqlx::query_as(&format!(
            r#"
            INSERT INTO grid.readings
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            SELECT $1,$2,$3,$4,$5,'draft',$6,$7,$7
            FROM grid.sites s
            WHERE s.tenant_id = $2 AND s.id = $3 AND s.deleted_at IS NULL
            {READING_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .bind(title)
        .bind(body)
        .bind(&metadata)
        .bind(created_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid create child: {e}")))?;
        row.map(ReadingRow::into_reading)
            .ok_or_else(|| HelixError::not_found("parent not found"))
    }

    pub async fn get_reading(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
        reading_id: Uuid,
    ) -> HelixResult<Option<Reading>> {
        let row: Option<ReadingRow> = sqlx::query_as(&format!(
            "{READING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .bind(reading_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid get reading: {e}")))?;
        Ok(row.map(ReadingRow::into_reading))
    }

    async fn fetch_reading_any(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
        reading_id: Uuid,
    ) -> HelixResult<Option<Reading>> {
        let row: Option<ReadingRow> = sqlx::query_as(&format!(
            "{READING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .bind(reading_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid fetch reading: {e}")))?;
        Ok(row.map(ReadingRow::into_reading))
    }

    pub async fn update_reading(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
        reading_id: Uuid,
        update: ReadingUpdate,
    ) -> HelixResult<Reading> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE grid.readings SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(t) = update.title {
            builder.push(", title = ");
            builder.push_bind(t);
        }
        if let Some(b) = update.body {
            builder.push(", body = ");
            builder.push_bind(b);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND parent_id = ");
        builder.push_bind(site_id);
        builder.push(" AND id = ");
        builder.push_bind(reading_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {READING_RETURNING}"));

        let row: Option<ReadingRow> = builder
            .build_query_as::<ReadingRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("grid update reading: {e}")))?;

        row.map(ReadingRow::into_reading)
            .ok_or_else(|| HelixError::not_found("reading not found"))
    }

    pub async fn verify_reading(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
        reading_id: Uuid,
    ) -> HelixResult<Reading> {
        let reading = self
            .get_reading(tenant_id, site_id, reading_id)
            .await?
            .ok_or_else(|| HelixError::not_found("reading not found"))?;
        let next = next_reading_status(&reading.status, "verify")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<ReadingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.readings
            SET status = $1, verified_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND status = $6 AND deleted_at IS NULL
            {READING_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .bind(reading_id)
        .bind(&reading.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid verify reading: {e}")))?;

        row.map(ReadingRow::into_reading)
            .ok_or_else(|| HelixError::conflict("reading changed during verify; retry"))
    }

    pub async fn reject_reading(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
        reading_id: Uuid,
    ) -> HelixResult<Reading> {
        let reading = self
            .get_reading(tenant_id, site_id, reading_id)
            .await?
            .ok_or_else(|| HelixError::not_found("reading not found"))?;
        let next = next_reading_status(&reading.status, "reject")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<ReadingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.readings
            SET status = $1, rejected_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND status = $6 AND deleted_at IS NULL
            {READING_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .bind(reading_id)
        .bind(&reading.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid reject reading: {e}")))?;

        row.map(ReadingRow::into_reading)
            .ok_or_else(|| HelixError::conflict("reading changed during reject; retry"))
    }

    pub async fn soft_delete_reading(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
        reading_id: Uuid,
    ) -> HelixResult<Reading> {
        let reading = self
            .get_reading(tenant_id, site_id, reading_id)
            .await?
            .ok_or_else(|| HelixError::not_found("reading not found"))?;
        if reading.status == "deleted" {
            return Err(HelixError::validation("reading is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<ReadingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.readings
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {READING_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .bind(reading_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid soft-delete reading: {e}")))?;

        row.map(ReadingRow::into_reading)
            .ok_or_else(|| HelixError::not_found("reading not found"))
    }

    /// Restore a soft-deleted reading, returning it to its pre-delete status.
    pub async fn restore_reading(
        &self,
        tenant_id: TenantId,
        site_id: Uuid,
        reading_id: Uuid,
    ) -> HelixResult<Reading> {
        let reading = self
            .fetch_reading_any(tenant_id, site_id, reading_id)
            .await?
            .ok_or_else(|| HelixError::not_found("reading not found"))?;
        if reading.deleted_at.is_none() {
            return Err(HelixError::validation("reading is not deleted"));
        }
        let restored = if reading.rejected_at.is_some() {
            "rejected"
        } else if reading.verified_at.is_some() {
            "verified"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<ReadingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE grid.readings
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {READING_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(site_id)
        .bind(reading_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid restore reading: {e}")))?;

        row.map(ReadingRow::into_reading)
            .ok_or_else(|| HelixError::not_found("reading not found or not deleted"))
    }

    // --- Reports ---

    /// Per-site reading counts by status for non-deleted sites.
    pub async fn get_grid_summary(&self, tenant_id: TenantId) -> HelixResult<Vec<GridSummaryRow>> {
        let rows: Vec<GridSummaryRow> = sqlx::query_as(
            r#"
            SELECT s.id, s.name, s.status,
                   COUNT(r.id) AS total_readings,
                   COUNT(r.id) FILTER (WHERE r.status = 'draft') AS draft_readings,
                   COUNT(r.id) FILTER (WHERE r.status = 'verified') AS verified_readings,
                   COUNT(r.id) FILTER (WHERE r.status = 'rejected') AS rejected_readings
            FROM grid.sites s
            LEFT JOIN grid.readings r
                   ON r.parent_id = s.id AND r.tenant_id = s.tenant_id
                  AND r.deleted_at IS NULL
            WHERE s.tenant_id = $1 AND s.deleted_at IS NULL
            GROUP BY s.id, s.name, s.status, s.created_at
            ORDER BY s.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("grid summary: {e}")))?;
        Ok(rows)
    }
}
