//! HelixCore API Gateway — catalog, identity edge, reverse-proxy, enterprise platform APIs.

use audit_log::AuditEvent;
use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use helix_db::TenantStatus;
use serde::{Deserialize, Serialize};
use service_kit::{serve_with_shutdown, ApiError, AppState, RequireAuth, ServiceBuilder};
use shared_core::ids::TenantId;
use shared_core::project::PRODUCT_CATALOG;
use shared_core::semantic_state::{ProductMaturity, SemanticState};
use shared_core::tenancy::{Actor, Scope};
use shared_core::{ApiResponse, HelixError, HelixResult};
use std::sync::OnceLock;
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let builder = ServiceBuilder::new("gateway", 8080).await?;
    let cfg_addr = builder.config().listen_addr;
    let state = builder.into_state();

    // Start background outbox relay when Postgres is available.
    let _relay = service_kit::spawn_outbox_relay((*state.clients).clone());

    let app = ServiceBuilder::base_router(state.clone()).merge(
        Router::new()
            .route("/v1/catalog", get(catalog))
            .route("/v1/catalog/{slug}", get(catalog_one))
            .route("/v1/catalog/{slug}/state", get(catalog_state))
            .route("/v1/me", get(me))
            .route("/v1/routes", get(routes))
            .route("/v1/core/status", get(core_status))
            .route("/v1/core/inventory", get(core_inventory))
            .route("/v1/workspaces", get(list_all_workspaces))
            // Enterprise platform (Platform scope)
            .route(
                "/v1/platform/tenants",
                get(list_tenants).post(create_tenant),
            )
            .route("/v1/platform/tenants/{id}/suspend", post(suspend_tenant))
            .route("/v1/platform/tenants/{id}/activate", post(activate_tenant))
            .route(
                "/v1/tenants/{tenant_id}/api-keys",
                get(list_api_keys).post(issue_api_key),
            )
            .route(
                "/v1/tenants/{tenant_id}/api-keys/{id}/revoke",
                post(revoke_api_key),
            )
            // Resource ACL + governance
            .route(
                "/v1/acl/{resource_type}/{resource_id}",
                get(acl_list).post(acl_grant),
            )
            .route(
                "/v1/acl/{resource_type}/{resource_id}/check",
                get(acl_check),
            )
            .route(
                "/v1/acl/{resource_type}/{resource_id}/revoke",
                post(acl_revoke),
            )
            .route(
                "/v1/governance/retention",
                get(retention_list).post(retention_set),
            )
            .route("/v1/governance/holds", post(hold_place))
            .route("/v1/governance/holds/{id}/release", post(hold_release))
            .route("/v1/governance/purpose", post(purpose_bind))
            .route(
                "/v1/governance/can-delete/{resource_type}/{resource_id}",
                get(can_delete),
            )
            .route("/v1/regions", get(regions_list))
            .route("/v1/regions/{code}/status", post(region_status_update))
            // Recovery bin
            .route("/v1/recovery-bin", get(list_recovery_bin))
            .route("/v1/recovery-bin/{id}", get(get_recovery_bin))
            .route("/v1/recovery-bin/{id}/restore", post(restore_recovery_bin))
            .route(
                "/v1/recovery-bin/{id}/permanent-delete",
                post(permanent_delete_recovery_bin),
            )
            // WebSocket edge (Collab etc.) — upgrade at gateway then proxy to product
            .route("/p/{slug}/ws", get(ws_proxy_root))
            .route("/p/{slug}/ws/{*rest}", get(ws_proxy))
            .route("/p/{slug}/{*rest}", any(proxy_product))
            .route("/p/{slug}", any(proxy_product_root))
            .route("/core/{service}/{*rest}", any(proxy_core))
            .route("/core/{service}", any(proxy_core_root))
            .with_state(state.clone()),
    );

    serve_with_shutdown(cfg_addr, app, "gateway", state.clone()).await
}

fn http_client_for(cfg: &shared_core::config::CoreConfig) -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(cfg.proxy_timeout_secs))
            .connect_timeout(std::time::Duration::from_secs(
                cfg.proxy_connect_timeout_secs,
            ))
            .build()
            .expect("reqwest")
    })
}

