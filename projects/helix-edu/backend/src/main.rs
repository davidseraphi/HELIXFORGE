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
        .merge(domain_routes());

    let cfg = shared_core::CoreConfig::from_env("helix-edu", 8106)?;
    service_kit::serve_with_shutdown(cfg.listen_addr, app, "helix-edu", state).await?;
    Ok(())
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/domain/status", get(domain_status))
        .route("/v1/courses", get(list_courses).post(create_course))
        .route("/v1/courses/{id}", get(get_course).patch(update_course))
        .route("/v1/courses/{id}/publish", post(publish_course))
        .route("/v1/courses/{id}/unpublish", post(unpublish_course))
        .route("/v1/courses/{id}/delete", post(delete_course))
        .route("/v1/courses/{id}/restore", post(restore_course))
        .route("/v1/courses/{id}/enrollments", get(list_course_enrollments))
        .route("/v1/enrollments", get(list_enrollments).post(enroll))
        .route("/v1/enrollments/{id}", get(get_enrollment))
        .route("/v1/enrollments/{id}/progress", post(update_progress))
        .route("/v1/enrollments/{id}/withdraw", post(withdraw_enrollment))
}

async fn domain_status(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Read)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "domain": "helix-edu",
        "phase": "wave2_w4",
        "tenant": p.tenant_id.to_string(),
        "durable": state.clients.db.is_some(),
        "planes": {
            "courses": true,
            "enrollments": true,
            "publish_unpublish": true,
            "soft_delete": true,
            "withdraw": true,
            "progress_history": true,
            "audit": true,
            "metering": true,
            "nats": true
        }
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
    validate_course_input(&body.slug, &body.title)?;
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
    state
        .clients
        .bus
        .publish(
            "helix.edu.course.created",
            &serde_json::json!({
                "course_id": course.id,
                "slug": course.slug,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
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

#[derive(Deserialize, Default)]
struct UpdateCourse {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    level: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

async fn update_course(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCourse>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable courses"))?;
    let repo = EduRepo::new(pool.clone());
    let course = repo
        .update_course(
            p.tenant_id,
            id,
            body.title,
            body.description,
            body.level,
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
            action: "course.update".into(),
            resource_type: "course".into(),
            resource_id: course.id.to_string(),
            metadata: serde_json::json!({"slug": course.slug, "status": course.status}),
            residency_region: p.residency_region.clone(),
        })
        .await?;
    state
        .clients
        .billing
        .record_usage(
            p.tenant_id,
            "helix-edu",
            "courses.updated",
            1.0,
            "count",
            serde_json::json!({}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.edu.course.updated",
            &serde_json::json!({
                "course_id": course.id,
                "slug": course.slug,
                "tenant_id": p.tenant_id.to_string()
            }),
        )
        .await
        .ok();
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
        .billing
        .record_usage(
            p.tenant_id,
            "helix-edu",
            "courses.published",
            1.0,
            "count",
            serde_json::json!({}),
        )
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

async fn unpublish_course(
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
    let course = repo.unpublish_course(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "course.unpublish".into(),
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
            "helix.edu.course.unpublished",
            &serde_json::json!({"course_id": course.id, "slug": course.slug}),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(course))))
}

async fn delete_course(
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
    let course = repo.soft_delete_course(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "course.delete".into(),
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
            "helix.edu.course.deleted",
            &serde_json::json!({"course_id": course.id, "slug": course.slug}),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(course))))
}

async fn restore_course(
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
    let course = repo.restore_course(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "course.restore".into(),
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
            "helix.edu.course.restored",
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

async fn get_enrollment(
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
    let enrollment = repo
        .get_enrollment(p.tenant_id, id)
        .await?
        .ok_or_else(|| HelixError::not_found("enrollment not found"))?;
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
    validate_progress_pct(body.progress_pct)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable enrollments"))?;
    let repo = EduRepo::new(pool.clone());
    let enrollment = repo
        .update_progress(
            p.tenant_id,
            id,
            body.progress_pct,
            Some(p.user_id.as_uuid()),
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
            action: "enrollment.progress".into(),
            resource_type: "enrollment".into(),
            resource_id: enrollment.id.to_string(),
            metadata: serde_json::json!({
                "course_id": enrollment.course_id,
                "progress_pct": enrollment.progress_pct,
                "status": enrollment.status
            }),
            residency_region: p.residency_region.clone(),
        })
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

async fn withdraw_enrollment(
    State(state): State<AppState>,
    RequireAuth(p): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    p.require_scope(shared_core::tenancy::Scope::Write)?;
    let pool = state
        .clients
        .db
        .as_ref()
        .ok_or_else(|| HelixError::unavailable("Postgres required for durable enrollments"))?;
    let repo = EduRepo::new(pool.clone());
    let enrollment = repo.withdraw_enrollment(p.tenant_id, id).await?;
    state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(p.tenant_id),
            actor: Actor::User {
                user_id: p.user_id,
                tenant_id: p.tenant_id,
            },
            action: "enrollment.withdraw".into(),
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
            "enrollments.withdrawn",
            1.0,
            "count",
            serde_json::json!({"enrollment_id": enrollment.id}),
        )
        .await?;
    state
        .clients
        .bus
        .publish(
            "helix.edu.enrollment.withdrawn",
            &serde_json::json!({
                "enrollment_id": enrollment.id,
                "course_id": enrollment.course_id
            }),
        )
        .await
        .ok();
    Ok(Json(ApiResponse::ok(serde_json::json!(enrollment))))
}

fn validate_course_input(slug: &str, title: &str) -> HelixResult<()> {
    if slug.trim().is_empty() {
        return Err(HelixError::validation("slug required"));
    }
    if title.trim().is_empty() {
        return Err(HelixError::validation("title required"));
    }
    Ok(())
}

fn validate_progress_pct(progress_pct: i32) -> HelixResult<()> {
    if !(0..=100).contains(&progress_pct) {
        return Err(HelixError::validation("progress_pct must be 0..=100"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use service_kit::{AppState, ProductApp, ServiceBuilder};
    use shared_core::tenancy::{Principal, Scope};
    use shared_core::{TenantId, UserId};
    use tokio::sync::{Mutex, MutexGuard};
    use uuid::Uuid;

    use super::*;

    static INIT_ENV: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

    fn init_test_env() {
        INIT_ENV.call_once(|| {
            std::env::set_var("HELIX_ENV", "local");
            std::env::set_var("HELIX_LOCAL_DEV_UNSAFE", "1");
            std::env::set_var("HELIX_ALLOW_DEV_HEADERS", "1");
            std::env::set_var("HELIX_DEV_PLATFORM", "1");
            std::env::set_var("PORT", "18106");
            std::env::set_var("LOG_JSON", "false");
            std::env::set_var("HELIX_DB_POOL_MAX_CONNECTIONS", "4");
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        });
    }

    async fn locked_state() -> (AppState, MutexGuard<'static, ()>) {
        init_test_env();
        let guard = TEST_MUTEX.lock().await;
        let product = ProductApp::from_slug("helix-edu").expect("helix-edu product known");
        let builder = ServiceBuilder::new(product.slug, product.default_port)
            .await
            .expect("ServiceBuilder requires Postgres + optional NATS/MinIO");
        (builder.into_state(), guard)
    }

    fn dev_principal(label: &str) -> Principal {
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

    #[test]
    fn progress_pct_boundary_rejects_out_of_range() {
        assert!(validate_progress_pct(-1).is_err());
        assert!(validate_progress_pct(0).is_ok());
        assert!(validate_progress_pct(100).is_ok());
        assert!(validate_progress_pct(101).is_err());
    }

    #[test]
    fn course_input_requires_slug_and_title() {
        assert!(validate_course_input("", "Title").is_err());
        assert!(validate_course_input("slug", "").is_err());
        assert!(validate_course_input("  ", "Title").is_err());
        assert!(validate_course_input("slug", "Title").is_ok());
    }

    #[test]
    fn duplicate_enrollment_error_message_contains_enrolled() {
        let msg = "already enrolled in this course";
        assert!(msg.contains("enrolled"));
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn progress_update_creates_history_rows() {
        let (state, _guard) = locked_state().await;
        let principal = dev_principal("history-learner");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = EduRepo::new(pool.clone());

        let course = repo
            .create_course(
                principal.tenant_id,
                "history-course",
                "History Course",
                "",
                "beginner",
                serde_json::json!({}),
            )
            .await
            .expect("create course");
        let published = repo
            .publish_course(principal.tenant_id, course.id)
            .await
            .expect("publish course");
        let enrollment = repo
            .enroll(
                principal.tenant_id,
                published.id,
                principal.user_id,
                "learner",
            )
            .await
            .expect("enroll");

        repo.update_progress(
            principal.tenant_id,
            enrollment.id,
            25,
            Some(principal.user_id.as_uuid()),
        )
        .await
        .expect("progress 25");
        repo.update_progress(
            principal.tenant_id,
            enrollment.id,
            75,
            Some(principal.user_id.as_uuid()),
        )
        .await
        .expect("progress 75");
        repo.update_progress(
            principal.tenant_id,
            enrollment.id,
            100,
            Some(principal.user_id.as_uuid()),
        )
        .await
        .expect("progress 100");

        let history = repo
            .list_progress_history(principal.tenant_id, enrollment.id)
            .await
            .expect("list history");
        assert_eq!(history.len(), 3, "expected three progress history rows");
        assert_eq!(history[0].progress_pct, 100, "latest row is 100%");

        let completed = repo
            .get_enrollment(principal.tenant_id, enrollment.id)
            .await
            .expect("reload enrollment")
            .expect("enrollment exists");
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());

        // Rolling progress back clears completion.
        repo.update_progress(
            principal.tenant_id,
            enrollment.id,
            90,
            Some(principal.user_id.as_uuid()),
        )
        .await
        .expect("progress 90");
        let rolled_back = repo
            .get_enrollment(principal.tenant_id, enrollment.id)
            .await
            .expect("reload enrollment")
            .expect("enrollment exists");
        assert_eq!(rolled_back.status, "active");
        assert!(rolled_back.completed_at.is_none());
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn concurrent_enroll_same_learner_single_winner() {
        let (state, _guard) = locked_state().await;
        let principal = dev_principal("enroll-race");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = EduRepo::new(pool.clone());

        let course = repo
            .create_course(
                principal.tenant_id,
                "race-course",
                "Race Course",
                "",
                "beginner",
                serde_json::json!({}),
            )
            .await
            .expect("create course");
        let published = repo
            .publish_course(principal.tenant_id, course.id)
            .await
            .expect("publish course");

        // 8 concurrent enrollments of the same learner in the same course.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            let tenant_id = principal.tenant_id;
            let user_id = principal.user_id;
            handles.push(tokio::spawn(async move {
                repo.enroll(tenant_id, published.id, user_id, "racer").await
            }));
        }
        let mut winners = 0usize;
        let mut conflicts = 0usize;
        for h in handles {
            match h.await.expect("enroll task panicked") {
                Ok(_) => winners += 1,
                Err(e) if e.code == shared_core::ErrorCode::Conflict => conflicts += 1,
                Err(e) => panic!("unexpected enroll error: {e}"),
            }
        }
        assert_eq!(winners, 1, "exactly one enrollment may win");
        assert_eq!(conflicts, 7, "all losers must get Conflict");

        let enrollments = repo
            .list_enrollments(principal.tenant_id, Some(published.id))
            .await
            .expect("list enrollments");
        assert_eq!(enrollments.len(), 1, "exactly one enrollment row exists");
    }

    #[tokio::test]
    #[ignore = "requires HelixCore data plane (Postgres)"]
    async fn enroll_rejected_when_unpublished() {
        let (state, _guard) = locked_state().await;
        let principal = dev_principal("enroll-guard");
        let pool = state.clients.db.as_ref().expect("Postgres required");
        let repo = EduRepo::new(pool.clone());

        let course = repo
            .create_course(
                principal.tenant_id,
                "guard-course",
                "Guard Course",
                "",
                "beginner",
                serde_json::json!({}),
            )
            .await
            .expect("create course");

        // 8 concurrent enroll attempts on a draft course: all rejected.
        let mut handles = Vec::new();
        for _ in 0..8u32 {
            let repo = repo.clone();
            let tenant_id = principal.tenant_id;
            let user_id = principal.user_id;
            handles.push(tokio::spawn(async move {
                repo.enroll(tenant_id, course.id, user_id, "guard").await
            }));
        }
        let mut rejected = 0usize;
        for h in handles {
            match h.await.expect("enroll task panicked") {
                Err(e) if e.code == shared_core::ErrorCode::Validation => rejected += 1,
                Ok(_) => panic!("enroll on a draft course must be rejected"),
                Err(e) => panic!("unexpected enroll error: {e}"),
            }
        }
        assert_eq!(rejected, 8, "every enroll on a draft course must fail");

        let enrollments = repo
            .list_enrollments(principal.tenant_id, Some(course.id))
            .await
            .expect("list enrollments");
        assert!(enrollments.is_empty(), "no enrollment may leak in");
    }
}
