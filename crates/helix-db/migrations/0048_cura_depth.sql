-- HelixCura Prime W13 depth: case lifecycle, note sign/void lifecycle
CREATE SCHEMA IF NOT EXISTS cura;

ALTER TABLE cura.care_cases
    ADD COLUMN IF NOT EXISTS activated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS discharged_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE cura.notes
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS signed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS voided_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every note follows the new lifecycle
UPDATE cura.notes SET status = 'draft' WHERE status = 'open';
UPDATE cura.notes SET updated_at = created_at WHERE updated_at IS NULL;

-- Active case lookups
CREATE INDEX IF NOT EXISTS cura_cases_active_idx
    ON cura.care_cases (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active note lookups per case
CREATE INDEX IF NOT EXISTS cura_notes_active_idx
    ON cura.notes (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
