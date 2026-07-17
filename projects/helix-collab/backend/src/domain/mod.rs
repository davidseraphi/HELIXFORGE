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

#[cfg(test)]
pub mod test_support {
    //! Shared harness for HelixCollab backend unit tests.
    //!
    //! Requires the local HelixCore data plane (Postgres) to be running.
    //! NATS and MinIO fall back to local/memory defaults when absent.

    use std::sync::{Arc, Once};

    use service_kit::{ProductApp, ServiceBuilder};
    use shared_core::tenancy::{Principal, Scope};
    use shared_core::{TenantId, UserId};
    use tokio::sync::{Mutex, MutexGuard};
    use uuid::Uuid;

    use super::{CollabState, MlsEngine, RealtimeHub};

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    pub fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18101");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    /// Build a fresh CollabState while holding a global lock.
    ///
    /// Each integration-style test gets its own HelixCore clients so that DB pools
    /// and NATS tasks are tied to a single Tokio runtime and dropped cleanly when
    /// the test ends. The returned guard serialises these tests to avoid exhausting
    /// the Postgres connection limit.
    pub async fn locked_state() -> (CollabState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-collab").expect("helix-collab product known");
        let builder = ServiceBuilder::new(product.slug, product.default_port)
            .await
            .expect("ServiceBuilder requires Postgres + optional NATS/MinIO");
        let core = builder.into_state();
        let state = CollabState {
            core: core.clone(),
            hub: Arc::new(RealtimeHub::with_bus(core.clients.bus.clone())),
            mls: Arc::new(MlsEngine::new()),
        };

        // Integration tests run against a freshly-migrated, empty Postgres.
        // The dev principal's tenant is deterministic but not seeded, so create
        // it here before any audited operation tries to reference it.
        let local_dev_tenant = TenantId::from_uuid(Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        if let Some(tenants) = state.core.clients.tenants.as_ref() {
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

    pub async fn create_test_doc(
        state: &CollabState,
        principal: &Principal,
        title: &str,
        content: &str,
    ) -> helix_db::CollabDocument {
        let repo = state
            .core
            .clients
            .collab
            .as_ref()
            .expect("Postgres required for collab tests");
        repo.create_document_full_ex(
            principal.tenant_id,
            principal.user_id,
            title,
            content,
            None,
            None,
            false,
            false,
        )
        .await
        .expect("create test document")
    }
}
