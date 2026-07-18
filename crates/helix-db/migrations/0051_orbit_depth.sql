-- HelixOrbit Prime W16 depth: asset lifecycle, pass lifecycle
CREATE SCHEMA IF NOT EXISTS orbit;

ALTER TABLE orbit.assets
    ADD COLUMN IF NOT EXISTS commissioned_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS decommissioned_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE orbit.passes
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS planned_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS cancelled_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every pass follows the new lifecycle
UPDATE orbit.passes SET status = 'draft' WHERE status = 'open';
UPDATE orbit.passes SET updated_at = created_at WHERE updated_at IS NULL;

-- Active asset lookups
CREATE INDEX IF NOT EXISTS orbit_assets_active_idx
    ON orbit.assets (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active pass lookups per asset
CREATE INDEX IF NOT EXISTS orbit_passes_active_idx
    ON orbit.passes (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
