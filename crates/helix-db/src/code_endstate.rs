//! HelixCode end-state persistence (issues, PRs, protection, webhooks, CI fleet, agents events, MLS devices, settings, quotas).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use uuid::Uuid;

use crate::code::CodeRepoStore;

// —— Types ——

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub number: i32,
    pub title: String,
    pub body: String,
    pub state: String,
    pub author: String,
    pub labels: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePullRequest {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub number: i32,
    pub title: String,
    pub body: String,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author: String,
    pub head_sha: Option<String>,
    pub merge_sha: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePrReview {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub pr_id: Uuid,
    pub author: String,
    pub state: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBranchProtection {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub branch_pattern: String,
    pub require_pr: bool,
    pub require_approvals: i32,
    pub deny_force_push: bool,
    pub required_status_checks: JsonValue,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeWebhook {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub repo_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: JsonValue,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeWebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub tenant_id: TenantId,
    pub event: String,
    pub status: String,
    pub response_code: Option<i32>,
    pub payload: JsonValue,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRunner {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub labels: JsonValue,
    pub status: String,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAgentJobEvent {
    pub id: Uuid,
    pub job_id: Uuid,
    pub tenant_id: TenantId,
    pub seq: i32,
    pub kind: String,
    pub payload: JsonValue,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMlsDevice {
    pub tenant_id: TenantId,
    pub user_key: String,
    pub device_id: String,
    pub label: String,
    pub public_identity_b64: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeTenantQuota {
    pub tenant_id: TenantId,
    pub max_repos: i32,
    pub max_pipeline_runs_month: i32,
    pub max_agent_jobs_day: i32,
    pub max_sealed_bytes: i64,
}

impl CodeRepoStore {
    /// Allocate the next number for a scope atomically. The counter row is
    /// created on first use and incremented under a row lock in one
    /// statement, so concurrent allocators always receive distinct values —
    /// no MAX+1 read window (including the zero-row case), and no
    /// unique-violation 500 on the loser.
    async fn allocate_number(
        &self,
        tenant_id: TenantId,
        scope_kind: &str,
        scope_id: Uuid,
    ) -> HelixResult<i32> {
        let allocated: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO code.number_counters (tenant_id, scope_kind, scope_id, next_value)
            VALUES ($1, $2, $3, 2)
            ON CONFLICT (tenant_id, scope_kind, scope_id)
            DO UPDATE SET next_value = code.number_counters.next_value + 1
            RETURNING next_value - 1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(scope_kind)
        .bind(scope_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("allocate {scope_kind} number: {e}")))?;
        i32::try_from(allocated.0)
            .map_err(|e| HelixError::dependency(format!("allocate {scope_kind} overflow: {e}")))
    }

    // —— Issues ——
    pub async fn next_issue_number(&self, tenant_id: TenantId, repo_id: Uuid) -> HelixResult<i32> {
        self.allocate_number(tenant_id, "issue", repo_id).await
    }

    pub async fn create_issue(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        title: &str,
        body: &str,
        author: &str,
        labels: JsonValue,
    ) -> HelixResult<CodeIssue> {
        let id = Uuid::now_v7();
        let number = self.next_issue_number(tenant_id, repo_id).await?;
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.issues
            (id, tenant_id, repo_id, number, title, body, state, author, labels, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,'open',$7,$8,$9,$9)"#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(number)
        .bind(title)
        .bind(body)
        .bind(author)
        .bind(&labels)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("create issue: {e}")))?;
        Ok(CodeIssue {
            id,
            tenant_id,
            repo_id,
            number,
            title: title.into(),
            body: body.into(),
            state: "open".into(),
            author: author.into(),
            labels,
            created_at: now,
            updated_at: now,
            closed_at: None,
        })
    }

    pub async fn list_issues(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
    ) -> HelixResult<Vec<CodeIssue>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            number: i32,
            title: String,
            body: String,
            state: String,
            author: String,
            labels: JsonValue,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            closed_at: Option<DateTime<Utc>>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT * FROM code.issues WHERE tenant_id = $1 AND repo_id = $2 ORDER BY number DESC LIMIT 200"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list issues: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodeIssue {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                repo_id: r.repo_id,
                number: r.number,
                title: r.title,
                body: r.body,
                state: r.state,
                author: r.author,
                labels: r.labels,
                created_at: r.created_at,
                updated_at: r.updated_at,
                closed_at: r.closed_at,
            })
            .collect())
    }

    pub async fn update_issue_state(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        number: i32,
        state: &str,
    ) -> HelixResult<Option<CodeIssue>> {
        let closed = if state == "closed" {
            Some(Utc::now())
        } else {
            None
        };
        let row = sqlx::query_as::<_, (Uuid,)>(
            r#"UPDATE code.issues SET state = $4, updated_at = now(), closed_at = $5
               WHERE tenant_id = $1 AND repo_id = $2 AND number = $3 RETURNING id"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(number)
        .bind(state)
        .bind(closed)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("update issue: {e}")))?;
        if row.is_none() {
            return Ok(None);
        }
        let items = self.list_issues(tenant_id, repo_id).await?;
        Ok(items.into_iter().find(|i| i.number == number))
    }

    // —— PRs ——
    pub async fn next_pr_number(&self, tenant_id: TenantId, repo_id: Uuid) -> HelixResult<i32> {
        self.allocate_number(tenant_id, "pr", repo_id).await
    }

    pub async fn create_pr(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: &str,
        author: &str,
        head_sha: Option<&str>,
    ) -> HelixResult<CodePullRequest> {
        let id = Uuid::now_v7();
        let number = self.next_pr_number(tenant_id, repo_id).await?;
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.pull_requests
            (id, tenant_id, repo_id, number, title, body, state, source_branch, target_branch, author, head_sha, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,'open',$7,$8,$9,$10,$11,$11)"#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(number)
        .bind(title)
        .bind(body)
        .bind(source_branch)
        .bind(target_branch)
        .bind(author)
        .bind(head_sha)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("create pr: {e}")))?;
        Ok(CodePullRequest {
            id,
            tenant_id,
            repo_id,
            number,
            title: title.into(),
            body: body.into(),
            state: "open".into(),
            source_branch: source_branch.into(),
            target_branch: target_branch.into(),
            author: author.into(),
            head_sha: head_sha.map(|s| s.into()),
            merge_sha: None,
            created_at: now,
            updated_at: now,
            merged_at: None,
        })
    }

    pub async fn list_prs(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
    ) -> HelixResult<Vec<CodePullRequest>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            number: i32,
            title: String,
            body: String,
            state: String,
            source_branch: String,
            target_branch: String,
            author: String,
            head_sha: Option<String>,
            merge_sha: Option<String>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            merged_at: Option<DateTime<Utc>>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT * FROM code.pull_requests WHERE tenant_id = $1 AND repo_id = $2 ORDER BY number DESC LIMIT 200"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list prs: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodePullRequest {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                repo_id: r.repo_id,
                number: r.number,
                title: r.title,
                body: r.body,
                state: r.state,
                source_branch: r.source_branch,
                target_branch: r.target_branch,
                author: r.author,
                head_sha: r.head_sha,
                merge_sha: r.merge_sha,
                created_at: r.created_at,
                updated_at: r.updated_at,
                merged_at: r.merged_at,
            })
            .collect())
    }

    pub async fn get_pr(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        number: i32,
    ) -> HelixResult<Option<CodePullRequest>> {
        Ok(self
            .list_prs(tenant_id, repo_id)
            .await?
            .into_iter()
            .find(|p| p.number == number))
    }

    pub async fn mark_pr_merged(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        number: i32,
        merge_sha: &str,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"UPDATE code.pull_requests SET state = 'merged', merge_sha = $4, merged_at = now(), updated_at = now()
               WHERE tenant_id = $1 AND repo_id = $2 AND number = $3"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(number)
        .bind(merge_sha)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mark pr merged: {e}")))?;
        Ok(())
    }

    pub async fn add_pr_review(
        &self,
        tenant_id: TenantId,
        pr_id: Uuid,
        author: &str,
        state: &str,
        body: &str,
    ) -> HelixResult<CodePrReview> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.pr_reviews (id, tenant_id, pr_id, author, state, body, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(pr_id)
        .bind(author)
        .bind(state)
        .bind(body)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("pr review: {e}")))?;
        Ok(CodePrReview {
            id,
            tenant_id,
            pr_id,
            author: author.into(),
            state: state.into(),
            body: body.into(),
            created_at: now,
        })
    }

    pub async fn count_pr_approvals(&self, pr_id: Uuid) -> HelixResult<i64> {
        let n: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code.pr_reviews WHERE pr_id = $1 AND state = 'approve'"#,
        )
        .bind(pr_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("count approvals: {e}")))?;
        Ok(n.0)
    }

    // —— Protections ——
    pub async fn upsert_protection(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        branch_pattern: &str,
        require_pr: bool,
        require_approvals: i32,
        deny_force_push: bool,
        required_status_checks: JsonValue,
    ) -> HelixResult<CodeBranchProtection> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO code.branch_protections
               (id, tenant_id, repo_id, branch_pattern, require_pr, require_approvals, deny_force_push, required_status_checks, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               ON CONFLICT (tenant_id, repo_id, branch_pattern) DO UPDATE SET
                 require_pr = EXCLUDED.require_pr,
                 require_approvals = EXCLUDED.require_approvals,
                 deny_force_push = EXCLUDED.deny_force_push,
                 required_status_checks = EXCLUDED.required_status_checks
               RETURNING id"#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(branch_pattern)
        .bind(require_pr)
        .bind(require_approvals)
        .bind(deny_force_push)
        .bind(&required_status_checks)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("upsert protection: {e}")))?;
        Ok(CodeBranchProtection {
            id: row.0,
            tenant_id,
            repo_id,
            branch_pattern: branch_pattern.into(),
            require_pr,
            require_approvals,
            deny_force_push,
            required_status_checks,
            created_at: now,
        })
    }

    pub async fn list_protections(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
    ) -> HelixResult<Vec<CodeBranchProtection>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            branch_pattern: String,
            require_pr: bool,
            require_approvals: i32,
            deny_force_push: bool,
            required_status_checks: JsonValue,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT * FROM code.branch_protections WHERE tenant_id = $1 AND repo_id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list protections: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodeBranchProtection {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                repo_id: r.repo_id,
                branch_pattern: r.branch_pattern,
                require_pr: r.require_pr,
                require_approvals: r.require_approvals,
                deny_force_push: r.deny_force_push,
                required_status_checks: r.required_status_checks,
                created_at: r.created_at,
            })
            .collect())
    }

    /// Match branch name against protection patterns (exact or trailing *).
    pub fn protection_matches(pattern: &str, branch: &str) -> bool {
        if pattern == branch || pattern == "*" {
            return true;
        }
        if let Some(prefix) = pattern.strip_suffix('*') {
            return branch.starts_with(prefix);
        }
        false
    }

    pub async fn matching_protection(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        branch: &str,
    ) -> HelixResult<Option<CodeBranchProtection>> {
        let all = self.list_protections(tenant_id, repo_id).await?;
        Ok(all
            .into_iter()
            .find(|p| Self::protection_matches(&p.branch_pattern, branch)))
    }

    /// Latest pipeline run status for a named check (pipeline name) at a commit.
    /// Used for `required_status_checks` on PR merge.
    pub async fn latest_check_status(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        check_name: &str,
        commit_sha: &str,
    ) -> HelixResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"SELECT r.status
               FROM code.pipeline_runs r
               INNER JOIN code.pipelines p ON p.id = r.pipeline_id
               WHERE r.tenant_id = $1 AND r.repo_id = $2
                 AND p.name = $3
                 AND (
                   r.commit_sha = $4
                   OR r.commit_sha LIKE $4 || '%'
                   OR $4 LIKE r.commit_sha || '%'
                   OR r.trigger_ref = $4
                   OR r.trigger_ref LIKE '%' || $4 || '%'
                 )
               ORDER BY r.started_at DESC
               LIMIT 1"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(check_name)
        .bind(commit_sha)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("latest_check_status: {e}")))?;
        Ok(row.map(|r| r.0))
    }

    // —— Webhooks ——
    pub async fn create_webhook(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        url: &str,
        secret: &str,
        events: JsonValue,
    ) -> HelixResult<CodeWebhook> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.webhooks (id, tenant_id, repo_id, url, secret, events, active, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,true,$7)"#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(url)
        .bind(secret)
        .bind(&events)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("create webhook: {e}")))?;
        Ok(CodeWebhook {
            id,
            tenant_id,
            repo_id,
            url: url.into(),
            secret: secret.into(),
            events,
            active: true,
            created_at: now,
        })
    }

    pub async fn list_webhooks(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
    ) -> HelixResult<Vec<CodeWebhook>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            repo_id: Uuid,
            url: String,
            secret: String,
            events: JsonValue,
            active: bool,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> =
            sqlx::query_as(r#"SELECT * FROM code.webhooks WHERE tenant_id = $1 AND repo_id = $2"#)
                .bind(tenant_id.as_uuid())
                .bind(repo_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("list webhooks: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodeWebhook {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                repo_id: r.repo_id,
                url: r.url,
                secret: r.secret,
                events: r.events,
                active: r.active,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn record_webhook_delivery(
        &self,
        tenant_id: TenantId,
        webhook_id: Uuid,
        event: &str,
        status: &str,
        response_code: Option<i32>,
        payload: JsonValue,
    ) -> HelixResult<CodeWebhookDelivery> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.webhook_deliveries
               (id, webhook_id, tenant_id, event, status, response_code, payload, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
        )
        .bind(id)
        .bind(webhook_id)
        .bind(tenant_id.as_uuid())
        .bind(event)
        .bind(status)
        .bind(response_code)
        .bind(&payload)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("webhook delivery: {e}")))?;
        Ok(CodeWebhookDelivery {
            id,
            webhook_id,
            tenant_id,
            event: event.into(),
            status: status.into(),
            response_code,
            payload,
            created_at: now,
        })
    }

    // —— Pipeline runs list / cancel ——
    pub async fn list_pipeline_runs(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        limit: i64,
    ) -> HelixResult<Vec<crate::code::CodePipelineRun>> {
        // Reuse get shape via raw query of essential fields
        let limit = limit.clamp(1, 100);
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
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT id, pipeline_id, tenant_id, repo_id, status, trigger_ref, commit_sha,
                      COALESCE(log_text,'') AS log_text, workdir,
                      COALESCE(artifacts,'[]'::jsonb) AS artifacts, exit_code,
                      COALESCE(isolation,'host') AS isolation, started_at, finished_at
               FROM code.pipeline_runs
               WHERE tenant_id = $1 AND repo_id = $2
               ORDER BY started_at DESC LIMIT $3"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list runs: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::code::CodePipelineRun {
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
            })
            .collect())
    }

    pub async fn cancel_pipeline_run(
        &self,
        tenant_id: TenantId,
        run_id: Uuid,
    ) -> HelixResult<bool> {
        let r = sqlx::query(
            r#"UPDATE code.pipeline_runs SET cancel_requested = true, status = CASE
                 WHEN status IN ('queued','running') THEN 'cancelled' ELSE status END
               WHERE tenant_id = $1 AND id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cancel run: {e}")))?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn count_pipeline_runs_month(&self, tenant_id: TenantId) -> HelixResult<i64> {
        let n: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code.pipeline_runs
               WHERE tenant_id = $1 AND started_at >= date_trunc('month', now())"#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("count runs month: {e}")))?;
        Ok(n.0)
    }

    pub async fn count_repos(&self, tenant_id: TenantId) -> HelixResult<i64> {
        let n: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM code.repos WHERE tenant_id = $1"#)
            .bind(tenant_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("count repos: {e}")))?;
        Ok(n.0)
    }

    pub async fn count_agent_jobs_day(&self, tenant_id: TenantId) -> HelixResult<i64> {
        let n: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code.agent_jobs
               WHERE tenant_id = $1 AND created_at >= date_trunc('day', now())"#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("count agent day: {e}")))?;
        Ok(n.0)
    }

    /// Sum of sealed object byte lengths for the tenant (quota: max_sealed_bytes).
    pub async fn sum_sealed_bytes(&self, tenant_id: TenantId) -> HelixResult<i64> {
        let n: (Option<i64>,) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(byte_len), 0)::bigint FROM code.sealed_objects
               WHERE tenant_id = $1"#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("sum sealed bytes: {e}")))?;
        Ok(n.0.unwrap_or(0))
    }

    // —— Runners ——
    pub async fn upsert_runner(
        &self,
        tenant_id: TenantId,
        name: &str,
        labels: JsonValue,
    ) -> HelixResult<CodeRunner> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO code.runners (id, tenant_id, name, labels, status, last_seen)
               VALUES ($1,$2,$3,$4,'online',$5)
               ON CONFLICT (tenant_id, name) DO UPDATE SET labels = EXCLUDED.labels, last_seen = EXCLUDED.last_seen, status = 'online'
               RETURNING id"#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(&labels)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("upsert runner: {e}")))?;
        Ok(CodeRunner {
            id: row.0,
            tenant_id,
            name: name.into(),
            labels,
            status: "online".into(),
            last_seen: now,
        })
    }

    // —— Agent events / list ——
    pub async fn list_agent_jobs(
        &self,
        tenant_id: TenantId,
        repo_id: Uuid,
        limit: i64,
    ) -> HelixResult<Vec<crate::code::CodeAgentJob>> {
        let limit = limit.clamp(1, 100);
        // Minimal: fetch via get by querying ids
        #[derive(sqlx::FromRow)]
        struct IdRow {
            id: Uuid,
        }
        let ids: Vec<IdRow> = sqlx::query_as(
            r#"SELECT id FROM code.agent_jobs WHERE tenant_id = $1 AND repo_id = $2
               ORDER BY created_at DESC LIMIT $3"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list agent jobs: {e}")))?;
        let mut out = Vec::new();
        for r in ids {
            if let Some(j) = self.get_agent_job(tenant_id, r.id).await? {
                out.push(j);
            }
        }
        Ok(out)
    }

    pub async fn append_agent_event(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        kind: &str,
        payload: JsonValue,
    ) -> HelixResult<CodeAgentJobEvent> {
        let seq = self
            .allocate_number(tenant_id, "agent_event", job_id)
            .await?;
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.agent_job_events (id, job_id, tenant_id, seq, kind, payload, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
        )
        .bind(id)
        .bind(job_id)
        .bind(tenant_id.as_uuid())
        .bind(seq)
        .bind(kind)
        .bind(&payload)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("append event: {e}")))?;
        Ok(CodeAgentJobEvent {
            id,
            job_id,
            tenant_id,
            seq,
            kind: kind.into(),
            payload,
            created_at: now,
        })
    }

    pub async fn list_agent_events(
        &self,
        tenant_id: TenantId,
        job_id: Uuid,
        after_seq: i32,
    ) -> HelixResult<Vec<CodeAgentJobEvent>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            job_id: Uuid,
            tenant_id: Uuid,
            seq: i32,
            kind: String,
            payload: JsonValue,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT * FROM code.agent_job_events
               WHERE tenant_id = $1 AND job_id = $2 AND seq > $3 ORDER BY seq ASC LIMIT 500"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .bind(after_seq)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list events: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodeAgentJobEvent {
                id: r.id,
                job_id: r.job_id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                seq: r.seq,
                kind: r.kind,
                payload: r.payload,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn cancel_agent_job(&self, tenant_id: TenantId, job_id: Uuid) -> HelixResult<bool> {
        let r = sqlx::query(
            r#"UPDATE code.agent_jobs SET cancel_requested = true,
                 status = CASE WHEN status IN ('queued','running') THEN 'cancelled' ELSE status END
               WHERE tenant_id = $1 AND id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("cancel agent: {e}")))?;
        Ok(r.rows_affected() > 0)
    }

    // —— MLS devices / backup / groups list ——
    pub async fn upsert_mls_device(
        &self,
        tenant_id: TenantId,
        user_key: &str,
        device_id: &str,
        label: &str,
        public_identity_b64: &str,
    ) -> HelixResult<CodeMlsDevice> {
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO code.mls_devices (tenant_id, user_key, device_id, label, public_identity_b64, created_at)
               VALUES ($1,$2,$3,$4,$5,$6)
               ON CONFLICT (tenant_id, user_key, device_id) DO UPDATE SET label = EXCLUDED.label, public_identity_b64 = EXCLUDED.public_identity_b64"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_key)
        .bind(device_id)
        .bind(label)
        .bind(public_identity_b64)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("mls device: {e}")))?;
        Ok(CodeMlsDevice {
            tenant_id,
            user_key: user_key.into(),
            device_id: device_id.into(),
            label: label.into(),
            public_identity_b64: public_identity_b64.into(),
            created_at: now,
        })
    }

    pub async fn list_mls_devices(
        &self,
        tenant_id: TenantId,
        user_key: &str,
    ) -> HelixResult<Vec<CodeMlsDevice>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            tenant_id: Uuid,
            user_key: String,
            device_id: String,
            label: String,
            public_identity_b64: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT * FROM code.mls_devices WHERE tenant_id = $1 AND user_key = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_key)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list devices: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| CodeMlsDevice {
                tenant_id: TenantId::from_uuid(r.tenant_id),
                user_key: r.user_key,
                device_id: r.device_id,
                label: r.label,
                public_identity_b64: r.public_identity_b64,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn put_mls_key_backup(
        &self,
        tenant_id: TenantId,
        user_key: &str,
        ciphertext: &[u8],
    ) -> HelixResult<()> {
        sqlx::query(
            r#"INSERT INTO code.mls_key_backups (tenant_id, user_key, ciphertext, updated_at)
               VALUES ($1,$2,$3,now())
               ON CONFLICT (tenant_id, user_key) DO UPDATE SET ciphertext = EXCLUDED.ciphertext, updated_at = now()"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_key)
        .bind(ciphertext)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("key backup: {e}")))?;
        Ok(())
    }

    pub async fn get_mls_key_backup(
        &self,
        tenant_id: TenantId,
        user_key: &str,
    ) -> HelixResult<Option<Vec<u8>>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            r#"SELECT ciphertext FROM code.mls_key_backups WHERE tenant_id = $1 AND user_key = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("get backup: {e}")))?;
        Ok(row.map(|r| r.0))
    }

    pub async fn list_mls_groups_meta(&self, tenant_id: TenantId) -> HelixResult<Vec<JsonValue>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            group_key: String,
            repo_id: Option<Uuid>,
            name: String,
            epoch: i64,
            member_count: i32,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"SELECT id, group_key, repo_id, name, epoch, member_count FROM code.mls_groups_meta
               WHERE tenant_id = $1 ORDER BY updated_at DESC LIMIT 100"#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("list mls groups: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "group_key": r.group_key,
                    "repo_id": r.repo_id,
                    "name": r.name,
                    "epoch": r.epoch,
                    "member_count": r.member_count,
                })
            })
            .collect())
    }

    // —— Settings / quotas ——
    pub async fn get_user_settings(
        &self,
        tenant_id: TenantId,
        user_id: Uuid,
    ) -> HelixResult<JsonValue> {
        let row: Option<(JsonValue,)> = sqlx::query_as(
            r#"SELECT settings FROM code.user_settings WHERE tenant_id = $1 AND user_id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("get settings: {e}")))?;
        Ok(row.map(|r| r.0).unwrap_or_else(|| serde_json::json!({})))
    }

    pub async fn put_user_settings(
        &self,
        tenant_id: TenantId,
        user_id: Uuid,
        settings: JsonValue,
    ) -> HelixResult<()> {
        sqlx::query(
            r#"INSERT INTO code.user_settings (tenant_id, user_id, settings, updated_at)
               VALUES ($1,$2,$3,now())
               ON CONFLICT (tenant_id, user_id) DO UPDATE SET settings = EXCLUDED.settings, updated_at = now()"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(user_id)
        .bind(&settings)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("put settings: {e}")))?;
        Ok(())
    }

    pub async fn get_or_default_quota(&self, tenant_id: TenantId) -> HelixResult<CodeTenantQuota> {
        #[derive(sqlx::FromRow)]
        struct Row {
            tenant_id: Uuid,
            max_repos: i32,
            max_pipeline_runs_month: i32,
            max_agent_jobs_day: i32,
            max_sealed_bytes: i64,
        }
        if let Some(r) =
            sqlx::query_as::<_, Row>(r#"SELECT * FROM code.tenant_quotas WHERE tenant_id = $1"#)
                .bind(tenant_id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| HelixError::dependency(format!("get quota: {e}")))?
        {
            return Ok(CodeTenantQuota {
                tenant_id: TenantId::from_uuid(r.tenant_id),
                max_repos: r.max_repos,
                max_pipeline_runs_month: r.max_pipeline_runs_month,
                max_agent_jobs_day: r.max_agent_jobs_day,
                max_sealed_bytes: r.max_sealed_bytes,
            });
        }
        sqlx::query(
            r#"INSERT INTO code.tenant_quotas (tenant_id) VALUES ($1) ON CONFLICT DO NOTHING"#,
        )
        .bind(tenant_id.as_uuid())
        .execute(&self.pool)
        .await
        .ok();
        Ok(CodeTenantQuota {
            tenant_id,
            max_repos: 100,
            max_pipeline_runs_month: 500,
            max_agent_jobs_day: 200,
            max_sealed_bytes: 1_073_741_824,
        })
    }

    pub async fn get_pipeline_artifact(
        &self,
        tenant_id: TenantId,
        artifact_id: Uuid,
    ) -> HelixResult<Option<crate::code::CodePipelineArtifact>> {
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
        let row: Option<Row> = sqlx::query_as(
            r#"SELECT * FROM code.pipeline_artifacts WHERE tenant_id = $1 AND id = $2"#,
        )
        .bind(tenant_id.as_uuid())
        .bind(artifact_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("get artifact: {e}")))?;
        Ok(row.map(|r| crate::code::CodePipelineArtifact {
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
        }))
    }
}

#[cfg(test)]
mod protection_tests {
    use super::CodeRepoStore;

    #[test]
    fn protection_glob_match() {
        assert!(CodeRepoStore::protection_matches("main", "main"));
        assert!(CodeRepoStore::protection_matches(
            "release/*",
            "release/1.0"
        ));
        assert!(!CodeRepoStore::protection_matches("release/*", "main"));
        assert!(CodeRepoStore::protection_matches("*", "anything"));
    }
}
