-- HelixTerra Prime W14 depth: field lifecycle, observation lifecycle
CREATE SCHEMA IF NOT EXISTS terra;

ALTER TABLE terra.fields
    ADD COLUMN IF NOT EXISTS activated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS retired_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE terra.observations
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS confirmed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS dismissed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every observation follows the new lifecycle
UPDATE terra.observations SET status = 'draft' WHERE status = 'open';
UPDATE terra.observations SET updated_at = created_at WHERE updated_at IS NULL;

-- Active field lookups
CREATE INDEX IF NOT EXISTS terra_fields_active_idx
    ON terra.fields (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active observation lookups per field
CREATE INDEX IF NOT EXISTS terra_observations_active_idx
    ON terra.observations (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
