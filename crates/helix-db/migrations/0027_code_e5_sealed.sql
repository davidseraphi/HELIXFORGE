-- HelixCode E5: sealed object depth (envelope metadata + forge crypto groups)

ALTER TABLE code.sealed_objects
    ADD COLUMN IF NOT EXISTS name TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS purpose TEXT NOT NULL DEFAULT 'forge.secret',
    ADD COLUMN IF NOT EXISTS envelope_kind TEXT NOT NULL DEFAULT 'hva4',
    ADD COLUMN IF NOT EXISTS content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    ADD COLUMN IF NOT EXISTS plaintext_sha256 TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS created_by TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS group_id UUID,
    ADD COLUMN IF NOT EXISTS cleartext_forbidden BOOLEAN NOT NULL DEFAULT true;

-- content_sha256 remains the ciphertext content hash (storage integrity)

CREATE TABLE IF NOT EXISTS code.crypto_groups (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    purpose TEXT NOT NULL DEFAULT 'forge.mls-like',
    owner_user TEXT NOT NULL,
    -- HVA4-wrapped group DEK (32 bytes) for the owner; members get their own wrap rows
    wrapped_dek_b64 TEXT NOT NULL,
    epoch BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS code_crypto_groups_tenant_idx ON code.crypto_groups (tenant_id);

CREATE TABLE IF NOT EXISTS code.crypto_group_members (
    id UUID PRIMARY KEY,
    group_id UUID NOT NULL REFERENCES code.crypto_groups(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    user_key TEXT NOT NULL,
    wrapped_dek_b64 TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (group_id, user_key)
);

CREATE INDEX IF NOT EXISTS code_crypto_group_members_group_idx ON code.crypto_group_members (group_id);


