-- HelixCode end-state: collab, CI fleet, agents depth, MLS devices, settings, quotas

CREATE TABLE IF NOT EXISTS code.issues (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    number INT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    state TEXT NOT NULL DEFAULT 'open',
    author TEXT NOT NULL DEFAULT '',
    labels JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at TIMESTAMPTZ,
    UNIQUE (tenant_id, repo_id, number)
);
CREATE INDEX IF NOT EXISTS code_issues_repo_idx ON code.issues (tenant_id, repo_id);

CREATE TABLE IF NOT EXISTS code.pull_requests (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    number INT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    state TEXT NOT NULL DEFAULT 'open',
    source_branch TEXT NOT NULL,
    target_branch TEXT NOT NULL DEFAULT 'main',
    author TEXT NOT NULL DEFAULT '',
    head_sha TEXT,
    merge_sha TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    merged_at TIMESTAMPTZ,
    UNIQUE (tenant_id, repo_id, number)
);
CREATE INDEX IF NOT EXISTS code_prs_repo_idx ON code.pull_requests (tenant_id, repo_id);

CREATE TABLE IF NOT EXISTS code.pr_reviews (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    pr_id UUID NOT NULL REFERENCES code.pull_requests(id) ON DELETE CASCADE,
    author TEXT NOT NULL DEFAULT '',
    state TEXT NOT NULL DEFAULT 'comment',
    body TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS code.branch_protections (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    branch_pattern TEXT NOT NULL,
    require_pr BOOLEAN NOT NULL DEFAULT true,
    require_approvals INT NOT NULL DEFAULT 0,
    deny_force_push BOOLEAN NOT NULL DEFAULT true,
    required_status_checks JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, repo_id, branch_pattern)
);

CREATE TABLE IF NOT EXISTS code.webhooks (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret TEXT NOT NULL DEFAULT '',
    events JSONB NOT NULL DEFAULT '["*"]'::jsonb,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS code.webhook_deliveries (
    id UUID PRIMARY KEY,
    webhook_id UUID NOT NULL REFERENCES code.webhooks(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    event TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    response_code INT,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE code.pipeline_runs
    ADD COLUMN IF NOT EXISTS matrix_index INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS parent_run_id UUID,
    ADD COLUMN IF NOT EXISTS cancel_requested BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS worker_id TEXT,
    ADD COLUMN IF NOT EXISTS matrix_combo JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE TABLE IF NOT EXISTS code.runners (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    labels JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'online',
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)
);

CREATE TABLE IF NOT EXISTS code.pipeline_secret_names (
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    vault_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, repo_id, name)
);

ALTER TABLE code.agent_jobs
    ADD COLUMN IF NOT EXISTS policy JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS context_paths JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS cancel_requested BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS step_budget INT NOT NULL DEFAULT 20;

CREATE TABLE IF NOT EXISTS code.agent_job_events (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES code.agent_jobs(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    seq INT NOT NULL,
    kind TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (job_id, seq)
);

CREATE TABLE IF NOT EXISTS code.mls_devices (
    tenant_id UUID NOT NULL,
    user_key TEXT NOT NULL,
    device_id TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    public_identity_b64 TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, user_key, device_id)
);

CREATE TABLE IF NOT EXISTS code.mls_key_backups (
    tenant_id UUID NOT NULL,
    user_key TEXT NOT NULL,
    ciphertext BYTEA NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, user_key)
);

CREATE TABLE IF NOT EXISTS code.user_settings (
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, user_id)
);

CREATE TABLE IF NOT EXISTS code.tenant_quotas (
    tenant_id UUID PRIMARY KEY,
    max_repos INT NOT NULL DEFAULT 100,
    max_pipeline_runs_month INT NOT NULL DEFAULT 500,
    max_agent_jobs_day INT NOT NULL DEFAULT 200,
    max_sealed_bytes BIGINT NOT NULL DEFAULT 1073741824,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE code.workspaces
    ADD COLUMN IF NOT EXISTS lsp_config JSONB NOT NULL DEFAULT '{}'::jsonb;
