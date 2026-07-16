//! Product service scaffold — every Helix product API reuses this.

use agent_framework::{AgentRun, AgentSpec};
use audit_log::AuditEvent;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use helix_db::AgentRunStore;
use serde::{Deserialize, Serialize};
use shared_core::project::product_by_slug;
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError};
use uuid::Uuid;

use crate::context::AppState;
use crate::error_map::ApiError;
use crate::middleware::RequireAuth;
use crate::ServiceBuilder;

/// Descriptor for a product API binary.
#[derive(Clone)]
pub struct ProductApp {
    pub slug: &'static str,
    pub title: &'static str,
    pub default_port: u16,
}

impl ProductApp {
    pub fn from_slug(slug: &'static str) -> Result<Self, HelixError> {
        let meta = product_by_slug(slug)
            .ok_or_else(|| HelixError::internal(format!("unknown product slug {slug}")))?;
        Ok(Self {
            slug: meta.slug,
            title: meta.title,
            default_port: meta.default_port,
        })
    }
}

/// Fully wired product service: health + domain + agents + audit.
pub struct ProductService;

impl ProductService {
    /// Build the standard product router (domain routes may be merged by caller).
    pub fn router(state: AppState, product: ProductApp) -> Router<AppState> {
        let _state = state;
        let slug = product.slug;
        Router::new()
            .route(
                "/v1/product",
                get({
                    let p = product.clone();
                    move || async move { product_info(p) }
                }),
            )
            .route(
                "/v1/workspaces",
                get(list_workspaces).post(create_workspace),
            )
            .route("/v1/agents", get(list_agents).post(run_agent))
            .route("/v1/agents/runs/{id}", get(get_run))
            .route("/v1/audit/recent", get(recent_audit))
            .route("/v1/usage", get(usage_summary))
            .layer(axum::Extension(ProductSlug(slug.to_string())))
    }

    /// Run a product API from `main`.
    pub async fn run(slug: &'static str) -> Result<(), HelixError> {
        let product = ProductApp::from_slug(slug)?;
        let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
        // Capture config before consuming the builder.
        let cfg = builder.config().clone();

        // Register a default product agent
        builder.clients().agents.register_agent(AgentSpec {
            name: format!("{slug}-assistant"),
            description: format!("{} default assistant", product.title),
            system_prompt: format!("You are the {title} assistant.", title = product.title),
            tools: vec!["echo".into(), "product_catalog".into()],
            max_steps: 8,
        });

        let state = builder.into_state();
        let app =
            ServiceBuilder::base_router(state.clone()).merge(Self::router(state.clone(), product));

        crate::serve_with_shutdown(cfg.listen_addr, app, slug, state).await?;
        Ok(())
    }

    /// Run with additional domain routes (standard product surface + custom handlers).
    pub async fn run_with_domain_routes(
        slug: &'static str,
        domain_routes: Router<AppState>,
    ) -> Result<(), HelixError> {
        let product = ProductApp::from_slug(slug)?;
        let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
        // Capture config before consuming the builder.
        let cfg = builder.config().clone();

        // Register a default product agent
        builder.clients().agents.register_agent(AgentSpec {
            name: format!("{slug}-assistant"),
            description: format!("{} default assistant", product.title),
            system_prompt: format!("You are the {title} assistant.", title = product.title),
            tools: vec!["echo".into(), "product_catalog".into()],
            max_steps: 8,
        });

        let state = builder.into_state();
        let app = ServiceBuilder::base_router(state.clone())
            .merge(Self::router(state.clone(), product))
            .merge(domain_routes);

        crate::serve_with_shutdown(cfg.listen_addr, app, slug, state).await?;
        Ok(())
    }
}

#[derive(Clone)]
struct ProductSlug(String);

#[derive(Serialize)]
struct ProductInfo {
    slug: String,
    title: String,
    description: String,
    tier: String,
    order: u8,
    nats_prefix: String,
}

fn product_info(product: ProductApp) -> Json<ApiResponse<ProductInfo>> {
    let meta = product_by_slug(product.slug).expect("validated");
    Json(ApiResponse::ok(ProductInfo {
        slug: meta.slug.into(),
        title: meta.title.into(),
        description: meta.description.into(),
        tier: format!("{:?}", meta.tier).to_lowercase(),
        order: meta.order,
        nats_prefix: meta.nats_prefix.into(),
    }))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub product: String,
}

#[derive(Deserialize)]
struct CreateWorkspace {
    name: String,
}

