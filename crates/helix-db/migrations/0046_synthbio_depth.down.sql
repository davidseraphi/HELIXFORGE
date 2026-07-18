DROP INDEX IF EXISTS synthbio_sims_active_idx;
DROP INDEX IF EXISTS synthbio_designs_active_idx;
ALTER TABLE synthbio.sims
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS failed_at,
    DROP COLUMN IF EXISTS completed_at,
    DROP COLUMN IF EXISTS started_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE synthbio.designs
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS approved_at,
    DROP COLUMN IF EXISTS submitted_at;