/// Resolve product upstream from config. Outside local/dev, a configured upstream is required.
fn resolve_product_upstream(
    cfg: &shared_core::config::CoreConfig,
    slug: &str,
    default_port: u16,
) -> HelixResult<String> {
    let is_local = cfg.environment == "local" || cfg.environment == "dev";
    let override_key = slug.to_ascii_lowercase();
    if let Some(url) = cfg.product_overrides.get(&override_key) {
        if !url.trim().is_empty() {
            return Ok(url.trim_end_matches('/').to_string());
        }
    }
    if let Some(host_tmpl) = cfg.product_upstream_template.as_deref() {
        let host = host_tmpl.replace("{slug}", slug);
        return Ok(format!("http://{host}:{default_port}"));
    }
    if let Some(base) = cfg.product_upstream_base.as_deref() {
        // base is a hostname suffix: e.g. "svc.cluster.local" → {slug}.svc.cluster.local
        return Ok(format!(
            "http://{slug}.{}:{default_port}",
            base.trim_start_matches('.')
        ));
    }
    if is_local {
        return Ok(format!("http://127.0.0.1:{default_port}"));
    }
    Err(HelixError::dependency(format!(
        "no upstream configured for product {slug}; set HELIX_PRODUCT_HOST or HELIX_UPSTREAM_{}",
        override_key.to_ascii_uppercase().replace('-', "_")
    )))
}

#[derive(Serialize)]
struct CatalogEntry {
    order: u8,
    slug: &'static str,
    title: &'static str,
    description: &'static str,
    tier: String,
    maturity: String,
    semantic_state: String,
    default_port: u16,
    upstream: String,
    gateway_prefix: String,
}

async fn catalog(State(state): State<AppState>) -> Json<ApiResponse<Vec<CatalogEntry>>> {
    let items = PRODUCT_CATALOG
        .iter()
        .map(|p| CatalogEntry {
            order: p.order,
            slug: p.slug,
            title: p.title,
            description: p.description,
            tier: format!("{:?}", p.tier).to_lowercase(),
            maturity: format!("{:?}", p.maturity).to_lowercase(),
            semantic_state: p.maturity.default_semantic_state().as_str().to_string(),
            default_port: p.default_port,
            upstream: resolve_product_upstream(&state.clients.config, p.slug, p.default_port)
                .unwrap_or_else(|_| format!("http://127.0.0.1:{}", p.default_port)),
            gateway_prefix: format!("/p/{}", p.slug),
        })
        .collect();
    Json(ApiResponse::ok(items))
}

async fn catalog_one(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ApiResponse<CatalogEntry>>, ApiError> {
    let p = PRODUCT_CATALOG
        .iter()
        .find(|p| p.slug == slug)
        .ok_or_else(|| HelixError::not_found(format!("product {slug}")))?;
    Ok(Json(ApiResponse::ok(CatalogEntry {
        order: p.order,
        slug: p.slug,
        title: p.title,
        description: p.description,
        tier: format!("{:?}", p.tier).to_lowercase(),
        maturity: format!("{:?}", p.maturity).to_lowercase(),
        semantic_state: p.maturity.default_semantic_state().as_str().to_string(),
        default_port: p.default_port,
        upstream: resolve_product_upstream(&state.clients.config, p.slug, p.default_port)
            .unwrap_or_else(|_| format!("http://127.0.0.1:{}", p.default_port)),
        gateway_prefix: format!("/p/{}", p.slug),
    })))
}

static HTTP: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

#[derive(Serialize)]
struct CatalogState {
    slug: &'static str,
    maturity: &'static str,
    semantic_state: &'static str,
    upstream_reachable: bool,
    detail: String,
}

async fn catalog_state(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ApiResponse<CatalogState>>, ApiError> {
    let p = PRODUCT_CATALOG
        .iter()
        .find(|p| p.slug == slug)
        .ok_or_else(|| HelixError::not_found(format!("product {slug}")))?;
    let upstream = resolve_product_upstream(&state.clients.config, p.slug, p.default_port)?;
    let (reachable, semantic, detail) = probe_product_state(&upstream, p.maturity).await;
    Ok(Json(ApiResponse::ok(CatalogState {
        slug: p.slug,
        maturity: p.maturity.as_str(),
        semantic_state: semantic.as_str(),
        upstream_reachable: reachable,
        detail,
    })))
}

async fn probe_product_state(
    upstream: &str,
    maturity: ProductMaturity,
) -> (bool, SemanticState, String) {
    if matches!(maturity, ProductMaturity::Scaffold) {
        return (
            false,
            SemanticState::Unknown,
            "scaffold product has no runtime".to_string(),
        );
    }
    let url = format!("{}/health", upstream.trim_end_matches('/'));
    match http_client().get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            (true, SemanticState::Completed, "healthy".to_string())
        }
        Ok(resp) => (
            false,
            SemanticState::Failed,
            format!("unhealthy: {}", resp.status()),
        ),
        Err(e) => (false, SemanticState::Failed, format!("unreachable: {e}")),
    }
}

