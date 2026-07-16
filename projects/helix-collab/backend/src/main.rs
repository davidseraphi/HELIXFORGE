//! HelixCollab API — real-time collaborative workspace.
//!
//! Reuses HelixCore via `service_kit` + durable Postgres via `helix_db`.

mod domain;

use service_kit::{serve_with_shutdown, ProductApp, ProductService, ServiceBuilder};
use shared_core::HelixResult;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-collab")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;

    builder.clients().agents.register_agent(agent_framework::AgentSpec {
        name: format!("{}-assistant", product.slug),
        description: format!("{} collaborative assistant", product.title),
        system_prompt: format!(
            "You are the {} assistant. Help users co-author documents safely with versioning and presence.",
            product.title
        ),
        tools: vec![
            "echo".into(),
            "product_catalog".into(),
            "utc_now".into(),
            "tenant_context".into(),
        ],
        max_steps: 8,
    });

    let addr = builder.config().listen_addr;
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .nest_service("/", domain::routes(state.clone()));

    serve_with_shutdown(addr, app, "helix-collab", state.clone()).await
}
