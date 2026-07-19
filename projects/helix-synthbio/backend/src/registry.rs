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
        .route(
            "/v1/inventory/samples",
            get(list_samples).post(register_sample),
        )
        .route("/v1/inventory/samples/{id}", get(get_sample))
        .route("/v1/inventory/samples/{id}/custody", post(custody_event))
        .route("/v1/inventory/samples/{id}/aliquot", post(aliquot))
        .route("/v1/measurements", post(record_measurement))
        .route(
            "/v1/inventory/samples/{id}/measurements",
            get(list_measurements),
        )
        .route("/v1/measurements/{id}/accept", post(accept_measurement))
        .route("/v1/measurements/{id}/reject", post(reject_measurement))
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

// ——— inventory ———

#[derive(Deserialize)]
struct RegisterSampleReq {
    name: String,
    #[serde(default = "default_kind")]
    kind: String,
    #[serde(default)]
    design_id: Option<Uuid>,
    #[serde(default)]
    location: String,
}

#[derive(Deserialize)]
struct CustodyReq {
    event: String,
    #[serde(default)]
    to_location: String,
    #[serde(default)]
    notes: String,
}

#[derive(Deserialize)]
struct AliquotReq {
    name: String,
}

fn default_kind() -> String {
    "other".into()
}

async fn register_sample(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<RegisterSampleReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let actor = actor(&p);
    let sample = repo
        .register_sample(
            p.tenant_id,
            &body.name,
            &body.kind,
            body.design_id,
            &body.location,
            &actor,
        )
        .await?;
    audit(
        &state,
        &p,
        "synthbio.inventory.register",
        "synthbio.sample",
        sample.id,
        serde_json::json!({"accession": sample.accession, "kind": sample.kind}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(sample))))
}

async fn list_samples(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let items = repo.list_samples(p.tenant_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "items": items
    }))))
}

async fn get_sample(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let detail = repo
        .sample_detail(p.tenant_id, id)
        .await?
        .ok_or_else(|| shared_core::HelixError::not_found("sample not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(detail))))
}

async fn custody_event(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<CustodyReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let actor = actor(&p);
    let sample = repo
        .custody_event(p.tenant_id, id, &body.event, &body.to_location, &actor, &body.notes)
        .await?;
    audit(
        &state,
        &p,
        "synthbio.inventory.custody",
        "synthbio.sample",
        id,
        serde_json::json!({"event": body.event, "to": body.to_location}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(sample))))
}

async fn aliquot(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<AliquotReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let actor = actor(&p);
    let child = repo.aliquot(p.tenant_id, id, &body.name, &actor).await?;
    audit(
        &state,
        &p,
        "synthbio.inventory.aliquot",
        "synthbio.sample",
        child.id,
        serde_json::json!({"parent": id, "accession": child.accession}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(child))))
}

// ——— measurements ———

#[derive(Deserialize)]
struct MeasurementReq {
    sample_id: Uuid,
    #[serde(default)]
    design_version_id: Option<Uuid>,
    #[serde(default = "default_measurement_kind")]
    kind: String,
    #[serde(default)]
    method: String,
    #[serde(default)]
    value: Option<f64>,
    #[serde(default)]
    unit: String,
    #[serde(default)]
    uncertainty: Option<f64>,
    #[serde(default)]
    raw: serde_json::Value,
}

#[derive(Deserialize)]
struct VerdictReq {
    #[serde(default)]
    analyst: String,
}

fn default_measurement_kind() -> String {
    "other".into()
}

async fn record_measurement(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<MeasurementReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let input = helix_db::MeasurementInput {
        sample_id: body.sample_id,
        design_version_id: body.design_version_id,
        kind: body.kind,
        method: body.method,
        value: body.value,
        unit: body.unit,
        uncertainty: body.uncertainty,
        raw: body.raw,
    };
    let actor = actor(&p);
    let m = repo.record_measurement(p.tenant_id, &input, &actor).await?;
    audit(
        &state,
        &p,
        "synthbio.measurement.record",
        "synthbio.measurement",
        m.id,
        serde_json::json!({"accession": m.accession, "kind": m.kind, "sample": m.sample_id}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(m))))
}

async fn list_measurements(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let items = repo.list_measurements(p.tenant_id, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "items": items
    }))))
}

async fn accept_measurement(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<VerdictReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    verdict_measurement(state, p, id, "accept", body.analyst).await
}

async fn reject_measurement(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<VerdictReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    verdict_measurement(state, p, id, "reject", body.analyst).await
}

async fn verdict_measurement(
    state: AppState,
    p: shared_core::tenancy::Principal,
    id: Uuid,
    action: &str,
    analyst: String,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = require_pool(&state)?;
    let repo = RegistryRepo::new(pool);
    let who = if analyst.is_empty() { actor(&p) } else { analyst };
    let m = repo
        .transition_measurement(p.tenant_id, id, action, &who)
        .await?;
    audit(
        &state,
        &p,
        &format!("synthbio.measurement.{action}"),
        "synthbio.measurement",
        id,
        serde_json::json!({"accession": m.accession}),
    )
    .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(m))))
}
