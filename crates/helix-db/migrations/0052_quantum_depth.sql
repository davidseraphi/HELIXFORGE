-- HelixQuantum Forge W17 depth: job lifecycle, circuit lifecycle
CREATE SCHEMA IF NOT EXISTS quantum;

ALTER TABLE quantum.jobs
    ADD COLUMN IF NOT EXISTS submitted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS failed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE quantum.circuits
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS validated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every circuit follows the new lifecycle
UPDATE quantum.circuits SET status = 'draft' WHERE status = 'open';
UPDATE quantum.circuits SET updated_at = created_at WHERE updated_at IS NULL;

-- Active job lookups
CREATE INDEX IF NOT EXISTS quantum_jobs_active_idx
    ON quantum.jobs (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active circuit lookups per job
CREATE INDEX IF NOT EXISTS quantum_circuits_active_idx
    ON quantum.circuits (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
