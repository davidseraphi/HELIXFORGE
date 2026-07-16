//! HelixEdu API — adaptive AI learning & certification (durable via helix_db).

use audit_log::AuditEvent;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use helix_db::EduRepo;
use serde::Deserialize;
use service_kit::{ApiError, AppState, ProductApp, ProductService, RequireAuth, ServiceBuilder};
use shared_core::tenancy::Actor;
use shared_core::{ApiResponse, HelixError, HelixResult};
use uuid::Uuid;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let product = ProductApp::from_slug("helix-edu")?;
    let builder = ServiceBuilder::new(product.slug, product.default_port).await?;
    builder
        .clients()
        .agents
        .register_agent(agent_framework::AgentSpec {
            name: format!("{}-assistant", product.slug),
            description: format!("{} assistant", product.title),
            system_prompt: format!(
            "You are the {} learning assistant. Help design courses and track learner progress.",
            product.title
        ),
            tools: vec!["echo".into(), "product_catalog".into()],
            max_steps: 8,
        });
    let state = builder.into_state();
    let app = ServiceBuilder::base_router(state.clone())
        .merge(ProductService::router(state.clone(), product))
        .nest_service("/", domain_routes().with_state(state.clone()));

    let cfg = shared_core::CoreConfig::from_env("helix-edu", 8106)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-edu", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/courses", get(list_courses).post(create_course))
        .route("/v1/courses/{id}", get(get_course))
        .route("/v1/courses/{id}/publish", post(publish_course))
        .route("/v1/courses/{id}/enrollments", get(list_course_enrollments))
        .route("/v1/enrollments", get(list_enrollments).post(enroll))
        .route("/v1/enrollments/{id}/progress", post(update_progress))
        .route("/v1/domain/status", get(domain_status))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "ready",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some()
    }))))
}

async fn list_courses(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = EduRepo::new(pool.clone());
        let items = repo.list_courses(p.tenant_id).await?;
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "durable": true,
            "items": items
        }))));
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": false,
        "items": []
    }))))
}

#[derive(Deserialize)]
struct CreateCourse {
    slug: String,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_level")]
    level: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

fn default_level() -> String {
    "beginner".into()
}

async fn create_course(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<CreateCourse>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    if body.slug.trim().is_empty() {
        return Err(HelixError::validation("slug required").into());
    }
    if body.title.trim().is_empty() {
        return Err(HelixError::validation("title required").into());
    }
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable courses"))?;
    let repo = EduRepo::new(pool.clone());
    let course = repo
        .create_course(
            p.tenant_id,
            body.slug.trim(),
            body.title.trim(),
            &body.description,
            &body.level,
            body.metadata,
        )
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "course.create".into(),
            resource_type: "course".into(),
            resource_id: course.id.to_string(),
            metadata: serde_json::json!({"slug": course.slug, "title": course.title}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-edu",
            "courses.created",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(course))))
}

async fn get_course(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable courses"))?;
    let repo = EduRepo::new(pool.clone());
    let course = repo
        .get_course(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("course not found"))?;
    Ok(Json(ApiResponse::ok(serde_json::json!(course))))
}

async fn publish_course(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable courses"))?;
    let repo = EduRepo::new(pool.clone());
    let course = repo.publish_course(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "course.publish".into(),
            resource_type: "course".into(),
            resource_id: course.id.to_string(),
            metadata: serde_json::json!({"slug": course.slug}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.edu.course.published",
            &serde_json::json!({"course_id": course.id, "slug": course.slug}),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(course))))
}

#[derive(Deserialize)]
struct EnrollmentsQuery {
    course_id: Option<Uuid>,
}

async fn list_enrollments(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Query(q): Query<EnrollmentsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    if let Some(pool) = state.clients.db.as_ref() {
        let repo = EduRepo::new(pool.clone());
        let items = repo.list_enrollments(p.tenant_id, q.course_id).await?;
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "durable": true,
            "items": items
        }))));
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": false,
        "items": []
    }))))
}

async fn list_course_enrollments(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable enrollments"))?;
    let repo = EduRepo::new(pool.clone());
    let items = repo.list_enrollments(p.tenant_id, Some(id)).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "durable": true,
        "course_id": id,
        "items": items
    }))))
}

#[derive(Deserialize)]
struct EnrollBody {
    course_id: Uuid,
    #[serde(default)]
    learner_label: String,
}

async fn enroll(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Json(body): Json<EnrollBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable enrollments"))?;
    let label = if body.learner_label.trim().is_empty() {
        p.user_id.to_string()
    } else {
        body.learner_label
    };
    let repo = EduRepo::new(pool.clone());
    let enrollment = repo
        .enroll(p.tenant_id, body.course_id, p.user_id, &label)
        .await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "enrollment.create".into(),
            resource_type: "enrollment".into(),
            resource_id: enrollment.id.to_string(),
            metadata: serde_json::json!({"course_id": enrollment.course_id}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-edu",
            "enrollments.created",
            1.0,
            "count",
            serde_json::json!({"course_id": body.course_id}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.edu.enrollment.created",
            &serde_json::json!({
                "enrollment_id": enrollment.id,
                "course_id": enrollment.course_id
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(enrollment))))
}

#[derive(Deserialize)]
struct ProgressBody {
    progress_pct: i32,
}

async fn update_progress(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<ProgressBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable enrollments"))?;
    let repo = EduRepo::new(pool.clone());
    let enrollment = repo
        .update_progress(p.tenant_id, id, body.progress_pct)
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-edu",
            "progress.updates",
            1.0,
            "count",
            serde_json::json!({
                "enrollment_id": id,
                "progress_pct": body.progress_pct
            }),
        )
        .await?;
    if enrollment.status == "completed" {
        state
            .clients
            .bus
            .publish(
                "helix.edu.enrollment.completed",
                &serde_json::json!({
                    "enrollment_id": enrollment.id,
                    "course_id": enrollment.course_id
                }),
            )
            .await
            .ok();
    }
    Ok(Json(ApiResponse::ok(serde_json::json!(enrollment))))
}
