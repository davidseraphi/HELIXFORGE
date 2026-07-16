//! Deploy keys, sticky LSP registry, debug session rows.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use uuid::Uuid;

use crate::code::CodeRepoStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDeployKey {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDeployKeyIssued {
    pub key: CodeDeployKey,
    /// Shown once at creation.
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLspSessionReg {
    pub session_id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub instance_id: String,
    pub server_cmd: String,
    pub root_path: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDebugSession {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub config: String,
    pub adapter: String,
    pub status: String,
    pub breakpoints: JsonValue,
    pub instance_id: String,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

pub fn hash_deploy_token(token: &str) -> String {
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    hex::encode(h.finalize())
}

impl CodeRepoStore {
    pub async fn create_deploy_key(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        name: &str,
        scope: &str,
    ) -> HelixResult<CodeDeployKeyIssued> {
        let scope = if scope == "write" { "write" } else { "read" };
        let id = Uuid::now_v7();
        let raw = format!("hdk_{}", Uuid::now_v7().simple());
        let token_hash = hash_deploy_token(&raw);
        let token_prefix = raw.chars().take(12).collect::<String>();
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.deploy_keys
               (id, tenant_id, repo_id, name, token_hash, token_prefix, scope, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(name)
        .bind(&token_hash)
        .bind(&token_prefix)
        .bind(scope)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("create deploy key: {e}")))?;
        Ok(CodeDeployKeyIssued {
            key: CodeDeployKey {
                id,
                tenant_id,
                repo_id,
                name: name.into(),
                token_prefix,
                scope: scope.into(),
                created_at: now,
                last_used_at: None,
                revoked_at: None,
            },
            token: raw,
        })
    }

    pub async fn list_deploy_keys(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
    ) -> HelixResult<Vec<CodeDeployKey>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            name: String,
            token_prefix: String,
            scope: String,
            created_at: DateTime<Utc>,
            last_used_at: Option<DateTime<Utc>>,
            revoked_at: Option<DateTime<Utc>>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT id, tenant_id, repo_id, name, token_prefix, scope, created_at, last_used_at, revoked_at
               FROM code.deploy_keys WHERE tenant_id = $1 AND repo_id = $2 AND revoked_at IS NULL
               ORDER BY created_at DESC"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list deploy keys: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodeDeployKey {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                repo_id: r.repo_id,
                name: r.name,
                token_prefix: r.token_prefix,
                scope: r.scope,
                created_at: r.created_at,
                last_used_at: r.last_used_at,
                revoked_at: r.revoked_at,
            })
            .collect())
    }

    pub async fn revoke_deploy_key(&self, tenant_id: TenantId, key_id: Uuid) -> HelixResult<bool> {
        let r = sqlx::query(
            r#"UPDATE code.deploy_keys SET revoked_at = now()
               WHERE tenant_id = $1 AND id = $2 AND revoked_at IS NULL"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(key_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("revoke deploy key: {e}")))?;
        Ok(r.rows_affected() > 0)
    }

    /// Resolve deploy key → (tenant, repo_id, scope). Marks last_used.
    pub async fn resolve_deploy_key(
        &self,
        token: &str,
    ) -> HelixResult<Option<(TenantId, Uuid, String)>> {
        let hash = hash_deploy_token(token.trim());
        #[derive(sqlx::FromRow)]
        struct Row {
            tenant_id: Uuid,
            repo_id: Uuid,
            scope: String,
            revoked_at: Option<DateTime<Utc>>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"SELECT tenant_id, repo_id, scope, revoked_at FROM code.deploy_keys WHERE token_hash = $1"#,
        )
        .bind(&hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("resolve deploy key: {e}")))?;
        let Some(r) = row else {
            return Ok(None);
        };
        if r.revoked_at.is_some() {
            return Ok(None);
        }
        let _ = sqlx::query(
            r#"UPDATE code.deploy_keys SET last_used_at = now() WHERE token_hash = $1"#,
        )
        .bind(&hash)
        .execute(&self.pool)
        .await;
        Ok(Some((TenantId::from_uuid(r.tenant_id), r.repo_id, r.scope)))
    }

    pub async fn register_lsp_session(
        &self,
        session_id: Uuid,
        tenant_id: TenantId,
        repo_id: Uuid,
        instance_id: &str,
        server_cmd: &str,
        root_path: &str,
        ttl_secs: i64,
    ) -> HelixResult<CodeLspSessionReg> {
        let now = Utc::now();
        let expires = now + Duration::seconds(ttl_secs.max(60));
        sqlx::query(
            r#"INSERT INTO code.lsp_session_registry
               (session_id, tenant_id, repo_id, instance_id, server_cmd, root_path, created_at, expires_at, last_heartbeat)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$7)
               ON CONFLICT (session_id) DO UPDATE SET last_heartbeat = EXCLUDED.last_heartbeat, expires_at = EXCLUDED.expires_at"#,
        )
        .bind(session_id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(instance_id)
        .bind(server_cmd)
        .bind(root_path)
        .bind(now)
        .bind(expires)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("register lsp session: {e}")))?;
        Ok(CodeLspSessionReg {
            session_id,
            tenant_id,
            repo_id,
            instance_id: instance_id.into(),
            server_cmd: server_cmd.into(),
            root_path: root_path.into(),
            created_at: now,
            expires_at: expires,
            last_heartbeat: now,
        })
    }

    pub async fn get_lsp_session_reg(
        &self,
        session_id: Uuid,
    ) -> HelixResult<Option<CodeLspSessionReg>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            session_id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            instance_id: String,
            server_cmd: String,
            root_path: String,
            created_at: DateTime<Utc>,
            expires_at: DateTime<Utc>,
            last_heartbeat: DateTime<Utc>,
        }
        let row: Option<Row> =
            sqlx::query_as(r#"SELECT * FROM code.lsp_session_registry WHERE session_id = $1"#)
                .bind(session_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("get lsp reg: {e}")))?;
        Ok(row.map(|r| CodeLspSessionReg {
            session_id: r.session_id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            repo_id: r.repo_id,
            instance_id: r.instance_id,
            server_cmd: r.server_cmd,
            root_path: r.root_path,
            created_at: r.created_at,
            expires_at: r.expires_at,
            last_heartbeat: r.last_heartbeat,
        }))
    }

    pub async fn heartbeat_lsp_session(&self, session_id: Uuid) -> HelixResult<()> {
        let _ = sqlx::query(
            r#"UPDATE code.lsp_session_registry SET last_heartbeat = now(), expires_at = now() + interval '2 hours'
               WHERE session_id = $1"#,
        )
        .bind(session_id)
        .execute(&self.pool)
        .await;
        Ok(())
    }

    pub async fn delete_lsp_session_reg(&self, session_id: Uuid) -> HelixResult<()> {
        let _ = sqlx::query(r#"DELETE FROM code.lsp_session_registry WHERE session_id = $1"#)
            .bind(session_id)
            .execute(&self.pool)
            .await;
        Ok(())
    }

    pub async fn create_debug_session(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        config: &str,
        adapter: &str,
        instance_id: &str,
    ) -> HelixResult<CodeDebugSession> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.debug_sessions
               (id, tenant_id, repo_id, config, adapter, status, breakpoints, instance_id, created_at)
               VALUES ($1,$2,$3,$4,$5,'ready','[]'::jsonb,$6,$7)"#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(config)
        .bind(adapter)
        .bind(instance_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("create debug session: {e}")))?;
        Ok(CodeDebugSession {
            id,
            tenant_id,
            repo_id,
            config: config.into(),
            adapter: adapter.into(),
            status: "ready".into(),
            breakpoints: serde_json::json!([]),
            instance_id: instance_id.into(),
            created_at: now,
            finished_at: None,
        })
    }

    pub async fn set_debug_breakpoints(
        &self,
        tenant_id: TenantId,
        session_id: Uuid,
        breakpoints: JsonValue,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"UPDATE code.debug_sessions SET breakpoints = $3 WHERE tenant_id = $1 AND id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(session_id)
        .bind(&breakpoints)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("set breakpoints: {e}")))?;
        Ok(())
    }

    pub async fn get_debug_session(
        &self,
        tenant_id: TenantId,
        session_id: Uuid,
    ) -> HelixResult<Option<CodeDebugSession>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            config: String,
            adapter: String,
            status: String,
            breakpoints: JsonValue,
            instance_id: String,
            created_at: DateTime<Utc>,
            finished_at: Option<DateTime<Utc>>,
        }
        let row: Option<Row> =
            sqlx::query_as(r#"SELECT * FROM code.debug_sessions WHERE tenant_id = $1 AND id = $2"#)
                .bind(tenant_id.as_uuid())
                .bind(session_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("get debug: {e}")))?;
        Ok(row.map(|r| CodeDebugSession {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            repo_id: r.repo_id,
            config: r.config,
            adapter: r.adapter,
            status: r.status,
            breakpoints: r.breakpoints,
            instance_id: r.instance_id,
            created_at: r.created_at,
            finished_at: r.finished_at,
        }))
    }

    pub async fn finish_debug_session(
        &self,
        tenant_id: TenantId,
        session_id: Uuid,
        status: &str,
    ) -> HelixResult<()> {
        let finished = status == "stopped" || status == "failed" || status == "exited";
        if finished {
            sqlx::query(
                r#"UPDATE code.debug_sessions SET status = $3, finished_at = now()
                   WHERE tenant_id = $1 AND id = $2"#,
            )
            .bind(tenant_id.as_uuid())
            .bind(session_id)
            .bind(status)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("finish debug: {e}")))?;
        } else {
            sqlx::query(
                r#"UPDATE code.debug_sessions SET status = $3 WHERE tenant_id = $1 AND id = $2"#,
            )
            .bind(tenant_id.as_uuid())
            .bind(session_id)
            .bind(status)
            .execute(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("status debug: {e}")))?;
        }
        Ok(())
    }

    // —— Per-tenant break-glass ——
    pub async fn get_tenant_breakglass(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<CodeTenantBreakglass> {
        #[derive(sqlx::FromRow)]
        struct Row {
            tenant_id: Uuid,
            allow_direct_push: bool,
            allow_force_push: bool,
            allow_ci_all: bool,
            allow_term_all: bool,
            allow_host_fallback: bool,
            allow_host_isolation: bool,
            updated_at: DateTime<Utc>,
            updated_by: String,
        }
        let row: Option<Row> =
            sqlx::query_as(r#"SELECT * FROM code.tenant_breakglass WHERE tenant_id = $1"#)
                .bind(tenant_id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("get breakglass: {e}")))?;
        Ok(row
            .map(|r| CodeTenantBreakglass {
                tenant_id: TenantId::from_uuid(r.tenant_id),
                allow_direct_push: r.allow_direct_push,
                allow_force_push: r.allow_force_push,
                allow_ci_all: r.allow_ci_all,
                allow_term_all: r.allow_term_all,
                allow_host_fallback: r.allow_host_fallback,
                allow_host_isolation: r.allow_host_isolation,
                updated_at: r.updated_at,
                updated_by: r.updated_by,
            })
            .unwrap_or_else(|| CodeTenantBreakglass::default_for(tenant_id)))
    }

    pub async fn put_tenant_breakglass(
        &self,
        tenant_id: TenantId,
        flags: &CodeTenantBreakglass,
        updated_by: &str,
    ) -> HelixResult<CodeTenantBreakglass> {
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.tenant_breakglass
               (tenant_id, allow_direct_push, allow_force_push, allow_ci_all, allow_term_all,
                allow_host_fallback, allow_host_isolation, updated_at, updated_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               ON CONFLICT (tenant_id) DO UPDATE SET
                 allow_direct_push = EXCLUDED.allow_direct_push,
                 allow_force_push = EXCLUDED.allow_force_push,
                 allow_ci_all = EXCLUDED.allow_ci_all,
                 allow_term_all = EXCLUDED.allow_term_all,
                 allow_host_fallback = EXCLUDED.allow_host_fallback,
                 allow_host_isolation = EXCLUDED.allow_host_isolation,
                 updated_at = EXCLUDED.updated_at,
                 updated_by = EXCLUDED.updated_by"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(flags.allow_direct_push)
        .bind(flags.allow_force_push)
        .bind(flags.allow_ci_all)
        .bind(flags.allow_term_all)
        .bind(flags.allow_host_fallback)
        .bind(flags.allow_host_isolation)
        .bind(now)
        .bind(updated_by)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("put breakglass: {e}")))?;
        let mut out = flags.clone();
        out.tenant_id = tenant_id;
        out.updated_at = now;
        out.updated_by = updated_by.into();
        Ok(out)
    }

    // —— Process session sticky (terminal / dap) ——
    pub async fn register_process_session(
        &self,
        session_id: Uuid,
        tenant_id: TenantId,
        kind: &str,
        instance_id: &str,
        repo_id: Option<Uuid>,
        meta: JsonValue,
        ttl_secs: i64,
    ) -> HelixResult<CodeProcessSession> {
        let now = Utc::now();
        let expires = now + Duration::seconds(ttl_secs.max(60));
        sqlx::query(
            r#"INSERT INTO code.process_sessions
               (session_id, tenant_id, kind, instance_id, repo_id, meta, created_at, expires_at, last_heartbeat)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$7)
               ON CONFLICT (session_id) DO UPDATE SET
                 last_heartbeat = EXCLUDED.last_heartbeat,
                 expires_at = EXCLUDED.expires_at,
                 instance_id = EXCLUDED.instance_id"#,
        )
        .bind(session_id)
        .bind(tenant_id.as_uuid())
        .bind(kind)
        .bind(instance_id)
        .bind(repo_id)
        .bind(&meta)
        .bind(now)
        .bind(expires)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("register process session: {e}")))?;
        Ok(CodeProcessSession {
            session_id,
            tenant_id,
            kind: kind.into(),
            instance_id: instance_id.into(),
            repo_id,
            meta,
            created_at: now,
            expires_at: expires,
            last_heartbeat: now,
        })
    }

    pub async fn get_process_session(
        &self,
        session_id: Uuid,
    ) -> HelixResult<Option<CodeProcessSession>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            session_id: Uuid,
            tenant_id: Uuid,
            kind: String,
            instance_id: String,
            repo_id: Option<Uuid>,
            meta: JsonValue,
            created_at: DateTime<Utc>,
            expires_at: DateTime<Utc>,
            last_heartbeat: DateTime<Utc>,
        }
        let row: Option<Row> =
            sqlx::query_as(r#"SELECT * FROM code.process_sessions WHERE session_id = $1"#)
                .bind(session_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("get process session: {e}")))?;
        Ok(row.map(|r| CodeProcessSession {
            session_id: r.session_id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            kind: r.kind,
            instance_id: r.instance_id,
            repo_id: r.repo_id,
            meta: r.meta,
            created_at: r.created_at,
            expires_at: r.expires_at,
            last_heartbeat: r.last_heartbeat,
        }))
    }

    pub async fn delete_process_session(&self, session_id: Uuid) -> HelixResult<()> {
        let _ = sqlx::query(r#"DELETE FROM code.process_sessions WHERE session_id = $1"#)
            .bind(session_id)
            .execute(&self.pool)
            .await;
        Ok(())
    }

    /// Ensure session is local to this instance; else sticky_miss.
    pub async fn require_process_local(
        &self,
        session_id: Uuid,
        kind: &str,
        local_instance: &str,
    ) -> HelixResult<CodeProcessSession> {
        let reg = self
            .get_process_session(session_id)
            .await?
            .ok_or_else(|| HelixError::not_found(format!("{kind} session not found")))?;
        if reg.kind != kind {
            return Err(HelixError::not_found(format!("{kind} session not found")));
        }
        if reg.instance_id != local_instance {
            return Err(HelixError::validation(format!(
                "sticky_miss: {kind} session on instance '{}' (this node is '{local_instance}')",
                reg.instance_id
            )));
        }
        Ok(reg)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeTenantBreakglass {
    pub tenant_id: TenantId,
    pub allow_direct_push: bool,
    pub allow_force_push: bool,
    pub allow_ci_all: bool,
    pub allow_term_all: bool,
    pub allow_host_fallback: bool,
    pub allow_host_isolation: bool,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
}

impl CodeTenantBreakglass {
    pub fn default_for(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            allow_direct_push: false,
            allow_force_push: false,
            allow_ci_all: false,
            allow_term_all: false,
            allow_host_fallback: false,
            allow_host_isolation: false,
            updated_at: Utc::now(),
            updated_by: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeProcessSession {
    pub session_id: Uuid,
    pub tenant_id: TenantId,
    pub kind: String,
    pub instance_id: String,
    pub repo_id: Option<Uuid>,
    pub meta: JsonValue,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}
