-- Foundation Integrity 011.4: recovery bin, restore, permanent-delete authority,
-- and policy exceptions.

-- 1. Recovery bin: tenant-scoped soft-deleted resources with a 30-day default
--    retention window.
CREATE TABLE IF NOT EXISTS helix_core.recovery_bin (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES helix_core.tenants(id) ON DELETE CASCADE,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    original_table TEXT,
    original_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    deleted_by TEXT NOT NULL,
    deleted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    retain_until TIMESTAMPTZ NOT NULL,
    restored_at TIMESTAMPTZ,
    permanently_deleted_at TIMESTAMPTZ,
    reason TEXT,
    seq BIGSERIAL NOT NULL
);

CREATE INDEX IF NOT EXISTS recovery_bin_tenant_idx
    ON helix_core.recovery_bin (tenant_id);
CREATE INDEX IF NOT EXISTS recovery_bin_expiry_idx
    ON helix_core.recovery_bin (retain_until)
    WHERE permanently_deleted_at IS NULL AND restored_at IS NULL;

-- 2. Policy exceptions: time-bound, justified, approved deviations from default
--    retention/governance policy.
CREATE TABLE IF NOT EXISTS helix_core.policy_exceptions (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES helix_core.tenants(id) ON DELETE CASCADE,
    resource_type TEXT,
    resource_id TEXT,
    policy_kind TEXT NOT NULL,
    justification TEXT NOT NULL,
    approved_by TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    revoked_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS policy_exceptions_tenant_idx
    ON helix_core.policy_exceptions (tenant_id);
CREATE INDEX IF NOT EXISTS policy_exceptions_active_idx
    ON helix_core.policy_exceptions (tenant_id, policy_kind, revoked_at, starts_at, expires_at);
