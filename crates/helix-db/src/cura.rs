//! HelixCura Prime durable store — `cura` schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareCase {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub discharged_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareNote {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub parent_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub signed_at: Option<DateTime<Utc>>,
    pub voided_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CuraSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_notes: i64,
    pub draft_notes: i64,
    pub signed_notes: i64,
    pub voided_notes: i64,
}

#[derive(Debug, Clone, Default)]
pub struct CaseUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct NoteUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Validate a case lifecycle transition and return the resulting status.
pub fn next_case_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "activate") => Ok("active"),
        ("active", "discharge") => Ok("discharged"),
        ("discharged", "reopen") => Ok("active"),
        (_, "activate") => Err(HelixError::validation(format!(
            "cannot activate a {current} case"
        ))),
        (_, "discharge") => Err(HelixError::validation(format!(
            "cannot discharge a {current} case"
        ))),
        (_, "reopen") => Err(HelixError::validation(format!(
            "cannot reopen a {current} case"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown case action {action}"
        ))),
    }
}

/// Validate a note lifecycle transition and return the resulting status.
pub fn next_note_status(current: &str, action: &str) -> HelixResult<&'static str> {
    match (current, action) {
        ("draft", "sign") => Ok("signed"),
        ("draft", "void") | ("signed", "void") => Ok("voided"),
        (_, "sign") => Err(HelixError::validation(format!(
            "cannot sign a {current} note"
        ))),
        (_, "void") => Err(HelixError::validation(format!(
            "cannot void a {current} note"
        ))),
        _ => Err(HelixError::validation(format!(
            "unknown note action {action}"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct CaseRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    description: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    activated_at: Option<DateTime<Utc>>,
    discharged_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl CaseRow {
    fn into_case(self) -> CareCase {
        CareCase {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            name: self.name,
            description: self.description,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            activated_at: self.activated_at,
            discharged_at: self.discharged_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct NoteRow {
    id: Uuid,
    tenant_id: Uuid,
    parent_id: Uuid,
    title: String,
    body: String,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    signed_at: Option<DateTime<Utc>>,
    voided_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

impl NoteRow {
    fn into_note(self) -> CareNote {
        CareNote {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            parent_id: self.parent_id,
            title: self.title,
            body: self.body,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
            updated_at: self.updated_at,
            signed_at: self.signed_at,
            voided_at: self.voided_at,
            deleted_at: self.deleted_at,
        }
    }
}

const CASE_SELECT: &str = r#"
    SELECT id, tenant_id, name, description, status, metadata, created_at,
           activated_at, discharged_at, deleted_at
    FROM cura.care_cases
"#;

const CASE_RETURNING: &str = r#"
    RETURNING id, tenant_id, name, description, status, metadata, created_at,
              activated_at, discharged_at, deleted_at
"#;

const NOTE_SELECT: &str = r#"
    SELECT id, tenant_id, parent_id, title, body, status, metadata, created_at,
           updated_at, signed_at, voided_at, deleted_at
    FROM cura.notes
"#;

const NOTE_RETURNING: &str = r#"
    RETURNING id, tenant_id, parent_id, title, body, status, metadata, created_at,
              updated_at, signed_at, voided_at, deleted_at
"#;

#[derive(Clone)]
pub struct CuraRepo {
    pool: PgPool,
}

impl CuraRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Cases ---

    pub async fn list_parents(&self, tenant_id: TenantId) -> HelixResult<Vec<CareCase>> {
        let rows: Vec<CaseRow> = sqlx::query_as(&format!(
            "{CASE_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura list: {e}")))?;
        Ok(rows.into_iter().map(CaseRow::into_case).collect())
    }

    pub async fn create_parent(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<CareCase> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let row: CaseRow = sqlx::query_as(&format!(
            r#"
            INSERT INTO cura.care_cases
                (id, tenant_id, name, description, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$6)
            {CASE_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(&metadata)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura create: {e}")))?;
        Ok(row.into_case())
    }

    pub async fn get_parent(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<CareCase>> {
        let row: Option<CaseRow> = sqlx::query_as(&format!(
            "{CASE_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura get: {e}")))?;
        Ok(row.map(CaseRow::into_case))
    }

    async fn fetch_case_any(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<CareCase>> {
        let row: Option<CaseRow> =
            sqlx::query_as(&format!("{CASE_SELECT} WHERE tenant_id = $1 AND id = $2"))
                .bind(tenant_id.as_uuid())
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("cura fetch case: {e}")))?;
        Ok(row.map(CaseRow::into_case))
    }

    pub async fn update_case(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
        update: CaseUpdate,
    ) -> HelixResult<CareCase> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE cura.care_cases SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(n) = update.name {
            builder.push(", name = ");
            builder.push_bind(n);
        }
        if let Some(d) = update.description {
            builder.push(", description = ");
            builder.push_bind(d);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(case_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(format!(" {CASE_RETURNING}"));

        let row: Option<CaseRow> = builder
            .build_query_as::<CaseRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("cura update case: {e}")))?;

        row.map(CaseRow::into_case)
            .ok_or_else(|| HelixError::not_found("case not found"))
    }

    pub async fn activate_case(&self, tenant_id: TenantId, case_id: Uuid) -> HelixResult<CareCase> {
        let case = self
            .get_parent(tenant_id, case_id)
            .await?
            .ok_or_else(|| HelixError::not_found("case not found"))?;
        let next = next_case_status(&case.status, "activate")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<CaseRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.care_cases
            SET status = $1, activated_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {CASE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .bind(&case.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura activate case: {e}")))?;

        row.map(CaseRow::into_case)
            .ok_or_else(|| HelixError::conflict("case changed during activate; retry"))
    }

    /// Discharge an active case. Rejected while draft notes remain. The
    /// active-status and no-draft-notes guards are part of the UPDATE
    /// itself, so a concurrent discharge or a note created mid-flight
    /// cannot slip through a check-then-act window; the earlier reads only
    /// shape the error returned for the steady-state cases.
    pub async fn discharge_case(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
    ) -> HelixResult<CareCase> {
        let case = self
            .get_parent(tenant_id, case_id)
            .await?
            .ok_or_else(|| HelixError::not_found("case not found"))?;
        let next = next_case_status(&case.status, "discharge")?;

        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM cura.notes WHERE tenant_id = $1 AND parent_id = $2 AND status = 'draft' AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura discharge note count: {e}")))?;
        if drafts > 0 {
            return Err(HelixError::validation(format!(
                "case has {drafts} draft note(s); sign or void them first"
            )));
        }

        let now = Utc::now();
        let row: Option<CaseRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.care_cases
            SET status = $1, discharged_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = 'active' AND deleted_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM cura.notes n
                  WHERE n.tenant_id = $3 AND n.parent_id = $4
                    AND n.status = 'draft' AND n.deleted_at IS NULL
              )
            {CASE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura discharge case: {e}")))?;

        row.map(CaseRow::into_case).ok_or_else(|| {
            HelixError::conflict("case changed during discharge or gained a draft note; retry")
        })
    }

    pub async fn reopen_case(&self, tenant_id: TenantId, case_id: Uuid) -> HelixResult<CareCase> {
        let case = self
            .get_parent(tenant_id, case_id)
            .await?
            .ok_or_else(|| HelixError::not_found("case not found"))?;
        let next = next_case_status(&case.status, "reopen")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<CaseRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.care_cases
            SET status = $1, discharged_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND status = $5 AND deleted_at IS NULL
            {CASE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .bind(&case.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura reopen case: {e}")))?;

        row.map(CaseRow::into_case)
            .ok_or_else(|| HelixError::conflict("case changed during reopen; retry"))
    }

    pub async fn soft_delete_case(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
    ) -> HelixResult<CareCase> {
        let case = self
            .get_parent(tenant_id, case_id)
            .await?
            .ok_or_else(|| HelixError::not_found("case not found"))?;
        if case.status == "deleted" {
            return Err(HelixError::validation("case is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<CaseRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.care_cases
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            {CASE_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura soft-delete case: {e}")))?;

        row.map(CaseRow::into_case)
            .ok_or_else(|| HelixError::not_found("case not found"))
    }

    /// Restore a soft-deleted case, returning it to its pre-delete status.
    pub async fn restore_case(&self, tenant_id: TenantId, case_id: Uuid) -> HelixResult<CareCase> {
        let case = self
            .fetch_case_any(tenant_id, case_id)
            .await?
            .ok_or_else(|| HelixError::not_found("case not found"))?;
        if case.deleted_at.is_none() {
            return Err(HelixError::validation("case is not deleted"));
        }
        let restored = if case.discharged_at.is_some() {
            "discharged"
        } else if case.activated_at.is_some() {
            "active"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<CaseRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.care_cases
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND id = $4 AND deleted_at IS NOT NULL
            {CASE_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura restore case: {e}")))?;

        row.map(CaseRow::into_case)
            .ok_or_else(|| HelixError::not_found("case not found or not deleted"))
    }

    // --- Notes ---

    pub async fn list_children(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
    ) -> HelixResult<Vec<CareNote>> {
        let rows: Vec<NoteRow> = sqlx::query_as(&format!(
            "{NOTE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura list children: {e}")))?;
        Ok(rows.into_iter().map(NoteRow::into_note).collect())
    }

    pub async fn create_child(
        &self,
        tenant_id: TenantId,
        parent_id: Uuid,
        title: &str,
        body: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<CareNote> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        // The non-deleted-parent guard is part of the INSERT itself: a case
        // soft-deleted between a separate check and insert cannot leak notes.
        let row: Option<NoteRow> = sqlx::query_as(&format!(
            r#"
            INSERT INTO cura.notes
                (id, tenant_id, parent_id, title, body, status, metadata, created_at, updated_at)
            SELECT $1,$2,$3,$4,$5,'draft',$6,$7,$7
            FROM cura.care_cases c
            WHERE c.tenant_id = $2 AND c.id = $3 AND c.deleted_at IS NULL
            {NOTE_RETURNING}
            "#
        ))
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(parent_id)
        .bind(title)
        .bind(body)
        .bind(&metadata)
        .bind(created_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura create child: {e}")))?;
        row.map(NoteRow::into_note)
            .ok_or_else(|| HelixError::not_found("parent not found"))
    }

    pub async fn get_note(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
        note_id: Uuid,
    ) -> HelixResult<Option<CareNote>> {
        let row: Option<NoteRow> = sqlx::query_as(&format!(
            "{NOTE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .bind(note_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura get note: {e}")))?;
        Ok(row.map(NoteRow::into_note))
    }

    async fn fetch_note_any(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
        note_id: Uuid,
    ) -> HelixResult<Option<CareNote>> {
        let row: Option<NoteRow> = sqlx::query_as(&format!(
            "{NOTE_SELECT} WHERE tenant_id = $1 AND parent_id = $2 AND id = $3"
        ))
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .bind(note_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura fetch note: {e}")))?;
        Ok(row.map(NoteRow::into_note))
    }

    /// Edit a note. Signed or voided notes are immutable.
    pub async fn update_note(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
        note_id: Uuid,
        update: NoteUpdate,
    ) -> HelixResult<CareNote> {
        let note = self
            .get_note(tenant_id, case_id, note_id)
            .await?
            .ok_or_else(|| HelixError::not_found("note not found"))?;
        if note.status != "draft" {
            return Err(HelixError::validation(format!(
                "cannot edit a {} note; void it and write a new one",
                note.status
            )));
        }

        let mut builder = sqlx::QueryBuilder::new("UPDATE cura.notes SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(t) = update.title {
            builder.push(", title = ");
            builder.push_bind(t);
        }
        if let Some(b) = update.body {
            builder.push(", body = ");
            builder.push_bind(b);
        }
        if let Some(m) = update.metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND parent_id = ");
        builder.push_bind(case_id);
        builder.push(" AND id = ");
        builder.push_bind(note_id);
        // The draft-only guard is part of the UPDATE itself: a sign landing
        // between the read above and this write cannot be overwritten —
        // signed notes stay immutable under race.
        builder.push(" AND status = 'draft' AND deleted_at IS NULL");
        builder.push(format!(" {NOTE_RETURNING}"));

        let row: Option<NoteRow> = builder
            .build_query_as::<NoteRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("cura update note: {e}")))?;

        row.map(NoteRow::into_note)
            .ok_or_else(|| HelixError::conflict("note changed during edit; retry"))
    }

    pub async fn sign_note(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
        note_id: Uuid,
    ) -> HelixResult<CareNote> {
        let note = self
            .get_note(tenant_id, case_id, note_id)
            .await?
            .ok_or_else(|| HelixError::not_found("note not found"))?;
        let next = next_note_status(&note.status, "sign")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<NoteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.notes
            SET status = $1, signed_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND status = $6 AND deleted_at IS NULL
            {NOTE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .bind(note_id)
        .bind(&note.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura sign note: {e}")))?;

        row.map(NoteRow::into_note)
            .ok_or_else(|| HelixError::conflict("note changed during sign; retry"))
    }

    pub async fn void_note(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
        note_id: Uuid,
    ) -> HelixResult<CareNote> {
        let note = self
            .get_note(tenant_id, case_id, note_id)
            .await?
            .ok_or_else(|| HelixError::not_found("note not found"))?;
        let next = next_note_status(&note.status, "void")?;
        let now = Utc::now();
        // The expected-from status is part of the UPDATE: a concurrent
        // transition in between loses instead of overwriting.
        let row: Option<NoteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.notes
            SET status = $1, voided_at = $2, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND status = $6 AND deleted_at IS NULL
            {NOTE_RETURNING}
            "#
        ))
        .bind(next)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .bind(note_id)
        .bind(&note.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura void note: {e}")))?;

        row.map(NoteRow::into_note)
            .ok_or_else(|| HelixError::conflict("note changed during void; retry"))
    }

    pub async fn soft_delete_note(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
        note_id: Uuid,
    ) -> HelixResult<CareNote> {
        let note = self
            .get_note(tenant_id, case_id, note_id)
            .await?
            .ok_or_else(|| HelixError::not_found("note not found"))?;
        if note.status == "deleted" {
            return Err(HelixError::validation("note is already deleted"));
        }
        let deleted_at = Utc::now();
        let row: Option<NoteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.notes
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND parent_id = $3 AND id = $4 AND deleted_at IS NULL
            {NOTE_RETURNING}
            "#
        ))
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .bind(note_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura soft-delete note: {e}")))?;

        row.map(NoteRow::into_note)
            .ok_or_else(|| HelixError::not_found("note not found"))
    }

    /// Restore a soft-deleted note, returning it to its pre-delete status.
    pub async fn restore_note(
        &self,
        tenant_id: TenantId,
        case_id: Uuid,
        note_id: Uuid,
    ) -> HelixResult<CareNote> {
        let note = self
            .fetch_note_any(tenant_id, case_id, note_id)
            .await?
            .ok_or_else(|| HelixError::not_found("note not found"))?;
        if note.deleted_at.is_none() {
            return Err(HelixError::validation("note is not deleted"));
        }
        let restored = if note.voided_at.is_some() {
            "voided"
        } else if note.signed_at.is_some() {
            "signed"
        } else {
            "draft"
        };
        let now = Utc::now();
        let row: Option<NoteRow> = sqlx::query_as(&format!(
            r#"
            UPDATE cura.notes
            SET status = $1, deleted_at = NULL, updated_at = $2
            WHERE tenant_id = $3 AND parent_id = $4 AND id = $5 AND deleted_at IS NOT NULL
            {NOTE_RETURNING}
            "#
        ))
        .bind(restored)
        .bind(now)
        .bind(tenant_id.as_uuid())
        .bind(case_id)
        .bind(note_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura restore note: {e}")))?;

        row.map(NoteRow::into_note)
            .ok_or_else(|| HelixError::not_found("note not found or not deleted"))
    }

    // --- Reports ---

    /// Per-case note counts by status for non-deleted cases.
    pub async fn get_cura_summary(&self, tenant_id: TenantId) -> HelixResult<Vec<CuraSummaryRow>> {
        let rows: Vec<CuraSummaryRow> = sqlx::query_as(
            r#"
            SELECT c.id, c.name, c.status,
                   COUNT(n.id) AS total_notes,
                   COUNT(n.id) FILTER (WHERE n.status = 'draft') AS draft_notes,
                   COUNT(n.id) FILTER (WHERE n.status = 'signed') AS signed_notes,
                   COUNT(n.id) FILTER (WHERE n.status = 'voided') AS voided_notes
            FROM cura.care_cases c
            LEFT JOIN cura.notes n
                   ON n.parent_id = c.id AND n.tenant_id = c.tenant_id
                  AND n.deleted_at IS NULL
            WHERE c.tenant_id = $1 AND c.deleted_at IS NULL
            GROUP BY c.id, c.name, c.status, c.created_at
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cura summary: {e}")))?;
        Ok(rows)
    }
}
