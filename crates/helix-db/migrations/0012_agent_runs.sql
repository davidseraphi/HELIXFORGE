-- Durable agent run records (Agent Hub)
CREATE TABLE IF NOT EXISTS helix_core.agent_runs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    agent TEXT NOT NULL,
    status TEXT NOT NULL,
    input JSONB NOT NULL DEFAULT '{}'::jsonb,
    output JSONB,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS helix_core_agent_runs_tenant_idx
    ON helix_core.agent_runs (tenant_id, started_at DESC);
CREATE INDEX IF NOT EXISTS helix_core_agent_runs_agent_idx
    ON helix_core.agent_runs (agent);
