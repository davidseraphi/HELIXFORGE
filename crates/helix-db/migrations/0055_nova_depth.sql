-- HelixNova Labs W20 depth: experiment lifecycle, finding lifecycle
CREATE SCHEMA IF NOT EXISTS nova;

ALTER TABLE nova.experiments
    ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS concluded_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE nova.findings
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS confirmed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS rejected_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every finding follows the new lifecycle
UPDATE nova.findings SET status = 'draft' WHERE status = 'open';
UPDATE nova.findings SET updated_at = created_at WHERE updated_at IS NULL;

-- Active experiment lookups
CREATE INDEX IF NOT EXISTS nova_experiments_active_idx
    ON nova.experiments (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active finding lookups per experiment
CREATE INDEX IF NOT EXISTS nova_findings_active_idx
    ON nova.findings (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
