//! HelixCore Agent Hub — registry, durable runs, audit + NATS.

use agent_framework::{AgentRun, AgentSpec};
use async_trait::async_trait;
use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Duration;
use helix_db::{AgentRunStore, JobRepo};
use job_engine::{JobContext, JobHandler, JobOutcome, JobRegistry, JobWorker};
use serde::Deserialize;
use service_kit::{serve_with_shutdown, ApiError, AppState, RequireAuth, ServiceBuilder};
use shared_core::ids::{JobId, TenantId, UserId};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let builder = ServiceBuilder::new("agent-hub", 8081).await?;
    let cfg_addr = builder.config().listen_addr;

    builder.clients().agents.register_agent(AgentSpec {
        name: "platform-orchestrator".into(),
        description: "Cross-product orchestration agent".into(),
        system_prompt: "Coordinate HelixForge products safely.".into(),
        tools: vec![
            "echo".into(),
            "product_catalog".into(),
            "utc_now".into(),
            "tenant_context".into(),
        ],
        max_steps: 16,
    });
    builder.clients().agents.register_agent(AgentSpec {
        name: "security-auditor".into(),
        description: "Reviews actions for zero-trust compliance".into(),
        system_prompt: "Audit for least privilege and residency.".into(),
        tools: vec!["echo".into(), "tenant_context".into(), "utc_now".into()],
        max_steps: 6,
    });

    let state = builder.into_state();

    // Spawn a durable job worker when Postgres is available. Agent runs submitted
    // through the API are queued as jobs and executed by this worker.
    if let Some(pool) = state.clients.db.clone() {
        let registry = JobRegistry::new();
        registry.register(
            "agent.run",
            Arc::new(AgentRunJobHandler {
                runtime: state.clients.agents.clone(),
                db: Some(pool.clone()),
            }),
        );
        let worker = JobWorker::new(
            JobRepo::new(pool.clone()),
            registry,
            format!("{}-worker", state.clients.config.service_name),
        )
        .with_lease_duration(Duration::seconds(60));
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        tokio::spawn(async move {
            if let Err(e) = worker
                .run(shutdown_rx, std::time::Duration::from_secs(2))
                .await
            {
                tracing::error!(error = %e, "agent job worker exited");
            }
        });
        tracing::info!("durable agent job worker spawned");
    }

    let app = ServiceBuilder::base_router(state.clone()).merge(
        Router::new()
            .route("/v1/agents", get(list_agents).post(register_agent))
            .route("/v1/agents/run", post(run_agent))
            .route("/v1/agents/runs", get(list_runs))
            .route("/v1/agents/runs/{id}", get(get_run))
            .route("/v1/agents/runs/{id}/cancel", post(cancel_run))
            .route("/v1/agents/jobs/{id}", get(get_job))
            .route("/v1/agents/product/{product}", post(register_product_agent))
            .route("/v1/tools", get(list_tools))
            .with_state(state.clone()),
    );

    serve_with_shutdown(cfg_addr, app, "agent-hub", state.clone()).await
}

async fn cancel_run(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<AgentRun>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    let run = state
        .clients
        .agents
        .cancel_run(id)
        .ok_or_else(|| HelixError::not_found(format!("run {id}")))?;
    if run.tenant_id != principal.tenant_id
        && !principal.has_scope(&shared_core::tenancy::Scope::Platform)
    {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    Ok(Json(ApiResponse::ok(run)))
}

async fn list_agents(
    State(state): State<AppState>,
    RequireAuth(_p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<AgentSpec>>>, ApiError> {
    Ok(Json(ApiResponse::ok(state.clients.agents.list_agents())))
}

async fn register_agent(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(spec): Json<AgentSpec>,
) -> Result<Json<ApiResponse<AgentSpec>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Admin)?;
    if spec.name.trim().is_empty() {
        return Err(HelixError::validation("agent name required").into());
    }
    state.clients.agents.register_agent(spec.clone());
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "agent.register".into(),
            resource_type: "agent".into(),
            resource_id: spec.name.clone(),
            metadata: serde_json::json!({"tools": spec.tools}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(spec)))
}

