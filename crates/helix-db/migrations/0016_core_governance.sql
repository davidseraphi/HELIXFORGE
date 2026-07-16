-- HelixCore governance: resource ACL, retention, legal hold, purpose binding, multi-region

-- Generic resource ACL (doc, matter, patient, dataset, etc.)
CREATE TABLE IF NOT EXISTS helix_core.resource_acl (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    principal_kind TEXT NOT NULL, -- user | api_key | role | tenant
    principal_id TEXT NOT NULL,
    permissions TEXT[] NOT NULL DEFAULT ARRAY['read']::TEXT[],
    granted_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    UNIQUE (tenant_id, resource_type, resource_id, principal_kind, principal_id)
);

CREATE INDEX IF NOT EXISTS resource_acl_lookup_idx
    ON helix_core.resource_acl (tenant_id, resource_type, resource_id);
CREATE INDEX IF NOT EXISTS resource_acl_principal_idx
    ON helix_core.resource_acl (tenant_id, principal_kind, principal_id);

-- Retention policies per resource type (or resource instance)
CREATE TABLE IF NOT EXISTS helix_core.retention_policies (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT, -- NULL = type-wide default
    retain_days INT NOT NULL CHECK (retain_days >= 0),
    action_on_expiry TEXT NOT NULL DEFAULT 'review', -- review | delete | anonymize
    purpose TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS retention_policies_tenant_type_idx
    ON helix_core.retention_policies (tenant_id, resource_type);

-- Legal holds block delete/anonymize
CREATE TABLE IF NOT EXISTS helix_core.legal_holds (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    placed_by TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    released_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS legal_holds_active_idx
    ON helix_core.legal_holds (tenant_id, resource_type, resource_id)
    WHERE active = true;

-- Purpose / consent binding (processing purpose must match for sensitive ops)
CREATE TABLE IF NOT EXISTS helix_core.purpose_bindings (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    purpose TEXT NOT NULL,
    legal_basis TEXT NOT NULL DEFAULT 'consent',
    subject_ref TEXT, -- patient/user id when applicable
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS purpose_bindings_lookup_idx
    ON helix_core.purpose_bindings (tenant_id, resource_type, resource_id)
    WHERE active = true;

-- Multi-region registry (write affinity + status)
CREATE TABLE IF NOT EXISTS helix_core.regions (
    code TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    write_enabled BOOLEAN NOT NULL DEFAULT true,
    read_enabled BOOLEAN NOT NULL DEFAULT true,
    endpoint_hint TEXT,
    status TEXT NOT NULL DEFAULT 'healthy', -- healthy | degraded | offline
    lag_seconds INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO helix_core.regions (code, name, is_primary, write_enabled, read_enabled, endpoint_hint, status)
VALUES
    ('local', 'Local development', true, true, true, 'http://127.0.0.1:8080', 'healthy'),
    ('eu-west', 'EU West', false, true, true, NULL, 'standby'),
    ('us-east', 'US East', false, true, true, NULL, 'standby')
ON CONFLICT (code) DO NOTHING;
