-- Per-tenant break-glass + process session sticky registry (terminal / DAP HA).

CREATE TABLE IF NOT EXISTS code.tenant_breakglass (
    tenant_id UUID PRIMARY KEY,
    allow_direct_push BOOLEAN NOT NULL DEFAULT false,
    allow_force_push BOOLEAN NOT NULL DEFAULT false,
    allow_ci_all BOOLEAN NOT NULL DEFAULT false,
    allow_term_all BOOLEAN NOT NULL DEFAULT false,
    allow_host_fallback BOOLEAN NOT NULL DEFAULT false,
    allow_host_isolation BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS code.process_sessions (
    session_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    kind TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    repo_id UUID,
    meta JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS code_process_sessions_tenant_idx
    ON code.process_sessions (tenant_id, kind);
CREATE INDEX IF NOT EXISTS code_process_sessions_instance_idx
    ON code.process_sessions (instance_id);
