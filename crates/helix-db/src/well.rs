//! HelixWell habits + check-in persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::{TenantId, UserId};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub owner_id: UserId,
    pub name: String,
    pub description: String,
    pub cadence: String,
    pub target_per_period: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub paused_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitLog {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub habit_id: Uuid,
    pub user_id: UserId,
    pub quantity: i32,
    pub notes: String,
    pub logged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckIn {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    /// Skipped fields are missing (NULL), never zero.
    pub mood: Option<i32>,
    pub energy: Option<i32>,
    pub notes: String,
    pub tags: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
    pub edit_version: i32,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Append-only snapshot of a check-in's values before an edit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckInEdit {
    pub id: Uuid,
    pub checkin_id: Uuid,
    pub mood: Option<i32>,
    pub energy: Option<i32>,
    pub notes: String,
    pub tags: serde_json::Value,
    pub edited_by: UserId,
    pub edited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HabitSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub cadence: String,
    pub target_per_period: i32,
    pub total_logs: i64,
    pub total_quantity: i64,
    pub last_logged_at: Option<DateTime<Utc>>,
    pub logs_last_7_days: i64,
}

#[derive(Debug, Clone, Default)]
pub struct HabitUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cadence: Option<String>,
    pub target_per_period: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

/// Field updates for a check-in. An outer `None` leaves the field untouched;
/// `Some(None)` clears a scale field back to missing.
#[derive(Debug, Clone, Default)]
pub struct CheckInUpdate {
    pub mood: Option<Option<i32>>,
    pub energy: Option<Option<i32>>,
    pub notes: Option<String>,
    pub tags: Option<serde_json::Value>,
}

impl CheckInUpdate {
    pub fn has_changes(&self) -> bool {
        self.mood.is_some() || self.energy.is_some() || self.notes.is_some() || self.tags.is_some()
    }
}

/// A skipped (missing) scale value is valid; a present one must be 1..=10.
pub fn validate_optional_scale(value: Option<i32>, field: &str) -> HelixResult<()> {
    if let Some(v) = value {
        if !(1..=10).contains(&v) {
            return Err(HelixError::validation(format!("{field} must be 1..=10")));
        }
    }
    Ok(())
}

/// Validate a habit lifecycle transition and return the resulting status.
pub fn next_habit_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("active", "pause") => Ok("paused"),
        ("paused", "resume") => Ok("active"),
        ("active", "end") | ("paused", "end") => Ok("ended"),
        (_, "pause") => Err(HelixError::validation(format!(
            "cannot pause a {current} habit"
        ))),
        (_, "resume") => Err(HelixError::validation(format!(
            "cannot resume a {current} habit"
        ))),
        (_, "end") => Err(HelixError::validation(format!(
            "cannot end a {current} habit"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown habit action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct HabitRow {
    id: Uuid,
    tenant_id: Uuid,
    owner_id: Uuid,
    name: String,
    description: String,
    cadence: String,
    target_per_period: i32,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    paused_at: Option<DateTime<Utc>>,
    ended_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl HabitRow {
    fn into_habit(self) -> Habit {
        Habit {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            owner_id: UserId::from_uuid(self.owner_id),
            name: self.name,
            description: self.description,
            cadence: self.cadence,
            target_per_period: self.target_per_period,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            paused_at: self.paused_at,
            ended_at: self.ended_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CheckInRow {
    id: Uuid,
    tenant_id: Uuid,
    user_id: Uuid,
    mood: Option<i32>,
    energy: Option<i32>,
    notes: String,
    tags: serde_json::Value,
    recorded_at: DateTime<Utc>,
    edit_version: i32,
    updated_at: Option<DateTime<Utc>>,
}

impl CheckInRow {
    fn into_checkin(self) -> CheckIn {
        CheckIn {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            user_id: UserId::from_uuid(self.user_id),
            mood: self.mood,
            energy: self.energy,
            notes: self.notes,
            tags: self.tags,
            recorded_at: self.recorded_at,
            edit_version: self.edit_version,
            updated_at: self.updated_at,
        }
    }
}

const HABIT_SELECT: &str = r#"
    SELECT id, tenant_id, owner_id, name, description, cadence,
           target_per_period, status, metadata, created_at,
           paused_at, ended_at, deleted_at
    FROM well.habits
"#;

const HABIT_RETURNING: &str = r#"
    RETURNING id, tenant_id, owner_id, name, description, cadence,
              target_per_period, status, metadata, created_at,
              paused_at, ended_at, deleted_at
"#;

const CHECKIN_SELECT: &str = r#"
    SELECT id, tenant_id, user_id, mood, energy, notes, tags, recorded_at,
           edit_version, updated_at
    FROM well.checkins
"#;

#[derive(Clone)]
pub struct WellRepo {
    pool: PgPool,
}

impl WellRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_habits(
        &self,
        tenant_id: TenantId,
        owner_id: Option<UserId>,
    ) -> HelixResult<Vec<Habit>> {
        let rows: Vec<HabitRow> = if let Some(owner) = owner_id {
            sqlx::query_as(&format!(
                "{HABIT_SELECT} WHERE tenant_id = $1 AND owner_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
            ))
            .bind(tenant_id.as_uuid())
            .bind(owner.as_uuid())
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(&format!(
                "{HABIT_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
            ))
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("well list habits: {e}")))?;
        Ok(rows.into_iter().map(HabitRow::into_habit).collect())
    }

    pub async fn create_habit(
        &self,
        tenant_id: TenantId,
        owner_id: UserId,
        name: &str,
        description: &str,
        cadence: &str,
        target_per_period: i32,
        metadata: serde_json::Value,
    ) -> HelixResult<Habit> {
        if target_per_period < 1 {
            return Err(HelixError::validation("target_per_period must be >= 1"));
        }
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let cadence = if cadence.trim().is_empty() {
            "daily"
        } else {
            cadence.trim()
        };
        let row: HabitRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO well.habits
                (id, tenant_id, owner_id, name, description, cadence, target_per_period,
                 status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,'active',$8,$9,$9)
            {HABIT_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(owner_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(cadence)
        .bind(target_per_period)
        .bind(&metadata)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well create habit: {e}")))?;
        Ok(row.into_habit())
    }

    pub async fn get_habit(
        &self,
        tenant_id: TenantId,
        habit_id: Uuid,
    ) -> HelixResult<Option<Habit>> {
        let row: Option<HabitRow> = sqlx::query_as(&format!(
            "{HABIT_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(habit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well get habit: {e}")))?;
        Ok(row.map(HabitRow::into_habit))
    }

    async fn fetch_habit_any(
        &self,
        tenant_id: TenantId,
        habit_id: Uuid,
    ) -> HelixResult<Option<Habit>> {
        let row: Option<HabitRow> =
            sqlx::query_as(&format!("{HABIT_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(habit_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("well fetch habit: {e}")))?;
        Ok(row.map(HabitRow::into_habit))
    }

    pub async fn update_habit(
        &self,
        tenant_id: TenantId,
        habit_id: Uuid,
        update: HabitUpdate,
    ) -> HelixResult<Habit> {
        if let Some(t) = update.target_per_period {
            if t < 1 {
                return Err(HelixError::validation("target_per_period must be >= 1"));
            }
        }
        let mut builder = sqlx::QueryBuilder::new("UPDATE well.habits SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(n) = update.name {
            builder.push(", name = ");
            builder.push_bind(n);
        }
        if let Some(d) = update.description {
            builder.push(", description = ");
            builder.push_bind(d);
        }
        if let Some(c) = update.cadence {
            let c = if c.trim().is_empty() {
                "daily".to_string()
            } else {
                c.trim().to_string()
            };
            builder.push(", cadence = ");
            builder.push_bind(c);
        }
        if let Some(t) = update.target_per_period {
            builder.push(", target_per_period = ");
            builder.push_bind(t);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(habit_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {HABIT_RETURNING}"));

        let row: Option<HabitRow> = builder
            .build_query_as::<HabitRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("well update habit: {e}")))?;

        row.map(HabitRow::into_habit)
            .ok_or_else(|| HelixError::not_found("habit not found"))
    }

    async fn transition_habit(
        &self,
        tenant_id: TenantId,
        habit_id: Uuid,
        action: &str,
    ) -> HelixResult<Habit> {
        let habit = self
            .get_habit(tenant_id, habit_id)
            .await?
            .ok_or_else(|| HelixError::not_found("habit not found"))?;
        let next = next_habit_status(&habit.status, action)?;
        let now = Utc::now();
        let (paused_at, ended_at) = match next {
            "paused" => (Some(now), habit.ended_at),
            "active" => (None, habit.ended_at),
            "ended" => (habit.paused_at, Some(now)),
            _ => (habit.paused_at, habit.ended_at),
        };
        let row: Option<HabitRow> = sqlx::query_as(&format!(
            r#"
            UPDATE well.habits
            SET status = $1, paused_at = $2, ended_at = $3, updated_at = $4
            WHERE tenant_id = $5 AND id = $6 AND deleted_at IS NULL
            {HABIT_RETURNING}
            "#
        ))
        .bind(next)
        .bind(paused_at)
        .bind(ended_at)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(habit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well {action} habit: {e}")))?;

        row.map(HabitRow::into_habit)
            .ok_or_else(|| HelixError::not_found("habit not found"))
    }

    pub async fn pause_habit(&self, tenant_id: TenantId, habit_id: Uuid) -> HelixResult<Habit> {
        self.transition_habit(tenant_id, habit_id, "pause").await
    }

    pub async fn resume_habit(&self, tenant_id: TenantId, habit_id: Uuid) -> HelixResult<Habit> {
        self.transition_habit(tenant_id, habit_id, "resume").await
    }

    pub async fn end_habit(&self, tenant_id: TenantId, habit_id: Uuid) -> HelixResult<Habit> {
        self.transition_habit(tenant_id, habit_id, "end").await
    }

    pub async fn soft_delete_habit(
        &self,
        tenant_id: TenantId,
        habit_id: Uuid,
    ) -> HelixResult<Habit> {
        let habit = self
            .get_habit(tenant_id, habit_id)
            .await?
            .ok_or_else(|| HelixError::not_found("habit not found"))?;
        if habit.status == "deleted" {
            return Err(HelixError::validation("habit is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<HabitRow> = sqlx::query_as(&format!(
            r#"
            UPDATE well.habits
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {HABIT_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(habit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well soft-delete habit: {e}")))?;

        row.map(HabitRow::into_habit)
            .ok_or_else(|| HelixError::not_found("habit not found"))
    }

    /// Restore a soft-deleted habit, returning it to its pre-delete status.
    pub async fn restore_habit(&self, tenant_id: TenantId, habit_id: Uuid) -> HelixResult<Habit> {
        let habit = self
            .fetch_habit_any(tenant_id, habit_id)
            .await?
            .ok_or_else(|| HelixError::not_found("habit not found"))?;
        if habit.deleted_at.is_none() {
            return Err(HelixError::validation("habit is not deleted"));
        }
        let restored = if habit.ended_at.is_some() {
            "ended"
        } else if habit.paused_at.is_some() {
            "paused"
        } else {
            "active"
        };
        let now = Utc::now();
        let row: Option<HabitRow> = sqlx::query_as(&format!(
            r#"
            UPDATE well.habits
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {HABIT_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(habit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well restore habit: {e}")))?;

        row.map(HabitRow::into_habit)
            .ok_or_else(|| HelixError::not_found("habit not found or not deleted"))
    }

    pub async fn list_checkins(
        &self,
        tenant_id: TenantId,
        user_id: Option<UserId>,
        limit: i64,
    ) -> HelixResult<Vec<CheckIn>> {
        let lim = limit.clamp(1, 500);
        let rows: Vec<CheckInRow> = if let Some(uid) = user_id {
            sqlx::query_as(&format!(
                "{CHECKIN_SELECT} WHERE tenant_id = $1 AND user_id = $2 AND deleted_at IS NULL ORDER BY recorded_at DESC LIMIT $3"
            ))
            .bind(tenant_id.as_uuid())
            .bind(uid.as_uuid())
            .bind(lim)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(&format!(
                "{CHECKIN_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY recorded_at DESC LIMIT $2"
            ))
            .bind(tenant_id.as_uuid())
            .bind(lim)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("well list checkins: {e}")))?;
        Ok(rows.into_iter().map(CheckInRow::into_checkin).collect())
    }

    pub async fn create_checkin(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        mood: Option<i32>,
        energy: Option<i32>,
        notes: &str,
        tags: serde_json::Value,
    ) -> HelixResult<CheckIn> {
        validate_optional_scale(mood, "mood")?;
        validate_optional_scale(energy, "energy")?;
        let id = Uuid::now_v7();
        let recorded_at = Utc::now();
        let row: CheckInRow = sqlx::query_as(
            r#"
            INSERT INTO well.checkins
                (id, tenant_id, user_id, mood, energy, notes, tags, recorded_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING id, tenant_id, user_id, mood, energy, notes, tags, recorded_at,
                      edit_version, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(mood)
        .bind(energy)
        .bind(notes)
        .bind(&tags)
        .bind(recorded_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well create checkin: {e}")))?;
        Ok(row.into_checkin())
    }

    pub async fn get_checkin(
        &self,
        tenant_id: TenantId,
        checkin_id: Uuid,
    ) -> HelixResult<Option<CheckIn>> {
        let row: Option<CheckInRow> = sqlx::query_as(&format!(
            "{CHECKIN_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(checkin_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well get checkin: {e}")))?;
        Ok(row.map(CheckInRow::into_checkin))
    }

    /// Apply an edit transactionally: snapshot the current values into
    /// `well.checkin_edits`, then update the row and bump `edit_version`.
    pub async fn update_checkin(
        &self,
        tenant_id: TenantId,
        checkin_id: Uuid,
        editor_id: UserId,
        update: CheckInUpdate,
    ) -> HelixResult<CheckIn> {
        if !update.has_changes() {
            return Err(HelixError::validation("no check-in changes provided"));
        }
        if let Some(m) = update.mood {
            validate_optional_scale(m, "mood")?;
        }
        if let Some(e) = update.energy {
            validate_optional_scale(e, "energy")?;
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("well checkin edit begin: {e}")))?;

        let row: Option<CheckInRow> = sqlx::query_as(&format!(
            "{CHECKIN_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE"
        ))
        .bind(tenant_id.as_uuid())
        .bind(checkin_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("well lock checkin: {e}")))?;
        let current = row
            .map(CheckInRow::into_checkin)
            .ok_or_else(|| HelixError::not_found("check-in not found"))?;

        sqlx::query(
            r#"
            INSERT INTO well.checkin_edits
                (id, tenant_id, checkin_id, mood, energy, notes, tags, edited_by, edited_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(tenant_id.as_uuid())
        .bind(checkin_id)
        .bind(current.mood)
        .bind(current.energy)
        .bind(&current.notes)
        .bind(&current.tags)
        .bind(editor_id.as_uuid())
        .bind(Utc::now())
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("well snapshot checkin edit: {e}")))?;

        let mut builder = sqlx::QueryBuilder::new("UPDATE well.checkins SET updated_at = ");
        builder.push_bind(Utc::now());
        builder.push(", edit_version = edit_version + 1");
        if let Some(m) = update.mood {
            builder.push(", mood = ");
            builder.push_bind(m);
        }
        if let Some(e) = update.energy {
            builder.push(", energy = ");
            builder.push_bind(e);
        }
        if let Some(n) = update.notes {
            builder.push(", notes = ");
            builder.push_bind(n);
        }
        if let Some(t) = update.tags {
            builder.push(", tags = ");
            builder.push_bind(t);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(checkin_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(
            " RETURNING id, tenant_id, user_id, mood, energy, notes, tags, recorded_at, edit_version, updated_at",
        );

        let row: Option<CheckInRow> = builder
            .build_query_as::<CheckInRow>()
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("well update checkin: {e}")))?;
        let updated = row
            .map(CheckInRow::into_checkin)
            .ok_or_else(|| HelixError::not_found("check-in not found"))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("well commit checkin edit: {e}")))?;
        Ok(updated)
    }

    pub async fn soft_delete_checkin(
        &self,
        tenant_id: TenantId,
        checkin_id: Uuid,
    ) -> HelixResult<CheckIn> {
        let deleted_at = Utc::now();
        let row: Option<CheckInRow> = sqlx::query_as(
            r#"
            UPDATE well.checkins
            SET deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            RETURNING id, tenant_id, user_id, mood, energy, notes, tags, recorded_at,
                      edit_version, updated_at
            "#,
        )
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(checkin_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well soft-delete checkin: {e}")))?;

        row.map(CheckInRow::into_checkin)
            .ok_or_else(|| HelixError::not_found("check-in not found"))
    }

    pub async fn list_checkin_edits(
        &self,
        tenant_id: TenantId,
        checkin_id: Uuid,
    ) -> HelixResult<Vec<CheckInEdit>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            checkin_id: Uuid,
            mood: Option<i32>,
            energy: Option<i32>,
            notes: String,
            tags: serde_json::Value,
            edited_by: Uuid,
            edited_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, checkin_id, mood, energy, notes, tags, edited_by, edited_at
            FROM well.checkin_edits
            WHERE tenant_id = $1 AND checkin_id = $2
            ORDER BY edited_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(checkin_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well list checkin edits: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CheckInEdit {
                id: r.id,
                checkin_id: r.checkin_id,
                mood: r.mood,
                energy: r.energy,
                notes: r.notes,
                tags: r.tags,
                edited_by: UserId::from_uuid(r.edited_by),
                edited_at: r.edited_at,
            })
            .collect())
    }

    pub async fn log_habit(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        habit_id: Uuid,
        quantity: i32,
        notes: &str,
    ) -> HelixResult<HabitLog> {
        if quantity < 1 {
            return Err(HelixError::validation("quantity must be >= 1"));
        }
        let id = Uuid::now_v7();
        let logged_at = Utc::now();
        // The active-habit guard is part of the INSERT itself: a habit
        // paused between a separate check and insert cannot leak logs.
        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"
            INSERT INTO well.habit_logs
                (id, tenant_id, habit_id, user_id, quantity, notes, logged_at)
            SELECT $1, $2, $3, $4, $5, $6, $7
            FROM well.habits h
            WHERE h.tenant_id = $2 AND h.id = $3 AND h.status = 'active' AND h.deleted_at IS NULL
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(habit_id)
        .bind(user_id.as_uuid())
        .bind(quantity)
        .bind(notes)
        .bind(logged_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well log habit: {e}")))?;
        if inserted.is_none() {
            return Err(HelixError::validation("habit not found or not active"));
        }
        Ok(HabitLog {
            id,
            tenant_id,
            habit_id,
            user_id,
            quantity,
            notes: notes.into(),
            logged_at,
        })
    }

    pub async fn list_habit_logs(
        &self,
        tenant_id: TenantId,
        habit_id: Uuid,
        limit: i64,
    ) -> HelixResult<Vec<HabitLog>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            habit_id: Uuid,
            user_id: Uuid,
            quantity: i32,
            notes: String,
            logged_at: DateTime<Utc>,
        }
        let lim = limit.clamp(1, 500);
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, habit_id, user_id, quantity, notes, logged_at
            FROM well.habit_logs
            WHERE tenant_id = $1 AND habit_id = $2
            ORDER BY logged_at DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(habit_id)
        .bind(lim)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well list habit logs: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| HabitLog {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                habit_id: r.habit_id,
                user_id: UserId::from_uuid(r.user_id),
                quantity: r.quantity,
                notes: r.notes,
                logged_at: r.logged_at,
            })
            .collect())
    }

    /// Per-habit totals and recent activity for non-deleted habits.
    pub async fn get_habit_summary(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<HabitSummaryRow>> {
        let rows: Vec<HabitSummaryRow> = sqlx::query_as(
            r#"
            SELECT h.id, h.name, h.status, h.cadence, h.target_per_period,
                   COUNT(l.id) AS total_logs,
                   COALESCE(SUM(l.quantity), 0) AS total_quantity,
                   MAX(l.logged_at) AS last_logged_at,
                   COUNT(l.id) FILTER (WHERE l.logged_at >= now() - interval '7 days') AS logs_last_7_days
            FROM well.habits h
            LEFT JOIN well.habit_logs l
                   ON l.habit_id = h.id AND l.tenant_id = h.tenant_id
            WHERE h.tenant_id = $1 AND h.deleted_at IS NULL
            GROUP BY h.id, h.name, h.status, h.cadence, h.target_per_period, h.created_at
            ORDER BY h.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well habit summary: {e}")))?;
        Ok(rows)
    }
}
