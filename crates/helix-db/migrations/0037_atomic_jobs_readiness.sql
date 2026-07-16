-- Foundation Integrity 011.3: atomic writes, durable visible jobs, truthful
-- readiness, and fresh release gates.

-- 1. Outbox: durable, idempotent event dispatch queue committed together with
--    domain changes and audit events.
CREATE TABLE IF NOT EXISTS helix_core.outbox (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES helix_core.tenants(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL,
    topic TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    headers JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at TIMESTAMPTZ,
    error TEXT,
    attempt_count INT NOT NULL DEFAULT 0,
    seq BIGSERIAL NOT NULL,
    UNIQUE (tenant_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS outbox_unprocessed_idx
    ON helix_core.outbox (tenant_id, topic)
    WHERE processed_at IS NULL;

-- 2. Jobs: durable, visible, long-running work with lifecycle, lease, and
--    checkpoint support.
CREATE TABLE IF NOT EXISTS helix_core.jobs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES helix_core.tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN (
            'queued', 'starting', 'running', 'waiting', 'blocked',
            'cancelling', 'cancelled', 'failed', 'completed', 'unknown'
        )),
    requested TEXT NOT NULL,
    stages JSONB NOT NULL DEFAULT '[]'::jsonb,
    checkpoints JSONB NOT NULL DEFAULT '[]'::jsonb,
    process_identity TEXT,
    lease_expires_at TIMESTAMPTZ,
    cancel_requested BOOLEAN NOT NULL DEFAULT false,
    retry_count INT NOT NULL DEFAULT 0,
    max_retries INT NOT NULL DEFAULT 3,
    started_at TIMESTAMPTZ,
    last_heartbeat_at TIMESTAMPTZ,
    elapsed_ms BIGINT NOT NULL DEFAULT 0,
    resource_usage JSONB NOT NULL DEFAULT '{}'::jsonb,
    error TEXT,
    final_output JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS jobs_tenant_status_idx
    ON helix_core.jobs (tenant_id, status);

CREATE INDEX IF NOT EXISTS jobs_worker_poll_idx
    ON helix_core.jobs (status, lease_expires_at)
    WHERE status IN ('queued', 'running', 'waiting', 'blocked', 'cancelling');
