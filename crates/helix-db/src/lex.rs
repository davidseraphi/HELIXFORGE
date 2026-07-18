//! HelixLex Prime durable store — `lex` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matter {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub opened_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filing {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub filed_at: Option<DateTime<Utc>>,
    pub withdrawn_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LexSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_filings: i64,
    pub draft_filings: i64,
    pub filed_filings: i64,
    pub withdrawn_filings: i64,
}

#[derive(Debug, Clone, Default)]
pub struct MatterUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct FilingUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a matter lifecycle transition and return the resulting status.
pub fn next_matter_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "open") => Ok("open"),
        ("open", "close") => Ok("closed"),
        ("closed", "reopen") => Ok("open"),
        (_, "open") => Err(HelixError::validation(format!(
            "cannot open a {current} matter"
        ))),
        (_, "close") => Err(HelixError::validation(format!(
            "cannot close a {current} matter"
        ))),
        (_, "reopen") => Err(HelixError::validation(format!(
            "cannot reopen a {current} matter"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown matter action {action}"
        ))),
    }
}

/// Validate a filing lifecycle transition and return the resulting status.
pub fn next_filing_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "file") => Ok("filed"),
        ("draft", "withdraw") | ("filed", "withdraw") => Ok("withdrawn"),
        (_, "file") => Err(HelixError::validation(format!(
            "cannot file a {current} filing"
        ))),
        (_, "withdraw") => Err(HelixError::validation(format!(
            "cannot withdraw a {current} filing"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown filing action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct MatterRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    opened_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl MatterRow {
    fn into_matter(self) -> Matter {
        Matter {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            opened_at: self.opened_at,
            closed_at: self.closed_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FilingRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    filed_at: Option<DateTime<Utc>>,
    withdrawn_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl FilingRow {
    fn into_filing(self) -> Filing {
        Filing {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            filed_at: self.filed_at,
            withdrawn_at: self.withdrawn_at,
            deleted_at: self.deleted_at,
        }
    }
}

const MATTER_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           opened_at, closed_at, deleted_at
    FROM lex.matters
"#;

const MATTER_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              opened_at, closed_at, deleted_at
"#;

const FILING_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, filed_at, withdrawn_at, deleted_at
    FROM lex.filings
"#;

const FILING_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, filed_at, withdrawn_at, deleted_at
"#;

#[derive(Clone)]
pub struct LexRepo {
    pool: PgPool,
}

impl LexRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Matters ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<Matter>> {
        let rows: Vec<MatterRow> = sqlx::query_as(&format!(
            "{MATTER_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex list: {e}")))?;
        Ok(rows.into_iter().map(MatterRow::into_matter).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Matter> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: MatterRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO lex.matters
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {MATTER_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("lex create: {e}")))?;
        Ok(row.into_matter())
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Matter>> {
        let row: Option<MatterRow> = sqlx::query_as(&format!(
            "{MATTER_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex get: {e}")))?;
        Ok(row.map(MatterRow::into_matter))
    }

    async fn fetch_matter_any(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Matter>> {
        let row: Option<MatterRow> =
            sqlx::query_as(&format!("{MATTER_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("lex fetch matter: {e}")))?;
        Ok(row.map(MatterRow::into_matter))
    }

    pub async fn update_matter(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
        update: MatterUpdate,
    ) -> HelixResult<Matter> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE lex.matters SET updated_at = ");
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
        builder.push_bind(matter_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {MATTER_RETURNING}"));

        let row: Option<MatterRow> = builder
            .build_query_as::<MatterRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("lex update matter: {e}")))?;

        row.map(MatterRow::into_matter)
            .ok_or_else(|| HelixError::not_found("matter not found"))
    }

    pub async fn open_matter(&self, tenant_id: TenantId, matter_id: Uuid) -> HelixResult<Matter> {
        let matter = self
            .get_parent(tenant_id, matter_id)
            .await?
            .ok_or_else(|| HelixError::not_found("matter not found"))?;
        let next = next_matter_status(&matter.status, "open")?;
        let now = Utc::now();
        let row: Option<MatterRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.matters
            SET status = $1, opened_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {MATTER_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex open matter: {e}")))?;

        row.map(MatterRow::into_matter)
            .ok_or_else(|| HelixError::not_found("matter not found"))
    }

    /// Close an open matter. Rejected while draft filings remain.
    pub async fn close_matter(&self, tenant_id: TenantId, matter_id: Uuid) -> HelixResult<Matter> {
        let matter = self
            .get_parent(tenant_id, matter_id)
            .await?
            .ok_or_else(|| HelixError::not_found("matter not found"))?;
        let next = next_matter_status(&matter.status, "close")?;

        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM lex.filings WHERE tenant_id = $1 AND parent_id = $2 AND status = 'draft' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex close filing count: {e}")))?;
        if drafts > 0 {
            return Err(HelixError::validation(format!(
                "matter has {drafts} draft filing(s); file or withdraw them first"
            )));
        }

        let now = Utc::now();
        let row: Option<MatterRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.matters
            SET status = $1, closed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {MATTER_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex close matter: {e}")))?;

        row.map(MatterRow::into_matter)
            .ok_or_else(|| HelixError::not_found("matter not found"))
    }

    pub async fn reopen_matter(&self, tenant_id: TenantId, matter_id: Uuid) -> HelixResult<Matter> {
        let matter = self
            .get_parent(tenant_id, matter_id)
            .await?
            .ok_or_else(|| HelixError::not_found("matter not found"))?;
        let next = next_matter_status(&matter.status, "reopen")?;
        let now = Utc::now();
        let row: Option<MatterRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.matters
            SET status = $1, closed_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {MATTER_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex reopen matter: {e}")))?;

        row.map(MatterRow::into_matter)
            .ok_or_else(|| HelixError::not_found("matter not found"))
    }

    pub async fn soft_delete_matter(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
    ) -> HelixResult<Matter> {
        let matter = self
            .get_parent(tenant_id, matter_id)
            .await?
            .ok_or_else(|| HelixError::not_found("matter not found"))?;
        if matter.status == "deleted" {
            return Err(HelixError::validation("matter is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<MatterRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.matters
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {MATTER_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex soft-delete matter: {e}")))?;

        row.map(MatterRow::into_matter)
            .ok_or_else(|| HelixError::not_found("matter not found"))
    }

    /// Restore a soft-deleted matter, returning it to its pre-delete status.
    pub async fn restore_matter(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
    ) -> HelixResult<Matter> {
        let matter = self
            .fetch_matter_any(tenant_id, matter_id)
            .await?
            .ok_or_else(|| HelixError::not_found("matter not found"))?;
        if matter.deleted_at.is_none() {
            return Err(HelixError::validation("matter is not deleted"));
        }
        let restored = if matter.closed_at.is_some() {
            "closed"
        } else if matter.opened_at.is_some() {
            "open"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<MatterRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.matters
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {MATTER_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex restore matter: {e}")))?;

        row.map(MatterRow::into_matter)
            .ok_or_else(|| HelixError::not_found("matter not found or not deleted"))
    }

    // --- Filings ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Filing>> {
        let rows: Vec<FilingRow> = sqlx::query_as(&format!(
            "{FILING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex list children: {e}")))?;
        Ok(rows.into_iter().map(FilingRow::into_filing).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Filing> {
        let _parent = self
            .get_parent(tenant_id, parent_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: FilingRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO lex.filings
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,'draft',$6,$7,$7)
            {FILING_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .bind(title)
        .bind(body)
        .bind(&metadata)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex create child: {e}")))?;
        Ok(row.into_filing())
    }

    pub async fn get_filing(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
        filing_id: Uuid,
    ) -> HelixResult<Option<Filing>> {
        let row: Option<FilingRow> = sqlx::query_as(&format!(
            "{FILING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .bind(filing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex get filing: {e}")))?;
        Ok(row.map(FilingRow::into_filing))
    }

    async fn fetch_filing_any(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
        filing_id: Uuid,
    ) -> HelixResult<Option<Filing>> {
        let row: Option<FilingRow> = sqlx::query_as(&format!(
            "{FILING_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .bind(filing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex fetch filing: {e}")))?;
        Ok(row.map(FilingRow::into_filing))
    }

    pub async fn update_filing(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
        filing_id: Uuid,
        update: FilingUpdate,
    ) -> HelixResult<Filing> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE lex.filings SET updated_at = ");
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
        builder.push_bind(matter_id);
        builder.push(" AND id = ");
        builder.push_bind(filing_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {FILING_RETURNING}"));

        let row: Option<FilingRow> = builder
            .build_query_as::<FilingRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("lex update filing: {e}")))?;

        row.map(FilingRow::into_filing)
            .ok_or_else(|| HelixError::not_found("filing not found"))
    }

    pub async fn file_filing(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
        filing_id: Uuid,
    ) -> HelixResult<Filing> {
        let filing = self
            .get_filing(tenant_id, matter_id, filing_id)
            .await?
            .ok_or_else(|| HelixError::not_found("filing not found"))?;
        let next = next_filing_status(&filing.status, "file")?;
        let now = Utc::now();
        let row: Option<FilingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.filings
            SET status = $1, filed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NULL
            {FILING_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .bind(filing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex file filing: {e}")))?;

        row.map(FilingRow::into_filing)
            .ok_or_else(|| HelixError::not_found("filing not found"))
    }

    pub async fn withdraw_filing(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
        filing_id: Uuid,
    ) -> HelixResult<Filing> {
        let filing = self
            .get_filing(tenant_id, matter_id, filing_id)
            .await?
            .ok_or_else(|| HelixError::not_found("filing not found"))?;
        let next = next_filing_status(&filing.status, "withdraw")?;
        let now = Utc::now();
        let row: Option<FilingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.filings
            SET status = $1, withdrawn_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NULL
            {FILING_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .bind(filing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex withdraw filing: {e}")))?;

        row.map(FilingRow::into_filing)
            .ok_or_else(|| HelixError::not_found("filing not found"))
    }

    pub async fn soft_delete_filing(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
        filing_id: Uuid,
    ) -> HelixResult<Filing> {
        let filing = self
            .get_filing(tenant_id, matter_id, filing_id)
            .await?
            .ok_or_else(|| HelixError::not_found("filing not found"))?;
        if filing.status == "deleted" {
            return Err(HelixError::validation("filing is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<FilingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.filings
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {FILING_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .bind(filing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex soft-delete filing: {e}")))?;

        row.map(FilingRow::into_filing)
            .ok_or_else(|| HelixError::not_found("filing not found"))
    }

    /// Restore a soft-deleted filing, returning it to its pre-delete status.
    pub async fn restore_filing(
        &self,
        tenant_id: TenantId,
        matter_id: Uuid,
        filing_id: Uuid,
    ) -> HelixResult<Filing> {
        let filing = self
            .fetch_filing_any(tenant_id, matter_id, filing_id)
            .await?
            .ok_or_else(|| HelixError::not_found("filing not found"))?;
        if filing.deleted_at.is_none() {
            return Err(HelixError::validation("filing is not deleted"));
        }
        let restored = if filing.withdrawn_at.is_some() {
            "withdrawn"
        } else if filing.filed_at.is_some() {
            "filed"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<FilingRow> = sqlx::query_as(&format!(
            r#"
            UPDATE lex.filings
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {FILING_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(matter_id)
        .bind(filing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex restore filing: {e}")))?;

        row.map(FilingRow::into_filing)
            .ok_or_else(|| HelixError::not_found("filing not found or not deleted"))
    }

    // --- Reports ---

    /// Per-matter filing counts by status for non-deleted matters.
    pub async fn get_lex_summary(&self, tenant_id: TenantId) -> HelixResult<Vec<LexSummaryRow>> {
        let rows: Vec<LexSummaryRow> = sqlx::query_as(
            r#"
            SELECT m.id, m.name, m.status,
                   COUNT(f.id) AS total_filings,
                   COUNT(f.id) FILTER (WHERE f.status = 'draft') AS draft_filings,
                   COUNT(f.id) FILTER (WHERE f.status = 'filed') AS filed_filings,
                   COUNT(f.id) FILTER (WHERE f.status = 'withdrawn') AS withdrawn_filings
            FROM lex.matters m
            LEFT JOIN lex.filings f
                   ON f.parent_id = m.id AND f.tenant_id = m.tenant_id
                  AND f.deleted_at IS NULL
            WHERE m.tenant_id = $1 AND m.deleted_at IS NULL
            GROUP BY m.id, m.name, m.status, m.created_at
            ORDER BY m.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("lex summary: {e}")))?;
        Ok(rows)
    }
}
