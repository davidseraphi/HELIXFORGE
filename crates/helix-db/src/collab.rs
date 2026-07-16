//! HelixCollab document + presence persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::{TenantId, UserId};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabDocument {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub workspace_id: Option<Uuid>,
    #[serde(default)]
    pub folder_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub version: u32,
    #[serde(default)]
    pub encrypted: bool,
    /// Client-held keys; server stores ciphertext only (no vault open).
    #[serde(default)]
    pub client_e2ee: bool,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub archived_at: Option<DateTime<Utc>>,
    pub created_by: Option<UserId>,
    pub updated_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabFolder {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub workspace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub created_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentComment {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub document_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: UserId,
    pub author_label: String,
    pub body: String,
    #[serde(default)]
    pub anchor_start: Option<i32>,
    #[serde(default)]
    pub anchor_end: Option<i32>,
    #[serde(default)]
    pub anchor_quote: String,
    #[serde(default)]
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub resolved_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub mentions: Vec<Mention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocActivity {
    pub id: Uuid,
    pub document_id: Uuid,
    pub actor_id: Option<UserId>,
    pub actor_label: String,
    pub action: String,
    pub detail: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub document_id: Uuid,
    pub mentioned_user_id: Option<UserId>,
    pub mentioned_label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPatch {
    pub base_version: u32,
    pub content: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRevision {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version: u32,
    pub content: String,
    pub author_id: Option<UserId>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresencePeer {
    pub document_id: Uuid,
    pub user_id: UserId,
    pub display_name: String,
    pub cursor_pos: i32,
    pub last_seen: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CollabRepo {
    pool: PgPool,
}

impl CollabRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    const DOC_COLS: &'static str = r#"
            id, tenant_id, workspace_id, folder_id, title, content, version,
            encrypted, client_e2ee, pinned, archived_at,
            created_by, updated_by, created_at, updated_at
    "#;

    pub async fn list_documents(&self, tenant_id: TenantId) -> HelixResult<Vec<CollabDocument>> {
        self.list_documents_filtered(tenant_id, None, None, false)
            .await
    }

    /// List docs optionally scoped to workspace and/or folder.
    /// `root_only`: when true and folder_id is None, only docs with folder_id IS NULL.
    pub async fn list_documents_filtered(
        &self,
        tenant_id: TenantId,
        workspace_id: Option<Uuid>,
        folder_id: Option<Uuid>,
        root_only: bool,
    ) -> HelixResult<Vec<CollabDocument>> {
        let rows = sqlx::query_as::<_, DocRow>(&format!(
            r#"
            SELECT {}
            FROM collab.documents
            WHERE tenant_id = $1
              AND ($2::uuid IS NULL OR workspace_id = $2)
              AND (
                ($3::uuid IS NOT NULL AND folder_id = $3)
                OR ($3::uuid IS NULL AND $4::bool = false)
                OR ($3::uuid IS NULL AND $4::bool = true AND folder_id IS NULL)
              )
              AND archived_at IS NULL
            ORDER BY pinned DESC, updated_at DESC
            "#,
            Self::DOC_COLS
        ))
        .bind(tenant_id.as_uuid())
        .bind(workspace_id)
        .bind(folder_id)
        .bind(root_only)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("collab list: {e}")))?;
        Ok(rows.into_iter().map(DocRow::into_doc).collect())
    }

    pub async fn get_document(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<CollabDocument> {
        let row = sqlx::query_as::<_, DocRow>(&format!(
            r#"
            SELECT {}
            FROM collab.documents
            WHERE tenant_id = $1 AND id = $2
            "#,
            Self::DOC_COLS
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("collab get: {e}")))?
        .ok_or_else(|| HelixError::not_found(format!("document {id}")))?;
        Ok(row.into_doc())
    }

    /// Resolve document by id only (WS handshake). Caller must enforce tenant match.
    pub async fn get_document_by_id(&self, id: Uuid) -> HelixResult<CollabDocument> {
        let row = sqlx::query_as::<_, DocRow>(&format!(
            r#"
            SELECT {}
            FROM collab.documents
            WHERE id = $1
            "#,
            Self::DOC_COLS
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("collab get by id: {e}")))?
        .ok_or_else(|| HelixError::not_found(format!("document {id}")))?;
        Ok(row.into_doc())
    }

    pub async fn create_document(
        &self,
        tenant_id: TenantId,
        author: UserId,
        title: &str,
        content: &str,
        workspace_id: Option<Uuid>,
    ) -> HelixResult<CollabDocument> {
        self.create_document_in(tenant_id, author, title, content, workspace_id, None)
            .await
    }

    pub async fn create_document_in(
        &self,
        tenant_id: TenantId,
        author: UserId,
        title: &str,
        content: &str,
        workspace_id: Option<Uuid>,
        folder_id: Option<Uuid>,
    ) -> HelixResult<CollabDocument> {
        self.create_document_full(
            tenant_id,
            author,
            title,
            content,
            workspace_id,
            folder_id,
            false,
        )
        .await
    }

    pub async fn create_document_full(
        &self,
        tenant_id: TenantId,
        author: UserId,
        title: &str,
        content: &str,
        workspace_id: Option<Uuid>,
        folder_id: Option<Uuid>,
        encrypted: bool,
    ) -> HelixResult<CollabDocument> {
        self.create_document_full_ex(
            tenant_id,
            author,
            title,
            content,
            workspace_id,
            folder_id,
            encrypted,
            false,
        )
        .await
    }

    pub async fn create_document_full_ex(
        &self,
        tenant_id: TenantId,
        author: UserId,
        title: &str,
        content: &str,
        workspace_id: Option<Uuid>,
        folder_id: Option<Uuid>,
        encrypted: bool,
        client_e2ee: bool,
    ) -> HelixResult<CollabDocument> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let encrypted = encrypted || client_e2ee;
        sqlx::query(
            r#"
            INSERT INTO collab.documents
                (id, tenant_id, workspace_id, folder_id, title, content, version, encrypted,
                 client_e2ee, created_by, updated_by, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,1,$7,$8,$9,$9,$10,$10)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(workspace_id)
        .bind(folder_id)
        .bind(title)
        .bind(content)
        .bind(encrypted)
        .bind(client_e2ee)
        .bind(author.as_uuid())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("collab create: {e}")))?;

        sqlx::query(
            r#"
            INSERT INTO collab.document_revisions (id, document_id, version, content, author_id, created_at)
            VALUES ($1, $2, 1, $3, $4, $5)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(id)
        .bind(content)
        .bind(author.as_uuid())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("collab revision: {e}")))?;

        self.get_document(tenant_id, id).await
    }

    /// Optimistic concurrency patch: fails with Conflict when base_version mismatches.
    pub async fn apply_patch(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        author: UserId,
        patch: DocumentPatch,
    ) -> HelixResult<CollabDocument> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("collab tx: {e}")))?;

        let current = sqlx::query_as::<_, DocRow>(&format!(
            r#"
            SELECT {}
            FROM collab.documents
            WHERE tenant_id = $1 AND id = $2
            FOR UPDATE
            "#,
            Self::DOC_COLS
        ))
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("collab lock: {e}")))?
        .ok_or_else(|| HelixError::not_found(format!("document {id}")))?;

        if current.version as u32 != patch.base_version {
            return Err(HelixError::conflict(format!(
                "version conflict: expected {}, found {}",
                patch.base_version, current.version
            )));
        }

        let new_version = current.version + 1;
        let now = Utc::now();

        let title = patch
            .title
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(current.title.as_str());

        sqlx::query(
            r#"
            UPDATE collab.documents
            SET content = $1, title = $2, version = $3, updated_by = $4, updated_at = $5
            WHERE id = $6
            "#,
        )
        .bind(&patch.content)
        .bind(title)
        .bind(new_version)
        .bind(author.as_uuid())
        .bind(now)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("collab update: {e}")))?;

        sqlx::query(
            r#"
            INSERT INTO collab.document_revisions (id, document_id, version, content, author_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(id)
        .bind(new_version)
        .bind(&patch.content)
        .bind(author.as_uuid())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("collab rev insert: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("collab commit: {e}")))?;

        self.get_document(tenant_id, id).await
    }

    pub async fn list_revisions(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        limit: i64,
    ) -> HelixResult<Vec<DocumentRevision>> {
        // Ensure document belongs to tenant.
        let _ = self.get_document(tenant_id, document_id).await?;
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            document_id: Uuid,
            version: i32,
            content: String,
            author_id: Option<Uuid>,
            created_at: DateTime<Utc>,
        }
        let limit = limit.clamp(1, 200);
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, document_id, version, content, author_id, created_at
            FROM collab.document_revisions
            WHERE document_id = $1
            ORDER BY version DESC
            LIMIT $2
            "#,
        )
        .bind(document_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("collab revisions: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| DocumentRevision {
                id: r.id,
                document_id: r.document_id,
                version: r.version as u32,
                content: r.content,
                author_id: r.author_id.map(UserId::from_uuid),
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn get_revision(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        version: u32,
    ) -> HelixResult<DocumentRevision> {
        let _ = self.get_document(tenant_id, document_id).await?;
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            document_id: Uuid,
            version: i32,
            content: String,
            author_id: Option<Uuid>,
            created_at: DateTime<Utc>,
        }
        let row: Row = sqlx::query_as(
            r#"
            SELECT id, document_id, version, content, author_id, created_at
            FROM collab.document_revisions
            WHERE document_id = $1 AND version = $2
            "#,
        )
        .bind(document_id)
        .bind(version as i32)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("collab get rev: {e}")))?
        .ok_or_else(|| HelixError::not_found(format!("revision {document_id}@{version}")))?;
        Ok(DocumentRevision {
            id: row.id,
            document_id: row.document_id,
            version: row.version as u32,
            content: row.content,
            author_id: row.author_id.map(UserId::from_uuid),
            created_at: row.created_at,
        })
    }

    /// Restore a historical revision as a new version (optimistic content write).
    pub async fn restore_revision(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        author: UserId,
        version: u32,
    ) -> HelixResult<CollabDocument> {
        let current = self.get_document(tenant_id, document_id).await?;
        let rev = self.get_revision(tenant_id, document_id, version).await?;
        self.apply_patch(
            tenant_id,
            document_id,
            author,
            DocumentPatch {
                base_version: current.version,
                content: rev.content,
                title: None,
            },
        )
        .await
    }

    pub async fn delete_document(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<()> {
        let res = sqlx::query("DELETE FROM collab.documents WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id.as_uuid())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("collab delete: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("document {id}")));
        }
        Ok(())
    }

    pub async fn upsert_presence(
        &self,
        document_id: Uuid,
        user_id: UserId,
        display_name: &str,
        cursor_pos: i32,
    ) -> HelixResult<PresencePeer> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.presence (document_id, user_id, display_name, cursor_pos, last_seen)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (document_id, user_id) DO UPDATE
            SET display_name = EXCLUDED.display_name,
                cursor_pos = EXCLUDED.cursor_pos,
                last_seen = EXCLUDED.last_seen
            "#,
        )
        .bind(document_id)
        .bind(user_id.as_uuid())
        .bind(display_name)
        .bind(cursor_pos)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("presence upsert: {e}")))?;

        Ok(PresencePeer {
            document_id,
            user_id,
            display_name: display_name.into(),
            cursor_pos,
            last_seen: now,
        })
    }

    pub async fn list_presence(&self, document_id: Uuid) -> HelixResult<Vec<PresencePeer>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            document_id: Uuid,
            user_id: Uuid,
            display_name: String,
            cursor_pos: i32,
            last_seen: DateTime<Utc>,
        }

        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT document_id, user_id, display_name, cursor_pos, last_seen
            FROM collab.presence
            WHERE document_id = $1
              AND last_seen > now() - interval '2 minutes'
            ORDER BY last_seen DESC
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("presence list: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| PresencePeer {
                document_id: r.document_id,
                user_id: UserId::from_uuid(r.user_id),
                display_name: r.display_name,
                cursor_pos: r.cursor_pos,
                last_seen: r.last_seen,
            })
            .collect())
    }

    // ----- folders -----

    pub async fn list_folders(
        &self,
        tenant_id: TenantId,
        workspace_id: Uuid,
    ) -> HelixResult<Vec<CollabFolder>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            workspace_id: Uuid,
            parent_id: Option<Uuid>,
            name: String,
            created_by: Option<Uuid>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, workspace_id, parent_id, name, created_by, created_at, updated_at
            FROM collab.folders
            WHERE tenant_id = $1 AND workspace_id = $2
            ORDER BY name ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("folder list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CollabFolder {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                workspace_id: r.workspace_id,
                parent_id: r.parent_id,
                name: r.name,
                created_by: r.created_by.map(UserId::from_uuid),
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn create_folder(
        &self,
        tenant_id: TenantId,
        workspace_id: Uuid,
        parent_id: Option<Uuid>,
        name: &str,
        author: UserId,
    ) -> HelixResult<CollabFolder> {
        let name = name.trim();
        if name.is_empty() {
            return Err(HelixError::validation("folder name required"));
        }
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.folders
                (id, tenant_id, workspace_id, parent_id, name, created_by, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(workspace_id)
        .bind(parent_id)
        .bind(name)
        .bind(author.as_uuid())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("folder create: {e}")))?;
        Ok(CollabFolder {
            id,
            tenant_id,
            workspace_id,
            parent_id,
            name: name.into(),
            created_by: Some(author),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn rename_folder(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        name: &str,
    ) -> HelixResult<CollabFolder> {
        let name = name.trim();
        if name.is_empty() {
            return Err(HelixError::validation("folder name required"));
        }
        let res = sqlx::query(
            r#"
            UPDATE collab.folders
            SET name = $3, updated_at = now()
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("folder rename: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("folder {id}")));
        }
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            workspace_id: Uuid,
            parent_id: Option<Uuid>,
            name: String,
            created_by: Option<Uuid>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }
        let r: Row = sqlx::query_as(
            r#"
            SELECT id, tenant_id, workspace_id, parent_id, name, created_by, created_at, updated_at
            FROM collab.folders WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("folder reload: {e}")))?;
        Ok(CollabFolder {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            workspace_id: r.workspace_id,
            parent_id: r.parent_id,
            name: r.name,
            created_by: r.created_by.map(UserId::from_uuid),
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }

    pub async fn delete_folder(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<()> {
        // Move docs to root then delete folder tree (children cascade).
        sqlx::query(
            "UPDATE collab.documents SET folder_id = NULL WHERE tenant_id = $1 AND folder_id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("folder clear docs: {e}")))?;
        let res = sqlx::query("DELETE FROM collab.folders WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id.as_uuid())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("folder delete: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("folder {id}")));
        }
        Ok(())
    }

    pub async fn move_document(
        &self,
        tenant_id: TenantId,
        doc_id: Uuid,
        folder_id: Option<Uuid>,
        workspace_id: Option<Uuid>,
    ) -> HelixResult<CollabDocument> {
        sqlx::query(
            r#"
            UPDATE collab.documents
            SET folder_id = $3,
                workspace_id = COALESCE($4, workspace_id),
                updated_at = now()
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(doc_id)
        .bind(folder_id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("doc move: {e}")))?;
        self.get_document(tenant_id, doc_id).await
    }

    // ----- comments + mentions -----

    pub async fn list_comments(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
    ) -> HelixResult<Vec<DocumentComment>> {
        let _ = self.get_document(tenant_id, document_id).await?;
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            document_id: Uuid,
            parent_id: Option<Uuid>,
            author_id: Uuid,
            author_label: String,
            body: String,
            #[sqlx(default)]
            anchor_start: Option<i32>,
            #[sqlx(default)]
            anchor_end: Option<i32>,
            #[sqlx(default)]
            anchor_quote: String,
            #[sqlx(default)]
            resolved_at: Option<DateTime<Utc>>,
            #[sqlx(default)]
            resolved_by: Option<Uuid>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, document_id, parent_id, author_id, author_label, body,
                   anchor_start, anchor_end, COALESCE(anchor_quote, '') AS anchor_quote,
                   resolved_at, resolved_by, created_at, updated_at
            FROM collab.comments
            WHERE document_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
        )
        .bind(document_id)
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("comments list: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let mentions = self.list_mentions_for_comment(r.id).await?;
            out.push(DocumentComment {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                document_id: r.document_id,
                parent_id: r.parent_id,
                author_id: UserId::from_uuid(r.author_id),
                author_label: r.author_label,
                body: r.body,
                anchor_start: r.anchor_start,
                anchor_end: r.anchor_end,
                anchor_quote: r.anchor_quote,
                resolved_at: r.resolved_at,
                resolved_by: r.resolved_by.map(UserId::from_uuid),
                created_at: r.created_at,
                updated_at: r.updated_at,
                mentions,
            });
        }
        Ok(out)
    }

    async fn list_mentions_for_comment(&self, comment_id: Uuid) -> HelixResult<Vec<Mention>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            comment_id: Uuid,
            document_id: Uuid,
            mentioned_user_id: Option<Uuid>,
            mentioned_label: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, comment_id, document_id, mentioned_user_id, mentioned_label, created_at
            FROM collab.mentions
            WHERE comment_id = $1
            "#,
        )
        .bind(comment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mentions list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Mention {
                id: r.id,
                comment_id: r.comment_id,
                document_id: r.document_id,
                mentioned_user_id: r.mentioned_user_id.map(UserId::from_uuid),
                mentioned_label: r.mentioned_label,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn add_comment(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        author: UserId,
        author_label: &str,
        body: &str,
        parent_id: Option<Uuid>,
    ) -> HelixResult<DocumentComment> {
        self.add_comment_anchored(
            tenant_id,
            document_id,
            author,
            author_label,
            body,
            parent_id,
            None,
            None,
            "",
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_comment_anchored(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        author: UserId,
        author_label: &str,
        body: &str,
        parent_id: Option<Uuid>,
        anchor_start: Option<i32>,
        anchor_end: Option<i32>,
        anchor_quote: &str,
    ) -> HelixResult<DocumentComment> {
        let body = body.trim();
        if body.is_empty() {
            return Err(HelixError::validation("comment body required"));
        }
        let _ = self.get_document(tenant_id, document_id).await?;
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO collab.comments
                (id, tenant_id, document_id, parent_id, author_id, author_label, body,
                 anchor_start, anchor_end, anchor_quote, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$11)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(parent_id)
        .bind(author.as_uuid())
        .bind(author_label)
        .bind(body)
        .bind(anchor_start)
        .bind(anchor_end)
        .bind(anchor_quote)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("comment insert: {e}")))?;

        let mentions = parse_mentions(body);
        for m in &mentions {
            let mid = Uuid::now_v7();
            let uid = Uuid::parse_str(m).ok();
            sqlx::query(
                r#"
                INSERT INTO collab.mentions
                    (id, tenant_id, document_id, comment_id, mentioned_user_id, mentioned_label, created_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7)
                "#,
            )
            .bind(mid)
            .bind(tenant_id.as_uuid())
            .bind(document_id)
            .bind(id)
            .bind(uid)
            .bind(m)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("mention insert: {e}")))?;
        }

        let _ = self
            .record_activity(
                tenant_id,
                document_id,
                Some(author),
                author_label,
                "comment.created",
                serde_json::json!({"comment_id": id, "anchored": anchor_start.is_some()}),
            )
            .await;

        let stored = self.list_mentions_for_comment(id).await?;
        Ok(DocumentComment {
            id,
            tenant_id,
            document_id,
            parent_id,
            author_id: author,
            author_label: author_label.into(),
            body: body.into(),
            anchor_start,
            anchor_end,
            anchor_quote: anchor_quote.into(),
            resolved_at: None,
            resolved_by: None,
            created_at: now,
            updated_at: now,
            mentions: stored,
        })
    }

    pub async fn resolve_comment(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        comment_id: Uuid,
        actor: UserId,
        resolved: bool,
    ) -> HelixResult<DocumentComment> {
        if resolved {
            sqlx::query(
                r#"
                UPDATE collab.comments
                SET resolved_at = now(), resolved_by = $4, updated_at = now()
                WHERE tenant_id = $1 AND document_id = $2 AND id = $3 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(document_id)
            .bind(comment_id)
            .bind(actor.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("comment resolve: {e}")))?;
        } else {
            sqlx::query(
                r#"
                UPDATE collab.comments
                SET resolved_at = NULL, resolved_by = NULL, updated_at = now()
                WHERE tenant_id = $1 AND document_id = $2 AND id = $3 AND deleted_at IS NULL
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(document_id)
            .bind(comment_id)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("comment unresolve: {e}")))?;
        }
        let _ = self
            .record_activity(
                tenant_id,
                document_id,
                Some(actor),
                &actor.to_string(),
                if resolved {
                    "comment.resolved"
                } else {
                    "comment.unresolved"
                },
                serde_json::json!({"comment_id": comment_id}),
            )
            .await;
        self.list_comments(tenant_id, document_id)
            .await?
            .into_iter()
            .find(|c| c.id == comment_id)
            .ok_or_else(|| HelixError::not_found(format!("comment {comment_id}")))
    }

    pub async fn set_document_flags(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        pinned: Option<bool>,
        archive: Option<bool>,
    ) -> HelixResult<CollabDocument> {
        if let Some(p) = pinned {
            sqlx::query(
                "UPDATE collab.documents SET pinned = $3, updated_at = now() WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tenant_id.as_uuid())
            .bind(id)
            .bind(p)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("pin: {e}")))?;
        }
        if let Some(a) = archive {
            if a {
                sqlx::query(
                    "UPDATE collab.documents SET archived_at = now(), updated_at = now() WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tenant_id.as_uuid())
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("archive: {e}")))?;
            } else {
                sqlx::query(
                    "UPDATE collab.documents SET archived_at = NULL, updated_at = now() WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tenant_id.as_uuid())
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("unarchive: {e}")))?;
            }
        }
        self.get_document(tenant_id, id).await
    }

    pub async fn set_encrypted_content(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        author: UserId,
        ciphertext: &str,
        encrypted: bool,
        base_version: u32,
    ) -> HelixResult<CollabDocument> {
        self.set_encrypted_content_ex(
            tenant_id,
            id,
            author,
            ciphertext,
            encrypted,
            false,
            base_version,
        )
        .await
    }

    pub async fn set_encrypted_content_ex(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        author: UserId,
        ciphertext: &str,
        encrypted: bool,
        client_e2ee: bool,
        base_version: u32,
    ) -> HelixResult<CollabDocument> {
        self.apply_patch(
            tenant_id,
            id,
            author,
            DocumentPatch {
                base_version,
                content: ciphertext.into(),
                title: None,
            },
        )
        .await?;
        sqlx::query(
            r#"
            UPDATE collab.documents
            SET encrypted = $3, client_e2ee = $4
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .bind(encrypted || client_e2ee)
        .bind(client_e2ee)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("set encrypted: {e}")))?;
        self.get_document(tenant_id, id).await
    }

    pub async fn record_activity(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        actor_id: Option<UserId>,
        actor_label: &str,
        action: &str,
        detail: serde_json::Value,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            INSERT INTO collab.activity
                (id, tenant_id, document_id, actor_id, actor_label, action, detail, created_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,now())
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(actor_id.map(|u| u.as_uuid()))
        .bind(actor_label)
        .bind(action)
        .bind(detail)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("activity: {e}")))?;
        Ok(())
    }

    pub async fn list_activity(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        limit: i64,
    ) -> HelixResult<Vec<DocActivity>> {
        let _ = self.get_document(tenant_id, document_id).await?;
        let limit = limit.clamp(1, 200);
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            document_id: Uuid,
            actor_id: Option<Uuid>,
            actor_label: String,
            action: String,
            detail: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, document_id, actor_id, actor_label, action, detail, created_at
            FROM collab.activity
            WHERE document_id = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(document_id)
        .bind(tenant_id.as_uuid())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("activity list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| DocActivity {
                id: r.id,
                document_id: r.document_id,
                actor_id: r.actor_id.map(UserId::from_uuid),
                actor_label: r.actor_label,
                action: r.action,
                detail: r.detail,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn update_comment(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        comment_id: Uuid,
        author: UserId,
        body: &str,
    ) -> HelixResult<DocumentComment> {
        let body = body.trim();
        if body.is_empty() {
            return Err(HelixError::validation("comment body required"));
        }
        let res = sqlx::query(
            r#"
            UPDATE collab.comments
            SET body = $4, updated_at = now()
            WHERE tenant_id = $1 AND document_id = $2 AND id = $3
              AND author_id = $5 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(comment_id)
        .bind(body)
        .bind(author.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("comment update: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("comment {comment_id}")));
        }
        // Refresh mentions: delete old, re-parse
        sqlx::query("DELETE FROM collab.mentions WHERE comment_id = $1")
            .bind(comment_id)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("mention clear: {e}")))?;
        let now = Utc::now();
        for m in parse_mentions(body) {
            let mid = Uuid::now_v7();
            let uid = Uuid::parse_str(&m).ok();
            sqlx::query(
                r#"
                INSERT INTO collab.mentions
                    (id, tenant_id, document_id, comment_id, mentioned_user_id, mentioned_label, created_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7)
                "#,
            )
            .bind(mid)
            .bind(tenant_id.as_uuid())
            .bind(document_id)
            .bind(comment_id)
            .bind(uid)
            .bind(&m)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("mention insert: {e}")))?;
        }
        let all = self.list_comments(tenant_id, document_id).await?;
        all.into_iter()
            .find(|c| c.id == comment_id)
            .ok_or_else(|| HelixError::not_found(format!("comment {comment_id}")))
    }

    pub async fn delete_comment(
        &self,
        tenant_id: TenantId,
        document_id: Uuid,
        comment_id: Uuid,
    ) -> HelixResult<()> {
        let res = sqlx::query(
            r#"
            UPDATE collab.comments
            SET deleted_at = now(), updated_at = now()
            WHERE tenant_id = $1 AND document_id = $2 AND id = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(document_id)
        .bind(comment_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("comment delete: {e}")))?;
        if res.rows_affected() == 0 {
            return Err(HelixError::not_found(format!("comment {comment_id}")));
        }
        Ok(())
    }

    pub async fn list_mentions_for_user(
        &self,
        tenant_id: TenantId,
        label_or_user: &str,
        limit: i64,
    ) -> HelixResult<Vec<Mention>> {
        let limit = limit.clamp(1, 100);
        let uid = Uuid::parse_str(label_or_user).ok();
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            comment_id: Uuid,
            document_id: Uuid,
            mentioned_user_id: Option<Uuid>,
            mentioned_label: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, comment_id, document_id, mentioned_user_id, mentioned_label, created_at
            FROM collab.mentions
            WHERE tenant_id = $1
              AND (
                ($2::uuid IS NOT NULL AND mentioned_user_id = $2)
                OR mentioned_label = $3
              )
            ORDER BY created_at DESC
            LIMIT $4
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(uid)
        .bind(label_or_user)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mentions inbox: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Mention {
                id: r.id,
                comment_id: r.comment_id,
                document_id: r.document_id,
                mentioned_user_id: r.mentioned_user_id.map(UserId::from_uuid),
                mentioned_label: r.mentioned_label,
                created_at: r.created_at,
            })
            .collect())
    }
}

