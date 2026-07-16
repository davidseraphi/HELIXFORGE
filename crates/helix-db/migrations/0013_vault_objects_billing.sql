-- Vault object refs (MinIO/S3-compatible) + tenant plan assignment

CREATE TABLE IF NOT EXISTS helix_core.vault_objects (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    object_key TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    size_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS vault_objects_tenant_idx
    ON helix_core.vault_objects (tenant_id);

CREATE TABLE IF NOT EXISTS helix_core.tenant_plans (
    tenant_id UUID PRIMARY KEY,
    plan_id TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
