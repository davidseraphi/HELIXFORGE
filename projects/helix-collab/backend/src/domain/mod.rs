//! HelixCollab domain: documents, presence, real-time sync, optional CRDT, sovereign APIs.

mod crdt;
mod crypto_doc;
mod documents;
mod mls_api;
mod mls_engine;
mod policy;
mod realtime;
mod sovereign;
mod webauthn_api;
mod workspace_api;

use axum::Router;
use service_kit::AppState;
use std::sync::Arc;

use mls_engine::MlsEngine;
use realtime::{spawn_nats_bridge, RealtimeHub};

#[derive(Clone)]
pub struct CollabState {
    pub core: AppState,
    pub hub: Arc<RealtimeHub>,
    pub mls: Arc<MlsEngine>,
}

pub fn routes(core: AppState) -> Router {
    let hub = Arc::new(RealtimeHub::with_bus(core.clients.bus.clone()));
    spawn_nats_bridge(hub.clone(), core.clients.bus.clone());
    let state = CollabState {
        core,
        hub,
        mls: Arc::new(MlsEngine::new()),
    };
    Router::new()
        .merge(documents::routes())
        .merge(workspace_api::routes())
        .merge(realtime::routes())
        .merge(sovereign::routes())
        .merge(mls_api::routes())
        .merge(webauthn_api::routes())
        .with_state(state)
}
