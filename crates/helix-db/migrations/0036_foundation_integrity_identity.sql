-- Foundation Integrity 011.2: stable identity, safe registration, tenant separation,
-- and per-resource access.

-- 1. Tenant membership: explicit user-to-tenant bindings and roles.
CREATE TABLE IF NOT EXISTS helix_core.memberships (
    tenant_id UUID NOT NULL REFERENCES helix_core.tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    role TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'admin', 'member', 'guest')),
    invited_by UUID,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, user_id)
);

CREATE INDEX IF NOT EXISTS memberships_user_idx ON helix_core.memberships (user_id);

-- 2. Ensure shared tables have a non-null tenant_id and reference helix_core.tenants.
ALTER TABLE helix_core.workspaces
    ALTER COLUMN tenant_id SET NOT NULL,
    ADD CONSTRAINT workspaces_tenant_fk FOREIGN KEY (tenant_id) REFERENCES helix_core.tenants(id) ON DELETE CASCADE;

ALTER TABLE helix_core.meter_events
    ALTER COLUMN tenant_id SET NOT NULL,
    ADD CONSTRAINT meter_events_tenant_fk FOREIGN KEY (tenant_id) REFERENCES helix_core.tenants(id) ON DELETE CASCADE;

ALTER TABLE helix_core.resource_acl
    ALTER COLUMN tenant_id SET NOT NULL,
    ADD CONSTRAINT resource_acl_tenant_fk FOREIGN KEY (tenant_id) REFERENCES helix_core.tenants(id) ON DELETE CASCADE;

ALTER TABLE audit.events
    ALTER COLUMN tenant_id SET NOT NULL,
    ADD CONSTRAINT audit_events_tenant_fk FOREIGN KEY (tenant_id) REFERENCES helix_core.tenants(id) ON DELETE CASCADE;

-- 3. Helper to set the tenant context used by RLS policies.
CREATE OR REPLACE FUNCTION helix_core.set_tenant_context(p_tenant_id UUID)
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant', p_tenant_id::TEXT, false);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- 4. Enable RLS on tenant-scoped tables.
ALTER TABLE helix_core.tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE helix_core.memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE helix_core.workspaces ENABLE ROW LEVEL SECURITY;
ALTER TABLE helix_core.meter_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE helix_core.resource_acl ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit.events ENABLE ROW LEVEL SECURITY;

-- 5. RLS policies: tenant-scoped reads/writes are limited to the current tenant
--    set via helix_core.set_tenant_context(). Platform/admin bypass is handled by
--    the application layer resetting the context to the target tenant after
--    authorization.
CREATE POLICY tenant_isolation_policy ON helix_core.tenants
    USING (id = current_setting('app.current_tenant')::UUID)
    WITH CHECK (id = current_setting('app.current_tenant')::UUID);

CREATE POLICY tenant_isolation_policy ON helix_core.memberships
    USING (tenant_id = current_setting('app.current_tenant')::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant')::UUID);

CREATE POLICY tenant_isolation_policy ON helix_core.workspaces
    USING (tenant_id = current_setting('app.current_tenant')::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant')::UUID);

CREATE POLICY tenant_isolation_policy ON helix_core.meter_events
    USING (tenant_id = current_setting('app.current_tenant')::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant')::UUID);

CREATE POLICY tenant_isolation_policy ON helix_core.resource_acl
    USING (tenant_id = current_setting('app.current_tenant')::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant')::UUID);

CREATE POLICY tenant_isolation_policy ON audit.events
    USING (tenant_id = current_setting('app.current_tenant')::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant')::UUID);
