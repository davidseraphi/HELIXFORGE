-- HelixNetwork W9 depth: profile lifecycle, connection lifecycle, opportunity lifecycle
CREATE SCHEMA IF NOT EXISTS network;

ALTER TABLE network.profiles
    ADD COLUMN IF NOT EXISTS deactivated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE network.connections
    ADD COLUMN IF NOT EXISTS responded_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS blocked_by UUID;

ALTER TABLE network.opportunities
    ADD COLUMN IF NOT EXISTS closed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Active profile lookups
CREATE INDEX IF NOT EXISTS network_profiles_active_idx
    ON network.profiles (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Pair lookups for blocked-pair and revival checks
CREATE INDEX IF NOT EXISTS network_connections_pair_idx
    ON network.connections (tenant_id, from_profile_id, to_profile_id, status);

-- Active opportunity lookups
CREATE INDEX IF NOT EXISTS network_opportunities_active_idx
    ON network.opportunities (tenant_id, status)
    WHERE deleted_at IS NULL;