#[derive(Deserialize)]
struct RunBody {
    agent: String,
    #[serde(default)]
    input: serde_json::Value,
    /// Force synchronous in-memory execution even when the durable data plane is up.
    #[serde(default)]
    synchronous: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AgentJobPayload {
    agent: String,
    tenant_id: TenantId,
    user_id: UserId,
    input: serde_json::Value,
}

async fn run_agent(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<RunBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;

    // When Postgres is available, queue the agent run as a durable job by default.
    if let Some(pool) = state.clients.db.as_ref() {
        if !body.synchronous {
            let payload = AgentJobPayload {
                agent: body.agent.clone(),
                tenant_id: principal.tenant_id,
                user_id: principal.user_id,
                input: body.input.clone(),
            };
            let requested =
                serde_json::to_string(&payload).map_err(|e| HelixError::internal(e.to_string()))?;
            let mut tx = pool
                .begin()
                .await
                .map_err(|e| HelixError::dependency(format!("job tx: {e}")))?;
            let job = JobRepo::new(pool.clone())
                .create_in_tx(
                    &mut tx,
                    principal.tenant_id,
                    principal.user_id,
                    "agent.run",
                    requested,
                    3,
                )
                .await?;
            tx.commit()
                .await
                .map_err(|e| HelixError::dependency(format!("job commit: {e}")))?;

            state
                .clients
                .audit
                .append(AuditEvent {
                    tenant_id: Some(principal.tenant_id),
                    actor: Actor::User {
                        user_id: principal.user_id,
                        tenant_id: principal.tenant_id,
                    },
                    action: "agent.run.queued".into(),
                    resource_type: "agent_job".into(),
                    resource_id: job.id.to_string(),
                    metadata: serde_json::json!({"agent": body.agent}),
                    residency_region: principal.residency_region.clone(),
                })
                .await?;
            state.clients.metrics.inc("agent_hub.runs", 1);
            let _ = state
                .clients
                .bus
                .publish(
                    "helix.core.agent.queued",
                    &serde_json::json!({
                        "job_id": job.id.to_string(),
                        "agent": body.agent,
                        "tenant_id": principal.tenant_id.to_string(),
                    }),
                )
                .await;
            return Ok(Json(ApiResponse::ok(serde_json::json!({
                "durable": true,
                "job_id": job.id.to_string(),
                "status": "queued",
                "agent": body.agent,
            }))));
        }
    }

    // Synchronous (or local-only) path: run in memory and optionally persist the record.
    let run = state
        .clients
        .agents
        .run(
            &body.agent,
            principal.tenant_id,
            principal.user_id,
            body.input,
        )
        .await?;

    if let Some(pool) = state.clients.db.as_ref() {
        let store = AgentRunStore::new(pool.clone());
        store.save(&run).await?;
    }

    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "agent.run".into(),
            resource_type: "agent_run".into(),
            resource_id: run.id.to_string(),
            metadata: serde_json::json!({
                "agent": run.agent,
                "status": format!("{:?}", run.status),
                "steps": run.steps.len(),
                "durable": state.clients.has_db()
            }),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            principal.tenant_id,
            "helix-core",
            "agents.runs",
            1.0,
            "count",
            serde_json::json!({"agent": run.agent}),
        )
        .await?;
    let _ = state
        .clients
        .bus
        .publish(
            "helix.core.agent.completed",
            &serde_json::json!({
                "run_id": run.id,
                "agent": run.agent,
                "tenant_id": principal.tenant_id.to_string(),
                "status": format!("{:?}", run.status)
            }),
        )
        .await;
    state.clients.metrics.inc("agent_hub.runs", 1);
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": false,
        "run": run
    }))))
}

#[derive(Deserialize)]
struct RunsQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

