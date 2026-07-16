-- HelixCollab sovereign stack (horizons A–C scaffolding + durable state).

-- Classification drives crypto policy (Public..Sovereign).
ALTER TABLE collab.documents
    ADD COLUMN IF NOT EXISTS classification TEXT NOT NULL DEFAULT 'internal',
    ADD COLUMN IF NOT EXISTS space_id UUID,
    ADD COLUMN IF NOT EXISTS sealed_comments BOOLEAN NOT NULL DEFAULT false;

-- Spaces tree (Horizon B) under workspace.
CREATE TABLE IF NOT EXISTS collab.spaces (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    parent_id UUID REFERENCES collab.spaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    classification TEXT NOT NULL DEFAULT 'internal',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS collab_spaces_ws_idx ON collab.spaces (workspace_id, parent_id);

-- Device key registry (Horizon A): public keys only; private material never leaves device.
CREATE TABLE IF NOT EXISTS collab.device_keys (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    device_label TEXT NOT NULL DEFAULT '',
    public_key_b64 TEXT NOT NULL,
    credential_id TEXT,
    algorithm TEXT NOT NULL DEFAULT 'ECDSA_P256',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS collab_device_keys_user_idx
    ON collab.device_keys (tenant_id, user_id) WHERE revoked_at IS NULL;

-- Wrapped DEK shares for multi-device / threshold (Horizon A/C).
CREATE TABLE IF NOT EXISTS collab.key_shares (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES collab.documents(id) ON DELETE CASCADE,
    device_key_id UUID REFERENCES collab.device_keys(id) ON DELETE CASCADE,
    -- HC1 or SPKI-wrapped DEK blob (opaque to server).
    wrapped_dek TEXT NOT NULL,
    share_kind TEXT NOT NULL DEFAULT 'device', -- device | threshold_shard
    threshold_n INT,
    threshold_k INT,
    shard_index INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS collab_key_shares_doc_idx ON collab.key_shares (document_id);

-- Sealed durable CRDT state (Horizon A JetStream companion in Postgres).
CREATE TABLE IF NOT EXISTS collab.sealed_crdt_state (
    document_id UUID PRIMARY KEY REFERENCES collab.documents(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    sealed_state TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by UUID
);

-- Attachments (Horizon B) — ciphertext only when sealed.
CREATE TABLE IF NOT EXISTS collab.attachments (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES collab.documents(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    size_bytes BIGINT NOT NULL DEFAULT 0,
    -- MinIO object key or inline small blob reference.
    object_key TEXT NOT NULL,
    client_sealed BOOLEAN NOT NULL DEFAULT false,
    sha256_hex TEXT NOT NULL DEFAULT '',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS collab_attachments_doc_idx ON collab.attachments (document_id);

-- Residency proofs (Horizon C).
CREATE TABLE IF NOT EXISTS collab.residency_proofs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    document_id UUID,
    workspace_id UUID,
    claimed_region TEXT NOT NULL,
    evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
    verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Federation receipts (Horizon C).
CREATE TABLE IF NOT EXISTS collab.federation_receipts (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    remote_deployment TEXT NOT NULL,
    document_id UUID,
    direction TEXT NOT NULL, -- export | import
    payload_hash TEXT NOT NULL,
    signature_b64 TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Threshold recovery ceremonies (Horizon C bookkeeping; shards stay opaque).
CREATE TABLE IF NOT EXISTS collab.recovery_ceremonies (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES collab.documents(id) ON DELETE CASCADE,
    k INT NOT NULL,
    n INT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open', -- open | completed | aborted
    meta JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);
