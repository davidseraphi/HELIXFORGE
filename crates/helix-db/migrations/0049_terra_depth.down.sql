DROP INDEX IF EXISTS terra_observations_active_idx;
DROP INDEX IF EXISTS terra_fields_active_idx;
ALTER TABLE terra.observations
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS dismissed_at,
    DROP COLUMN IF EXISTS confirmed_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE terra.fields
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS retired_at,
    DROP COLUMN IF EXISTS activated_at;
