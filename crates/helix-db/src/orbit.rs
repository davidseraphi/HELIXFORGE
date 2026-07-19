//! HelixOrbit Prime durable store — `orbit` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceAsset {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub commissioned_at: Option<DateTime<Utc>>,
    pub decommissioned_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pass {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub planned_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrbitSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_passes: i64,
    pub draft_passes: i64,
    pub planned_passes: i64,
    pub completed_passes: i64,
    pub cancelled_passes: i64,
}

#[derive(Debug, Clone, Default)]
pub struct AssetUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct PassUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate an asset lifecycle transition and return the resulting status.
pub fn next_asset_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "commission") => Ok("active"),
        ("active", "decommission") => Ok("decommissioned"),
        ("decommissioned", "recommission") => Ok("active"),
        (_, "commission") => Err(HelixError::validation(format!(
            "cannot commission a {current} asset"
        ))),
        (_, "decommission") => Err(HelixError::validation(format!(
            "cannot decommission a {current} asset"
        ))),
        (_, "recommission") => Err(HelixError::validation(format!(
            "cannot recommission a {current} asset"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown asset action {action}"
        ))),
    }
}

/// Validate a pass lifecycle transition and return the resulting status.
pub fn next_pass_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "plan") => Ok("planned"),
        ("planned", "complete") => Ok("completed"),
        ("draft", "cancel") | ("planned", "cancel") => Ok("cancelled"),
        (_, "plan") => Err(HelixError::validation(format!(
            "cannot plan a {current} pass"
        ))),
        (_, "complete") => Err(HelixError::validation(format!(
            "cannot complete a {current} pass"
        ))),
        (_, "cancel") => Err(HelixError::validation(format!(
            "cannot cancel a {current} pass"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown pass action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct AssetRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    commissioned_at: Option<DateTime<Utc>>,
    decommissioned_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl AssetRow {
    fn into_asset(self) -> SpaceAsset {
        SpaceAsset {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            commissioned_at: self.commissioned_at,
            decommissioned_at: self.decommissioned_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PassRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    planned_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl PassRow {
    fn into_pass(self) -> Pass {
        Pass {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            planned_at: self.planned_at,
            completed_at: self.completed_at,
            cancelled_at: self.cancelled_at,
            deleted_at: self.deleted_at,
        }
    }
}

const ASSET_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           commissioned_at, decommissioned_at, deleted_at
    FROM orbit.assets
"#;

const ASSET_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              commissioned_at, decommissioned_at, deleted_at
"#;

const PASS_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, planned_at, completed_at, cancelled_at, deleted_at
    FROM orbit.passes
"#;

const PASS_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, planned_at, completed_at, cancelled_at, deleted_at
"#;

#[derive(Clone)]
pub struct OrbitRepo {
    pool: PgPool,
}

impl OrbitRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Assets ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<SpaceAsset>> {
        let rows: Vec<AssetRow> = sqlx::query_as(&format!(
            "{ASSET_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit list: {e}")))?;
        Ok(rows.into_iter().map(AssetRow::into_asset).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<SpaceAsset> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: AssetRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO orbit.assets
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {ASSET_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("orbit create: {e}")))?;
        Ok(row.into_asset())
    }

    pub async fn get_parent(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<SpaceAsset>> {
        let row: Option<AssetRow> = sqlx::query_as(&format!(
            "{ASSET_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit get: {e}")))?;
        Ok(row.map(AssetRow::into_asset))
    }

    async fn fetch_asset_any(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<SpaceAsset>> {
        let row: Option<AssetRow> =
            sqlx::query_as(&format!("{ASSET_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("orbit fetch asset: {e}")))?;
        Ok(row.map(AssetRow::into_asset))
    }

    pub async fn update_asset(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        update: AssetUpdate,
    ) -> HelixResult<SpaceAsset> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE orbit.assets SET updated_at = ");
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
        builder.push_bind(asset_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {ASSET_RETURNING}"));

        let row: Option<AssetRow> = builder
            .build_query_as::<AssetRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("orbit update asset: {e}")))?;

        row.map(AssetRow::into_asset)
            .ok_or_else(|| HelixError::not_found("asset not found"))
    }

    pub async fn commission_asset(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
    ) -> HelixResult<SpaceAsset> {
        let asset = self
            .get_parent(tenant_id, asset_id)
            .await?
            .ok_or_else(|| HelixError::not_found("asset not found"))?;
        let next = next_asset_status(&asset.status, "commission")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<AssetRow> = sqlx::query_as(&format!(
            r#"
            UPDATE orbit.assets
            SET status = $1, commissioned_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {ASSET_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .bind(&asset.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit commission asset: {e}")))?;

        row.map(AssetRow::into_asset)
            .ok_or_else(|| HelixError::conflict("asset changed during commission; retry"))
    }

    /// Decommission an active asset. Rejected while draft or planned passes
    /// remain. The active-status and no-open-passes guards are part of the
    /// UPDATE itself, so a concurrent decommission or a pass created
    /// mid-flight cannot slip through a check-then-act window; the earlier
    /// reads only shape the error returned for the steady-state cases.
    pub async fn decommission_asset(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
    ) -> HelixResult<SpaceAsset> {
        let asset = self
            .get_parent(tenant_id, asset_id)
            .await?
            .ok_or_else(|| HelixError::not_found("asset not found"))?;
        let next = next_asset_status(&asset.status, "decommission")?;

        let open_passes: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM orbit.passes WHERE tenant_id = $1 AND parent_id = $2 AND status IN ('draft','planned') AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit decommission pass count: {e}")))?;
        if open_passes > 0 {
            return Err(HelixError::validation(format!(
                "asset has {open_passes} draft or planned pass(es); complete or cancel them first"
            )));
        }

        let now = Utc::now();
        let row: Option<AssetRow> = sqlx::query_as(&format!(
            r#"
            UPDATE orbit.assets
            SET status = $1, decommissioned_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = 'active' AND deleted_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM orbit.passes p
                  WHERE p.tenant_id = $3 AND p.parent_id = $4
                    AND p.status IN ('draft','planned') AND p.deleted_at IS NULL
              )
            {ASSET_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit decommission asset: {e}")))?;

        row.map(AssetRow::into_asset).ok_or_else(|| {
            HelixError::conflict("asset changed during decommission or gained an open pass; retry")
        })
    }

    pub async fn recommission_asset(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
    ) -> HelixResult<SpaceAsset> {
        let asset = self
            .get_parent(tenant_id, asset_id)
            .await?
            .ok_or_else(|| HelixError::not_found("asset not found"))?;
        let next = next_asset_status(&asset.status, "recommission")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<AssetRow> = sqlx::query_as(&format!(
            r#"
            UPDATE orbit.assets
            SET status = $1, decommissioned_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {ASSET_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .bind(&asset.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit recommission asset: {e}")))?;

        row.map(AssetRow::into_asset)
            .ok_or_else(|| HelixError::conflict("asset changed during recommission; retry"))
    }

    pub async fn soft_delete_asset(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
    ) -> HelixResult<SpaceAsset> {
        let asset = self
            .get_parent(tenant_id, asset_id)
            .await?
            .ok_or_else(|| HelixError::not_found("asset not found"))?;
        if asset.status == "deleted" {
            return Err(HelixError::validation("asset is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<AssetRow> = sqlx::query_as(&format!(
            r#"
            UPDATE orbit.assets
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {ASSET_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit soft-delete asset: {e}")))?;

        row.map(AssetRow::into_asset)
            .ok_or_else(|| HelixError::not_found("asset not found"))
    }

    /// Restore a soft-deleted asset, returning it to its pre-delete status.
    pub async fn restore_asset(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
    ) -> HelixResult<SpaceAsset> {
        let asset = self
            .fetch_asset_any(tenant_id, asset_id)
            .await?
            .ok_or_else(|| HelixError::not_found("asset not found"))?;
        if asset.deleted_at.is_none() {
            return Err(HelixError::validation("asset is not deleted"));
        }
        let restored = if asset.decommissioned_at.is_some() {
            "decommissioned"
        } else if asset.commissioned_at.is_some() {
            "active"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<AssetRow> = sqlx::query_as(&format!(
            r#"
            UPDATE orbit.assets
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {ASSET_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit restore asset: {e}")))?;

        row.map(AssetRow::into_asset)
            .ok_or_else(|| HelixError::not_found("asset not found or not deleted"))
    }

    // --- Passes ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<Pass>> {
        let rows: Vec<PassRow> = sqlx::query_as(&format!(
            "{PASS_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit list children: {e}")))?;
        Ok(rows.into_iter().map(PassRow::into_pass).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Pass> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        // The non-deleted-parent guard is part of the INSERT itself: an asset
        // soft-deleted between a separate check and insert cannot leak passes.
        let row: Option<PassRow> = sqlx::query_as(&format!(
            r#"
            INSERT INTO orbit.passes
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            SELECT $1,$2,$3,$4,$5,'draft',$6,$7,$7
            FROM orbit.assets a
            WHERE a.tenant_id = $2 AND a.id = $3 AND a.deleted_at IS NULL
            {PASS_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("orbit create child: {e}")))?;
        row.map(PassRow::into_pass)
            .ok_or_else(|| HelixError::not_found("parent not found"))
    }

    pub async fn get_pass(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
    ) -> HelixResult<Option<Pass>> {
        let row: Option<PassRow> = sqlx::query_as(&format!(
            "{PASS_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .bind(pass_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit get pass: {e}")))?;
        Ok(row.map(PassRow::into_pass))
    }

    async fn fetch_pass_any(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
    ) -> HelixResult<Option<Pass>> {
        let row: Option<PassRow> = sqlx::query_as(&format!(
            "{PASS_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .bind(pass_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit fetch pass: {e}")))?;
        Ok(row.map(PassRow::into_pass))
    }

    pub async fn update_pass(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
        update: PassUpdate,
    ) -> HelixResult<Pass> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE orbit.passes SET updated_at = ");
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
        builder.push_bind(asset_id);
        builder.push(" AND id = ");
        builder.push_bind(pass_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {PASS_RETURNING}"));

        let row: Option<PassRow> = builder
            .build_query_as::<PassRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("orbit update pass: {e}")))?;

        row.map(PassRow::into_pass)
            .ok_or_else(|| HelixError::not_found("pass not found"))
    }

    async fn transition_pass(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
        action: &str,
    ) -> HelixResult<Pass> {
        let pass = self
            .get_pass(tenant_id, asset_id, pass_id)
            .await?
            .ok_or_else(|| HelixError::not_found("pass not found"))?;
        let next = next_pass_status(&pass.status, action)?;
        let now = Utc::now();
        let (planned_at, completed_at, cancelled_at) = match next {
            "planned" => (Some(now), None, None),
            "completed" => (pass.planned_at, Some(now), None),
            "cancelled" => (pass.planned_at, None, Some(now)),
            _ => (pass.planned_at, pass.completed_at, pass.cancelled_at),
        };
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<PassRow> = sqlx::query_as(&format!(
            r#"
            UPDATE orbit.passes
            SET status = $1, planned_at = $2, completed_at = $3, cancelled_at = $4, updated_at = $5
            WHERE tenant_id = $6 AND parent_id = $7 AND id = $8 AND status = $9 AND deleted_at IS NULL
            {PASS_RETURNING}
            "#
        ))
        .bind(next)
        .bind(planned_at)
        .bind(completed_at)
        .bind(cancelled_at)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .bind(pass_id)
        .bind(&pass.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit {action} pass: {e}")))?;

        row.map(PassRow::into_pass)
            .ok_or_else(|| HelixError::conflict("pass changed during transition; retry"))
    }

    pub async fn plan_pass(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
    ) -> HelixResult<Pass> {
        self.transition_pass(tenant_id, asset_id, pass_id, "plan")
            .await
    }

    pub async fn complete_pass(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
    ) -> HelixResult<Pass> {
        self.transition_pass(tenant_id, asset_id, pass_id, "complete")
            .await
    }

    pub async fn cancel_pass(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
    ) -> HelixResult<Pass> {
        self.transition_pass(tenant_id, asset_id, pass_id, "cancel")
            .await
    }

    pub async fn soft_delete_pass(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
    ) -> HelixResult<Pass> {
        let pass = self
            .get_pass(tenant_id, asset_id, pass_id)
            .await?
            .ok_or_else(|| HelixError::not_found("pass not found"))?;
        if pass.status == "deleted" {
            return Err(HelixError::validation("pass is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<PassRow> = sqlx::query_as(&format!(
            r#"
            UPDATE orbit.passes
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {PASS_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .bind(pass_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit soft-delete pass: {e}")))?;

        row.map(PassRow::into_pass)
            .ok_or_else(|| HelixError::not_found("pass not found"))
    }

    /// Restore a soft-deleted pass, returning it to its pre-delete status.
    pub async fn restore_pass(
        &self,
        tenant_id: TenantId,
        asset_id: Uuid,
        pass_id: Uuid,
    ) -> HelixResult<Pass> {
        let pass = self
            .fetch_pass_any(tenant_id, asset_id, pass_id)
            .await?
            .ok_or_else(|| HelixError::not_found("pass not found"))?;
        if pass.deleted_at.is_none() {
            return Err(HelixError::validation("pass is not deleted"));
        }
        let restored = if pass.cancelled_at.is_some() {
            "cancelled"
        } else if pass.completed_at.is_some() {
            "completed"
        } else if pass.planned_at.is_some() {
            "planned"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<PassRow> = sqlx::query_as(&format!(
            r#"
            UPDATE orbit.passes
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {PASS_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(asset_id)
        .bind(pass_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("orbit restore pass: {e}")))?;

        row.map(PassRow::into_pass)
            .ok_or_else(|| HelixError::not_found("pass not found or not deleted"))
    }

    // --- Reports ---

    /// Per-asset pass counts by status for non-deleted assets.
    pub async fn get_orbit_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<OrbitSummaryRow>> {
        let rows: Vec<OrbitSummaryRow> = sqlx::query_as(
            r#"
            SELECT a.id, a.name, a.status,
                   COUNT(p.id) AS total_passes,
                   COUNT(p.id) FILTER (WHERE p.status = 'draft') AS draft_passes,
                   COUNT(p.id) FILTER (WHERE p.status = 'planned') AS planned_passes,
                   COUNT(p.id) FILTER (WHERE p.status = 'completed') AS completed_passes,
                   COUNT(p.id) FILTER (WHERE p.status = 'cancelled') AS cancelled_passes
            FROM orbit.assets a
            LEFT JOIN orbit.passes p
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
        .map_err(|e| HelixError::dependency(format!("orbit summary: {e}")))?;
        Ok(rows)
    }
}
