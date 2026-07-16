-- Residuals: deploy keys, sticky LSP session registry, debug sessions metadata

CREATE TABLE IF NOT EXISTS code.deploy_keys (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL REFERENCES code.repos(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    token_prefix TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT 'read', -- read | write
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    UNIQUE (tenant_id, token_hash)
);
CREATE INDEX IF NOT EXISTS code_deploy_keys_repo_idx ON code.deploy_keys (tenant_id, repo_id);

CREATE TABLE IF NOT EXISTS code.lsp_session_registry (
    session_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL,
    instance_id TEXT NOT NULL,
    server_cmd TEXT NOT NULL DEFAULT '',
    root_path TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS code_lsp_sess_tenant_idx ON code.lsp_session_registry (tenant_id, repo_id);

CREATE TABLE IF NOT EXISTS code.debug_sessions (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL,
    config TEXT NOT NULL DEFAULT 'launch',
    adapter TEXT NOT NULL DEFAULT 'none',
    status TEXT NOT NULL DEFAULT 'ready',
    breakpoints JSONB NOT NULL DEFAULT '[]'::jsonb,
    instance_id TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);
