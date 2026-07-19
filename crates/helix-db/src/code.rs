//! HelixCode forge persistence (repos, refs, workspaces, pipelines, agents, sealed index).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRepo {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub default_branch: String,
    pub description: String,
    pub visibility: String,
    pub storage_kind: String,
    pub head_sha: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRef {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub target_sha: String,
    pub is_symbolic: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeWorkspace {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub name: String,
    pub branch: String,
    pub root_path: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePipeline {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub name: String,
    pub definition: JsonValue,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePipelineRun {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub status: String,
    pub trigger_ref: String,
    pub commit_sha: Option<String>,
    pub log_text: String,
    pub workdir: Option<String>,
    pub artifacts: JsonValue,
    pub exit_code: Option<i32>,
    /// `host` or `docker` (see HelixCode container isolation).
    #[serde(default = "default_isolation")]
    pub isolation: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

fn default_isolation() -> String {
    "host".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePipelineArtifact {
    pub id: Uuid,
    pub run_id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub name: String,
    pub storage_key: String,
    pub content_type: String,
    pub byte_len: i64,
    pub sha256: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAgentJob {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub kind: String,
    pub status: String,
    pub prompt: String,
    pub result_summary: String,
    pub workdir: Option<String>,
    pub commit_sha: Option<String>,
    pub log_text: String,
    pub files_changed: JsonValue,
    pub agent_run_ids: JsonValue,
    pub mesh_steps: JsonValue,
    /// `host` or `docker` isolation for agent shell steps.
    #[serde(default = "default_isolation")]
    pub isolation: String,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedObjectMeta {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Option<Uuid>,
    /// SHA-256 of **ciphertext** (storage integrity; never plaintext keying).
    pub content_sha256: String,
    pub storage_key: String,
    pub classification: String,
    pub byte_len: i64,
    pub name: String,
    pub purpose: String,
    pub envelope_kind: String,
    pub content_type: String,
    pub plaintext_sha256: String,
    pub created_by: String,
    pub group_id: Option<Uuid>,
    pub cleartext_forbidden: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoGroup {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub purpose: String,
    pub owner_user: String,
    pub epoch: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct RepoRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    default_branch: String,
    description: String,
    visibility: String,
    storage_kind: String,
    head_sha: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn map_repo(r: RepoRow) -> CodeRepo {
    CodeRepo {
        id: r.id,
        tenant_id: TenantId::from_uuid(r.tenant_id),
        name: r.name,
        default_branch: r.default_branch,
        description: r.description,
        visibility: r.visibility,
        storage_kind: r.storage_kind,
        head_sha: r.head_sha,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

const REPO_SELECT: &str = r#"
    SELECT id, tenant_id, name, default_branch, description,
           COALESCE(visibility, 'private') AS visibility,
           COALESCE(storage_kind, 'bare_fs') AS storage_kind,
           head_sha, created_at,
           COALESCE(updated_at, created_at) AS updated_at
    FROM code.repos
"#;

#[derive(Clone)]
pub struct CodeRepoStore {
    pub(crate) pool: PgPool,
}

impl CodeRepoStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, tenant_id: TenantId) -> HelixResult<Vec<CodeRepo>> {
        let q = format!("{REPO_SELECT} WHERE tenant_id = $1 ORDER BY created_at DESC");
        let rows: Vec<RepoRow> = sqlx::query_as(&q)
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("code list: {e}")))?;
        Ok(rows.into_iter().map(map_repo).collect())
    }

    pub async fn get(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<Option<CodeRepo>> {
        let q = format!("{REPO_SELECT} WHERE tenant_id = $1 AND id = $2");
        let row: Option<RepoRow> = sqlx::query_as(&q)
            .bind(tenant_id.as_uuid())
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("code get: {e}")))?;
        Ok(row.map(map_repo))
    }

    pub async fn get_by_name(
        &self,
        tenant_id: TenantId,
        name: &str,
    ) -> HelixResult<Option<CodeRepo>> {
        let q = format!("{REPO_SELECT} WHERE tenant_id = $1 AND name = $2");
        let row: Option<RepoRow> = sqlx::query_as(&q)
            .bind(tenant_id.as_uuid())
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("code get_by_name: {e}")))?;
        Ok(row.map(map_repo))
    }

    pub async fn create(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        visibility: &str,
    ) -> HelixResult<CodeRepo> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let visibility = if visibility.is_empty() {
            "private"
        } else {
            visibility
        };
        sqlx::query(
            r#"
            INSERT INTO code.repos (
                id, tenant_id, name, default_branch, description,
                visibility, storage_kind, head_sha, created_at, updated_at
            )
            VALUES ($1,$2,$3,'main',$4,$5,'bare_fs',NULL,$6,$6)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(visibility)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code create: {e}")))?;
        Ok(CodeRepo {
            id,
            tenant_id,
            name: name.into(),
            default_branch: "main".into(),
            description: description.into(),
            visibility: visibility.into(),
            storage_kind: "bare_fs".into(),
            head_sha: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn set_head_sha(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        head_sha: &str,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            UPDATE code.repos
            SET head_sha = $3, updated_at = now()
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .bind(head_sha)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code set_head: {e}")))?;
        Ok(())
    }

    pub async fn upsert_ref(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        name: &str,
        target_sha: &str,
        is_symbolic: bool,
    ) -> HelixResult<CodeRef> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let row: (Uuid, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO code.refs (id, repo_id, tenant_id, name, target_sha, is_symbolic, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            ON CONFLICT (repo_id, name) DO UPDATE
              SET target_sha = EXCLUDED.target_sha,
                  is_symbolic = EXCLUDED.is_symbolic,
                  updated_at = EXCLUDED.updated_at
            RETURNING id, updated_at
            "#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(target_sha)
        .bind(is_symbolic)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code upsert_ref: {e}")))?;
        Ok(CodeRef {
            id: row.0,
            repo_id,
            tenant_id,
            name: name.into(),
            target_sha: target_sha.into(),
            is_symbolic,
            updated_at: row.1,
        })
    }

    pub async fn list_refs(&self, tenant_id: TenantId, repo_id: Uuid) -> HelixResult<Vec<CodeRef>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            repo_id: Uuid,
            tenant_id: Uuid,
            name: String,
            target_sha: String,
            is_symbolic: bool,
            updated_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, repo_id, tenant_id, name, target_sha, is_symbolic, updated_at
            FROM code.refs WHERE tenant_id = $1 AND repo_id = $2 ORDER BY name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code list_refs: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodeRef {
                id: r.id,
                repo_id: r.repo_id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                name: r.name,
                target_sha: r.target_sha,
                is_symbolic: r.is_symbolic,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn create_workspace(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        name: &str,
        branch: &str,
        root_path: &str,
        created_by: &str,
    ) -> HelixResult<CodeWorkspace> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        // The repo-exists guard is part of the INSERT itself: a repo
        // deleted (or foreign) slips no workspace past the write, and the
        // caller gets a clean not_found instead of an FK-violation 500.
        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"
            INSERT INTO code.workspaces (
                id, tenant_id, repo_id, name, branch, root_path, created_by, created_at, updated_at
            )
            SELECT $1,$2,$3,$4,$5,$6,$7,$8,$8
            FROM code.repos r
            WHERE r.tenant_id = $2 AND r.id = $3
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(name)
        .bind(branch)
        .bind(root_path)
        .bind(created_by)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code create_workspace: {e}")))?;
        if inserted.is_none() {
            return Err(HelixError::not_found("repo not found"));
        }
        Ok(CodeWorkspace {
            id,
            tenant_id,
            repo_id,
            name: name.into(),
            branch: branch.into(),
            root_path: root_path.into(),
            created_by: created_by.into(),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list_workspaces(
        &self,
        tenant_id: TenantId,
        repo_id: Option<Uuid>,
    ) -> HelixResult<Vec<CodeWorkspace>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            name: String,
            branch: String,
            root_path: String,
            created_by: String,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = if let Some(rid) = repo_id {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, repo_id, name, branch, root_path, created_by, created_at, updated_at
                FROM code.workspaces WHERE tenant_id = $1 AND repo_id = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(rid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, repo_id, name, branch, root_path, created_by, created_at, updated_at
                FROM code.workspaces WHERE tenant_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("code list_workspaces: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodeWorkspace {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                repo_id: r.repo_id,
                name: r.name,
                branch: r.branch,
                root_path: r.root_path,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn create_pipeline(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        name: &str,
        definition: JsonValue,
    ) -> HelixResult<CodePipeline> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        // The repo-exists guard is part of the INSERT itself: a repo
        // deleted (or foreign) slips no pipeline past the write, and the
        // caller gets a clean not_found instead of an FK-violation 500.
        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"
            INSERT INTO code.pipelines (id, tenant_id, repo_id, name, definition, enabled, created_at)
            SELECT $1,$2,$3,$4,$5,true,$6
            FROM code.repos r
            WHERE r.tenant_id = $2 AND r.id = $3
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(name)
        .bind(&definition)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code create_pipeline: {e}")))?;
        if inserted.is_none() {
            return Err(HelixError::not_found("repo not found"));
        }
        Ok(CodePipeline {
            id,
            tenant_id,
            repo_id,
            name: name.into(),
            definition,
            enabled: true,
            created_at: now,
        })
    }

    pub async fn list_pipelines(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
    ) -> HelixResult<Vec<CodePipeline>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            name: String,
            definition: JsonValue,
            enabled: bool,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, repo_id, name, definition, enabled, created_at
            FROM code.pipelines WHERE tenant_id = $1 AND repo_id = $2 ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code list_pipelines: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodePipeline {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                repo_id: r.repo_id,
                name: r.name,
                definition: r.definition,
                enabled: r.enabled,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn get_pipeline(
        &self,
        tenant_id: TenantId,
        pipeline_id: Uuid,
    ) -> HelixResult<Option<CodePipeline>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            name: String,
            definition: JsonValue,
            enabled: bool,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, repo_id, name, definition, enabled, created_at
            FROM code.pipelines WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(pipeline_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code get_pipeline: {e}")))?;
        Ok(row.map(|r| CodePipeline {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            repo_id: r.repo_id,
            name: r.name,
            definition: r.definition,
            enabled: r.enabled,
            created_at: r.created_at,
        }))
    }

    pub async fn create_pipeline_run(
        &self,
        tenant_id: TenantId,
        pipeline_id: Uuid,
        repo_id: Uuid,
        trigger_ref: &str,
        commit_sha: Option<&str>,
    ) -> HelixResult<CodePipelineRun> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO code.pipeline_runs (
                id, pipeline_id, tenant_id, repo_id, status, trigger_ref, commit_sha, log_text, started_at
            ) VALUES ($1,$2,$3,$4,'queued',$5,$6,'',$7)
            "#,
        )
        .bind(id)
        .bind(pipeline_id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(trigger_ref)
        .bind(commit_sha)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code create_run: {e}")))?;
        Ok(CodePipelineRun {
            id,
            pipeline_id,
            tenant_id,
            repo_id,
            status: "queued".into(),
            trigger_ref: trigger_ref.into(),
            commit_sha: commit_sha.map(|s| s.into()),
            log_text: String::new(),
            workdir: None,
            artifacts: JsonValue::Array(vec![]),
            exit_code: None,
            isolation: "host".into(),
            started_at: now,
            finished_at: None,
        })
    }

    pub async fn finish_pipeline_run(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
        status: &str,
        log_text: &str,
        workdir: Option<&str>,
        artifacts: JsonValue,
        exit_code: Option<i32>,
        isolation: &str,
    ) -> HelixResult<()> {
        // The terminal guard is part of the UPDATE: a concurrent finish (or
        // a finish racing a cancel) loses instead of overwriting the
        // acknowledged terminal state.
        let done: Option<(Uuid,)> = sqlx::query_as(
            r#"
            UPDATE code.pipeline_runs
            SET status = $3,
                log_text = $4,
                workdir = $5,
                artifacts = $6,
                exit_code = $7,
                isolation = $8,
                finished_at = now()
            WHERE tenant_id = $1 AND id = $2 AND finished_at IS NULL
            RETURNING id
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .bind(status)
        .bind(log_text)
        .bind(workdir)
        .bind(&artifacts)
        .bind(exit_code)
        .bind(isolation)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code finish_run: {e}")))?;
        if done.is_none() {
            return Err(HelixError::conflict("pipeline run already finished"));
        }
        Ok(())
    }

    pub async fn insert_pipeline_artifact(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
        repo_id: Uuid,
        name: &str,
        storage_key: &str,
        content_type: &str,
        byte_len: i64,
        sha256: &str,
    ) -> HelixResult<CodePipelineArtifact> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO code.pipeline_artifacts (
                id, run_id, tenant_id, repo_id, name, storage_key, content_type, byte_len, sha256, created_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#,
        )
        .bind(id)
        .bind(run_id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(name)
        .bind(storage_key)
        .bind(content_type)
        .bind(byte_len)
        .bind(sha256)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code insert_artifact: {e}")))?;
        Ok(CodePipelineArtifact {
            id,
            run_id,
            tenant_id,
            repo_id,
            name: name.into(),
            storage_key: storage_key.into(),
            content_type: content_type.into(),
            byte_len,
            sha256: sha256.into(),
            created_at: now,
        })
    }

    pub async fn list_pipeline_artifacts(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
    ) -> HelixResult<Vec<CodePipelineArtifact>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            run_id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            name: String,
            storage_key: String,
            content_type: String,
            byte_len: i64,
            sha256: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, run_id, tenant_id, repo_id, name, storage_key, content_type, byte_len, sha256, created_at
            FROM code.pipeline_artifacts
            WHERE tenant_id = $1 AND run_id = $2
            ORDER BY created_at ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code list_artifacts: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodePipelineArtifact {
                id: r.id,
                run_id: r.run_id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                repo_id: r.repo_id,
                name: r.name,
                storage_key: r.storage_key,
                content_type: r.content_type,
                byte_len: r.byte_len,
                sha256: r.sha256,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn get_pipeline_run(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
    ) -> HelixResult<Option<CodePipelineRun>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            pipeline_id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            status: String,
            trigger_ref: String,
            commit_sha: Option<String>,
            log_text: String,
            workdir: Option<String>,
            artifacts: JsonValue,
            exit_code: Option<i32>,
            isolation: String,
            started_at: DateTime<Utc>,
            finished_at: Option<DateTime<Utc>>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, pipeline_id, tenant_id, repo_id, status, trigger_ref,
                   commit_sha, log_text,
                   workdir,
                   COALESCE(artifacts, '[]'::jsonb) AS artifacts,
                   exit_code,
                   COALESCE(isolation, 'host') AS isolation,
                   started_at, finished_at
            FROM code.pipeline_runs WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code get_run: {e}")))?;
        Ok(row.map(|r| CodePipelineRun {
            id: r.id,
            pipeline_id: r.pipeline_id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            repo_id: r.repo_id,
            status: r.status,
            trigger_ref: r.trigger_ref,
            commit_sha: r.commit_sha,
            log_text: r.log_text,
            workdir: r.workdir,
            artifacts: r.artifacts,
            exit_code: r.exit_code,
            isolation: r.isolation,
            started_at: r.started_at,
            finished_at: r.finished_at,
        }))
    }

    pub async fn create_agent_job(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        workspace_id: Option<Uuid>,
        kind: &str,
        prompt: &str,
    ) -> HelixResult<CodeAgentJob> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO code.agent_jobs (
                id, tenant_id, repo_id, workspace_id, kind, status, prompt, result_summary, created_at
            ) VALUES ($1,$2,$3,$4,$5,'queued',$6,'',$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(workspace_id)
        .bind(kind)
        .bind(prompt)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code create_agent_job: {e}")))?;
        Ok(CodeAgentJob {
            id,
            tenant_id,
            repo_id,
            workspace_id,
            kind: kind.into(),
            status: "queued".into(),
            prompt: prompt.into(),
            result_summary: String::new(),
            workdir: None,
            commit_sha: None,
            log_text: String::new(),
            files_changed: JsonValue::Array(vec![]),
            agent_run_ids: JsonValue::Array(vec![]),
            mesh_steps: JsonValue::Array(vec![]),
            isolation: "host".into(),
            created_at: now,
            finished_at: None,
        })
    }

    pub async fn finish_agent_job(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        status: &str,
        result_summary: &str,
        workdir: Option<&str>,
        commit_sha: Option<&str>,
        log_text: &str,
        files_changed: JsonValue,
        agent_run_ids: JsonValue,
        mesh_steps: JsonValue,
        isolation: &str,
    ) -> HelixResult<()> {
        // The terminal guard is part of the UPDATE: a concurrent finish (or
        // a finish racing a cancel) loses instead of overwriting the
        // acknowledged terminal state.
        let done: Option<(Uuid,)> = sqlx::query_as(
            r#"
            UPDATE code.agent_jobs
            SET status = $3,
                result_summary = $4,
                workdir = $5,
                commit_sha = $6,
                log_text = $7,
                files_changed = $8,
                agent_run_ids = $9,
                mesh_steps = $10,
                isolation = $11,
                finished_at = now()
            WHERE tenant_id = $1 AND id = $2 AND finished_at IS NULL
            RETURNING id
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(status)
        .bind(result_summary)
        .bind(workdir)
        .bind(commit_sha)
        .bind(log_text)
        .bind(&files_changed)
        .bind(&agent_run_ids)
        .bind(&mesh_steps)
        .bind(isolation)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code finish_agent_job: {e}")))?;
        if done.is_none() {
            return Err(HelixError::conflict("agent job already finished"));
        }
        Ok(())
    }

    pub async fn get_agent_job(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
    ) -> HelixResult<Option<CodeAgentJob>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            workspace_id: Option<Uuid>,
            kind: String,
            status: String,
            prompt: String,
            result_summary: String,
            workdir: Option<String>,
            commit_sha: Option<String>,
            log_text: String,
            files_changed: JsonValue,
            agent_run_ids: JsonValue,
            mesh_steps: JsonValue,
            isolation: String,
            created_at: DateTime<Utc>,
            finished_at: Option<DateTime<Utc>>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, repo_id, workspace_id, kind, status, prompt,
                   result_summary,
                   workdir, commit_sha,
                   COALESCE(log_text, '') AS log_text,
                   COALESCE(files_changed, '[]'::jsonb) AS files_changed,
                   COALESCE(agent_run_ids, '[]'::jsonb) AS agent_run_ids,
                   COALESCE(mesh_steps, '[]'::jsonb) AS mesh_steps,
                   COALESCE(isolation, 'host') AS isolation,
                   created_at, finished_at
            FROM code.agent_jobs WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code get_agent_job: {e}")))?;
        Ok(row.map(|r| CodeAgentJob {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            repo_id: r.repo_id,
            workspace_id: r.workspace_id,
            kind: r.kind,
            status: r.status,
            prompt: r.prompt,
            result_summary: r.result_summary,
            workdir: r.workdir,
            commit_sha: r.commit_sha,
            log_text: r.log_text,
            files_changed: r.files_changed,
            agent_run_ids: r.agent_run_ids,
            mesh_steps: r.mesh_steps,
            isolation: r.isolation,
            created_at: r.created_at,
            finished_at: r.finished_at,
        }))
    }

    pub async fn insert_sealed_object(
        &self,
        tenant_id: TenantId,
        repo_id: Option<Uuid>,
        content_sha256: &str,
        storage_key: &str,
        classification: &str,
        byte_len: i64,
        name: &str,
        purpose: &str,
        envelope_kind: &str,
        content_type: &str,
        plaintext_sha256: &str,
        created_by: &str,
        group_id: Option<Uuid>,
    ) -> HelixResult<SealedObjectMeta> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO code.sealed_objects (
                id, tenant_id, repo_id, content_sha256, storage_key, classification, byte_len,
                name, purpose, envelope_kind, content_type, plaintext_sha256, created_by,
                group_id, cleartext_forbidden, created_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,true,$15)
            ON CONFLICT (tenant_id, content_sha256) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(content_sha256)
        .bind(storage_key)
        .bind(classification)
        .bind(byte_len)
        .bind(name)
        .bind(purpose)
        .bind(envelope_kind)
        .bind(content_type)
        .bind(plaintext_sha256)
        .bind(created_by)
        .bind(group_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code insert_sealed: {e}")))?;
        // Prefer row currently in DB (handles conflict / concurrent insert).
        if let Some(existing) = self.get_sealed_by_hash(tenant_id, content_sha256).await? {
            return Ok(existing);
        }
        Ok(SealedObjectMeta {
            id,
            tenant_id,
            repo_id,
            content_sha256: content_sha256.into(),
            storage_key: storage_key.into(),
            classification: classification.into(),
            byte_len,
            name: name.into(),
            purpose: purpose.into(),
            envelope_kind: envelope_kind.into(),
            content_type: content_type.into(),
            plaintext_sha256: plaintext_sha256.into(),
            created_by: created_by.into(),
            group_id,
            cleartext_forbidden: true,
            created_at: now,
        })
    }

    fn map_sealed_row(
        id: Uuid,
        tenant_id: Uuid,
        repo_id: Option<Uuid>,
        content_sha256: String,
        storage_key: String,
        classification: String,
        byte_len: i64,
        name: String,
        purpose: String,
        envelope_kind: String,
        content_type: String,
        plaintext_sha256: String,
        created_by: String,
        group_id: Option<Uuid>,
        cleartext_forbidden: bool,
        created_at: DateTime<Utc>,
    ) -> SealedObjectMeta {
        SealedObjectMeta {
            id,
            tenant_id: TenantId::from_uuid(tenant_id),
            repo_id,
            content_sha256,
            storage_key,
            classification,
            byte_len,
            name,
            purpose,
            envelope_kind,
            content_type,
            plaintext_sha256,
            created_by,
            group_id,
            cleartext_forbidden,
            created_at,
        }
    }

    pub async fn get_sealed_by_hash(
        &self,
        tenant_id: TenantId,
        content_sha256: &str,
    ) -> HelixResult<Option<SealedObjectMeta>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Option<Uuid>,
            content_sha256: String,
            storage_key: String,
            classification: String,
            byte_len: i64,
            name: String,
            purpose: String,
            envelope_kind: String,
            content_type: String,
            plaintext_sha256: String,
            created_by: String,
            group_id: Option<Uuid>,
            cleartext_forbidden: bool,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, repo_id, content_sha256, storage_key, classification, byte_len,
                   COALESCE(name,'') AS name,
                   COALESCE(purpose,'forge.secret') AS purpose,
                   COALESCE(envelope_kind,'hva4') AS envelope_kind,
                   COALESCE(content_type,'application/octet-stream') AS content_type,
                   COALESCE(plaintext_sha256,'') AS plaintext_sha256,
                   COALESCE(created_by,'') AS created_by,
                   group_id,
                   COALESCE(cleartext_forbidden, true) AS cleartext_forbidden,
                   created_at
            FROM code.sealed_objects
            WHERE tenant_id = $1 AND content_sha256 = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(content_sha256)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code get_sealed_hash: {e}")))?;
        Ok(row.map(|r| {
            Self::map_sealed_row(
                r.id,
                r.tenant_id,
                r.repo_id,
                r.content_sha256,
                r.storage_key,
                r.classification,
                r.byte_len,
                r.name,
                r.purpose,
                r.envelope_kind,
                r.content_type,
                r.plaintext_sha256,
                r.created_by,
                r.group_id,
                r.cleartext_forbidden,
                r.created_at,
            )
        }))
    }

    pub async fn get_sealed(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<SealedObjectMeta>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Option<Uuid>,
            content_sha256: String,
            storage_key: String,
            classification: String,
            byte_len: i64,
            name: String,
            purpose: String,
            envelope_kind: String,
            content_type: String,
            plaintext_sha256: String,
            created_by: String,
            group_id: Option<Uuid>,
            cleartext_forbidden: bool,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, repo_id, content_sha256, storage_key, classification, byte_len,
                   COALESCE(name,'') AS name,
                   COALESCE(purpose,'forge.secret') AS purpose,
                   COALESCE(envelope_kind,'hva4') AS envelope_kind,
                   COALESCE(content_type,'application/octet-stream') AS content_type,
                   COALESCE(plaintext_sha256,'') AS plaintext_sha256,
                   COALESCE(created_by,'') AS created_by,
                   group_id,
                   COALESCE(cleartext_forbidden, true) AS cleartext_forbidden,
                   created_at
            FROM code.sealed_objects
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code get_sealed: {e}")))?;
        Ok(row.map(|r| {
            Self::map_sealed_row(
                r.id,
                r.tenant_id,
                r.repo_id,
                r.content_sha256,
                r.storage_key,
                r.classification,
                r.byte_len,
                r.name,
                r.purpose,
                r.envelope_kind,
                r.content_type,
                r.plaintext_sha256,
                r.created_by,
                r.group_id,
                r.cleartext_forbidden,
                r.created_at,
            )
        }))
    }

    pub async fn list_sealed(
        &self,
        tenant_id: TenantId,
        repo_id: Option<Uuid>,
    ) -> HelixResult<Vec<SealedObjectMeta>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Option<Uuid>,
            content_sha256: String,
            storage_key: String,
            classification: String,
            byte_len: i64,
            name: String,
            purpose: String,
            envelope_kind: String,
            content_type: String,
            plaintext_sha256: String,
            created_by: String,
            group_id: Option<Uuid>,
            cleartext_forbidden: bool,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = if let Some(rid) = repo_id {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, repo_id, content_sha256, storage_key, classification, byte_len,
                       COALESCE(name,'') AS name,
                       COALESCE(purpose,'forge.secret') AS purpose,
                       COALESCE(envelope_kind,'hva4') AS envelope_kind,
                       COALESCE(content_type,'application/octet-stream') AS content_type,
                       COALESCE(plaintext_sha256,'') AS plaintext_sha256,
                       COALESCE(created_by,'') AS created_by,
                       group_id,
                       COALESCE(cleartext_forbidden, true) AS cleartext_forbidden,
                       created_at
                FROM code.sealed_objects
                WHERE tenant_id = $1 AND repo_id = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(rid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, repo_id, content_sha256, storage_key, classification, byte_len,
                       COALESCE(name,'') AS name,
                       COALESCE(purpose,'forge.secret') AS purpose,
                       COALESCE(envelope_kind,'hva4') AS envelope_kind,
                       COALESCE(content_type,'application/octet-stream') AS content_type,
                       COALESCE(plaintext_sha256,'') AS plaintext_sha256,
                       COALESCE(created_by,'') AS created_by,
                       group_id,
                       COALESCE(cleartext_forbidden, true) AS cleartext_forbidden,
                       created_at
                FROM code.sealed_objects
                WHERE tenant_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(tenant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| HelixError::dependency(format!("code list_sealed: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                Self::map_sealed_row(
                    r.id,
                    r.tenant_id,
                    r.repo_id,
                    r.content_sha256,
                    r.storage_key,
                    r.classification,
                    r.byte_len,
                    r.name,
                    r.purpose,
                    r.envelope_kind,
                    r.content_type,
                    r.plaintext_sha256,
                    r.created_by,
                    r.group_id,
                    r.cleartext_forbidden,
                    r.created_at,
                )
            })
            .collect())
    }

    pub async fn delete_sealed(&self, tenant_id: TenantId, id: Uuid) -> HelixResult<bool> {
        let res =
            sqlx::query(r#"DELETE FROM code.sealed_objects WHERE tenant_id = $1 AND id = $2"#)
                .bind(tenant_id.as_uuid())
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("code delete_sealed: {e}")))?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn create_crypto_group(
        &self,
        tenant_id: TenantId,
        name: &str,
        purpose: &str,
        owner_user: &str,
        wrapped_dek_b64: &str,
    ) -> HelixResult<CryptoGroup> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO code.crypto_groups (
                id, tenant_id, name, purpose, owner_user, wrapped_dek_b64, epoch, created_at
            ) VALUES ($1,$2,$3,$4,$5,$6,1,$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(purpose)
        .bind(owner_user)
        .bind(wrapped_dek_b64)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code create_crypto_group: {e}")))?;
        // Owner as member
        let mid = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO code.crypto_group_members (
                id, group_id, tenant_id, user_key, wrapped_dek_b64, created_at
            ) VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(mid)
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(owner_user)
        .bind(wrapped_dek_b64)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code group member owner: {e}")))?;
        Ok(CryptoGroup {
            id,
            tenant_id,
            name: name.into(),
            purpose: purpose.into(),
            owner_user: owner_user.into(),
            epoch: 1,
            created_at: now,
        })
    }

    pub async fn get_crypto_group(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> HelixResult<Option<CryptoGroup>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            purpose: String,
            owner_user: String,
            epoch: i64,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, purpose, owner_user, epoch, created_at
            FROM code.crypto_groups WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code get_crypto_group: {e}")))?;
        Ok(row.map(|r| CryptoGroup {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            name: r.name,
            purpose: r.purpose,
            owner_user: r.owner_user,
            epoch: r.epoch,
            created_at: r.created_at,
        }))
    }

    pub async fn get_member_wrapped_dek(
        &self,
        tenant_id: TenantId,
        group_id: Uuid,
        user_key: &str,
    ) -> HelixResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT wrapped_dek_b64 FROM code.crypto_group_members
            WHERE tenant_id = $1 AND group_id = $2 AND user_key = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(group_id)
        .bind(user_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code get member dek: {e}")))?;
        Ok(row.map(|r| r.0))
    }

    pub async fn add_crypto_group_member(
        &self,
        tenant_id: TenantId,
        group_id: Uuid,
        user_key: &str,
        wrapped_dek_b64: &str,
    ) -> HelixResult<()> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO code.crypto_group_members (
                id, group_id, tenant_id, user_key, wrapped_dek_b64, created_at
            ) VALUES ($1,$2,$3,$4,$5,$6)
            ON CONFLICT (group_id, user_key) DO UPDATE
              SET wrapped_dek_b64 = EXCLUDED.wrapped_dek_b64
            "#,
        )
        .bind(id)
        .bind(group_id)
        .bind(tenant_id.as_uuid())
        .bind(user_key)
        .bind(wrapped_dek_b64)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code add member: {e}")))?;
        Ok(())
    }

    pub async fn list_crypto_groups(&self, tenant_id: TenantId) -> HelixResult<Vec<CryptoGroup>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            purpose: String,
            owner_user: String,
            epoch: i64,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, purpose, owner_user, epoch, created_at
            FROM code.crypto_groups WHERE tenant_id = $1 ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("code list_crypto_groups: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CryptoGroup {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                name: r.name,
                purpose: r.purpose,
                owner_user: r.owner_user,
                epoch: r.epoch,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn upsert_mls_user_blob(
        &self,
        tenant_id: TenantId,
        user_key: &str,
        blob: &[u8],
    ) -> HelixResult<()> {
        sqlx::query(
            r#"
            INSERT INTO code.mls_user_blobs (tenant_id, user_key, blob, updated_at)
            VALUES ($1,$2,$3,now())
            ON CONFLICT (tenant_id, user_key) DO UPDATE
              SET blob = EXCLUDED.blob, updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_key)
        .bind(blob)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mls upsert blob: {e}")))?;
        Ok(())
    }

    pub async fn get_mls_user_blob(
        &self,
        tenant_id: TenantId,
        user_key: &str,
    ) -> HelixResult<Option<Vec<u8>>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            r#"SELECT blob FROM code.mls_user_blobs WHERE tenant_id = $1 AND user_key = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mls get blob: {e}")))?;
        Ok(row.map(|r| r.0))
    }

    pub async fn upsert_mls_group_meta(
        &self,
        tenant_id: TenantId,
        group_key: &str,
        repo_id: Option<Uuid>,
        name: &str,
        epoch: i64,
        member_count: i32,
    ) -> HelixResult<Uuid> {
        let id = Uuid::now_v7();
        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO code.mls_groups_meta (
                id, tenant_id, group_key, repo_id, name, epoch, member_count, created_at, updated_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,now(),now())
            ON CONFLICT (tenant_id, group_key) DO UPDATE
              SET epoch = EXCLUDED.epoch,
                  member_count = EXCLUDED.member_count,
                  name = EXCLUDED.name,
                  repo_id = COALESCE(EXCLUDED.repo_id, code.mls_groups_meta.repo_id),
                  updated_at = now()
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(group_key)
        .bind(repo_id)
        .bind(name)
        .bind(epoch)
        .bind(member_count)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mls group meta: {e}")))?;
        Ok(row.0)
    }
}
