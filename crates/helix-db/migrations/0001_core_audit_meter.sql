-- HelixCore durable audit + metering foundation
CREATE SCHEMA IF NOT EXISTS helix_core;
CREATE SCHEMA IF NOT EXISTS audit;

CREATE TABLE IF NOT EXISTS helix_core.tenants (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    residency_region TEXT NOT NULL DEFAULT 'local',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit.events (
    id UUID PRIMARY KEY,
    tenant_id UUID,
    actor JSONB NOT NULL,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL,
    prev_hash TEXT NOT NULL,
    entry_hash TEXT NOT NULL,
    residency_region TEXT NOT NULL DEFAULT 'local',
    seq BIGSERIAL NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS audit_events_entry_hash_uidx ON audit.events (entry_hash);
CREATE INDEX IF NOT EXISTS audit_events_created_at_idx ON audit.events (created_at DESC);
CREATE INDEX IF NOT EXISTS audit_events_tenant_idx ON audit.events (tenant_id);
CREATE INDEX IF NOT EXISTS audit_events_seq_idx ON audit.events (seq);

CREATE TABLE IF NOT EXISTS helix_core.meter_events (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    product TEXT NOT NULL,
    metric TEXT NOT NULL,
    quantity DOUBLE PRECISION NOT NULL CHECK (quantity >= 0),
    unit TEXT NOT NULL,
    dimensions JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS meter_events_tenant_product_idx
    ON helix_core.meter_events (tenant_id, product, metric);
CREATE INDEX IF NOT EXISTS meter_events_occurred_at_idx
    ON helix_core.meter_events (occurred_at DESC);

CREATE TABLE IF NOT EXISTS helix_core.workspaces (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    product_slug TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, product_slug, name)
);

CREATE INDEX IF NOT EXISTS workspaces_tenant_product_idx
    ON helix_core.workspaces (tenant_id, product_slug);
