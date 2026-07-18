DROP INDEX IF EXISTS nova_findings_active_idx;
DROP INDEX IF EXISTS nova_experiments_active_idx;
ALTER TABLE nova.findings
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS rejected_at,
    DROP COLUMN IF EXISTS confirmed_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE nova.experiments
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS concluded_at,
    DROP COLUMN IF EXISTS started_at;
