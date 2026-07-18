//! HelixForge Studio durable store — `studio` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StudioSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_pages: i64,
    pub open_pages: i64,
    pub archived_pages: i64,
}

#[derive(Debug, Clone, Default)]
pub struct AppUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct PageUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate an app lifecycle transition and return the resulting status.
pub fn next_app_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "publish") => Ok("published"),
        ("published", "unpublish") => Ok("draft"),
        (_, "publish") => Err(HelixError::validation(format!(
            "cannot publish a {current} app"
        ))),
        (_, "unpublish") => Err(HelixError::validation(format!(
            "cannot unpublish a {current} app"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown app action {action}"
        ))),
    }
}

/// Validate a page lifecycle transition and return the resulting status.
pub fn next_page_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("open", "archive") => Ok("archived"),
        ("archived", "reopen") => Ok("open"),
        (_, "archive") => Err(HelixError::validation(format!(
            "cannot archive a {current} page"
        ))),
        (_, "reopen") => Err(HelixError::validation(format!(
            "cannot reopen a {current} page"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown page action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct AppRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl AppRow {
    fn into_app(self) -> App {
        App {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            published_at: self.published_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PageRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    archived_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl PageRow {
    fn into_page(self) -> Page {
        Page {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            archived_at: self.archived_at,
            deleted_at: self.deleted_at,
        }
    }
}

const APP_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           published_at, deleted_at
    FROM studio.apps
"#;

const APP_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              published_at, deleted_at
"#;

const PAGE_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, archived_at, deleted_at
    FROM studio.pages
"#;

const PAGE_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, archived_at, deleted_at
"#;

#[derive(Clone)]
pub struct StudioRepo {
    pool: PgPool,
}

impl StudioRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Apps ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<App>> {
        let rows: Vec<AppRow> = sqlx::query_as(&format!(
            "{APP_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio list: {e}")))?;
        Ok(rows.into_iter().map(AppRow::into_app).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<App> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: AppRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO studio.apps
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {APP_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("studio create: {e}")))?;
        Ok(row.into_app())
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<App>> {
        let row: Option<AppRow> = sqlx::query_as(&format!(
            "{APP_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio get: {e}")))?;
        Ok(row.map(AppRow::into_app))
    }

    async fn fetch_app_any(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<App>> {
        let row: Option<AppRow> =
            sqlx::query_as(&format!("{APP_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("studio fetch app: {e}")))?;
        Ok(row.map(AppRow::into_app))
    }

    pub async fn update_app(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        update: AppUpdate,
    ) -> HelixResult<App> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE studio.apps SET updated_at = ");
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
        builder.push_bind(app_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {APP_RETURNING}"));

        let row: Option<AppRow> = builder
            .build_query_as::<AppRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("studio update app: {e}")))?;

        row.map(AppRow::into_app)
            .ok_or_else(|| HelixError::not_found("app not found"))
    }

    /// Publish a draft app. Requires at least one non-deleted page.
    pub async fn publish_app(&self, tenant_id: TenantId, app_id: Uuid) -> HelixResult<App> {
        let app = self
            .get_parent(tenant_id, app_id)
            .await?
            .ok_or_else(|| HelixError::not_found("app not found"))?;
        let next = next_app_status(&app.status, "publish")?;

        let page_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM studio.pages WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio publish page count: {e}")))?;
        if page_count == 0 {
            return Err(HelixError::validation(
                "app needs at least one page to publish",
            ));
        }

        let now = Utc::now();
        let row: Option<AppRow> = sqlx::query_as(&format!(
            r#"
            UPDATE studio.apps
            SET status = $1, published_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {APP_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio publish app: {e}")))?;

        row.map(AppRow::into_app)
            .ok_or_else(|| HelixError::not_found("app not found"))
    }

    pub async fn unpublish_app(&self, tenant_id: TenantId, app_id: Uuid) -> HelixResult<App> {
        let app = self
            .get_parent(tenant_id, app_id)
            .await?
            .ok_or_else(|| HelixError::not_found("app not found"))?;
        let next = next_app_status(&app.status, "unpublish")?;
        let now = Utc::now();
        let row: Option<AppRow> = sqlx::query_as(&format!(
            r#"
            UPDATE studio.apps
            SET status = $1, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {APP_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio unpublish app: {e}")))?;

        row.map(AppRow::into_app)
            .ok_or_else(|| HelixError::not_found("app not found"))
    }

    pub async fn soft_delete_app(&self, tenant_id: TenantId, app_id: Uuid) -> HelixResult<App> {
        let app = self
            .get_parent(tenant_id, app_id)
            .await?
            .ok_or_else(|| HelixError::not_found("app not found"))?;
        if app.status == "deleted" {
            return Err(HelixError::validation("app is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<AppRow> = sqlx::query_as(&format!(
            r#"
            UPDATE studio.apps
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {APP_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio soft-delete app: {e}")))?;

        row.map(AppRow::into_app)
            .ok_or_else(|| HelixError::not_found("app not found"))
    }

    /// Restore a soft-deleted app, returning it to its pre-delete status.
    pub async fn restore_app(&self, tenant_id: TenantId, app_id: Uuid) -> HelixResult<App> {
        let app = self
            .fetch_app_any(tenant_id, app_id)
            .await?
            .ok_or_else(|| HelixError::not_found("app not found"))?;
        if app.deleted_at.is_none() {
            return Err(HelixError::validation("app is not deleted"));
        }
        let restored = if app.published_at.is_some() {
            "published"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<AppRow> = sqlx::query_as(&format!(
            r#"
            UPDATE studio.apps
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {APP_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio restore app: {e}")))?;

        row.map(AppRow::into_app)
            .ok_or_else(|| HelixError::not_found("app not found or not deleted"))
    }

    // --- Pages ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Page>> {
        let rows: Vec<PageRow> = sqlx::query_as(&format!(
            "{PAGE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio list children: {e}")))?;
        Ok(rows.into_iter().map(PageRow::into_page).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Page> {
        let _parent = self
            .get_parent(tenant_id, parent_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: PageRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO studio.pages
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,'open',$6,$7,$7)
            {PAGE_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("studio create child: {e}")))?;
        Ok(row.into_page())
    }

    pub async fn get_page(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        page_id: Uuid,
    ) -> HelixResult<Option<Page>> {
        let row: Option<PageRow> = sqlx::query_as(&format!(
            "{PAGE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio get page: {e}")))?;
        Ok(row.map(PageRow::into_page))
    }

    async fn fetch_page_any(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        page_id: Uuid,
    ) -> HelixResult<Option<Page>> {
        let row: Option<PageRow> = sqlx::query_as(&format!(
            "{PAGE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio fetch page: {e}")))?;
        Ok(row.map(PageRow::into_page))
    }

    pub async fn update_page(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        page_id: Uuid,
        update: PageUpdate,
    ) -> HelixResult<Page> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE studio.pages SET updated_at = ");
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
        builder.push_bind(app_id);
        builder.push(" AND id = ");
        builder.push_bind(page_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {PAGE_RETURNING}"));

        let row: Option<PageRow> = builder
            .build_query_as::<PageRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("studio update page: {e}")))?;

        row.map(PageRow::into_page)
            .ok_or_else(|| HelixError::not_found("page not found"))
    }

    async fn transition_page(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        page_id: Uuid,
        action: &str,
    ) -> HelixResult<Page> {
        let page = self
            .get_page(tenant_id, app_id, page_id)
            .await?
            .ok_or_else(|| HelixError::not_found("page not found"))?;
        let next = next_page_status(&page.status, action)?;
        let now = Utc::now();
        let archived_at = if next == "archived" { Some(now) } else { None };
        let row: Option<PageRow> = sqlx::query_as(&format!(
            r#"
            UPDATE studio.pages
            SET status = $1, archived_at = $2, updated_at = $3
            WHERE tenant_id = $4 AND parent_id = $5 AND id = $6 AND deleted_at IS NULL
            {PAGE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(archived_at)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio {action} page: {e}")))?;

        row.map(PageRow::into_page)
            .ok_or_else(|| HelixError::not_found("page not found"))
    }

    pub async fn archive_page(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        page_id: Uuid,
    ) -> HelixResult<Page> {
        self.transition_page(tenant_id, app_id, page_id, "archive")
            .await
    }

    pub async fn reopen_page(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        page_id: Uuid,
    ) -> HelixResult<Page> {
        self.transition_page(tenant_id, app_id, page_id, "reopen")
            .await
    }

    pub async fn soft_delete_page(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        page_id: Uuid,
    ) -> HelixResult<Page> {
        let page = self
            .get_page(tenant_id, app_id, page_id)
            .await?
            .ok_or_else(|| HelixError::not_found("page not found"))?;
        if page.status == "deleted" {
            return Err(HelixError::validation("page is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<PageRow> = sqlx::query_as(&format!(
            r#"
            UPDATE studio.pages
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {PAGE_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio soft-delete page: {e}")))?;

        row.map(PageRow::into_page)
            .ok_or_else(|| HelixError::not_found("page not found"))
    }

    /// Restore a soft-deleted page, returning it to its pre-delete status.
    pub async fn restore_page(
        &self,
        tenant_id: TenantId,
        app_id: Uuid,
        page_id: Uuid,
    ) -> HelixResult<Page> {
        let page = self
            .fetch_page_any(tenant_id, app_id, page_id)
            .await?
            .ok_or_else(|| HelixError::not_found("page not found"))?;
        if page.deleted_at.is_none() {
            return Err(HelixError::validation("page is not deleted"));
        }
        let restored = if page.archived_at.is_some() {
            "archived"
        } else {
            "open"
        };
        let now = Utc::now();
        let row: Option<PageRow> = sqlx::query_as(&format!(
            r#"
            UPDATE studio.pages
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {PAGE_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(app_id)
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio restore page: {e}")))?;

        row.map(PageRow::into_page)
            .ok_or_else(|| HelixError::not_found("page not found or not deleted"))
    }

    // --- Reports ---

    /// Per-app page counts for non-deleted apps.
    pub async fn get_studio_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<StudioSummaryRow>> {
        let rows: Vec<StudioSummaryRow> = sqlx::query_as(
            r#"
            SELECT a.id, a.name, a.status,
                   COUNT(p.id) AS total_pages,
                   COUNT(p.id) FILTER (WHERE p.status = 'open') AS open_pages,
                   COUNT(p.id) FILTER (WHERE p.status = 'archived') AS archived_pages
            FROM studio.apps a
            LEFT JOIN studio.pages p
                   ON p.parent_id = a.id AND p.tenant_id = a.tenant_id
                  AND p.deleted_at IS NULL
            WHERE a.tenant_id = $1 AND a.deleted_at IS NULL
            GROUP BY a.id, a.name, a.status, a.created_at
            ORDER BY a.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("studio summary: {e}")))?;
        Ok(rows)
    }
}
