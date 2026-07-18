-- HelixClimate Prime W15 depth: scenario lifecycle, risk-score lifecycle
CREATE SCHEMA IF NOT EXISTS climate;

ALTER TABLE climate.scenarios
    ADD COLUMN IF NOT EXISTS activated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE climate.risk_scores
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS assessed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS dismissed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every score follows the new lifecycle
UPDATE climate.risk_scores SET status = 'draft' WHERE status = 'open';
UPDATE climate.risk_scores SET updated_at = created_at WHERE updated_at IS NULL;

-- Active scenario lookups
CREATE INDEX IF NOT EXISTS climate_scenarios_active_idx
    ON climate.scenarios (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active score lookups per scenario
CREATE INDEX IF NOT EXISTS climate_scores_active_idx
    ON climate.risk_scores (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
