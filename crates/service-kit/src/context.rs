use agent_framework::AgentRuntime;
use audit_log::AuditSink;
use auth_client::AuthClient;
use billing_client::BillingClient;
use helix_db::{
    ApiKeyStore, CollabRepo, DbPool, DbStatus, GovernanceRepo, MembershipRepo, RegionRepo,
    ResourceAclRepo, TenantRepo, WorkspaceRepo,
};
use nats_client::HelixBus;
use observability::MetricsRegistry;
use shared_core::config::CoreConfig;
use std::sync::Arc;
use vault_client::{KeyManagement, ObjectStore, VaultClient};

use crate::middleware::RateLimiter;

/// Bundle of HelixCore clients injected into every service.
#[derive(Clone)]
pub struct HelixCoreClients {
    pub auth: AuthClient,
    pub bus: HelixBus,
    pub vault: VaultClient,
    pub objects: ObjectStore,
    pub billing: BillingClient,
    pub audit: Arc<dyn AuditSink>,
    pub agents: Arc<AgentRuntime>,
    pub metrics: MetricsRegistry,
    pub kms: Arc<dyn KeyManagement>,
    pub config: CoreConfig,
    /// Present when Postgres is reachable and migrations applied.
    pub db: Option<DbPool>,
    pub db_status: DbStatus,
    pub workspaces: Option<WorkspaceRepo>,
    pub collab: Option<CollabRepo>,
    pub tenants: Option<TenantRepo>,
    pub memberships: Option<MembershipRepo>,
    pub api_keys: Option<ApiKeyStore>,
    pub acl: Option<ResourceAclRepo>,
    pub governance: Option<GovernanceRepo>,
    pub regions: Option<RegionRepo>,
    pub rate_limiter: RateLimiter,
}

impl HelixCoreClients {
    pub fn audit_sink(&self) -> Arc<dyn AuditSink> {
        self.audit.clone()
    }

    pub fn has_db(&self) -> bool {
        self.db.is_some()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub clients: Arc<HelixCoreClients>,
}
