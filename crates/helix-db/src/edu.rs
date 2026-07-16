//! HelixEdu course + enrollment persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::{TenantId, UserId};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub level: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub course_id: Uuid,
    pub learner_id: UserId,
    pub learner_label: String,
    pub status: String,
    pub progress_pct: i32,
    pub enrolled_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct CourseRow {
    id: Uuid,
    tenant_id: Uuid,
    slug: String,
    title: String,
    description: String,
    level: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl CourseRow {
    fn into_course(self) -> Course {
        Course {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            slug: self.slug,
            title: self.title,
            description: self.description,
            level: self.level,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
        }
    }
}

const COURSE_SELECT: &str = r#"
    SELECT id, tenant_id, slug, title, description, level, status, metadata, created_at
    FROM edu.courses
"#;

#[derive(Clone)]
pub struct EduRepo {
    pool: PgPool,
}

impl EduRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_courses(&self, tenant_id: TenantId) -> HelixResult<Vec<Course>> {
        let rows: Vec<CourseRow> = sqlx::query_as(&format!(
            "{COURSE_SELECT} WHERE tenant_id = $1 ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu list courses: {e}")))?;
        Ok(rows.into_iter().map(CourseRow::into_course).collect())
    }

    pub async fn create_course(
        &self,
        tenant_id: TenantId,
        slug: &str,
        title: &str,
        description: &str,
        level: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Course> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let level = if level.trim().is_empty() {
            "beginner"
        } else {
            level.trim()
        };
        sqlx::query(
            r#"
            INSERT INTO edu.courses
                (id, tenant_id, slug, title, description, level, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,'draft',$7,$8,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(slug)
        .bind(title)
        .bind(description)
        .bind(level)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu create course: {e}")))?;
        Ok(Course {
            id,
            tenant_id,
            slug: slug.into(),
            title: title.into(),
            description: description.into(),
            level: level.into(),
            status: "draft".into(),
            metadata,
            created_at,
        })
    }

    pub async fn get_course(
        &self,
        tenant_id: TenantId,
        course_id: Uuid,
    ) -> HelixResult<Option<Course>> {
        let row: Option<CourseRow> =
            sqlx::query_as(&format!("{COURSE_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(course_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("edu get course: {e}")))?;
        Ok(row.map(CourseRow::into_course))
    }

    pub async fn publish_course(
        &self,
        tenant_id: TenantId,
        course_id: Uuid,
    ) -> HelixResult<Course> {
        let updated_at = Utc::now();
        let res = sqlx::query(
            r#"
            UPDATE edu.courses
            SET status = 'published', updated_at = $1
            WHERE tenant_id = $2 AND id = $3
            "#,
        )
        .bind(updated_at)
        .bind(tenant_id.as_uuid())
        .bind(course_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu publish course: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found("course not found"));
        }
        self.get_course(tenant_id, course_id)
            .await?
            .ok_or_else(|| HelixError::not_found("course not found"))
    }

    pub async fn list_enrollments(
        &self,
        tenant_id: TenantId,
        course_id: Option<Uuid>,
    ) -> HelixResult<Vec<Enrollment>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            course_id: Uuid,
            learner_id: Uuid,
            learner_label: String,
            status: String,
            progress_pct: i32,
            enrolled_at: DateTime<Utc>,
            completed_at: Option<DateTime<Utc>>,
        }
        let rows: Vec<Row> = if let Some(cid) = course_id {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, course_id, learner_id, learner_label, status,
                       progress_pct, enrolled_at, completed_at
                FROM edu.enrollments
                WHERE tenant_id = $1 AND course_id = $2
                ORDER BY enrolled_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(cid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, course_id, learner_id, learner_label, status,
                       progress_pct, enrolled_at, completed_at
                FROM edu.enrollments
                WHERE tenant_id = $1
                ORDER BY enrolled_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("edu list enrollments: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| Enrollment {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                course_id: r.course_id,
                learner_id: UserId::from_uuid(r.learner_id),
                learner_label: r.learner_label,
                status: r.status,
                progress_pct: r.progress_pct,
                enrolled_at: r.enrolled_at,
                completed_at: r.completed_at,
            })
            .collect())
    }

    pub async fn enroll(
        &self,
        tenant_id: TenantId,
        course_id: Uuid,
        learner_id: UserId,
        learner_label: &str,
    ) -> HelixResult<Enrollment> {
        let course = self
            .get_course(tenant_id, course_id)
            .await?
            .ok_or_else(|| HelixError::not_found("course not found"))?;
        if course.status != "published" && course.status != "draft" {
            return Err(HelixError::validation(format!(
                "course {} is not open for enrollment",
                course.slug
            )));
        }

        let id = Uuid::now_v7();
        let enrolled_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO edu.enrollments
                (id, tenant_id, course_id, learner_id, learner_label, status, progress_pct, enrolled_at)
            VALUES ($1,$2,$3,$4,$5,'active',0,$6)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(course_id)
        .bind(learner_id.as_uuid())
        .bind(learner_label)
        .bind(enrolled_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                HelixError::conflict("already enrolled in this course")
            } else {
                HelixError::dependency(format!("edu enroll: {e}"))
            }
        })?;

        Ok(Enrollment {
            id,
            tenant_id,
            course_id,
            learner_id,
            learner_label: learner_label.into(),
            status: "active".into(),
            progress_pct: 0,
            enrolled_at,
            completed_at: None,
        })
    }

    pub async fn update_progress(
        &self,
        tenant_id: TenantId,
        enrollment_id: Uuid,
        progress_pct: i32,
    ) -> HelixResult<Enrollment> {
        if !(0..=100).contains(&progress_pct) {
            return Err(HelixError::validation("progress_pct must be 0..=100"));
        }
        let completed_at = if progress_pct >= 100 {
            Some(Utc::now())
        } else {
            None
        };
        let status = if progress_pct >= 100 {
            "completed"
        } else {
            "active"
        };
        let res = sqlx::query(
            r#"
            UPDATE edu.enrollments
            SET progress_pct = $1,
                status = $2,
                completed_at = COALESCE($3, completed_at)
            WHERE tenant_id = $4 AND id = $5
            "#,
        )
        .bind(progress_pct)
        .bind(status)
        .bind(completed_at)
        .bind(tenant_id.as_uuid())
        .bind(enrollment_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu update progress: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found("enrollment not found"));
        }
        self.get_enrollment(tenant_id, enrollment_id)
            .await?
            .ok_or_else(|| HelixError::not_found("enrollment not found"))
    }

    pub async fn get_enrollment(
        &self,
        tenant_id: TenantId,
        enrollment_id: Uuid,
    ) -> HelixResult<Option<Enrollment>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            course_id: Uuid,
            learner_id: Uuid,
            learner_label: String,
            status: String,
            progress_pct: i32,
            enrolled_at: DateTime<Utc>,
            completed_at: Option<DateTime<Utc>>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, course_id, learner_id, learner_label, status,
                   progress_pct, enrolled_at, completed_at
            FROM edu.enrollments
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(enrollment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu get enrollment: {e}")))?;
        Ok(row.map(|r| Enrollment {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            course_id: r.course_id,
            learner_id: UserId::from_uuid(r.learner_id),
            learner_label: r.learner_label,
            status: r.status,
            progress_pct: r.progress_pct,
            enrolled_at: r.enrolled_at,
            completed_at: r.completed_at,
        }))
    }
}
