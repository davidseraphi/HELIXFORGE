//! HelixFlow API — agentic automation & workflow engine (second-wave depth).

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::FlowRepo;
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-flow")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: format!("{}-assistant", product.slug),
            description: format!("{} assistant", product.title),
            system_prompt: format!("You are the {} workflow assistant.", product.title),
            tools: vec!["echo".into(), "product_catalog".into()],
            max_steps: 8,
        });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-flow", 8103)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-flow", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/domain/status", get(domain_status))
        .route("/v1/workflows", get(list_wf).post(create_wf))
        .route("/v1/workflows/{id}", get(get_wf))
        .route(
            "/v1/workflows/{id}/runs",
            get(list_runs_for_wf).post(run_wf_path),
        )
        .route("/v1/workflows/run", post(run_wf))
        .route("/v1/runs", get(list_runs))
        .route("/v1/runs/{id}", get(get_run))
        .route("/v1/runs/{id}/cancel", post(cancel_run))
        .route("/v1/runs/{id}/events", get(list_events))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-flow",
        "phase": "wave2_w1",
        "durable": state.clients.db.is_some(),
        "planes": {
            "workflows": true,
            "runs": true,
            "step_events": true,
            "in_process_execute": true,
            "cancel": true
        },
        "step_types": ["echo", "set", "fail", "noop"]
    }))))
}

async fn list_wf(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = FlowRepo::new(pool.clone());
        let items = repo.list(p.tenant_id).await?;
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

async fn get_wf(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let repo = FlowRepo::new(pool.clone());
    let wf = repo
        .get(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("workflow not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(wf))))
}

#[derive(Deserialize)]
struct CreateWf {
    name: String,
    #[serde(default = "default_steps")]
    steps: u32,
    #[serde(default)]
    definition: serde_json::Value,
}

fn default_steps() -> u32 {
    3
}

async fn create_wf(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateWf>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable workflows"))?;
    let repo = FlowRepo::new(pool.clone());
    let def = if body.definition.is_null() || body.definition == serde_json::json!({}) {
        // Default multi-step definition for second-wave demos
        serde_json::json!({
            "version": 1,
            "steps": [
                {"name": "hello", "type": "echo", "message": "helix-flow"},
                {"name": "stamp", "type": "set", "key": "ok", "value": true},
                {"name": "done", "type": "noop"}
            ]
        })
    } else {
        body.definition
    };
    let wf = repo
        .create(p.tenant_id, body.name.trim(), body.steps.max(1), def)
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "workflow.create".into(),
            resource_type: "workflow".into(),
            resource_id: wf.id.to_string(),
            metadata: serde_json::json!({"name": wf.name, "steps": wf.steps}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-flow",
            "workflows.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(wf))))
}

#[derive(Deserialize)]
struct RunBody {
    workflow_id: Uuid,
}

async fn run_wf(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<RunBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    execute_workflow(&state, &p, body.workflow_id).await
}

async fn run_wf_path(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    execute_workflow(&state, &p, id).await
}

async fn execute_workflow(
    state: &AppState,
    p: &shared_core::tenancy::Principal,
    workflow_id: Uuid,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for workflow runs"))?;
    let repo = FlowRepo::new(pool.clone());
    let wf = repo
        .get(p.tenant_id, workflow_id)
        .await?
        .ok_or_else(|| HelixError::not_found("workflow not found"))?;
    let run = repo.enqueue_run(p.tenant_id, workflow_id).await?;
    // In-process execute (synchronous for local forge depth)
    let finished = run_definition(state, &repo, p, &wf, run.id).await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-flow",
            "workflows.runs",
            1.0,
            "count",
            serde_json::json!({"workflow_id": workflow_id, "status": finished.status}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.flow.workflow.finished",
            &serde_json::json!({
                "run_id": finished.id,
                "workflow_id": finished.workflow_id,
                "status": finished.status
            }),
        )
        .await
        .ok();
    let events = repo.list_step_events(p.tenant_id, finished.id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "run": finished,
        "events": events
    }))))
}