async fn me(
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<shared_core::tenancy::Principal>>, ApiError> {
    Ok(Json(ApiResponse::ok(principal)))
}

/// Cross-product workspaces for the authenticated tenant (core edge API).
async fn list_all_workspaces(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(repo) = state.clients.workspaces.as_ref() {
        let rows = repo.list_for_tenant(principal.tenant_id).await?;
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "durable": true,
            "tenant_id": principal.tenant_id,
            "items": rows
        }))));
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": false,
        "tenant_id": principal.tenant_id,
        "items": []
    }))))
}

#[derive(Serialize)]
struct CoreRoute {
    service: &'static str,
    url: String,
    gateway_prefix: String,
}

async fn routes(State(state): State<AppState>) -> Json<ApiResponse<Vec<CoreRoute>>> {
    let e = &state.clients.config.endpoints;
    Json(ApiResponse::ok(vec![
        CoreRoute {
            service: "gateway",
            url: e.gateway.clone(),
            gateway_prefix: "/".into(),
        },
        CoreRoute {
            service: "agent-hub",
            url: e.agent_hub.clone(),
            gateway_prefix: "/core/agent-hub".into(),
        },
        CoreRoute {
            service: "vault",
            url: e.vault.clone(),
            gateway_prefix: "/core/vault".into(),
        },
        CoreRoute {
            service: "billing",
            url: e.billing.clone(),
            gateway_prefix: "/core/billing".into(),
        },
        CoreRoute {
            service: "observability",
            url: e.observability.clone(),
            gateway_prefix: "/core/observability".into(),
        },
        CoreRoute {
            service: "auth-adapter",
            url: e.auth_adapter.clone(),
            gateway_prefix: "/core/auth".into(),
        },
    ]))
}

async fn core_status(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let auth = state.clients.auth.health().await?;
    let otlp = state.clients.config.otlp_endpoint.clone();
    let kms_mode = state.clients.config.kms_mode.clone();
    let pay_provider = state.clients.config.payment_provider.clone();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "service": "gateway",
        "aetherid": {
            "mode": auth.mode,
            "kratos_reachable": auth.kratos_reachable,
            "dev_headers": state.clients.config.environment == "local"
                || state.clients.config.environment == "dev",
            "ory_login": "/core/auth/v1/ory/login"
        },
        "capabilities": {
            "vault_envelope": "HVA5",
            "kms_mode": kms_mode,
            "payment_provider": pay_provider.as_str(),
            "otlp": otlp,
            "gateway_proxy": true,
            "minio_objects": true,
            "api_keys": state.clients.api_keys.is_some(),
            "tenant_lifecycle": state.clients.tenants.is_some(),
            "resource_acl": state.clients.acl.is_some(),
            "governance": state.clients.governance.is_some(),
            "multi_region": state.clients.regions.is_some(),
            "ws_proxy": true,
            "rate_limit_rps": state.clients.config.rate_limit_rps,
            "max_body_bytes": state.clients.config.max_body_bytes,
            "security_headers": true,
            "graceful_shutdown": true
        },
        "enterprise_tier": "sovereign-core",
        "data_plane": {
            "postgres": state.clients.db_status.connected,
            "migrated": state.clients.db_status.migrated,
            "detail": state.clients.db_status.detail,
            "vault": if state.clients.has_db() {
                "postgres-aes-gcm-kms"
            } else {
                "memory-aes-gcm-kms"
            },
            "minio_bucket": state.clients.config.minio_bucket,
            "audit": if state.clients.has_db() { "postgres" } else { "memory" },
            "meter": if state.clients.has_db() { "postgres" } else { "memory" }
        },
        "products": PRODUCT_CATALOG.len(),
        "edge_mode": "gateway_proxy",
        "edge_note": "Product APIs via /p/{slug}/…; core via /core/{service}/…; direct ports still work"
    }))))
}

async fn core_inventory(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Read)?;
    let e = &state.clients.config.endpoints;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "core_services": [
            {"name": "gateway", "url": e.gateway, "port": 8080},
            {"name": "agent-hub", "url": e.agent_hub, "port": 8081},
            {"name": "vault", "url": e.vault, "port": 8082},
            {"name": "billing", "url": e.billing, "port": 8083},
            {"name": "observability", "url": e.observability, "port": 8084},
            {"name": "auth-adapter", "url": e.auth_adapter, "port": 8085},
        ],
        "products": PRODUCT_CATALOG.len(),
        "product_ports": PRODUCT_CATALOG.iter().map(|p| {
            serde_json::json!({"slug": p.slug, "port": p.default_port, "tier": format!("{:?}", p.tier).to_lowercase()})
        }).collect::<Vec<_>>(),
        "data_plane": ["postgres", "nats", "minio"],
        "optional_profiles": ["ory", "observability"],
    }))))
}

