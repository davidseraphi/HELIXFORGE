//! HelixClimate Prime durable store — `climate` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub assessed_at: Option<DateTime<Utc>>,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClimateSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_scores: i64,
    pub draft_scores: i64,
    pub assessed_scores: i64,
    pub dismissed_scores: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ScenarioUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct ScoreUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a scenario lifecycle transition and return the resulting status.
pub fn next_scenario_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "activate") => Ok("active"),
        ("active", "archive") => Ok("archived"),
        ("archived", "reopen") => Ok("active"),
        (_, "activate") => Err(HelixError::validation(format!(
            "cannot activate a {current} scenario"
        ))),
        (_, "archive") => Err(HelixError::validation(format!(
            "cannot archive a {current} scenario"
        ))),
        (_, "reopen") => Err(HelixError::validation(format!(
            "cannot reopen a {current} scenario"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown scenario action {action}"
        ))),
    }
}

/// Validate a risk-score lifecycle transition and return the resulting status.
pub fn next_score_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "assess") => Ok("assessed"),
        ("draft", "dismiss") | ("assessed", "dismiss") => Ok("dismissed"),
        (_, "assess") => Err(HelixError::validation(format!(
            "cannot assess a {current} score"
        ))),
        (_, "dismiss") => Err(HelixError::validation(format!(
            "cannot dismiss a {current} score"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown score action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct ScenarioRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    activated_at: Option<DateTime<Utc>>,
    archived_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ScenarioRow {
    fn into_scenario(self) -> Scenario {
        Scenario {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            activated_at: self.activated_at,
            archived_at: self.archived_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ScoreRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    assessed_at: Option<DateTime<Utc>>,
    dismissed_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ScoreRow {
    fn into_score(self) -> RiskScore {
        RiskScore {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            assessed_at: self.assessed_at,
            dismissed_at: self.dismissed_at,
            deleted_at: self.deleted_at,
        }
    }
}

const SCENARIO_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           activated_at, archived_at, deleted_at
    FROM climate.scenarios
"#;

const SCENARIO_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              activated_at, archived_at, deleted_at
"#;

const SCORE_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, assessed_at, dismissed_at, deleted_at
    FROM climate.risk_scores
"#;

const SCORE_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, assessed_at, dismissed_at, deleted_at
"#;

#[derive(Clone)]
pub struct ClimateRepo {
    pool: PgPool,
}

impl ClimateRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Scenarios ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<Scenario>> {
        let rows: Vec<ScenarioRow> = sqlx::query_as(&format!(
            "{SCENARIO_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate list: {e}")))?;
        Ok(rows.into_iter().map(ScenarioRow::into_scenario).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Scenario> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: ScenarioRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO climate.scenarios
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {SCENARIO_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("climate create: {e}")))?;
        Ok(row.into_scenario())
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Scenario>> {
        let row: Option<ScenarioRow> = sqlx::query_as(&format!(
            "{SCENARIO_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate get: {e}")))?;
        Ok(row.map(ScenarioRow::into_scenario))
    }

    async fn fetch_scenario_any(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<Scenario>> {
        let row: Option<ScenarioRow> = sqlx::query_as(&format!(
            "{SCENARIO_SELECT} WHERE tenant_id = $1 AND id = $2"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate fetch scenario: {e}")))?;
        Ok(row.map(ScenarioRow::into_scenario))
    }

    pub async fn update_scenario(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
        update: ScenarioUpdate,
    ) -> HelixResult<Scenario> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE climate.scenarios SET updated_at = ");
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
        builder.push_bind(scenario_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {SCENARIO_RETURNING}"));

        let row: Option<ScenarioRow> = builder
            .build_query_as::<ScenarioRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("climate update scenario: {e}")))?;

        row.map(ScenarioRow::into_scenario)
            .ok_or_else(|| HelixError::not_found("scenario not found"))
    }

    pub async fn activate_scenario(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
    ) -> HelixResult<Scenario> {
        let scenario = self
            .get_parent(tenant_id, scenario_id)
            .await?
            .ok_or_else(|| HelixError::not_found("scenario not found"))?;
        let next = next_scenario_status(&scenario.status, "activate")?;
        let now = Utc::now();
        let row: Option<ScenarioRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.scenarios
            SET status = $1, activated_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {SCENARIO_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate activate scenario: {e}")))?;

        row.map(ScenarioRow::into_scenario)
            .ok_or_else(|| HelixError::not_found("scenario not found"))
    }

    /// Archive an active scenario. Rejected while draft scores remain.
    pub async fn archive_scenario(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
    ) -> HelixResult<Scenario> {
        let scenario = self
            .get_parent(tenant_id, scenario_id)
            .await?
            .ok_or_else(|| HelixError::not_found("scenario not found"))?;
        let next = next_scenario_status(&scenario.status, "archive")?;

        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM climate.risk_scores WHERE tenant_id = $1 AND parent_id = $2 AND status = 'draft' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate archive score count: {e}")))?;
        if drafts > 0 {
            return Err(HelixError::validation(format!(
                "scenario has {drafts} draft score(s); assess or dismiss them first"
            )));
        }

        let now = Utc::now();
        let row: Option<ScenarioRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.scenarios
            SET status = $1, archived_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {SCENARIO_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate archive scenario: {e}")))?;

        row.map(ScenarioRow::into_scenario)
            .ok_or_else(|| HelixError::not_found("scenario not found"))
    }

    pub async fn reopen_scenario(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
    ) -> HelixResult<Scenario> {
        let scenario = self
            .get_parent(tenant_id, scenario_id)
            .await?
            .ok_or_else(|| HelixError::not_found("scenario not found"))?;
        let next = next_scenario_status(&scenario.status, "reopen")?;
        let now = Utc::now();
        let row: Option<ScenarioRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.scenarios
            SET status = $1, archived_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NULL
            {SCENARIO_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate reopen scenario: {e}")))?;

        row.map(ScenarioRow::into_scenario)
            .ok_or_else(|| HelixError::not_found("scenario not found"))
    }

    pub async fn soft_delete_scenario(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
    ) -> HelixResult<Scenario> {
        let scenario = self
            .get_parent(tenant_id, scenario_id)
            .await?
            .ok_or_else(|| HelixError::not_found("scenario not found"))?;
        if scenario.status == "deleted" {
            return Err(HelixError::validation("scenario is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<ScenarioRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.scenarios
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {SCENARIO_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate soft-delete scenario: {e}")))?;

        row.map(ScenarioRow::into_scenario)
            .ok_or_else(|| HelixError::not_found("scenario not found"))
    }

    /// Restore a soft-deleted scenario, returning it to its pre-delete status.
    pub async fn restore_scenario(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
    ) -> HelixResult<Scenario> {
        let scenario = self
            .fetch_scenario_any(tenant_id, scenario_id)
            .await?
            .ok_or_else(|| HelixError::not_found("scenario not found"))?;
        if scenario.deleted_at.is_none() {
            return Err(HelixError::validation("scenario is not deleted"));
        }
        let restored = if scenario.archived_at.is_some() {
            "archived"
        } else if scenario.activated_at.is_some() {
            "active"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<ScenarioRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.scenarios
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {SCENARIO_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate restore scenario: {e}")))?;

        row.map(ScenarioRow::into_scenario)
            .ok_or_else(|| HelixError::not_found("scenario not found or not deleted"))
    }

    // --- Risk scores ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<RiskScore>> {
        let rows: Vec<ScoreRow> = sqlx::query_as(&format!(
            "{SCORE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate list children: {e}")))?;
        Ok(rows.into_iter().map(ScoreRow::into_score).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<RiskScore> {
        let _parent = self
            .get_parent(tenant_id, parent_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: ScoreRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO climate.risk_scores
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,'draft',$6,$7,$7)
            {SCORE_RETURNING}
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
        .map_err(|e| HelixError::dependency(format!("climate create child: {e}")))?;
        Ok(row.into_score())
    }

    pub async fn get_score(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
        score_id: Uuid,
    ) -> HelixResult<Option<RiskScore>> {
        let row: Option<ScoreRow> = sqlx::query_as(&format!(
            "{SCORE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .bind(score_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate get score: {e}")))?;
        Ok(row.map(ScoreRow::into_score))
    }

    async fn fetch_score_any(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
        score_id: Uuid,
    ) -> HelixResult<Option<RiskScore>> {
        let row: Option<ScoreRow> = sqlx::query_as(&format!(
            "{SCORE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .bind(score_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate fetch score: {e}")))?;
        Ok(row.map(ScoreRow::into_score))
    }

    pub async fn update_score(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
        score_id: Uuid,
        update: ScoreUpdate,
    ) -> HelixResult<RiskScore> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE climate.risk_scores SET updated_at = ");
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
        builder.push_bind(scenario_id);
        builder.push(" AND id = ");
        builder.push_bind(score_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {SCORE_RETURNING}"));

        let row: Option<ScoreRow> = builder
            .build_query_as::<ScoreRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("climate update score: {e}")))?;

        row.map(ScoreRow::into_score)
            .ok_or_else(|| HelixError::not_found("score not found"))
    }

    pub async fn assess_score(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
        score_id: Uuid,
    ) -> HelixResult<RiskScore> {
        let score = self
            .get_score(tenant_id, scenario_id, score_id)
            .await?
            .ok_or_else(|| HelixError::not_found("score not found"))?;
        let next = next_score_status(&score.status, "assess")?;
        let now = Utc::now();
        let row: Option<ScoreRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.risk_scores
            SET status = $1, assessed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NULL
            {SCORE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .bind(score_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate assess score: {e}")))?;

        row.map(ScoreRow::into_score)
            .ok_or_else(|| HelixError::not_found("score not found"))
    }

    pub async fn dismiss_score(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
        score_id: Uuid,
    ) -> HelixResult<RiskScore> {
        let score = self
            .get_score(tenant_id, scenario_id, score_id)
            .await?
            .ok_or_else(|| HelixError::not_found("score not found"))?;
        let next = next_score_status(&score.status, "dismiss")?;
        let now = Utc::now();
        let row: Option<ScoreRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.risk_scores
            SET status = $1, dismissed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NULL
            {SCORE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .bind(score_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate dismiss score: {e}")))?;

        row.map(ScoreRow::into_score)
            .ok_or_else(|| HelixError::not_found("score not found"))
    }

    pub async fn soft_delete_score(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
        score_id: Uuid,
    ) -> HelixResult<RiskScore> {
        let score = self
            .get_score(tenant_id, scenario_id, score_id)
            .await?
            .ok_or_else(|| HelixError::not_found("score not found"))?;
        if score.status == "deleted" {
            return Err(HelixError::validation("score is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<ScoreRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.risk_scores
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {SCORE_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .bind(score_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate soft-delete score: {e}")))?;

        row.map(ScoreRow::into_score)
            .ok_or_else(|| HelixError::not_found("score not found"))
    }

    /// Restore a soft-deleted score, returning it to its pre-delete status.
    pub async fn restore_score(
        &self,
        tenant_id: TenantId,
        scenario_id: Uuid,
        score_id: Uuid,
    ) -> HelixResult<RiskScore> {
        let score = self
            .fetch_score_any(tenant_id, scenario_id, score_id)
            .await?
            .ok_or_else(|| HelixError::not_found("score not found"))?;
        if score.deleted_at.is_none() {
            return Err(HelixError::validation("score is not deleted"));
        }
        let restored = if score.dismissed_at.is_some() {
            "dismissed"
        } else if score.assessed_at.is_some() {
            "assessed"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<ScoreRow> = sqlx::query_as(&format!(
            r#"
            UPDATE climate.risk_scores
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {SCORE_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(scenario_id)
        .bind(score_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("climate restore score: {e}")))?;

        row.map(ScoreRow::into_score)
            .ok_or_else(|| HelixError::not_found("score not found or not deleted"))
    }

    // --- Reports ---

    /// Per-scenario score counts by status for non-deleted scenarios.
    pub async fn get_climate_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<ClimateSummaryRow>> {
        let rows: Vec<ClimateSummaryRow> = sqlx::query_as(
            r#"
            SELECT s.id, s.name, s.status,
                   COUNT(r.id) AS total_scores,
                   COUNT(r.id) FILTER (WHERE r.status = 'draft') AS draft_scores,
                   COUNT(r.id) FILTER (WHERE r.status = 'assessed') AS assessed_scores,
                   COUNT(r.id) FILTER (WHERE r.status = 'dismissed') AS dismissed_scores
            FROM climate.scenarios s
            LEFT JOIN climate.risk_scores r
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
        .map_err(|e| HelixError::dependency(format!("climate summary: {e}")))?;
        Ok(rows)
    }
}
