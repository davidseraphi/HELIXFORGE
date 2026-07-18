DROP INDEX IF EXISTS climate_scores_active_idx;
DROP INDEX IF EXISTS climate_scenarios_active_idx;
ALTER TABLE climate.risk_scores
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS dismissed_at,
    DROP COLUMN IF EXISTS assessed_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE climate.scenarios
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS archived_at,
    DROP COLUMN IF EXISTS activated_at;