#[derive(Deserialize)]
struct CreateTenantBody {
    name: String,
    #[serde(default)]
    residency_region: Option<String>,
    #[serde(default)]
    plan_id: Option<String>,
    #[serde(default)]
    tenant_id: Option<String>,
}

async fn list_tenants(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Platform)?;
    let repo = state
        .clients
        .tenants
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for tenants"))?;
    let items = repo.list(200).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn create_tenant(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<CreateTenantBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Platform)?;
    let repo = state
        .clients
        .tenants
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for tenants"))?;
    let id = if let Some(raw) = body.tenant_id.as_deref() {
        raw.parse()
            .map_err(|_| HelixError::validation("invalid tenant_id"))?
    } else {
        TenantId::new()
    };
    let region = body
        .residency_region
        .unwrap_or_else(|| state.clients.config.data_residency_region.clone());
    let rec = repo
        .create(id, body.name.trim(), &region, body.plan_id.as_deref())
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "tenant.create".into(),
            resource_type: "tenant".into(),
            resource_id: id.to_string(),
            metadata: serde_json::json!({"name": rec.name, "region": rec.residency_region}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "tenant": rec }))))
}

async fn suspend_tenant(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Platform)?;
    let tid: TenantId = id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    let repo = state
        .clients
        .tenants
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let rec = repo.set_status(tid, TenantStatus::Suspended).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(tid),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "tenant.suspend".into(),
            resource_type: "tenant".into(),
            resource_id: tid.to_string(),
            metadata: serde_json::json!({}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "tenant": rec }))))
}

async fn activate_tenant(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Platform)?;
    let tid: TenantId = id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    let repo = state
        .clients
        .tenants
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    let rec = repo.set_status(tid, TenantStatus::Active).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "tenant": rec }))))
}

#[derive(Deserialize)]
struct IssueKeyBody {
    name: String,
    #[serde(default)]
    scopes: Option<String>,
}

async fn issue_api_key(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
    Json(body): Json<IssueKeyBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    let store = state
        .clients
        .api_keys
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for api keys"))?;
    let scopes = body
        .scopes
        .as_deref()
        .and_then(Scope::parse_list)
        .unwrap_or_else(|| vec![Scope::Read, Scope::Write]);
    let issued = store.issue(tid, &body.name, &scopes, None).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(tid),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "api_key.issue".into(),
            resource_type: "api_key".into(),
            resource_id: issued.record.id.to_string(),
            metadata: serde_json::json!({"name": issued.record.name, "prefix": issued.record.key_prefix}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "key": issued.record,
        "secret": issued.secret,
        "note": "Store secret now — it is not shown again"
    }))))
}

async fn list_api_keys(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    let store = state
        .clients
        .api_keys
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "items": store.list(tid).await?
    }))))
}

async fn revoke_api_key(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((tenant_id, id)): Path<(String, Uuid)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    let tid: TenantId = tenant_id
        .parse()
        .map_err(|_| HelixError::validation("invalid tenant_id"))?;
    if tid != principal.tenant_id && !principal.has_scope(&Scope::Platform) {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    let store = state
        .clients
        .api_keys
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required"))?;
    store.revoke(tid, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(tid),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "api_key.revoke".into(),
            resource_type: "api_key".into(),
            resource_id: id.to_string(),
            metadata: serde_json::json!({}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"revoked": id}))))
}

// ----- Resource ACL -----

fn acl_repo(state: &AppState) -> Result<&helix_db::ResourceAclRepo, HelixError> {
    state
        .clients
        .acl
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for ACL"))
}

fn gov_repo(state: &AppState) -> Result<&helix_db::GovernanceRepo, HelixError> {
    state
        .clients
        .governance
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for governance"))
}

fn region_repo(state: &AppState) -> Result<&helix_db::RegionRepo, HelixError> {
    state
        .clients
        .regions
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for regions"))
}

#[derive(Deserialize)]
struct AclGrantBody {
    principal_kind: String,
    principal_id: String,
    permissions: Vec<String>,
}

async fn acl_list(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((resource_type, resource_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Read)?;
    let items = acl_repo(&state)?
        .list_for_resource(principal.tenant_id, &resource_type, &resource_id)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn acl_grant(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((resource_type, resource_id)): Path<(String, String)>,
    Json(body): Json<AclGrantBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    let perms: Vec<helix_db::AclPermission> = body
        .permissions
        .iter()
        .filter_map(|p| helix_db::AclPermission::parse(p))
        .collect();
    if perms.is_empty() {
        return Err(HelixError::validation("permissions required").into());
    }
    let entry = acl_repo(&state)?
        .grant(
            principal.tenant_id,
            &resource_type,
            &resource_id,
            &body.principal_kind,
            &body.principal_id,
            &perms,
            Some(&principal.user_id.to_string()),
        )
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "acl.grant".into(),
            resource_type: resource_type.clone(),
            resource_id: resource_id.clone(),
            metadata: serde_json::json!({"principal": body.principal_id, "perms": body.permissions}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "entry": entry }))))
}

