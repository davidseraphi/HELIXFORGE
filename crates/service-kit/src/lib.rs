//! Service kit — shared Axum bootstrap, middleware, and HelixCore client bundle.
//!
//! Every product backend and core service starts with [`ServiceBuilder`].

mod context;
mod error_map;
mod health;
mod middleware;
mod outbox_relay;
mod product;

pub use context::{AppState, HelixCoreClients};
pub use error_map::{map_error, ApiError};
pub use health::health_router;
pub use middleware::{
    inject_request_id, rate_limit_middleware, request_id_from_headers, request_id_middleware,
    session_token_from_headers, RateLimiter, RequireAuth,
};
pub use outbox_relay::spawn_outbox_relay;
pub use product::{ProductApp, ProductService, Workspace};

/// Re-export ACL permission enum for product crates.
pub use helix_db::{AclPermission, DeleteDecision};

use auth_client::{AuthClient, AuthConfig};
use axum::http::{HeaderValue, Method};
use axum::Router;
use billing_client::BillingClient;
use helix_db::{
    try_connect_and_migrate_with_config, ApiKeyStore, CollabRepo, GovernanceRepo, MembershipRepo,
    PgAuditSink, PgMetering, PgPlanStore, RegionRepo, ResourceAclRepo, TenantRepo, WorkspaceRepo,
};
use nats_client::HelixBus;
use observability::MetricsRegistry;
use shared_core::config::CoreConfig;
use shared_core::HelixResult;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use vault_client::{build_kms, ObjectStore, ObjectStoreConfig, VaultClient};

/// Build shared HelixCore clients from config (Postgres when available).
pub async fn build_core_clients(cfg: &CoreConfig) -> HelixResult<HelixCoreClients> {
    let auth = AuthClient::from_config(AuthConfig::from_core_config(cfg));

    let is_local = cfg.environment == "local";
    let bus = HelixBus::connect_with_config(cfg, is_local).await?;
    audit_log::set_hmac_secret(cfg.audit_hmac_secret.clone());
    let agents = agent_framework::AgentRuntime::with_defaults();
    let metrics = MetricsRegistry::new();
    let vault_key = cfg.vault_master_key.as_bytes();
    let kms = build_kms(vault_key, cfg);
    let objects = ObjectStore::new(ObjectStoreConfig {
        endpoint: cfg.minio_endpoint.clone(),
        bucket: cfg.minio_bucket.clone(),
        access_key: cfg.minio_access_key.clone(),
        secret_key: cfg.minio_secret_key.clone(),
        region: cfg.minio_region.clone(),
    })?;

    // MinIO is a hard dependency outside local (object refs, audit archive).
    if !is_local {
        objects.verify().await.map_err(|e| {
            shared_core::HelixError::unavailable(format!("MinIO required outside local: {e}"))
        })?;
    }

    let (db, db_status) =
        try_connect_and_migrate_with_config(&cfg.database_url, &cfg.db_pool).await;

    // Non-local: fail closed if Postgres is down (Kimi P1).
    if db.is_none() && !is_local {
        return Err(shared_core::HelixError::unavailable(format!(
            "Postgres required outside local (status: {})",
            db_status.detail
        )));
    }

    // Shared rate limit when Postgres is up (multi-replica safe).
    // Outside local, Postgres rate-limit errors fail closed rather than silently
    // falling back to per-process counting.
    let rate_limit_pg_fallback = is_local || cfg.rate_limit_pg_fallback;
    let use_pg_rate_limit = cfg.rate_limit_backend.eq_ignore_ascii_case("postgres")
        || cfg.rate_limit_backend == "1"
        || cfg.rate_limit_backend.eq_ignore_ascii_case("shared");
    let rate_limiter = match db.as_ref() {
        Some(pool) if use_pg_rate_limit => {
            RateLimiter::with_postgres(cfg.rate_limit_rps, pool.clone(), rate_limit_pg_fallback)
        }
        _ => RateLimiter::new(cfg.rate_limit_rps),
    };

    let (
        audit,
        billing,
        workspaces,
        collab,
        vault,
        tenants,
        memberships,
        api_keys,
        acl,
        governance,
        regions,
    ) = if let Some(ref pool) = db {
        info!(detail = %db_status.detail, "using durable Postgres enterprise data plane");
        (
            {
                let archive = helix_db::ObjectStoreArchiveSink::new(
                    objects.clone(),
                    &cfg.data_residency_region,
                );
                Arc::new(PgAuditSink::new(pool.clone()).with_archive(Arc::new(archive)))
                    as Arc<dyn audit_log::AuditSink>
            },
            BillingClient::with_plan_store(
                Arc::new(PgMetering::new(pool.clone())),
                Arc::new(PgPlanStore::new(pool.clone())),
            ),
            Some(WorkspaceRepo::new(pool.clone())),
            Some(CollabRepo::new(pool.clone())),
            VaultClient::new(Arc::new(helix_db::PgVault::with_kms(
                pool.clone(),
                vault_key,
                kms.clone(),
            ))),
            Some(TenantRepo::new(pool.clone())),
            Some(MembershipRepo::new(pool.clone())),
            Some(ApiKeyStore::new(pool.clone())),
            Some(ResourceAclRepo::new(pool.clone())),
            Some(GovernanceRepo::new(pool.clone())),
            Some(RegionRepo::new(pool.clone())),
        )
    } else {
        info!("local-only memory audit/meter/vault (Postgres not available)");
        (
            Arc::new(audit_log::MemoryAuditSink::new()) as Arc<dyn audit_log::AuditSink>,
            BillingClient::memory(),
            None,
            None,
            VaultClient::memory_with_kms(vault_key, kms.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
    };

    Ok(HelixCoreClients {
        auth,
        bus,
        vault,
        objects,
        billing,
        audit,
        agents: Arc::new(agents),
        metrics,
        kms: kms.clone(),
        config: cfg.clone(),
        db,
        db_status: db_status.clone(),
        workspaces,
        collab,
        tenants,
        memberships,
        api_keys,
        acl,
        governance,
        regions,
        rate_limiter,
    })
}

pub struct ServiceBuilder {
    cfg: CoreConfig,
    clients: HelixCoreClients,
}

impl ServiceBuilder {
    pub async fn new(service_name: &str, default_port: u16) -> HelixResult<Self> {
        let cfg = CoreConfig::from_env(service_name, default_port)?;
        observability::init_tracing_with_otlp(
            &cfg.service_name,
            cfg.log_json,
            cfg.otlp_endpoint.as_deref(),
            cfg.otlp_sample_rate,
        );
        let clients = build_core_clients(&cfg).await?;
        Ok(Self { cfg, clients })
    }

    pub fn config(&self) -> &CoreConfig {
        &self.cfg
    }

    pub fn clients(&self) -> &HelixCoreClients {
        &self.clients
    }

    pub fn into_state(self) -> AppState {
        AppState {
            clients: Arc::new(self.clients),
        }
    }

    /// Health router without global layers. Merge domain routes, then pass to
    /// [`serve_with_shutdown`] which applies enterprise middleware to *all* routes.
    pub fn base_router(_state: AppState) -> Router<AppState> {
        health_router()
    }

    /// Serve with graceful shutdown on Ctrl+C / SIGTERM.
    pub async fn serve(self, app: Router<AppState>) -> HelixResult<()> {
        let addr = self.cfg.listen_addr;
        let service_name = self.cfg.service_name.clone();
        let rate_limit_rps = self.cfg.rate_limit_rps;
        let max_body_bytes = self.cfg.max_body_bytes;
        let state = self.into_state();
        let app = layer_global(app, state);
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| shared_core::HelixError::internal(format!("bind {addr}: {e}")))?;
        info!(
            %addr,
            service = %service_name,
            rate_limit_rps,
            max_body_bytes,
            "listening (enterprise baseline)"
        );
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| shared_core::HelixError::internal(format!("serve: {e}")))?;
        info!(service = %service_name, "shutdown complete");
        Ok(())
    }
}

