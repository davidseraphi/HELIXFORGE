-- HelixCode horizons: durable OpenMLS user blobs + isolation metadata on runs/jobs

CREATE TABLE IF NOT EXISTS code.mls_user_blobs (
    tenant_id UUID NOT NULL,
    user_key TEXT NOT NULL,
    blob BYTEA NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, user_key)
);

CREATE TABLE IF NOT EXISTS code.mls_groups_meta (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    group_key TEXT NOT NULL,
    repo_id UUID REFERENCES code.repos(id) ON DELETE SET NULL,
    name TEXT NOT NULL DEFAULT '',
    epoch BIGINT NOT NULL DEFAULT 0,
    member_count INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, group_key)
);

CREATE INDEX IF NOT EXISTS code_mls_groups_tenant_idx ON code.mls_groups_meta (tenant_id);

ALTER TABLE code.pipeline_runs
    ADD COLUMN IF NOT EXISTS isolation TEXT NOT NULL DEFAULT 'host';

ALTER TABLE code.agent_jobs
    ADD COLUMN IF NOT EXISTS isolation TEXT NOT NULL DEFAULT 'host';