async fn list_runs(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Query(q): Query<RunsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = AgentRunStore::new(pool.clone());
        let items = store.list_for_tenant(principal.tenant_id, q.limit).await?;
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "durable": true,
            "items": items
        }))));
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": false,
        "items": []
    }))))
}

async fn get_run(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<AgentRun>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = AgentRunStore::new(pool.clone());
        match store.get_for_tenant(principal.tenant_id, id).await? {
            Some(run) => return Ok(Json(ApiResponse::ok(run))),
            None => return Err(HelixError::not_found(format!("run {id}")).into()),
        }
    }
    // Memory fallback only when Postgres is unavailable.
    let run = state
        .clients
        .agents
        .get_run(id)
        .ok_or_else(|| HelixError::not_found(format!("run {id}")))?;
    if run.tenant_id != principal.tenant_id
        && !principal.has_scope(&shared_core::tenancy::Scope::Platform)
    {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    Ok(Json(ApiResponse::ok(run)))
}

async fn get_job(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for jobs"))?;
    let job_id: JobId = id
        .parse()
        .map_err(|_| HelixError::validation("invalid job_id"))?;
    let repo = JobRepo::new(pool.clone());
    let job = repo
        .get(principal.tenant_id, job_id)
        .await?
        .ok_or_else(|| HelixError::not_found(format!("job {id}")))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "job": job,
        "durable": true,
    }))))
}

async fn list_tools(
    State(state): State<AppState>,
    RequireAuth(_p): RequireAuth,
) -> Result<Json<ApiResponse<Vec<(String, String)>>>, ApiError> {
    Ok(Json(ApiResponse::ok(state.clients.agents.tools().list())))
}

/// Products register a named assistant into the shared hub (Admin).
async fn register_product_agent(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(product): Path<String>,
    Json(mut spec): Json<AgentSpec>,
) -> Result<Json<ApiResponse<AgentSpec>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Admin)?;
    if product.trim().is_empty() {
        return Err(HelixError::validation("product slug required").into());
    }
    if shared_core::project::product_by_slug(&product).is_none() && product != "helix-core" {
        return Err(HelixError::not_found(format!("product {product}")).into());
    }
    if spec.name.trim().is_empty() {
        spec.name = format!("{product}-assistant");
    }
    if spec.tools.is_empty() {
        spec.tools = vec![
            "echo".into(),
            "product_catalog".into(),
            "tenant_context".into(),
        ];
    }
    if spec.max_steps == 0 {
        spec.max_steps = 8;
    }
    state.clients.agents.register_agent(spec.clone());
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "agent.product_register".into(),
            resource_type: "agent".into(),
            resource_id: spec.name.clone(),
            metadata: serde_json::json!({"product": product, "tools": spec.tools}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(spec)))
}

/// Job-engine adapter that executes a queued agent run on the shared runtime and
/// persists the resulting run record to Postgres.
struct AgentRunJobHandler {
    runtime: Arc<agent_framework::AgentRuntime>,
    db: Option<helix_db::DbPool>,
}

#[async_trait]
impl JobHandler for AgentRunJobHandler {
    async fn run(&self, job: &helix_db::Job, ctx: JobContext) -> HelixResult<JobOutcome> {
        let payload: AgentJobPayload = serde_json::from_str(&job.requested)
            .map_err(|e| HelixError::validation(format!("agent job payload: {e}")))?;

        // Mark that the worker is alive before potentially long tool calls.
        let _ = ctx.heartbeat(0, serde_json::json!({})).await;

        let run = self
            .runtime
            .run(
                &payload.agent,
                payload.tenant_id,
                payload.user_id,
                payload.input,
            )
            .await?;

        if let Some(pool) = self.db.as_ref() {
            let store = AgentRunStore::new(pool.clone());
            if let Err(e) = store.save(&run).await {
                tracing::warn!(error = %e, run_id = %run.id, "failed to persist agent run");
            }
        }

        Ok(JobOutcome::Completed(Some(
            serde_json::to_value(&run).map_err(|e| HelixError::internal(e.to_string()))?,
        )))
    }
}
