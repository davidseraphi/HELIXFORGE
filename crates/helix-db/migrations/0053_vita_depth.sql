-- HelixVita Prime W18 depth: study lifecycle, cohort lifecycle
CREATE SCHEMA IF NOT EXISTS vita;

ALTER TABLE vita.studies
    ADD COLUMN IF NOT EXISTS recruiting_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS terminated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE vita.cohorts
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS enrolled_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS withdrawn_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every cohort follows the new lifecycle
UPDATE vita.cohorts SET status = 'draft' WHERE status = 'open';
UPDATE vita.cohorts SET updated_at = created_at WHERE updated_at IS NULL;

-- Active study lookups
CREATE INDEX IF NOT EXISTS vita_studies_active_idx
    ON vita.studies (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active cohort lookups per study
CREATE INDEX IF NOT EXISTS vita_cohorts_active_idx
    ON vita.cohorts (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
