-- HelixCode + HelixFlow durable domain tables (reuse helix_core tenancy)
CREATE SCHEMA IF NOT EXISTS code;
CREATE SCHEMA IF NOT EXISTS flow;

CREATE TABLE IF NOT EXISTS code.repos (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    default_branch TEXT NOT NULL DEFAULT 'main',
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS code_repos_tenant_idx ON code.repos (tenant_id);

CREATE TABLE IF NOT EXISTS flow.workflows (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    steps INT NOT NULL DEFAULT 1 CHECK (steps >= 1),
    status TEXT NOT NULL DEFAULT 'draft',
    definition JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS flow_workflows_tenant_idx ON flow.workflows (tenant_id);

CREATE TABLE IF NOT EXISTS flow.runs (
    id UUID PRIMARY KEY,
    workflow_id UUID NOT NULL REFERENCES flow.workflows(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued',
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);