/// Extract @mentions from comment body (labels: @alice or UUIDs).
pub fn parse_mentions(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in body.split_whitespace() {
        let t = part.trim_matches(|c: char| {
            matches!(c, ',' | '.' | ';' | ':' | '!' | '?' | '(' | ')' | '[' | ']')
        });
        if let Some(rest) = t.strip_prefix('@') {
            if !rest.is_empty() && rest.len() <= 128 && !out.iter().any(|x| x == rest) {
                out.push(rest.to_string());
            }
        }
    }
    out
}

#[derive(sqlx::FromRow)]
struct DocRow {
    id: Uuid,
    tenant_id: Uuid,
    workspace_id: Option<Uuid>,
    #[sqlx(default)]
    folder_id: Option<Uuid>,
    title: String,
    content: String,
    version: i32,
    #[sqlx(default)]
    encrypted: bool,
    #[sqlx(default)]
    client_e2ee: bool,
    #[sqlx(default)]
    pinned: bool,
    #[sqlx(default)]
    archived_at: Option<DateTime<Utc>>,
    created_by: Option<Uuid>,
    updated_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl DocRow {
    fn into_doc(self) -> CollabDocument {
        CollabDocument {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            workspace_id: self.workspace_id,
            folder_id: self.folder_id,
            title: self.title,
            content: self.content,
            version: self.version as u32,
            encrypted: self.encrypted,
            client_e2ee: self.client_e2ee,
            pinned: self.pinned,
            archived_at: self.archived_at,
            created_by: self.created_by.map(UserId::from_uuid),
            updated_by: self.updated_by.map(UserId::from_uuid),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[cfg(test)]
mod mention_tests {
    use super::parse_mentions;

    #[test]
    fn parses_at_labels() {
        let m = parse_mentions("hey @alice and @bob, see @alice again");
        assert_eq!(m, vec!["alice".to_string(), "bob".to_string()]);
    }
}
