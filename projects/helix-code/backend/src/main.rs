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
        .nest_service("/", domain::routes(state.clone()));

    serve_with_shutdown(addr, app, "helix-code", state.clone()).await
}