#[derive(Deserialize)]
struct AclRevokeBody {
    principal_kind: String,
    principal_id: String,
}

async fn acl_revoke(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((resource_type, resource_id)): Path<(String, String)>,
    Json(body): Json<AclRevokeBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    acl_repo(&state)?
        .revoke(
            principal.tenant_id,
            &resource_type,
            &resource_id,
            &body.principal_kind,
            &body.principal_id,
        )
        .await?;
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "revoked": true }),
    )))
}

async fn acl_check(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((resource_type, resource_id)): Path<(String, String)>,
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Read)?;
    let need = q
        .get("permission")
        .and_then(|p| helix_db::AclPermission::parse(p))
        .unwrap_or(helix_db::AclPermission::Read);
    let allowed = acl_repo(&state)?
        .check(&principal, &resource_type, &resource_id, need)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "allowed": allowed,
        "permission": need.as_str(),
        "resource_type": resource_type,
        "resource_id": resource_id
    }))))
}

// ----- Governance -----

#[derive(Deserialize)]
struct RetentionBody {
    resource_type: String,
    #[serde(default)]
    resource_id: Option<String>,
    retain_days: i32,
    #[serde(default = "default_expiry_action")]
    action_on_expiry: String,
    #[serde(default)]
    purpose: Option<String>,
}

fn default_expiry_action() -> String {
    "review".into()
}

async fn retention_set(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<RetentionBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    let pol = gov_repo(&state)?
        .set_retention(
            principal.tenant_id,
            &body.resource_type,
            body.resource_id.as_deref(),
            body.retain_days,
            &body.action_on_expiry,
            body.purpose.as_deref(),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "policy": pol }))))
}

async fn retention_list(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Read)?;
    let items = gov_repo(&state)?
        .list_retention(principal.tenant_id, None)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

#[derive(Deserialize)]
struct HoldBody {
    resource_type: String,
    resource_id: String,
    reason: String,
}

async fn hold_place(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<HoldBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    let hold = gov_repo(&state)?
        .place_hold(
            principal.tenant_id,
            &body.resource_type,
            &body.resource_id,
            &body.reason,
            &principal.user_id.to_string(),
        )
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "legal_hold.place".into(),
            resource_type: body.resource_type,
            resource_id: body.resource_id,
            metadata: serde_json::json!({"reason": body.reason}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "hold": hold }))))
}

async fn hold_release(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    gov_repo(&state)?
        .release_hold(principal.tenant_id, id)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "released": id }))))
}

#[derive(Deserialize)]
struct PurposeBody {
    resource_type: String,
    resource_id: String,
    purpose: String,
    #[serde(default = "default_basis")]
    legal_basis: String,
    #[serde(default)]
    subject_ref: Option<String>,
}

fn default_basis() -> String {
    "consent".into()
}

async fn purpose_bind(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<PurposeBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Write)?;
    let b = gov_repo(&state)?
        .bind_purpose(
            principal.tenant_id,
            &body.resource_type,
            &body.resource_id,
            &body.purpose,
            &body.legal_basis,
            body.subject_ref.as_deref(),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "binding": b }))))
}

async fn can_delete(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path((resource_type, resource_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<helix_db::DeleteDecision>>, ApiError> {
    principal.require_scope(Scope::Read)?;
    let d = gov_repo(&state)?
        .can_delete(principal.tenant_id, &resource_type, &resource_id)
        .await?;
    Ok(Json(ApiResponse::ok(d)))
}

// ----- Multi-region -----

async fn regions_list(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Read)?;
    let items = region_repo(&state)?.list().await?;
    let service_region = &state.clients.config.data_residency_region;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "service_region": service_region,
        "items": items
    }))))
}

#[derive(Deserialize)]
struct RegionStatusBody {
    status: String,
    #[serde(default)]
    lag_seconds: i32,
    #[serde(default = "default_true")]
    write_enabled: bool,
    #[serde(default = "default_true")]
    read_enabled: bool,
}

fn default_true() -> bool {
    true
}

async fn region_status_update(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(code): Path<String>,
    Json(body): Json<RegionStatusBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Platform)?;
    let rec = region_repo(&state)?
        .upsert_status(
            &code,
            &body.status,
            body.lag_seconds,
            body.write_enabled,
            body.read_enabled,
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "region": rec }))))
}

