DROP INDEX IF EXISTS quantum_circuits_active_idx;
DROP INDEX IF EXISTS quantum_jobs_active_idx;
ALTER TABLE quantum.circuits
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS archived_at,
    DROP COLUMN IF EXISTS validated_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE quantum.jobs
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS failed_at,
    DROP COLUMN IF EXISTS completed_at,
    DROP COLUMN IF EXISTS submitted_at;
