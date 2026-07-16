-- Durable vault secrets (envelope ciphertext; plaintext never stored)
CREATE TABLE IF NOT EXISTS helix_core.secrets (
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1 CHECK (version >= 1),
    ciphertext BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS helix_core_secrets_tenant_idx ON helix_core.secrets (tenant_id);