async fn run_definition(
    _state: &AppState,
    repo: &FlowRepo,
    p: &shared_core::tenancy::Principal,
    wf: &helix_db::Workflow,
    run_id: Uuid,
) -> Result<helix_db::WorkflowRun, ApiError> {
    repo.update_run(
        p.tenant_id,
        run_id,
        "running",
        0,
        serde_json::json!({}),
        "",
        false,
    )
    .await?;
    let steps = wf
        .definition
        .get("steps")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();
    let mut ctx = serde_json::Map::new();
    for (i, step) in steps.iter().enumerate() {
        if repo.is_cancel_requested(p.tenant_id, run_id).await? {
            repo.update_run(
                p.tenant_id,
                run_id,
                "cancelled",
                i as i32,
                serde_json::Value::Object(ctx.clone()),
                "cancel_requested",
                true,
            )
            .await?;
            return Ok(repo
                .get_run(p.tenant_id, run_id)
                .await?
                .ok_or_else(|| HelixError::internal("run missing after cancel"))?);
        }
        let stype = step
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("echo")
            .to_ascii_lowercase();
        let sname = step
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or(&stype)
            .to_string();
        let (status, output) = match stype.as_str() {
            "echo" => {
                let msg = step.get("message").and_then(|m| m.as_str()).unwrap_or("ok");
                ("succeeded", serde_json::json!({ "message": msg }))
            }
            "set" => {
                let key = step.get("key").and_then(|k| k.as_str()).unwrap_or("key");
                let val = step
                    .get("value")
                    .cloned()
                    .unwrap_or(serde_json::json!(true));
                ctx.insert(key.to_string(), val.clone());
                ("succeeded", serde_json::json!({ "set": key, "value": val }))
            }
            "noop" => ("succeeded", serde_json::json!({ "noop": true })),
            "fail" => {
                let msg = step
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("step failed");
                let _ = repo
                    .append_step_event(
                        p.tenant_id,
                        run_id,
                        i as i32,
                        &sname,
                        &stype,
                        "failed",
                        serde_json::json!({ "error": msg }),
                    )
                    .await;
                repo.update_run(
                    p.tenant_id,
                    run_id,
                    "failed",
                    i as i32,
                    serde_json::Value::Object(ctx.clone()),
                    msg,
                    true,
                )
                .await?;
                return Ok(repo
                    .get_run(p.tenant_id, run_id)
                    .await?
                    .ok_or_else(|| HelixError::internal("run missing after fail"))?);
            }
            "http" | "http_post" => {
                // SSRF: not executed in-process by default
                (
                    "skipped",
                    serde_json::json!({
                        "note": "http steps disabled in local executor (SSRF policy)"
                    }),
                )
            }
            other => (
                "failed",
                serde_json::json!({ "error": format!("unknown step type: {other}") }),
            ),
        };
        let _ = repo
            .append_step_event(
                p.tenant_id,
                run_id,
                i as i32,
                &sname,
                &stype,
                status,
                output.clone(),
            )
            .await;
        repo.update_run(
            p.tenant_id,
            run_id,
            "running",
            (i + 1) as i32,
            serde_json::Value::Object(ctx.clone()),
            "",
            false,
        )
        .await?;
        if status == "failed" {
            repo.update_run(
                p.tenant_id,
                run_id,
                "failed",
                i as i32,
                serde_json::Value::Object(ctx.clone()),
                output
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("failed"),
                true,
            )
            .await?;
            return Ok(repo
                .get_run(p.tenant_id, run_id)
                .await?
                .ok_or_else(|| HelixError::internal("run missing"))?);
        }
    }
    repo.update_run(
        p.tenant_id,
        run_id,
        "succeeded",
        steps.len() as i32,
        serde_json::Value::Object(ctx),
        "",
        true,
    )
    .await?;
    let _ = repo.set_workflow_status(p.tenant_id, wf.id, "active").await;
    Ok(repo
        .get_run(p.tenant_id, run_id)
        .await?
        .ok_or_else(|| HelixError::internal("run missing after success"))?)
}

#[derive(Deserialize)]
struct RunsQuery {
    #[serde(default)]
    workflow_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    50
}

