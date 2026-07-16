//! Per-tenant security flags merged with process env break-glass.

use helix_db::{CodeRepoStore, CodeTenantBreakglass};
use shared_core::ids::TenantId;
use shared_core::HelixResult;

#[derive(Debug, Clone, Default)]
pub struct EffectiveBreakglass {
    pub allow_direct_push: bool,
    pub allow_force_push: bool,
    pub allow_ci_all: bool,
    pub allow_term_all: bool,
    pub allow_host_fallback: bool,
    pub allow_host_isolation: bool,
    pub sources: Vec<String>,
}

pub async fn load_effective(
    store: &CodeRepoStore,
    tenant_id: TenantId,
) -> HelixResult<EffectiveBreakglass> {
    let t = store.get_tenant_breakglass(tenant_id).await?;
    Ok(merge_with_env(&t))
}

pub fn merge_with_env(t: &CodeTenantBreakglass) -> EffectiveBreakglass {
    let mut e = EffectiveBreakglass::default();
    let mut src = Vec::new();

    if env_on("HELIX_CODE_ALLOW_DIRECT_PUSH") {
        e.allow_direct_push = true;
        src.push("env:ALLOW_DIRECT_PUSH".into());
    } else if t.allow_direct_push {
        e.allow_direct_push = true;
        src.push("tenant:allow_direct_push".into());
    }

    if env_on("HELIX_CODE_ALLOW_FORCE_PUSH") {
        e.allow_force_push = true;
        src.push("env:ALLOW_FORCE_PUSH".into());
    } else if t.allow_force_push {
        e.allow_force_push = true;
        src.push("tenant:allow_force_push".into());
    }

    if env_on("HELIX_CODE_CI_ALLOW_ALL") {
        e.allow_ci_all = true;
        src.push("env:CI_ALLOW_ALL".into());
    } else if t.allow_ci_all {
        e.allow_ci_all = true;
        src.push("tenant:allow_ci_all".into());
    }

    if env_on("HELIX_CODE_TERM_ALLOW_ALL") {
        e.allow_term_all = true;
        src.push("env:TERM_ALLOW_ALL".into());
    } else if t.allow_term_all {
        e.allow_term_all = true;
        src.push("tenant:allow_term_all".into());
    }

    if env_on("HELIX_CODE_ALLOW_HOST_FALLBACK") || env_on("HELIX_CODE_CI_ALLOW_ALL") {
        e.allow_host_fallback = true;
        src.push("env:HOST_FALLBACK".into());
    } else if t.allow_host_fallback || t.allow_ci_all {
        e.allow_host_fallback = true;
        src.push("tenant:allow_host_fallback".into());
    }

    if env_on("HELIX_CODE_ALLOW_HOST_ISOLATION") {
        e.allow_host_isolation = true;
        src.push("env:HOST_ISOLATION".into());
    } else if t.allow_host_isolation {
        e.allow_host_isolation = true;
        src.push("tenant:allow_host_isolation".into());
    }

    e.sources = src;
    e
}

fn env_on(k: &str) -> bool {
    std::env::var(k)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
