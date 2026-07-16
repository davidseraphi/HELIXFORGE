-- Enterprise HelixCore: tenant lifecycle, service API keys, vault key meta

ALTER TABLE helix_core.tenants
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS plan_id TEXT,
    ADD COLUMN IF NOT EXISTS suspended_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX IF NOT EXISTS tenants_status_idx ON helix_core.tenants (status);

CREATE TABLE IF NOT EXISTS helix_core.service_api_keys (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    key_prefix TEXT NOT NULL,
    key_hash TEXT NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT ARRAY['read']::TEXT[],
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ,
    UNIQUE (key_prefix)
);

CREATE INDEX IF NOT EXISTS service_api_keys_tenant_idx
    ON helix_core.service_api_keys (tenant_id)
    WHERE revoked_at IS NULL;

CREATE TABLE IF NOT EXISTS helix_core.vault_key_meta (
    id TEXT PRIMARY KEY DEFAULT 'default',
    version INT NOT NULL DEFAULT 1,
    algorithm TEXT NOT NULL DEFAULT 'HVA3',
    rotated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    note TEXT NOT NULL DEFAULT ''
);

INSERT INTO helix_core.vault_key_meta (id, version, algorithm, note)
VALUES ('default', 1, 'HVA3', 'initial')
ON CONFLICT (id) DO NOTHING;
