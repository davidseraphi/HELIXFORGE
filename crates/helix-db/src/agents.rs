//! Durable agent run persistence.

use agent_framework::{AgentRun, AgentStep, RunStatus};
use chrono::{DateTime, Utc};
use shared_core::ids::{TenantId, UserId};
use shared_core::time::UtcTimestamp;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AgentRunStore {
    pool: PgPool,
}

#[derive(sqlx::FromRow)]
struct RunRow {
    id: Uuid,
    tenant_id: Uuid,
    user_id: Uuid,
    agent: String,
    status: String,
    input: serde_json::Value,
    output: Option<serde_json::Value>,
    steps: serde_json::Value,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
}

impl RunRow {
    fn into_run(self) -> HelixResult<AgentRun> {
        let steps: Vec<AgentStep> = serde_json::from_value(self.steps)
            .map_err(|e| HelixError::internal(format!("agent steps decode: {e}")))?;
        let status = match self.status.as_str() {
            "succeeded" => RunStatus::Succeeded,
            "failed" => RunStatus::Failed,
            "cancelled" => RunStatus::Cancelled,
            "running" => RunStatus::Running,
            _ => RunStatus::Pending,
        };
        Ok(AgentRun {
            id: self.id,
            agent: self.agent,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            user_id: UserId::from_uuid(self.user_id),
            input: self.input,
            status,
            steps,
            output: self.output,
            started_at: UtcTimestamp(self.started_at),
            finished_at: self.finished_at.map(UtcTimestamp),
        })
    }
}

impl AgentRunStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, run: &AgentRun) -> HelixResult<()> {
        let steps = serde_json::to_value(&run.steps)
            .map_err(|e| HelixError::internal(format!("agent steps: {e}")))?;
        let status = match run.status {
            RunStatus::Succeeded => "succeeded",
            RunStatus::Failed => "failed",
            RunStatus::Cancelled => "cancelled",
            RunStatus::Running => "running",
            RunStatus::Pending => "pending",
        };
        sqlx::query(
            r#"
            INSERT INTO helix_core.agent_runs
                (id, tenant_id, user_id, agent, status, input, output, steps, started_at, finished_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (id) DO UPDATE SET
                status = EXCLUDED.status,
                output = EXCLUDED.output,
                steps = EXCLUDED.steps,
                finished_at = EXCLUDED.finished_at
            "#,
        )
        .bind(run.id)
        .bind(run.tenant_id.as_uuid())
        .bind(run.user_id.as_uuid())
        .bind(&run.agent)
        .bind(status)
        .bind(&run.input)
        .bind(&run.output)
        .bind(steps)
        .bind(run.started_at.inner())
        .bind(run.finished_at.map(|t| t.inner()))
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("agent run save: {e}")))?;
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> HelixResult<Option<AgentRun>> {
        let row: Option<RunRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, user_id, agent, status, input, output, steps, started_at, finished_at
            FROM helix_core.agent_runs WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("agent run get: {e}")))?;
        match row {
            Some(r) => Ok(Some(r.into_run()?)),
            None => Ok(None),
        }
    }

    pub async fn get_for_tenant(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<AgentRun>> {
        let row: Option<RunRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, user_id, agent, status, input, output, steps, started_at, finished_at
            FROM helix_core.agent_runs WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("agent run get_for_tenant: {e}")))?;
        match row {
            Some(r) => Ok(Some(r.into_run()?)),
            None => Ok(None),
        }
    }

    pub async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
        limit: i64,
    ) -> HelixResult<Vec<AgentRun>> {
        let lim = limit.clamp(1, 200);
        let rows: Vec<RunRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, user_id, agent, status, input, output, steps, started_at, finished_at
            FROM helix_core.agent_runs
            WHERE tenant_id = $1
            ORDER BY started_at DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(lim)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("agent run list: {e}")))?;
        rows.into_iter().map(|r| r.into_run()).collect()
    }
}
