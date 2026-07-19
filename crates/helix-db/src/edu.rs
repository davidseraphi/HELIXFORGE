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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressHistoryRecord {
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub tenant_id: TenantId,
    pub progress_pct: i32,
    pub status: String,
    pub actor_id: Option<Uuid>,
    pub recorded_at: DateTime<Utc>,
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
            "{COURSE_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
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
        let row: Option<CourseRow> = sqlx::query_as(&format!(
            "{COURSE_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu get course: {e}")))?;
        Ok(row.map(CourseRow::into_course))
    }

    pub async fn update_course(
        &self,
        tenant_id: TenantId,
        course_id: Uuid,
        title: Option<String>,
        description: Option<String>,
        level: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> HelixResult<Course> {
        let updated_at = Utc::now();
        let res = sqlx::query(
            r#"
            UPDATE edu.courses
            SET updated_at = $1,
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                level = COALESCE(NULLIF($4,''), level),
                metadata = COALESCE($5, metadata)
            WHERE tenant_id = $6 AND id = $7 AND deleted_at IS NULL
            "#,
        )
        .bind(updated_at)
        .bind(title)
        .bind(description)
        .bind(level)
        .bind(metadata)
        .bind(tenant_id.as_uuid())
        .bind(course_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu update course: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found("course not found"));
        }
        self.get_course(tenant_id, course_id)
            .await?
            .ok_or_else(|| HelixError::not_found("course not found"))
    }

    pub async fn soft_delete_course(
        &self,
        tenant_id: TenantId,
        course_id: Uuid,
    ) -> HelixResult<Course> {
        let deleted_at = Utc::now();
        let res = sqlx::query(
            r#"
            UPDATE edu.courses
            SET deleted_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(course_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu soft delete course: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found("course not found"));
        }
        // Return the course row without exposing deleted_at.
        let row: Option<CourseRow> =
            sqlx::query_as(&format!("{COURSE_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(course_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("edu get course after delete: {e}")))?;
        row.map(CourseRow::into_course)
            .ok_or_else(|| HelixError::not_found("course not found"))
    }

    pub async fn restore_course(
        &self,
        tenant_id: TenantId,
        course_id: Uuid,
    ) -> HelixResult<Course> {
        let updated_at = Utc::now();
        let res = sqlx::query(
            r#"
            UPDATE edu.courses
            SET deleted_at = NULL,
                status = 'draft',
                updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NOT NULL
            "#,
        )
        .bind(updated_at)
        .bind(tenant_id.as_uuid())
        .bind(course_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu restore course: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found("course not found"));
        }
        self.get_course(tenant_id, course_id)
            .await?
            .ok_or_else(|| HelixError::not_found("course not found"))
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
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
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

    pub async fn unpublish_course(
        &self,
        tenant_id: TenantId,
        course_id: Uuid,
    ) -> HelixResult<Course> {
        let updated_at = Utc::now();
        let res = sqlx::query(
            r#"
            UPDATE edu.courses
            SET status = 'draft', updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(updated_at)
        .bind(tenant_id.as_uuid())
        .bind(course_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu unpublish course: {e}")))?;
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
        let id = Uuid::now_v7();
        let enrolled_at = Utc::now();
        // The published-course guard is part of the INSERT itself: a course
        // unpublished or deleted between a separate check and insert cannot
        // leak enrollments.
        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"
            INSERT INTO edu.enrollments
                (id, tenant_id, course_id, learner_id, learner_label, status, progress_pct, enrolled_at)
            SELECT $1, $2, $3, $4, $5, 'active', 0, $6
            FROM edu.courses c
            WHERE c.tenant_id = $2 AND c.id = $3 AND c.status = 'published' AND c.deleted_at IS NULL
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(course_id)
        .bind(learner_id.as_uuid())
        .bind(learner_label)
        .bind(enrolled_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") {
                HelixError::conflict("already enrolled in this course")
            } else {
                HelixError::dependency(format!("edu enroll: {e}"))
            }
        })?;
        if inserted.is_none() {
            return Err(HelixError::validation("course not found or not published"));
        }

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

    pub async fn withdraw_enrollment(
        &self,
        tenant_id: TenantId,
        enrollment_id: Uuid,
    ) -> HelixResult<Enrollment> {
        // Single guarded update: a concurrent withdraw or progress update
        // cannot double-apply or reopen the row.
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
            UPDATE edu.enrollments
            SET status = 'withdrawn'
            WHERE tenant_id = $1 AND id = $2 AND status <> 'withdrawn'
            RETURNING id, tenant_id, course_id, learner_id, learner_label, status,
                      progress_pct, enrolled_at, completed_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(enrollment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu withdraw enrollment: {e}")))?;

        let Some(r) = row else {
            // Distinguish "not found" from "already withdrawn" without
            // holding any lock across two statements.
            let exists: Option<(String,)> = sqlx::query_as(
                "SELECT status FROM edu.enrollments WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tenant_id.as_uuid())
            .bind(enrollment_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("edu check enrollment: {e}")))?;
            return match exists {
                Some((status,)) if status == "withdrawn" => {
                    Err(HelixError::validation("enrollment is already withdrawn"))
                }
                _ => Err(HelixError::not_found("enrollment not found")),
            };
        };

        Ok(Enrollment {
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
    }

    pub async fn update_progress(
        &self,
        tenant_id: TenantId,
        enrollment_id: Uuid,
        progress_pct: i32,
        actor_id: Option<Uuid>,
    ) -> HelixResult<Enrollment> {
        if !(0..=100).contains(&progress_pct) {
            return Err(HelixError::validation("progress_pct must be 0..=100"));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("edu progress tx begin: {e}")))?;

        let existing: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT status
            FROM edu.enrollments
            WHERE tenant_id = $1 AND id = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(enrollment_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("edu lock enrollment: {e}")))?;

        let current_status = existing
            .ok_or_else(|| HelixError::not_found("enrollment not found"))?
            .0;
        if current_status == "withdrawn" {
            return Err(HelixError::validation(
                "cannot update progress on a withdrawn enrollment",
            ));
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

        sqlx::query(
            r#"
            UPDATE edu.enrollments
            SET progress_pct = $1,
                status = $2,
                completed_at = $3
            WHERE tenant_id = $4 AND id = $5
            "#,
        )
        .bind(progress_pct)
        .bind(status)
        .bind(completed_at)
        .bind(tenant_id.as_uuid())
        .bind(enrollment_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("edu update progress: {e}")))?;

        let history_id = Uuid::now_v7();
        let recorded_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO edu.enrollment_progress_history
                (id, enrollment_id, tenant_id, progress_pct, status, actor_id, recorded_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            "#,
        )
        .bind(history_id)
        .bind(enrollment_id)
        .bind(tenant_id.as_uuid())
        .bind(progress_pct)
        .bind(status)
        .bind(actor_id)
        .bind(recorded_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("edu insert progress history: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("edu progress tx commit: {e}")))?;

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

    pub async fn list_progress_history(
        &self,
        tenant_id: TenantId,
        enrollment_id: Uuid,
    ) -> HelixResult<Vec<ProgressHistoryRecord>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            enrollment_id: Uuid,
            tenant_id: Uuid,
            progress_pct: i32,
            status: String,
            actor_id: Option<Uuid>,
            recorded_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, enrollment_id, tenant_id, progress_pct, status, actor_id, recorded_at
            FROM edu.enrollment_progress_history
            WHERE tenant_id = $1 AND enrollment_id = $2
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(enrollment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("edu list progress history: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| ProgressHistoryRecord {
                id: r.id,
                enrollment_id: r.enrollment_id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                progress_pct: r.progress_pct,
                status: r.status,
                actor_id: r.actor_id,
                recorded_at: r.recorded_at,
            })
            .collect())
    }
}
