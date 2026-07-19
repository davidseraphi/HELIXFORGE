//! HelixCode API — sovereign code forge (extreme E0+).
//!
//! Dual-plane git: gitoxide read model + smart HTTP pack servers.
//! See `docs/SOVEREIGN_ROADMAP.md`.

#![recursion_limit = "512"]

mod domain;

use service_kit::{serve_with_shutdown, ProductApp, ProductService, ServiceBuilder};
use shared_core::HelixResult;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-code")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;

    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: format!("{}-assistant", product.slug),
            description: format!("{} forge assistant", product.title),
            system_prompt: format!(
            "You are the {} forge assistant. Help with repos, commits, pipelines, and workspaces.",
            product.title
        ),
            tools: vec![
                "echo".into(),
                "product_catalog".into(),
                "utc_now".into(),
                "tenant_context".into(),
            ],
            max_steps: 10,
        });
    // E4 mesh peer: patch-oriented agent (same tool sandbox; used in multi-agent jobs)
    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: "helix-code-patcher".into(),
            description: "HelixCode patch reviewer / apply coordinator".into(),
            system_prompt: "Review and acknowledge structured patches in forge sandboxes.".into(),
            tools: vec![
                "echo".into(),
                "product_catalog".into(),
                "utc_now".into(),
                "tenant_context".into(),
            ],
            max_steps: 6,
        });

    let addr = builder.config().listen_addr;
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        // Axum no longer supports nesting a service at the root; a root
        // fallback gives the same "everything else goes to the domain
        // router" behavior the nest provided.
        .fallback_service(domain::routes(state.clone()));

    serve_with_shutdown(addr, app, "helix-code", state.clone()).await
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{AppState, ProductApp, ServiceBuilder};
    use shared_core::TenantId;
    use tokio::sync::{Mutex, MutexGuard};
    use uuid::Uuid;

    use helix_db::CodeRepoStore;

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18102");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    pub async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-code").expect("helix-code product known");
        let builder = ServiceBuilder::new(product.slug, product.default_port)
            .await
            .expect("ServiceBuilder requires Postgres + optional NATS/MinIO");
        let state = builder.into_state();

        // Integration tests run against a migrated Postgres. The dev
        // principal's tenant is deterministic but not seeded, so create it
        // here before any audited operation tries to reference it.
        let local_dev_tenant = dev_tenant();
        if let Some(tenants) = state.clients.tenants.as_ref() {
            let _ = tenants
                .create(local_dev_tenant, "local-dev", "local", None)
                .await;
        }

        (state, guard)
    }

    fn dev_tenant() -> TenantId {
        TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ))
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_finish_pipeline_run_single_winner() {
        let (state, _guard) = locked_state().await;
        let tenant_id = dev_tenant();
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let store = CodeRepoStore::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let repo = store
            .create(tenant_id, &format!("dur-pipe-{suffix}"), "", "private")
            .await
            .expect("create repo");
        let pipeline = store
            .create_pipeline(tenant_id, repo.id, "gate", serde_json::json!({"steps": []}))
            .await
            .expect("create pipeline");
        let run = store
            .create_pipeline_run(tenant_id, pipeline.id, repo.id, "refs/heads/main", None)
            .await
            .expect("create run");

        // 8 racing finishes of one pipeline run.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let store = store.clone();
            handles.push(tokio::spawn(async move {
                store
                    .finish_pipeline_run(
                        tenant_id,
                        run.id,
                        "succeeded",
                        "done",
                        None,
                        serde_json::json!([]),
                        Some(0),
                        "host",
                    )
                    .await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("finish task panicked") {
                Ok(_) => winners += 1,
                Err(e) if e.code == shared_core::ErrorCode::Conflict => rejected += 1,
                Err(e) => panic!("unexpected finish error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing finish may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let finished = store
            .get_pipeline_run(tenant_id, run.id)
            .await
            .expect("get run")
            .expect("run exists");
        assert_eq!(finished.status, "succeeded");
        assert!(finished.finished_at.is_some());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_finish_agent_job_single_winner() {
        let (state, _guard) = locked_state().await;
        let tenant_id = dev_tenant();
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let store = CodeRepoStore::new(pool.clone());

        let suffix = Uuid::now_v7().simple().to_string();
        let repo = store
            .create(tenant_id, &format!("dur-agent-{suffix}"), "", "private")
            .await
            .expect("create repo");
        let job = store
            .create_agent_job(tenant_id, repo.id, None, "sandbox", "prove the gate")
            .await
            .expect("create agent job");

        // 8 racing finishes of one agent job.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let store = store.clone();
            handles.push(tokio::spawn(async move {
                store
                    .finish_agent_job(
                        tenant_id,
                        job.id,
                        "succeeded",
                        "summary",
                        None,
                        None,
                        "log",
                        serde_json::json!([]),
                        serde_json::json!([]),
                        serde_json::json!([]),
                        "host",
                    )
                    .await
            }));
        }
        let mut winners = 0usize;
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("finish task panicked") {
                Ok(_) => winners += 1,
                Err(e) if e.code == shared_core::ErrorCode::Conflict => rejected += 1,
                Err(e) => panic!("unexpected finish error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one racing finish may win");
        assert_eq!(rejected, 7, "all losers must be rejected");

        let finished = store
            .get_agent_job(tenant_id, job.id)
            .await
            .expect("get job")
            .expect("job exists");
        assert_eq!(finished.status, "succeeded");
        assert!(finished.finished_at.is_some());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn children_rejected_on_missing_repo() {
        let (state, _guard) = locked_state().await;
        let tenant_id = dev_tenant();
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let store = CodeRepoStore::new(pool.clone());

        let err = store
            .create_workspace(tenant_id, Uuid::now_v7(), "ghost", "main", "", "tester")
            .await
            .expect_err("workspace on a missing repo must fail");
        assert_eq!(err.code, shared_core::ErrorCode::NotFound);

        let err = store
            .create_pipeline(tenant_id, Uuid::now_v7(), "ghost", serde_json::json!({}))
            .await
            .expect_err("pipeline on a missing repo must fail");
        assert_eq!(err.code, shared_core::ErrorCode::NotFound);
    }
}