async fn list_workspaces(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    axum::Extension(ProductSlug(slug)): axum::Extension<ProductSlug>,
) -> Result<Json<ApiResponse<Vec<Workspace>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    state.clients.metrics.inc("workspaces.list", 1);

    if let Some(repo) = state.clients.workspaces.as_ref() {
        let rows = repo.list(principal.tenant_id, &slug).await?;
        let items = rows
            .into_iter()
            .map(|r| Workspace {
                id: r.id,
                name: r.name,
                product: r.product_slug,
            })
            .collect();
        return Ok(Json(ApiResponse::ok(items)));
    }

    // Offline fallback when Postgres is down
    Ok(Json(ApiResponse::ok(vec![Workspace {
        id: Uuid::nil(),
        name: format!("{slug} default (memory)"),
        product: slug,
    }])))
}

async fn create_workspace(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    axum::Extension(ProductSlug(slug)): axum::Extension<ProductSlug>,
    Json(body): Json<CreateWorkspace>,
) -> Result<Json<ApiResponse<Workspace>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.name.trim().is_empty() {
        return Err(HelixError::validation("name required").into());
    }

    let ws = if let Some(repo) = state.clients.workspaces.as_ref() {
        repo.ensure_tenant(
            principal.tenant_id,
            &principal.user_id.to_string(),
            &principal.residency_region,
        )
        .await?;
        let rec = repo
            .create(principal.tenant_id, &slug, body.name.trim())
            .await?;
        Workspace {
            id: rec.id,
            name: rec.name,
            product: rec.product_slug,
        }
    } else {
        Workspace {
            id: Uuid::now_v7(),
            name: body.name,
            product: slug.clone(),
        }
    };

    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "workspace.create".into(),
            resource_type: "workspace".into(),
            resource_id: ws.id.to_string(),
            metadata: serde_json::json!({"name": ws.name, "product": slug}),
            residency_region: principal.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            principal.tenant_id,
            &slug,
            "workspaces.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            &format!("helix.{slug}.workspace.created"),
            &serde_json::json!({"id": ws.id, "name": ws.name}),
        )
        .await
        .ok();
    state.clients.metrics.inc("workspaces.create", 1);
    Ok(Json(ApiResponse::ok(ws)))
}

#[derive(Deserialize)]
struct RunAgentBody {
    #[serde(default = "default_agent")]
    agent: String,
    #[serde(default)]
    input: serde_json::Value,
}

fn default_agent() -> String {
    "assistant".into()
}

async fn list_agents(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<Vec<AgentSpec>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(state.clients.agents.list_agents())))
}

async fn run_agent(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    axum::Extension(ProductSlug(slug)): axum::Extension<ProductSlug>,
    Json(body): Json<RunAgentBody>,
) -> Result<Json<ApiResponse<AgentRun>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Write)?;
    let agent_name = if body.agent == "assistant" {
        format!("{slug}-assistant")
    } else {
        body.agent
    };
    let run = state
        .clients
        .agents
        .run(
            &agent_name,
            principal.tenant_id,
            principal.user_id,
            body.input,
        )
        .await?;
    state
        .clients
        .billing
        .record_usage(
            principal.tenant_id,
            &slug,
            "agent.runs",
            1.0,
            "count",
            serde_json::json!({"agent": agent_name}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(run)))
}

async fn get_run(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<AgentRun>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let store = AgentRunStore::new(pool.clone());
        match store.get_for_tenant(principal.tenant_id, id).await? {
            Some(run) => return Ok(Json(ApiResponse::ok(run))),
            None => return Err(HelixError::not_found(format!("run {id}")).into()),
        }
    }
    let run = state
        .clients
        .agents
        .get_run(id)
        .ok_or_else(|| HelixError::not_found(format!("run {id}")))?;
    if run.tenant_id != principal.tenant_id
        && !principal.has_scope(&shared_core::tenancy::Scope::Platform)
    {
        return Err(HelixError::forbidden("tenant isolation").into());
    }
    Ok(Json(ApiResponse::ok(run)))
}

async fn recent_audit(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<Vec<audit_log::AuditEntry>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::AuditRead)?;
    // Tenant-scoped only; global audit chain is exposed via operator path, not here.
    let entries = state
        .clients
        .audit
        .list_for_tenant(principal.tenant_id, 50)
        .await?;
    Ok(Json(ApiResponse::ok(entries)))
}

async fn usage_summary(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    axum::Extension(ProductSlug(slug)): axum::Extension<ProductSlug>,
) -> Result<Json<ApiResponse<Vec<billing_client::UsageSummary>>>, ApiError> {
    principal.require_scope(shared_core::tenancy::Scope::Read)?;
    let summary = state
        .clients
        .billing
        .summarize(principal.tenant_id, &slug)
        .await?;
    Ok(Json(ApiResponse::ok(summary)))
}
