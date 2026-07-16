-- HelixCode extreme: forge planes (git metadata, workspaces, CI, agents, sealed index)
-- Extends 0003_code_flow.sql code.repos

ALTER TABLE code.repos
    ADD COLUMN IF NOT EXISTS visibility TEXT NOT NULL DEFAULT 'private',
    ADD COLUMN IF NOT EXISTS storage_kind TEXT NOT NULL DEFAULT 'bare_fs',
    ADD COLUMN IF NOT EXISTS head_sha TEXT,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now();

CREATE TABLE IF NOT EXISTS code.refs (
    id UUID PRIMARY KEY,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    target_sha TEXT NOT NULL,
    is_symbolic BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (repo_id, name)
);

CREATE INDEX IF NOT EXISTS code_refs_repo_idx ON code.refs (repo_id);
CREATE INDEX IF NOT EXISTS code_refs_tenant_idx ON code.refs (tenant_id);

CREATE TABLE IF NOT EXISTS code.workspaces (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    branch TEXT NOT NULL DEFAULT 'main',
    root_path TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS code_workspaces_tenant_idx ON code.workspaces (tenant_id);
CREATE INDEX IF NOT EXISTS code_workspaces_repo_idx ON code.workspaces (repo_id);

CREATE TABLE IF NOT EXISTS code.pipelines (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    definition JSONB NOT NULL DEFAULT '{}'::jsonb,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (repo_id, name)
);

CREATE INDEX IF NOT EXISTS code_pipelines_repo_idx ON code.pipelines (repo_id);

CREATE TABLE IF NOT EXISTS code.pipeline_runs (
    id UUID PRIMARY KEY,
    pipeline_id UUID NOT NULL REFERENCES code.pipelines(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued',
    trigger_ref TEXT NOT NULL DEFAULT 'refs/heads/main',
    commit_sha TEXT,
    log_text TEXT NOT NULL DEFAULT '',
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS code_pipeline_runs_pipeline_idx ON code.pipeline_runs (pipeline_id);

CREATE TABLE IF NOT EXISTS code.agent_jobs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    workspace_id UUID REFERENCES code.workspaces(id) ON DELETE SET NULL,
    kind TEXT NOT NULL DEFAULT 'sandbox',
    status TEXT NOT NULL DEFAULT 'queued',
    prompt TEXT NOT NULL DEFAULT '',
    result_summary TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS code_agent_jobs_repo_idx ON code.agent_jobs (repo_id);

-- Content-addressed sealed object index (bodies in MinIO; cleartext forbidden for sealed class)
CREATE TABLE IF NOT EXISTS code.sealed_objects (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID REFERENCES code.repos(id) ON DELETE CASCADE,
    content_sha256 TEXT NOT NULL,
    storage_key TEXT NOT NULL,
    classification TEXT NOT NULL DEFAULT 'internal',
    byte_len BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, content_sha256)
);

CREATE INDEX IF NOT EXISTS code_sealed_objects_tenant_idx ON code.sealed_objects (tenant_id);
