#!/usr/bin/env python3
"""Generate thin durable helix_db slices for products 10–20."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# order, schema, slug, package, port, entity_table, entity_name, second_table, second_name, action_prefix
PRODUCTS = [
    (10, "studio", "helix-forge-studio", "helix_forge_studio_api", 8110, "apps", "App", "pages", "Page", "app"),
    (11, "synthbio", "helix-synthbio", "helix_synthbio_api", 8111, "designs", "Design", "sims", "SimRun", "design"),
    (12, "lex", "helix-lex-prime", "helix_lex_prime_api", 8112, "matters", "Matter", "filings", "Filing", "matter"),
    (13, "cura", "helix-cura-prime", "helix_cura_prime_api", 8113, "care_cases", "CareCase", "notes", "CareNote", "care_case"),
    (14, "terra", "helix-terra-prime", "helix_terra_prime_api", 8114, "fields", "Field", "observations", "Observation", "field"),
    (15, "climate", "helix-climate-prime", "helix_climate_prime_api", 8115, "scenarios", "Scenario", "risk_scores", "RiskScore", "scenario"),
    (16, "orbit", "helix-orbit-prime", "helix_orbit_prime_api", 8116, "assets", "SpaceAsset", "passes", "Pass", "asset"),
    (17, "quantum", "helix-quantum-forge", "helix_quantum_forge_api", 8117, "jobs", "QuantumJob", "circuits", "Circuit", "job"),
    (18, "vita", "helix-vita-prime", "helix_vita_prime_api", 8118, "studies", "Study", "cohorts", "Cohort", "study"),
    (19, "grid", "helix-grid-prime", "helix_grid_prime_api", 8119, "sites", "GridSite", "readings", "Reading", "site"),
    (20, "nova", "helix-nova-labs", "helix_nova_labs_api", 8120, "experiments", "Experiment", "findings", "Finding", "experiment"),
]


def migration_sql() -> str:
    parts = [
        "-- Thin durable domain tables for products 10–20 (widen pass)\n"
    ]
    for _, schema, *_ in PRODUCTS:
        parts.append(f"CREATE SCHEMA IF NOT EXISTS {schema};\n")
    parts.append("\n")
    for _, schema, _, _, _, primary, _, secondary, _, _ in PRODUCTS:
        parts.append(
            f"""