/// Apply global enterprise middleware (request-id, rate-limit, body-limit,
/// trace, CORS) to a fully merged router.
pub fn layer_global(app: Router<AppState>, state: AppState) -> Router {
    let limiter = state.clients.rate_limiter.clone();
    let body_limit = state.clients.config.max_body_bytes;
    let cors = cors_for_env(&state.clients.config);
    app.layer(axum::middleware::from_fn(middleware::inject_request_id))
        .layer(axum::middleware::from_fn_with_state(
            limiter,
            middleware::rate_limit_middleware,
        ))
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn cors_for_env(cfg: &CoreConfig) -> CorsLayer {
    // Local: permissive for console/dev. Production: explicit allowlist via HELIX_CORS_ORIGINS.
    let allow_credentials = cfg.cors_allow_credentials;
    if cfg.environment == "local" {
        let mut layer = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        if allow_credentials {
            layer = layer.allow_credentials(true);
        }
        return layer;
    }
    let origins_str = cfg.cors_origins.as_deref().unwrap_or("");
    let origins: Vec<HeaderValue> = origins_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| HeaderValue::from_str(s).ok())
        .collect();
    if origins.is_empty() {
        // Fail closed: no default origin in non-local environments.
        return CorsLayer::new()
            .allow_origin(AllowOrigin::exact(HeaderValue::from_static("")))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(cors_allow_headers());
    }
    let mut layer = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(cors_allow_headers());
    if allow_credentials {
        layer = layer.allow_credentials(true);
    }
    layer
}

fn cors_allow_headers() -> Vec<axum::http::header::HeaderName> {
    use axum::http::header::HeaderName;
    vec![
        HeaderName::from_static("content-type"),
        HeaderName::from_static("authorization"),
        HeaderName::from_static("accept"),
        HeaderName::from_static("accept-language"),
        HeaderName::from_static("accept-encoding"),
        HeaderName::from_static("origin"),
        HeaderName::from_static("x-request-id"),
        HeaderName::from_static("x-helix-dev-user"),
        HeaderName::from_static("x-helix-dev-scopes"),
        HeaderName::from_static("x-helix-dev-residency"),
        HeaderName::from_static("x-helix-dev-master-key"),
        HeaderName::from_static("x-helix-api-key"),
        HeaderName::from_static("x-session-token"),
        HeaderName::from_static("x-helix-dev-login-token"),
    ]
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}

/// Bind and serve any router with graceful shutdown (for services that already built state).
pub async fn serve_with_shutdown(
    addr: std::net::SocketAddr,
    app: Router<AppState>,
    service_name: &str,
    state: AppState,
) -> HelixResult<()> {
    let app = layer_global(app, state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| shared_core::HelixError::internal(format!("bind {addr}: {e}")))?;
    info!(%addr, service = %service_name, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| shared_core::HelixError::internal(format!("serve: {e}")))?;
    info!(service = %service_name, "shutdown complete");
    Ok(())
}

// Re-export for callers that need empty DbStatus in tests
pub use helix_db::DbStatus;
