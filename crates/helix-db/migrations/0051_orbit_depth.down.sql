DROP INDEX IF EXISTS orbit_passes_active_idx;
DROP INDEX IF EXISTS orbit_assets_active_idx;
ALTER TABLE orbit.passes
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS cancelled_at,
    DROP COLUMN IF EXISTS completed_at,
    DROP COLUMN IF EXISTS planned_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE orbit.assets
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS decommissioned_at,
    DROP COLUMN IF EXISTS commissioned_at;