// ----- Recovery bin -----

async fn list_recovery_bin(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Query(q): Query<RecoveryBinQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Read)?;
    let items = gov_repo(&state)?
        .list_bin_for_tenant(principal.tenant_id, q.limit)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": items }))))
}

async fn get_recovery_bin(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<helix_db::governance::RecoveryBinEntry>>, ApiError> {
    principal.require_scope(Scope::Read)?;
    let entry = gov_repo(&state)?
        .get_bin_entry(principal.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found(format!("recovery bin entry {id}")))?;
    Ok(Json(ApiResponse::ok(entry)))
}

#[derive(Deserialize)]
struct RecoveryBinQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Deserialize)]
struct RecoveryBinActionBody {
    #[serde(default)]
    reason: String,
}

async fn restore_recovery_bin(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<RecoveryBinActionBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for recovery bin"))?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin tx: {e}")))?;
    let repo = helix_db::GovernanceRepo::new(pool.clone());
    let (resource_type, resource_id) = repo.restore_in_tx(&mut tx, principal.tenant_id, id).await?;
    tx.commit()
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin commit: {e}")))?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "recovery_bin.restore".into(),
            resource_type: resource_type.clone(),
            resource_id: resource_id.clone(),
            metadata: serde_json::json!({
                "bin_id": id.to_string(),
                "reason": body.reason,
            }),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "restored": id,
        "resource_type": resource_type,
        "resource_id": resource_id,
    }))))
}

async fn permanent_delete_recovery_bin(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<RecoveryBinActionBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    if !principal.can_permanently_delete() {
        return Err(HelixError::forbidden("admin/platform scope required").into());
    }
    if body.reason.trim().is_empty() {
        return Err(HelixError::validation("reason required for permanent deletion").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for recovery bin"))?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin tx: {e}")))?;
    let repo = helix_db::GovernanceRepo::new(pool.clone());
    let (resource_type, resource_id) = repo
        .permanently_delete_in_tx(&mut tx, principal.tenant_id, id)
        .await?;
    tx.commit()
        .await
        .map_err(|e| HelixError::dependency(format!("recovery bin commit: {e}")))?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "recovery_bin.permanent_delete".into(),
            resource_type: resource_type.clone(),
            resource_id: resource_id.clone(),
            metadata: serde_json::json!({
                "bin_id": id.to_string(),
                "reason": body.reason,
            }),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "permanently_deleted": id,
        "resource_type": resource_type,
        "resource_id": resource_id,
    }))))
}

// ----- WebSocket proxy -----

async fn ws_proxy_root(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    RequireAuth(principal): RequireAuth,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    ws_proxy_inner(state, slug, "/".into(), ws, headers, principal).await
}

async fn ws_proxy(
    State(state): State<AppState>,
    Path((slug, rest)): Path<(String, String)>,
    headers: HeaderMap,
    RequireAuth(principal): RequireAuth,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    let path = if rest.starts_with('/') {
        rest
    } else {
        format!("/{rest}")
    };
    ws_proxy_inner(state, slug, path, ws, headers, principal).await
}

async fn ws_proxy_inner(
    state: AppState,
    slug: String,
    path: String,
    ws: axum::extract::ws::WebSocketUpgrade,
    mut headers: HeaderMap,
    principal: shared_core::tenancy::Principal,
) -> Response {
    inject_principal_headers(&mut headers, &principal);
    let base = match product_upstream(&state, &slug) {
        Ok(u) => u,
        Err(e) => return ApiError(e).into_response(),
    };
    let http_url = format!("{}{}", base.trim_end_matches('/'), path);
    let ws_url = http_url
        .replacen("http://", "ws://", 1)
        .replacen("https://", "wss://", 1);

    // Forward auth headers to upstream handshake via custom request.
    ws.on_upgrade(move |client_socket| async move {
        if let Err(e) = bridge_websockets(client_socket, &ws_url, &headers).await {
            tracing::warn!(error = %e, %ws_url, "ws proxy bridge ended");
        }
    })
}

