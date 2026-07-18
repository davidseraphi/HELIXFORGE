-- HelixGrid Prime W19 depth: site lifecycle, reading lifecycle
CREATE SCHEMA IF NOT EXISTS grid;

ALTER TABLE grid.sites
    ADD COLUMN IF NOT EXISTS energized_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS offline_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE grid.readings
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS verified_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS rejected_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every reading follows the new lifecycle
UPDATE grid.readings SET status = 'draft' WHERE status = 'open';
UPDATE grid.readings SET updated_at = created_at WHERE updated_at IS NULL;

-- Active site lookups
CREATE INDEX IF NOT EXISTS grid_sites_active_idx
    ON grid.sites (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active reading lookups per site
CREATE INDEX IF NOT EXISTS grid_readings_active_idx
    ON grid.readings (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
