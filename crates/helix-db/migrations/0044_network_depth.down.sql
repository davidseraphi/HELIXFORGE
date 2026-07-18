DROP INDEX IF EXISTS network_opportunities_active_idx;
DROP INDEX IF EXISTS network_connections_pair_idx;
DROP INDEX IF EXISTS network_profiles_active_idx;
ALTER TABLE network.opportunities
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS closed_at;
ALTER TABLE network.connections
    DROP COLUMN IF EXISTS blocked_by,
    DROP COLUMN IF EXISTS responded_at;
ALTER TABLE network.profiles
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS deactivated_at;
