DROP INDEX IF EXISTS vita_cohorts_active_idx;
DROP INDEX IF EXISTS vita_studies_active_idx;
ALTER TABLE vita.cohorts
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS withdrawn_at,
    DROP COLUMN IF EXISTS enrolled_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE vita.studies
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS terminated_at,
    DROP COLUMN IF EXISTS completed_at,
    DROP COLUMN IF EXISTS recruiting_at;
