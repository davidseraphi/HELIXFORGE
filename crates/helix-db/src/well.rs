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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckIn {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub mood: i32,
    pub energy: i32,
    pub notes: String,
    pub tags: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
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
        }
    }
}

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
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, owner_id, name, description, cadence,
                       target_per_period, status, metadata, created_at
                FROM well.habits
                WHERE tenant_id = $1 AND owner_id = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(owner.as_uuid())
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, owner_id, name, description, cadence,
                       target_per_period, status, metadata, created_at
                FROM well.habits
                WHERE tenant_id = $1
                ORDER BY created_at DESC
                "#,
            )
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
        sqlx::query(
            r#"
            INSERT INTO well.habits
                (id, tenant_id, owner_id, name, description, cadence, target_per_period,
                 status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,'active',$8,$9,$9)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(owner_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(cadence)
        .bind(target_per_period)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well create habit: {e}")))?;
        Ok(Habit {
            id,
            tenant_id,
            owner_id,
            name: name.into(),
            description: description.into(),
            cadence: cadence.into(),
            target_per_period,
            status: "active".into(),
            metadata,
            created_at,
        })
    }

    pub async fn get_habit(
        &self,
        tenant_id: TenantId,
        habit_id: Uuid,
    ) -> HelixResult<Option<Habit>> {
        let row: Option<HabitRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, owner_id, name, description, cadence,
                   target_per_period, status, metadata, created_at
            FROM well.habits
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(habit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well get habit: {e}")))?;
        Ok(row.map(HabitRow::into_habit))
    }

    pub async fn list_checkins(
        &self,
        tenant_id: TenantId,
        user_id: Option<UserId>,
        limit: i64,
    ) -> HelixResult<Vec<CheckIn>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            user_id: Uuid,
            mood: i32,
            energy: i32,
            notes: String,
            tags: serde_json::Value,
            recorded_at: DateTime<Utc>,
        }
        let lim = limit.clamp(1, 500);
        let rows: Vec<Row> = if let Some(uid) = user_id {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, user_id, mood, energy, notes, tags, recorded_at
                FROM well.checkins
                WHERE tenant_id = $1 AND user_id = $2
                ORDER BY recorded_at DESC
                LIMIT $3
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(uid.as_uuid())
            .bind(lim)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, user_id, mood, energy, notes, tags, recorded_at
                FROM well.checkins
                WHERE tenant_id = $1
                ORDER BY recorded_at DESC
                LIMIT $2
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(lim)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("well list checkins: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CheckIn {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                user_id: UserId::from_uuid(r.user_id),
                mood: r.mood,
                energy: r.energy,
                notes: r.notes,
                tags: r.tags,
                recorded_at: r.recorded_at,
            })
            .collect())
    }

    pub async fn create_checkin(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        mood: i32,
        energy: i32,
        notes: &str,
        tags: serde_json::Value,
    ) -> HelixResult<CheckIn> {
        if !(1..=10).contains(&mood) {
            return Err(HelixError::validation("mood must be 1..=10"));
        }
        if !(1..=10).contains(&energy) {
            return Err(HelixError::validation("energy must be 1..=10"));
        }
        let id = Uuid::now_v7();
        let recorded_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO well.checkins
                (id, tenant_id, user_id, mood, energy, notes, tags, recorded_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
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
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well create checkin: {e}")))?;
        Ok(CheckIn {
            id,
            tenant_id,
            user_id,
            mood,
            energy,
            notes: notes.into(),
            tags,
            recorded_at,
        })
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
        let habit = self
            .get_habit(tenant_id, habit_id)
            .await?
            .ok_or_else(|| HelixError::not_found("habit not found"))?;
        if habit.status != "active" {
            return Err(HelixError::validation("habit is not active"));
        }
        let id = Uuid::now_v7();
        let logged_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO well.habit_logs
                (id, tenant_id, habit_id, user_id, quantity, notes, logged_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(habit_id)
        .bind(user_id.as_uuid())
        .bind(quantity)
        .bind(notes)
        .bind(logged_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("well log habit: {e}")))?;
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
}
