//! HelixCode extreme domain — forge planes + end-state collab/CI/LSP/agents.

mod agent_sandbox;
mod api;
mod branch_protection;
mod breakglass;
mod cmd_policy;
mod collab_api;
mod container;
mod dap_client;
mod endstate_api;
mod git_store;
mod lsp_api;
mod lsp_bridge;
mod mls_api;
mod mls_engine;
mod sandbox;
mod sealed_api;
mod smart_http;
mod tenant_policy;
mod terminal_policy;
mod webhook_policy;

use axum::Router;
use service_kit::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .merge(api::routes())
        .merge(smart_http::routes())
        .merge(lsp_api::routes())
        .merge(sealed_api::routes())
        .merge(mls_api::routes())
        .merge(collab_api::routes())
        .merge(endstate_api::routes())
        .with_state(state)
}
