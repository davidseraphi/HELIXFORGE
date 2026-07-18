//! HelixFlow workflow persistence (second-wave depth).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub steps: u32,
    pub status: String,
    pub definition: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub tenant_id: TenantId,
    pub status: String,
    pub current_step: i32,
    pub result: serde_json::Value,
    pub error_text: String,
    pub cancel_requested: bool,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEvent {
    pub id: Uuid,
    pub run_id: Uuid,
    pub tenant_id: TenantId,
    pub step_index: i32,
    pub step_name: String,
    pub step_type: String,
    pub status: String,
    pub output: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct FlowRepo {
    pool: PgPool,
}

impl FlowRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<Workflow>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            steps: i32,
            status: String,
            definition: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, steps, status, definition, created_at
            FROM flow.workflows WHERE tenant_id = $1 ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Workflow {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                name: r.name,
                steps: r.steps as u32,
                status: r.status,
                definition: r.definition,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn get(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<Workflow>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            steps: i32,
            status: String,
            definition: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, steps, status, definition, created_at
            FROM flow.workflows WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow get: {e}")))?;
        Ok(row.map(|r| Workflow {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            name: r.name,
            steps: r.steps as u32,
            status: r.status,
            definition: r.definition,
            created_at: r.created_at,
        }))
    }

    pub async fn create(
        &self,
        tenant_id: TenantId,
        name: &str,
        steps: u32,
        definition: serde_json::Value,
    ) -> HelixResult<Workflow> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let step_count = definition
            .get("steps")
            .and_then(|s| s.as_array())
            .map(|a| a.len() as u32)
            .unwrap_or(steps)
            .max(1);
        sqlx::query(
            r#"
            INSERT INTO flow.workflows (id, tenant_id, name, steps, status, definition, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(step_count as i32)
        .bind(&definition)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow create: {e}")))?;
        Ok(Workflow {
            id,
            tenant_id,
            name: name.into(),
            steps: step_count,
            status: "draft".into(),
            definition,
            created_at,
        })
    }

    pub async fn set_workflow_status(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        status: &str,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"UPDATE flow.workflows SET status = $3, updated_at = now()
               WHERE tenant_id = $1 AND id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .bind(status)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow status: {e}")))?;
        Ok(())
    }

    pub async fn enqueue_run(
        &self,
        tenant_id: TenantId,
        workflow_id: Uuid,
    ) -> HelixResult<WorkflowRun> {
        let _ = self
            .get(tenant_id, workflow_id)
            .await?
            .ok_or_else(|| HelixError::not_found("workflow not found"))?;
        let id = Uuid::now_v7();
        let started_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO flow.runs (id, workflow_id, tenant_id, status, current_step, result, error_text, cancel_requested, started_at)
            VALUES ($1,$2,$3,'queued',0,'{}'::jsonb,'',false,$4)
            "#,
        )
        .bind(id)
        .bind(workflow_id)
        .bind(tenant_id.as_uuid())
        .bind(started_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow run: {e}")))?;
        Ok(WorkflowRun {
            id,
            workflow_id,
            tenant_id,
            status: "queued".into(),
            current_step: 0,
            result: serde_json::json!({}),
            error_text: String::new(),
            cancel_requested: false,
            started_at,
            finished_at: None,
        })
    }

    pub async fn get_run(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
    ) -> HelixResult<Option<WorkflowRun>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            workflow_id: Uuid,
            tenant_id: Uuid,
            status: String,
            current_step: i32,
            result: serde_json::Value,
            error_text: String,
            cancel_requested: bool,
            started_at: DateTime<Utc>,
            finished_at: Option<DateTime<Utc>>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, workflow_id, tenant_id, status,
                   COALESCE(current_step, 0) AS current_step,
                   COALESCE(result, '{}'::jsonb) AS result,
                   COALESCE(error_text, '') AS error_text,
                   COALESCE(cancel_requested, false) AS cancel_requested,
                   started_at, finished_at
            FROM flow.runs WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow get run: {e}")))?;
        Ok(row.map(|r| WorkflowRun {
            id: r.id,
            workflow_id: r.workflow_id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            status: r.status,
            current_step: r.current_step,
            result: r.result,
            error_text: r.error_text,
            cancel_requested: r.cancel_requested,
            started_at: r.started_at,
            finished_at: r.finished_at,
        }))
    }

    pub async fn list_runs(
        &self,
        tenant_id: TenantId,
        workflow_id: Option<Uuid>,
        limit: i64,
    ) -> HelixResult<Vec<WorkflowRun>> {
        let limit = limit.clamp(1, 100);
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            workflow_id: Uuid,
            tenant_id: Uuid,
            status: String,
            current_step: i32,
            result: serde_json::Value,
            error_text: String,
            cancel_requested: bool,
            started_at: DateTime<Utc>,
            finished_at: Option<DateTime<Utc>>,
        }
        let rows: Vec<Row> = if let Some(wid) = workflow_id {
            sqlx::query_as(
                r#"
                SELECT id, workflow_id, tenant_id, status,
                       COALESCE(current_step, 0) AS current_step,
                       COALESCE(result, '{}'::jsonb) AS result,
                       COALESCE(error_text, '') AS error_text,
                       COALESCE(cancel_requested, false) AS cancel_requested,
                       started_at, finished_at
                FROM flow.runs WHERE tenant_id = $1 AND workflow_id = $2
                ORDER BY started_at DESC LIMIT $3
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(wid)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, workflow_id, tenant_id, status,
                       COALESCE(current_step, 0) AS current_step,
                       COALESCE(result, '{}'::jsonb) AS result,
                       COALESCE(error_text, '') AS error_text,
                       COALESCE(cancel_requested, false) AS cancel_requested,
                       started_at, finished_at
                FROM flow.runs WHERE tenant_id = $1
                ORDER BY started_at DESC LIMIT $2
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("flow list runs: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| WorkflowRun {
                id: r.id,
                workflow_id: r.workflow_id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                status: r.status,
                current_step: r.current_step,
                result: r.result,
                error_text: r.error_text,
                cancel_requested: r.cancel_requested,
                started_at: r.started_at,
                finished_at: r.finished_at,
            })
            .collect())
    }

    pub async fn update_run(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
        status: &str,
        current_step: i32,
        result: serde_json::Value,
        error_text: &str,
        finished: bool,
    ) -> HelixResult<()> {
        // Terminal runs are immutable: no progress or re-finish after finished_at.
        let res = if finished {
            sqlx::query(
                r#"UPDATE flow.runs SET status = $3, current_step = $4, result = $5, error_text = $6, finished_at = now()
                   WHERE tenant_id = $1 AND id = $2 AND finished_at IS NULL"#,
            )
            .bind(tenant_id.as_uuid())
            .bind(run_id)
            .bind(status)
            .bind(current_step)
            .bind(&result)
            .bind(error_text)
            .execute(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"UPDATE flow.runs SET status = $3, current_step = $4, result = $5, error_text = $6
                   WHERE tenant_id = $1 AND id = $2 AND finished_at IS NULL"#,
            )
            .bind(tenant_id.as_uuid())
            .bind(run_id)
            .bind(status)
            .bind(current_step)
            .bind(&result)
            .bind(error_text)
            .execute(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("flow update run: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::validation("run not found or already finished"));
        }
        Ok(())
    }

    pub async fn request_cancel(&self, tenant_id: TenantId, run_id: Uuid) -> HelixResult<bool> {
        let r = sqlx::query(
            r#"UPDATE flow.runs SET cancel_requested = true,
                   status = CASE WHEN status IN ('queued','running') THEN 'cancelled' ELSE status END,
                   finished_at = CASE WHEN status IN ('queued','running') THEN now() ELSE finished_at END
               WHERE tenant_id = $1 AND id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow cancel: {e}")))?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn is_cancel_requested(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
    ) -> HelixResult<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            r#"SELECT COALESCE(cancel_requested, false) FROM flow.runs WHERE tenant_id = $1 AND id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow cancel check: {e}")))?;
        Ok(row.map(|r| r.0).unwrap_or(false))
    }

    pub async fn append_step_event(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
        step_index: i32,
        step_name: &str,
        step_type: &str,
        status: &str,
        output: serde_json::Value,
    ) -> HelixResult<StepEvent> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            r#"INSERT INTO flow.step_events
               (id, run_id, tenant_id, step_index, step_name, step_type, status, output, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
        )
        .bind(id)
        .bind(run_id)
        .bind(tenant_id.as_uuid())
        .bind(step_index)
        .bind(step_name)
        .bind(step_type)
        .bind(status)
        .bind(&output)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow step event: {e}")))?;
        Ok(StepEvent {
            id,
            run_id,
            tenant_id,
            step_index,
            step_name: step_name.into(),
            step_type: step_type.into(),
            status: status.into(),
            output,
            created_at,
        })
    }

    pub async fn list_step_events(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
    ) -> HelixResult<Vec<StepEvent>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            run_id: Uuid,
            tenant_id: Uuid,
            step_index: i32,
            step_name: String,
            step_type: String,
            status: String,
            output: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT * FROM flow.step_events WHERE tenant_id = $1 AND run_id = $2
               ORDER BY step_index ASC, created_at ASC"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("flow list steps: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| StepEvent {
                id: r.id,
                run_id: r.run_id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                step_index: r.step_index,
                step_name: r.step_name,
                step_type: r.step_type,
                status: r.status,
                output: r.output,
                created_at: r.created_at,
            })
            .collect())
    }
}
