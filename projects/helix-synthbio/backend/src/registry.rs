//! HelixSynthBio registry routes — accessioned designs, immutable versions,
//! lineage, risk review, import, and evidence bundles.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::{Component, RegistryRepo, ReviewDecision, VersionInput};
use serde::Deserialize;
use service_kit::{ApiError, AppState, RequireAuth};
use shared_core::ApiResponse;
use uuid::Uuid;

use crate::{audit, require_pool};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/registry/designs",
            get(list_designs).post(create_design),
        )
        .route("/v1/registry/designs/{id}", get(get_design_360))
        .route(
            "/v1/registry/designs/{id}/versions",
            get(list_versions).post(add_version),
        )
        .route("/v1/registry/designs/{id}/bundle", get(get_bundle))
        .route("/v1/registry/designs/{id}/risk/review", post(review_risk))
        .route("/v1/registry/risk/queue", get(risk_queue))
        .route("/v1/registry/import", post(import_records))
}

// ——— payloads ———

#[derive(Deserialize)]
struct CreateDesignReq {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_access_class")]
    access_class: String,
    alphabet: String,
    topology: String,
    #[serde(default)]
    sequence_text: String,
    #[serde(default)]
    components: Vec<Component>,
    #[serde(default = "default_provenance")]
    provenance: String,
    #[serde(default)]
    notes: String,
    #[serde(default = "default_source_kind")]
    source_kind: String,
    #[serde(default)]
    source_name: String,
}

#[derive(Deserialize)]
struct AddVersionReq {
    alphabet: String,
    topology: String,
    #[serde(default)]
    sequence_text: String,
    #[serde(default)]
    components: Vec<Component>,
    #[serde(default = "default_provenance")]
    provenance: String,
    #[serde(default)]
    notes: String,
    #[serde(default = "default_source_kind")]
    source_kind: String,
    #[serde(default)]
    source_name: String,
}

#[derive(Deserialize)]
struct ReviewReq {
    state: String,
    reviewer: String,
    #[serde(default)]
    intended_use: String,
    #[serde(default)]
    policy_version: String,
    #[serde(default)]
    reasons: Vec<String>,
    #[serde(default)]
    conditions: String,
    #[serde(default)]
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    expected_state: Option<String>,
}

#[derive(Deserialize)]
struct ImportReq {
    #[serde(default = "default_format")]
    format: String,
    #[serde(default)]
    filename: String,
    content: String,
}

fn default_access_class() -> String {
    "internal".into()
}
fn default_provenance() -> String {
    "depositor-claimed".into()
}
fn default_source_kind() -> String {
    "manual".into()
}
fn default_format() -> String {
    "auto".into()
}

fn actor(p: &shared_core::tenancy::Principal) -> String {
    p.user_id.to_string()
}

// ——— handlers ———

async fn create_design(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateDesignReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let input = VersionInput {
        alphabet: body.alphabet,
        topology: body.topology,
        source_kind: body.source_kind,
        source_name: body.source_name,
        sequence_text: body.sequence_text,
        components: body.components,
        provenance: body.provenance,
        notes: body.notes,
    };
    let actor = actor(&p);
    let design = repo
        .create_design(
            p.tenant_id,
            &body.name,
            &body.description,
            &body.access_class,
            &input,
            &actor,
        )
        .await?;
    audit(
        &state,
        &p,
        "synthbio.registry.create",
        "synthbio.design",
        design.id,
        serde_json::json!({"accession": design.accession}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(design))))
}

async fn list_designs(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let items = repo.list_designs(p.tenant_id, false).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "items": items
    }))))
}

async fn get_design_360(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let view = repo
        .design_360(p.tenant_id, id)
        .await?
        .ok_or_else(|| shared_core::HelixError::not_found("design not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(view))))
}

async fn list_versions(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let items = repo.list_versions(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "items": items
    }))))
}

async fn add_version(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<AddVersionReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let input = VersionInput {
        alphabet: body.alphabet,
        topology: body.topology,
        source_kind: body.source_kind,
        source_name: body.source_name,
        sequence_text: body.sequence_text,
        components: body.components,
        provenance: body.provenance,
        notes: body.notes,
    };
    let actor = actor(&p);
    let version = repo.add_version(p.tenant_id, id, &input, &actor).await?;
    audit(
        &state,
        &p,
        "synthbio.registry.version",
        "synthbio.design",
        id,
        serde_json::json!({"version": version.version}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(version))))
}

async fn review_risk(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<ReviewReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let decision = ReviewDecision {
        state: body.state,
        intended_use: body.intended_use,
        policy_version: body.policy_version,
        reasons: body.reasons,
        conditions: body.conditions,
        expires_at: body.expires_at,
        expected_state: body.expected_state,
    };
    let case = repo
        .review_risk(p.tenant_id, id, &decision, &body.reviewer)
        .await?;
    audit(
        &state,
        &p,
        "synthbio.registry.review",
        "synthbio.design",
        id,
        serde_json::json!({"state": case.state, "reviewer": body.reviewer}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(case))))
}

async fn risk_queue(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let items: Vec<serde_json::Value> = repo
        .risk_queue(p.tenant_id)
        .await?
        .into_iter()
        .map(|(case, accession)| serde_json::json!({"case": case, "accession": accession}))
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "items": items
    }))))
}

async fn import_records(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<ImportReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let actor = actor(&p);
    let manifest = repo
        .import_records(p.tenant_id, &body.format, &body.content, &actor)
        .await?;
    audit(
        &state,
        &p,
        "synthbio.registry.import",
        "synthbio.import",
        Uuid::now_v7(),
        serde_json::json!({
            "filename": body.filename,
            "total": manifest.total_records,
            "accepted": manifest.accepted_count,
            "rejected": manifest.rejected_count,
        }),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(manifest))))
}

async fn get_bundle(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let bundle = repo
        .evidence_bundle(p.tenant_id, id)
        .await?
        .ok_or_else(|| shared_core::HelixError::not_found("design not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(bundle))))
}
