DROP INDEX IF EXISTS grid_readings_active_idx;
DROP INDEX IF EXISTS grid_sites_active_idx;
ALTER TABLE grid.readings
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS rejected_at,
    DROP COLUMN IF EXISTS verified_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE grid.sites
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS offline_at,
    DROP COLUMN IF EXISTS energized_at;