async fn bridge_websockets(
    client: axum::extract::ws::WebSocket,
    upstream_url: &str,
    headers: &HeaderMap,
) -> Result<(), String> {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::Message as TMsg;

    let mut req = upstream_url
        .into_client_request()
        .map_err(|e| format!("ws request: {e}"))?;
    for name in [
        "authorization",
        "x-session-token",
        "x-helix-dev-user",
        "cookie",
        "x-helix-api-key",
    ] {
        if let Some(v) = headers.get(name).and_then(|v| v.to_str().ok()) {
            req.headers_mut().insert(
                http::HeaderName::from_bytes(name.as_bytes()).map_err(|e| e.to_string())?,
                http::HeaderValue::from_str(v).map_err(|e| e.to_string())?,
            );
        }
    }

    let (upstream, _) = tokio_tungstenite::connect_async(req)
        .await
        .map_err(|e| format!("upstream connect {upstream_url}: {e}"))?;

    let (mut client_tx, mut client_rx) = client.split();
    let (mut up_tx, mut up_rx) = upstream.split();

    let c2u = async {
        while let Some(Ok(msg)) = client_rx.next().await {
            let mapped = match msg {
                axum::extract::ws::Message::Text(t) => TMsg::Text(t.to_string().into()),
                axum::extract::ws::Message::Binary(b) => TMsg::Binary(b.to_vec().into()),
                axum::extract::ws::Message::Ping(p) => TMsg::Ping(p.to_vec().into()),
                axum::extract::ws::Message::Pong(p) => TMsg::Pong(p.to_vec().into()),
                axum::extract::ws::Message::Close(_) => {
                    let _ = up_tx.close().await;
                    break;
                }
            };
            if up_tx.send(mapped).await.is_err() {
                break;
            }
        }
    };

    let u2c = async {
        while let Some(Ok(msg)) = up_rx.next().await {
            let mapped = match msg {
                TMsg::Text(t) => axum::extract::ws::Message::Text(t.to_string().into()),
                TMsg::Binary(b) => axum::extract::ws::Message::Binary(b.to_vec().into()),
                TMsg::Ping(p) => axum::extract::ws::Message::Ping(p.to_vec().into()),
                TMsg::Pong(p) => axum::extract::ws::Message::Pong(p.to_vec().into()),
                TMsg::Close(_) => {
                    let _ = client_tx
                        .send(axum::extract::ws::Message::Close(None))
                        .await;
                    break;
                }
                TMsg::Frame(_) => continue,
            };
            if client_tx.send(mapped).await.is_err() {
                break;
            }
        }
    };

    tokio::select! {
        _ = c2u => {},
        _ = u2c => {},
    }
    Ok(())
}

async fn proxy_product_root(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    method: Method,
    headers: HeaderMap,
    RequireAuth(principal): RequireAuth,
    req: Request,
) -> Response {
    let upstream = product_upstream(&state, &slug);
    proxy_to(
        upstream,
        "/",
        method,
        headers,
        req,
        &state.clients.config,
        &principal,
    )
    .await
}

