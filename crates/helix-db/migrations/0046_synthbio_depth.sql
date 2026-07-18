-- HelixSynthBio W11 depth: design review lifecycle, sim run lifecycle
CREATE SCHEMA IF NOT EXISTS synthbio;

ALTER TABLE synthbio.designs
    ADD COLUMN IF NOT EXISTS submitted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS approved_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE synthbio.sims
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS failed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Backfill sim updated_at so existing rows carry a sane value
UPDATE synthbio.sims SET updated_at = created_at WHERE updated_at IS NULL;

-- Active design lookups
CREATE INDEX IF NOT EXISTS synthbio_designs_active_idx
    ON synthbio.designs (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active sim lookups per design
CREATE INDEX IF NOT EXISTS synthbio_sims_active_idx
    ON synthbio.sims (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