CREATE TABLE IF NOT EXISTS {schema}.{primary} (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{{}}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS {schema}_{primary}_tenant_idx ON {schema}.{primary} (tenant_id);

CREATE TABLE IF NOT EXISTS {schema}.{secondary} (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES {schema}.{primary}(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{{}}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS {schema}_{secondary}_parent_idx ON {schema}.{secondary} (parent_id);
CREATE INDEX IF NOT EXISTS {schema}_{secondary}_tenant_idx ON {schema}.{secondary} (tenant_id);
"""
        )
    return "".join(parts)


def rust_module(
    schema: str,
    primary: str,
    entity: str,
    secondary: str,
    child: str,
) -> str:
    repo = f"{entity}Repo" if entity.endswith("s") else f"{entity}Repo"
    # Use schema-based repo names: StudioRepo, SynthbioRepo, etc.
    repo = f"{schema.capitalize()}Repo"
    return f'''//! Helix product durable store — `{schema}` schema (thin widen slice).

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use shared_core::ids::TenantId;
use shared_core::{{HelixError, HelixResult}};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {entity} {{
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {child} {{
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}}

#[derive(Clone)]
pub struct {repo} {{
    pool: PgPool,
}}

impl {repo} {{
    pub fn new(pool: PgPool) -> Self {{
        Self {{ pool }}
    }}

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<{entity}>> {{
        #[derive(sqlx::FromRow)]
        struct Row {{
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            description: String,
            status: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }}
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, status, metadata, created_at
            FROM {schema}.{primary}
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("{schema} list: {{e}}")))?;
        Ok(rows
            .into_iter()
            .map(|r| {entity} {{
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                name: r.name,
                description: r.description,
                status: r.status,
                metadata: r.metadata,
                created_at: r.created_at,
            }})
            .collect())
    }}

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<{entity}> {{
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO {schema}.{primary}
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("{schema} create: {{e}}")))?;
        Ok({entity} {{
            id,
            tenant_id,
            name: name.into(),
            description: description.into(),
            status: "draft".into(),
            metadata,
            created_at,
        }})
    }}

    pub async fn get_parent(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<{entity}>> {{
        #[derive(sqlx::FromRow)]
        struct Row {{
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            description: String,
            status: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }}
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, status, metadata, created_at
            FROM {schema}.{primary}
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("{schema} get: {{e}}")))?;
        Ok(row.map(|r| {entity} {{
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            name: r.name,
            description: r.description,
            status: r.status,
            metadata: r.metadata,
            created_at: r.created_at,
        }}))
    }}

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<{child}>> {{
        #[derive(sqlx::FromRow)]
        struct Row {{
            id: Uuid,
            tenant_id: Uuid,
            parent_id: Uuid,
            title: String,
            body: String,
            status: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }}
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at
            FROM {schema}.{secondary}
            WHERE tenant_id = $1 AND parent_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("{schema} list children: {{e}}")))?;
        Ok(rows
            .into_iter()
            .map(|r| {child} {{
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                parent_id: r.parent_id,
                title: r.title,
                body: r.body,
                status: r.status,
                metadata: r.metadata,
                created_at: r.created_at,
            }})
            .collect())
    }}

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<{child}> {{
        let _parent = self
            .get_parent(tenant_id, parent_id)
            .await?
            .ok_or_else(|| HelixError::not_found("parent not found"))?;
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO {schema}.{secondary}
                (id, tenant_id, parent_id, title, body, status, metadata, created_at)
            VALUES ($1,$2,$3,$4,$5,'open',$6,$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .bind(title)
        .bind(body)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("{schema} create child: {{e}}")))?;
        Ok({child} {{
            id,
            tenant_id,
            parent_id,
            title: title.into(),
            body: body.into(),
            status: "open".into(),
            metadata,
            created_at,
        }})
    }}
}}
'''


def api_main(
    slug: str,
    port: int,
    schema: str,
    entity: str,
    child: str,
    primary_route: str,
    child_route: str,
    action: str,
) -> str:
    repo = f"{schema.capitalize()}Repo"
    return f'''//! {slug} API — durable thin slice via helix_db.

use audit_log::AuditEvent;
use axum::extract::{{Path, State}};
use axum::routing::get;
use axum::{{Json, Router}};
use helix_db::{repo};
use serde::Deserialize;
use service_kit::{{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder}};
use shared_core::tenancy::Actor;
use shared_core::{{ApiResponse, HelixError, HelixResult}};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {{
    let product = ProductApp::from_slug("{slug}")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder.clients().agents.register_agent(agent_framework::AgentSpec {{
        name: format!("{{}}-assistant", product.slug),
        description: format!("{{}} assistant", product.title),
        system_prompt: format!("You are the {{}} assistant.", product.title),
        tools: vec!["echo".into(), "product_catalog".into()],
        max_steps: 8,
    }});
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .merge(domain_routes().with_state(state));

    let cfg = shared_core::CoreConfig::from_env("{slug}", {port})?;
    let listener = tokio::net::TcpListener::bind(cfg.listen_addr)
        .await
        .map_err(|e| HelixError::internal(format!("bind: {{e}}")))?;
    tracing::info!(addr = %cfg.listen_addr, service = "{slug}", "listening");
    axum::serve(listener, app)
        .await
        .map_err(|e| HelixError::internal(format!("serve: {{e}}")))?;
    Ok(())
}}

fn domain_routes() -> Router<AppState> {{
    Router::new()
        .route("/v1/{primary_route}", get(list_parents).post(create_parent))
        .route("/v1/{primary_route}/{{id}}", get(get_parent))
        .route(
            "/v1/{primary_route}/{{id}}/{child_route}",
            get(list_children).post(create_child),
        )
        .route("/v1/domain/status", get(domain_status))
}}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {{
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({{
        "domain": "ready",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some()
    }}))))
}}

async fn list_parents(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {{
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {{
        let repo = {repo}::new(pool.clone());
        let items = repo.list_parents(p.tenant_id).await?;
        return Ok(Json(ApiResponse::ok(serde_json::json!({{
            "durable": true,
            "items": items
        }}))));
    }}
    Ok(Json(ApiResponse::ok(serde_json::json!({{
        "durable": false,
        "items": []
    }}))))
}}

#[derive(Deserialize)]
struct CreateParent {{
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    metadata: serde_json::Value,
}}

async fn create_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateParent>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {{
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.name.trim().is_empty() {{
        return Err(HelixError::validation("name required").into());
    }}
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable store"))?;
    let repo = {repo}::new(pool.clone());
    let item = repo
        .create_parent(p.tenant_id, body.name.trim(), &body.description, body.metadata)
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {{
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {{
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            }},
            action: "{action}.create".into(),
            resource_type: "{action}".into(),
            resource_id: item.id.to_string(),
            metadata: serde_json::json!({{"name": item.name}}),
            residency_region: p.residency_region.clone(),
        }})
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "{slug}",
            "{action}s.created",
            1.0,
            "count",
            serde_json::json!({{}}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}}

async fn get_parent(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {{
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable store"))?;
    let repo = {repo}::new(pool.clone());
    let item = repo
        .get_parent(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}}

async fn list_children(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {{
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable store"))?;
    let repo = {repo}::new(pool.clone());
    let items = repo.list_children(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({{
        "durable": true,
        "parent_id": id,
        "items": items
    }}))))
}}

#[derive(Deserialize)]
struct CreateChild {{
    title: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    metadata: serde_json::Value,
}}

async fn create_child(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateChild>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {{
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.title.trim().is_empty() {{
        return Err(HelixError::validation("title required").into());
    }}
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable store"))?;
    let repo = {repo}::new(pool.clone());
    let item = repo
        .create_child(
            p.tenant_id,
            id,
            body.title.trim(),
            &body.body,
            body.metadata,
        )
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "{slug}",
            "children.created",
            1.0,
            "count",
            serde_json::json!({{"parent_id": id}}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(item))))
}}
'''


def patch_cargo(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    if "helix_db" in text:
        return
    text = text.replace(
        "agent_framework = { workspace = true }\n",
        "agent_framework = { workspace = true }\n"
        "helix_db = { workspace = true }\n"
        "audit_log = { workspace = true }\n",
    )
    path.write_text(text, encoding="utf-8")


def main() -> None:
    mig = ROOT / "crates" / "helix-db" / "migrations" / "0010_products_10_20.sql"
    mig.write_text(migration_sql(), encoding="utf-8")
    print(f"wrote {mig}")

    mods = []
    uses = []
    for _, schema, slug, package, port, primary, entity, secondary, child, action in PRODUCTS:
        mod_path = ROOT / "crates" / "helix-db" / "src" / f"{schema}.rs"
        mod_path.write_text(
            rust_module(schema, primary, entity, secondary, child), encoding="utf-8"
        )
        print(f"wrote {mod_path}")
        mods.append(f"pub mod {schema};")
        repo = f"{schema.capitalize()}Repo"
        uses.append(
            f"pub use {schema}::{{{entity}, {child}, {repo}}};"
        )

        # API
        dir_name = slug  # helix-forge-studio
        main_path = ROOT / "projects" / dir_name / "backend" / "src" / "main.rs"
        cargo_path = ROOT / "projects" / dir_name / "backend" / "Cargo.toml"
        if not main_path.exists():
            raise SystemExit(f"missing {main_path}")
        main_path.write_text(
            api_main(slug, port, schema, entity, child, primary, secondary, action),
            encoding="utf-8",
        )
        patch_cargo(cargo_path)
        print(f"wrote {main_path}")

    # Update lib.rs: append mods if missing
    lib = ROOT / "crates" / "helix-db" / "src" / "lib.rs"
    lib_text = lib.read_text(encoding="utf-8")
    for line in mods + uses:
        if line not in lib_text:
            # insert mod lines after existing pub mod well;
            if line.startswith("pub mod "):
                if "pub mod well;" in lib_text and line not in lib_text:
                    lib_text = lib_text.replace(
                        "pub mod well;\n", f"pub mod well;\n{line}\n"
                    )
                elif line not in lib_text:
                    lib_text = line + "\n" + lib_text
            else:
                if not lib_text.rstrip().endswith(line):
                    lib_text = lib_text.rstrip() + "\n" + line + "\n"
    # cleaner rewrite of lib.rs
    lib.write_text(
        """//! HelixForge database layer — pool, migrations, durable product repositories.

pub mod audit_pg;
pub mod capital;
pub mod climate;
pub mod code;
pub mod collab;
pub mod commerce;
pub mod cura;
pub mod edu;
pub mod flow;
pub mod grid;
pub mod insights;
pub mod lex;
pub mod meter_pg;
pub mod network;
pub mod nova;
pub mod orbit;
pub mod pool;
pub mod quantum;
pub mod studio;
pub mod synthbio;
pub mod terra;
pub mod vita;
pub mod well;
pub mod workspace;

pub use audit_pg::PgAuditSink;
pub use capital::{Account, CapitalRepo, Journal, JournalLine, JournalLineInput};
pub use climate::{ClimateRepo, RiskScore, Scenario};
pub use code::{CodeRepo, CodeRepoStore};
pub use collab::{CollabDocument, CollabRepo, DocumentPatch, PresencePeer};
pub use commerce::{CommerceRepo, Order, OrderItem, OrderLineInput, Product};
pub use cura::{CareCase, CareNote, CuraRepo};
pub use edu::{Course, EduRepo, Enrollment};
pub use flow::{FlowRepo, Workflow, WorkflowRun};
pub use grid::{GridRepo, GridSite, Reading};
pub use insights::{Dataset, InsightsRepo, MetricDef, MetricPoint};
pub use lex::{Filing, LexRepo, Matter};
pub use meter_pg::PgMetering;
pub use network::{Connection, NetworkRepo, Opportunity, Profile};
pub use nova::{Experiment, Finding, NovaRepo};
pub use orbit::{OrbitRepo, Pass, SpaceAsset};
pub use pool::{connect_and_migrate, try_connect_and_migrate, DbPool, DbStatus};
pub use quantum::{Circuit, QuantumJob, QuantumRepo};
pub use studio::{App, Page, StudioRepo};
pub use synthbio::{Design, SimRun, SynthbioRepo};
pub use terra::{Field, Observation, TerraRepo};
pub use vita::{Cohort, Study, VitaRepo};
pub use well::{CheckIn, Habit, HabitLog, WellRepo};
pub use workspace::{WorkspaceRecord, WorkspaceRepo};
""",
        encoding="utf-8",
    )
    print(f"rewrote {lib}")

    # Fix entity names that don't match capitalize(schema)+Repo for multiword
    # StudioRepo from studio - OK
    # CareCase for cura - module uses CareCase and CuraRepo - need to fix cura module repo name
    # SpaceAsset for orbit - OrbitRepo
    # QuantumJob for quantum - QuantumRepo
    # GridSite for grid - GridRepo

    # Fix modules that used wrong entity names in capitalize scheme
    # cura: entity CareCase, child CareNote, repo CuraRepo - regenerate with correct repo name
    # Our rust_module uses schema.capitalize() + "Repo" so:
    # cura -> CuraRepo OK
    # CareCase OK
    # quantum -> QuantumRepo OK, QuantumJob OK
    # orbit -> OrbitRepo OK, SpaceAsset OK
    # grid -> GridRepo OK, GridSite OK
    # studio -> StudioRepo, App, Page OK
    # synthbio -> SynthbioRepo OK
    # climate -> ClimateRepo OK
    # nova -> NovaRepo OK
    # terra -> TerraRepo OK
    # vita -> VitaRepo OK
    # lex -> LexRepo OK

    print("done")


if __name__ == "__main__":
    main()