async fn proxy_product(
    State(state): State<AppState>,
    Path((slug, rest)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    RequireAuth(principal): RequireAuth,
    req: Request,
) -> Response {
    let path = if rest.starts_with('/') {
        rest
    } else {
        format!("/{rest}")
    };
    let upstream = product_upstream(&state, &slug);
    proxy_to(
        upstream,
        &path,
        method,
        headers,
        req,
        &state.clients.config,
        &principal,
    )
    .await
}

async fn proxy_core_root(
    State(state): State<AppState>,
    Path(service): Path<String>,
    method: Method,
    headers: HeaderMap,
    RequireAuth(principal): RequireAuth,
    req: Request,
) -> Response {
    let upstream = core_upstream(&state, &service);
    proxy_to(
        upstream,
        "/",
        method,
        headers,
        req,
        &state.clients.config,
        &principal,
    )
    .await
}

async fn proxy_core(
    State(state): State<AppState>,
    Path((service, rest)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    RequireAuth(principal): RequireAuth,
    req: Request,
) -> Response {
    let path = if rest.starts_with('/') {
        rest
    } else {
        format!("/{rest}")
    };
    let upstream = core_upstream(&state, &service);
    proxy_to(
        upstream,
        &path,
        method,
        headers,
        req,
        &state.clients.config,
        &principal,
    )
    .await
}

fn product_upstream(state: &AppState, slug: &str) -> Result<String, HelixError> {
    let p = PRODUCT_CATALOG
        .iter()
        .find(|p| p.slug == slug)
        .ok_or_else(|| HelixError::not_found(format!("product {slug}")))?;
    resolve_product_upstream(&state.clients.config, p.slug, p.default_port)
}

fn core_upstream(state: &AppState, service: &str) -> Result<String, HelixError> {
    let e = &state.clients.config.endpoints;
    let url = match service {
        "agent-hub" | "agents" => e.agent_hub.clone(),
        "vault" => e.vault.clone(),
        "billing" => e.billing.clone(),
        "observability" | "obs" => e.observability.clone(),
        "auth" | "auth-adapter" => e.auth_adapter.clone(),
        other => {
            return Err(HelixError::not_found(format!("core service {other}")));
        }
    };
    Ok(url)
}

fn is_hop_by_hop(name: &str) -> bool {
    static HOP: std::sync::OnceLock<std::collections::HashSet<&'static str>> =
        std::sync::OnceLock::new();
    HOP.get_or_init(|| {
        [
            "connection",
            "keep-alive",
            "proxy-authenticate",
            "proxy-authorization",
            "proxy-connection",
            "te",
            "trailer",
            "transfer-encoding",
            "upgrade",
        ]
        .into_iter()
        .collect()
    })
    .contains(name.to_ascii_lowercase().as_str())
}

fn is_retryable(method: &Method) -> bool {
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::DELETE | Method::TRACE
    )
}

fn inject_principal_headers(headers: &mut HeaderMap, principal: &shared_core::tenancy::Principal) {
    headers.insert(
        "x-helix-tenant-id",
        principal.tenant_id.to_string().parse().unwrap(),
    );
    headers.insert(
        "x-helix-user-id",
        principal.user_id.to_string().parse().unwrap(),
    );
    let scopes = principal
        .scopes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(",");
    headers.insert("x-helix-scopes", scopes.parse().unwrap());
    headers.insert(
        "x-helix-residency",
        principal.residency_region.parse().unwrap(),
    );
}

async fn proxy_to(
    base: Result<String, HelixError>,
    path: &str,
    method: Method,
    mut headers: HeaderMap,
    req: Request,
    cfg: &shared_core::config::CoreConfig,
    principal: &shared_core::tenancy::Principal,
) -> Response {
    inject_principal_headers(&mut headers, principal);
    let base = match base {
        Ok(b) => b,
        Err(e) => return ApiError(e).into_response(),
    };

    let query = req
        .uri()
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    let url = format!("{}{}{}", base.trim_end_matches('/'), path, query);

    let fwd_host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("gateway")
        .to_string();

    // Buffer idempotent request bodies for safe retry; stream mutable bodies once.
    let idempotent = is_retryable(&method);
    let (body_bytes, mut body_stream): (Option<Bytes>, Option<Body>) = if idempotent {
        match axum::body::to_bytes(req.into_body(), 32 * 1024 * 1024).await {
            Ok(b) => (Some(b), None),
            Err(e) => {
                return ApiError(HelixError::internal(format!("proxy body: {e}"))).into_response();
            }
        }
    } else {
        (None, Some(req.into_body()))
    };

    let max_attempts = if idempotent {
        cfg.proxy_retry_count.saturating_add(1)
    } else {
        1
    };
    let mut last_err = None;
    for attempt in 0..max_attempts {
        let req_builder = http_client_for(cfg).request(
            method.as_str().parse().unwrap_or(reqwest::Method::GET),
            &url,
        );
        let mut b = req_builder;
        for (name, value) in headers.iter() {
            let lname = name.as_str().to_ascii_lowercase();
            if lname == "host" || lname == "content-length" || is_hop_by_hop(&lname) {
                continue;
            }
            if let Ok(v) = value.to_str() {
                b = b.header(name.as_str(), v);
            }
        }
        b = b
            .header("x-forwarded-proto", "http")
            .header("x-forwarded-host", &fwd_host)
            .header("x-forwarded-prefix", path);
        let body: reqwest::Body = if let Some(ref bytes) = body_bytes {
            bytes.clone().into()
        } else if let Some(body) = body_stream.take() {
            reqwest::Body::wrap_stream(body.into_data_stream())
        } else {
            reqwest::Body::from("")
        };
        match b.body(body).send().await {
            Ok(resp) => {
                let status =
                    StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
                let retryable_status = matches!(status.as_u16(), 502..=504);
                if attempt + 1 < max_attempts && retryable_status {
                    tokio::time::sleep(std::time::Duration::from_millis(
                        cfg.proxy_retry_backoff_ms,
                    ))
                    .await;
                    continue;
                }
                let mut out_headers = HeaderMap::new();
                for (k, v) in resp.headers().iter() {
                    if is_hop_by_hop(k.as_str()) {
                        continue;
                    }
                    if let Ok(name) = axum::http::HeaderName::try_from(k.as_str()) {
                        if let Ok(val) = HeaderValue::from_bytes(v.as_bytes()) {
                            out_headers.insert(name, val);
                        }
                    }
                }
                let stream = resp.bytes_stream();
                let mut response = Response::new(Body::from_stream(stream));
                *response.status_mut() = status;
                *response.headers_mut() = out_headers;
                return response;
            }
            Err(e) => {
                last_err = Some(e);
                if attempt + 1 < max_attempts {
                    tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                    continue;
                }
            }
        }
    }
    ApiError(HelixError::dependency(format!(
        "upstream {url}: {}",
        last_err
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown".into())
    )))
    .into_response()
}