async fn list_runs(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Query(q): Query<RunsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let repo = FlowRepo::new(pool.clone());
    let items = repo.list_runs(p.tenant_id, q.workflow_id, q.limit).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn list_runs_for_wf(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let repo = FlowRepo::new(pool.clone());
    let items = repo.list_runs(p.tenant_id, Some(id), 50).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn get_run(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let repo = FlowRepo::new(pool.clone());
    let run = repo
        .get_run(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("run not found"))?;
    let events = repo.list_step_events(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "run": run, "events": events }),
    )))
}

async fn cancel_run(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let repo = FlowRepo::new(pool.clone());
    let ok = repo.request_cancel(p.tenant_id, id).await?;
    if !ok {
        return Err(HelixError::not_found("run not found").into());
    }
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "cancelled": id }),
    )))
}

async fn list_events(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let repo = FlowRepo::new(pool.clone());
    let items = repo.list_step_events(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::tenancy::{Principal, Scope};
    use shared_core::{TenantId, UserId};
    use tokio::sync::{Mutex, MutexGuard};
    use uuid::Uuid;

    use super::*;

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18103");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-flow").expect("helix-flow product known");
        let builder = ServiceBuilder::new(product.slug, product.default_port)
            .await
            .expect("ServiceBuilder requires Postgres + optional NATS/MinIO");
        let state = builder.into_state();

        // Integration tests run against a freshly-migrated, empty Postgres.
        // The dev principal's tenant is deterministic but not seeded, so create
        // it here before any audited operation tries to reference it.
        let local_dev_tenant = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        if let Some(tenants) = state.clients.tenants.as_ref() {
            let _ = tenants
                .create(local_dev_tenant, "local-dev", "local", None)
                .await;
        }

        (state, guard)
    }

    pub fn dev_principal(label: &str) -> Principal {
        let tenant_id = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        let user_id = UserId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("helixforge-user:{label}").as_bytes(),
        ));
        Principal {
            user_id,
            tenant_id,
            org_id: None,
            scopes: vec![
                Scope::Read,
                Scope::Write,
                Scope::Admin,
                Scope::AuditRead,
                Scope::Platform,
            ],
            session_id: Some(format!("dev-session:{label}")),
            residency_region: "local".into(),
        }
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn finished_runs_are_immutable() {
        let (state, _guard) = locked_state().await;
        let p = dev_principal("flow-race");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = FlowRepo::new(pool.clone());

        let wf = repo
            .create(p.tenant_id, "immutable-check", 1, serde_json::json!({}))
            .await
            .expect("create workflow");
        let run = repo
            .enqueue_run(p.tenant_id, wf.id)
            .await
            .expect("enqueue run");

        // Progress once, then finish.
        repo.update_run(
            p.tenant_id,
            run.id,
            "running",
            1,
            serde_json::json!({}),
            "",
            false,
        )
        .await
        .expect("progress");
        repo.update_run(
            p.tenant_id,
            run.id,
            "completed",
            1,
            serde_json::json!({"ok": true}),
            "",
            true,
        )
        .await
        .expect("finish");

        // 8 concurrent update attempts after finish: all must be rejected.
        let mut handles = Vec::new();
        for i in 0..8u32 {
            let repo = repo.clone();
            let tenant_id = p.tenant_id;
            let run_id = run.id;
            handles.push(tokio::spawn(async move {
                let finished = i % 2 == 0;
                repo.update_run(
                    tenant_id,
                    run_id,
                    "running",
                    99,
                    serde_json::json!({}),
                    "",
                    finished,
                )
                .await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("update task panicked") {
                Err(e) if e.code == shared_core::ErrorCode::Validation => rejected += 1,
                Ok(_) => panic!("update after finish must be rejected"),
                Err(e) => panic!("unexpected update error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "every post-finish update must be rejected");

        let after = repo
            .get_run(p.tenant_id, run.id)
            .await
            .expect("get run")
            .expect("run exists");
        assert_eq!(after.status, "completed");
        assert_eq!(after.current_step, 1);
        assert!(after.finished_at.is_some());
    }
}
