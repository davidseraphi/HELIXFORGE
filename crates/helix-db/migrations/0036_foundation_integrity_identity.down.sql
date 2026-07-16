-- Rollback for Foundation Integrity 011.2.
-- Removes RLS, foreign keys, NOT NULL constraints, and the memberships table.
-- Existing tenant/user rows are preserved.

DROP POLICY IF EXISTS tenant_isolation_policy ON helix_core.tenants;
DROP POLICY IF EXISTS tenant_isolation_policy ON helix_core.memberships;
DROP POLICY IF EXISTS tenant_isolation_policy ON helix_core.workspaces;
DROP POLICY IF EXISTS tenant_isolation_policy ON helix_core.meter_events;
DROP POLICY IF EXISTS tenant_isolation_policy ON helix_core.resource_acl;
DROP POLICY IF EXISTS tenant_isolation_policy ON audit.events;

ALTER TABLE helix_core.tenants DISABLE ROW LEVEL SECURITY;
ALTER TABLE helix_core.memberships DISABLE ROW LEVEL SECURITY;
ALTER TABLE helix_core.workspaces DISABLE ROW LEVEL SECURITY;
ALTER TABLE helix_core.meter_events DISABLE ROW LEVEL SECURITY;
ALTER TABLE helix_core.resource_acl DISABLE ROW LEVEL SECURITY;
ALTER TABLE audit.events DISABLE ROW LEVEL SECURITY;

ALTER TABLE helix_core.workspaces
    DROP CONSTRAINT IF EXISTS workspaces_tenant_fk,
    ALTER COLUMN tenant_id DROP NOT NULL;

ALTER TABLE helix_core.meter_events
    DROP CONSTRAINT IF EXISTS meter_events_tenant_fk,
    ALTER COLUMN tenant_id DROP NOT NULL;

ALTER TABLE helix_core.resource_acl
    DROP CONSTRAINT IF EXISTS resource_acl_tenant_fk,
    ALTER COLUMN tenant_id DROP NOT NULL;

ALTER TABLE audit.events
    DROP CONSTRAINT IF EXISTS audit_events_tenant_fk,
    ALTER COLUMN tenant_id DROP NOT NULL;

DROP TABLE IF EXISTS helix_core.memberships;
