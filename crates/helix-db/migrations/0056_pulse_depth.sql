-- HelixPulse W21 depth: durable monitors + incidents (first durable slice)
CREATE SCHEMA IF NOT EXISTS pulse;

CREATE TABLE IF NOT EXISTS pulse.monitors (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    activated_at TIMESTAMPTZ,
    paused_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS pulse_monitors_tenant_idx ON pulse.monitors (tenant_id);
CREATE INDEX IF NOT EXISTS pulse_monitors_active_idx
    ON pulse.monitors (tenant_id, status)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS pulse.incidents (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES pulse.monitors(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS pulse_incidents_parent_idx ON pulse.incidents (parent_id);
CREATE INDEX IF NOT EXISTS pulse_incidents_tenant_idx ON pulse.incidents (tenant_id);
CREATE INDEX IF NOT EXISTS pulse_incidents_active_idx
    ON pulse.incidents (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
