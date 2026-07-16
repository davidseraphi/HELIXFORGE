-- Full-depth: MLS group state, WebAuthn credentials, residency hard policy, attachment bodies.

CREATE TABLE IF NOT EXISTS collab.mls_identities (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    identity_label TEXT NOT NULL,
    -- Serialized SignatureKeyPair + credential material (server holds for delivery-service role
    -- and for user agents that opt into server-held MLS identity; private keys are sealed HC1 optional).
    credential_blob BYTEA NOT NULL,
    signature_public_b64 TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, user_id, identity_label)
);

CREATE TABLE IF NOT EXISTS collab.mls_key_packages (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    identity_id UUID NOT NULL REFERENCES collab.mls_identities(id) ON DELETE CASCADE,
    key_package_tls BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    consumed_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS collab_mls_kp_user_idx
    ON collab.mls_key_packages (tenant_id, user_id) WHERE consumed_at IS NULL;

CREATE TABLE IF NOT EXISTS collab.mls_groups (
    group_id TEXT PRIMARY KEY,
    tenant_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES collab.documents(id) ON DELETE CASCADE,
    epoch BIGINT NOT NULL DEFAULT 0,
    ciphersuite TEXT NOT NULL DEFAULT 'MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519',
    -- Serialized group state blob for each member is in mls_member_state; this is group metadata.
    ratchet_tree BYTEA,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS collab_mls_groups_doc_idx ON collab.mls_groups (document_id);

CREATE TABLE IF NOT EXISTS collab.mls_member_state (
    group_id TEXT NOT NULL REFERENCES collab.mls_groups(group_id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    -- Opaque OpenMLS storage dump (JSON map from memory storage persistence)
    storage_json TEXT NOT NULL,
    leaf_index INT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (group_id, user_id)
);

CREATE TABLE IF NOT EXISTS collab.mls_messages (
    id UUID PRIMARY KEY,
    group_id TEXT NOT NULL REFERENCES collab.mls_groups(group_id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    sender_user_id UUID,
    epoch BIGINT NOT NULL DEFAULT 0,
    message_tls BYTEA NOT NULL,
    welcome_tls BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS collab_mls_msg_group_idx ON collab.mls_messages (group_id, created_at);

-- WebAuthn credentials bound to devices
ALTER TABLE collab.device_keys
    ADD COLUMN IF NOT EXISTS webauthn_cred_id BYTEA,
    ADD COLUMN IF NOT EXISTS webauthn_cose_key BYTEA,
    ADD COLUMN IF NOT EXISTS webauthn_counter BIGINT NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS collab.webauthn_challenges (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    challenge BYTEA NOT NULL,
    purpose TEXT NOT NULL, -- register | authenticate
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Required residency region for a document (hard enforce when set)
ALTER TABLE collab.documents
    ADD COLUMN IF NOT EXISTS required_region TEXT;

-- Attachment raw storage marker (MinIO key is object_key; body_stored flags bytes exist)
ALTER TABLE collab.attachments
    ADD COLUMN IF NOT EXISTS body_stored BOOLEAN NOT NULL DEFAULT false;
