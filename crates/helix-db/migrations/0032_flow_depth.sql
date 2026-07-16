-- HelixFlow second-wave depth: run steps, events, cancel, result payload.

ALTER TABLE flow.runs
    ADD COLUMN IF NOT EXISTS current_step INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS result JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS error_text TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS cancel_requested BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE IF NOT EXISTS flow.step_events (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES flow.runs(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    step_index INT NOT NULL,
    step_name TEXT NOT NULL DEFAULT '',
    step_type TEXT NOT NULL DEFAULT 'echo',
    status TEXT NOT NULL DEFAULT 'queued',
    output JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS flow_step_events_run_idx ON flow.step_events (run_id, step_index);
CREATE INDEX IF NOT EXISTS flow_runs_tenant_idx ON flow.runs (tenant_id, started_at DESC);
