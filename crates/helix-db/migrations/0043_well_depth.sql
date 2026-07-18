-- HelixWell W8 depth: habit lifecycle, optional check-in fields, edit history
CREATE SCHEMA IF NOT EXISTS well;

ALTER TABLE well.habits
    ADD COLUMN IF NOT EXISTS paused_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS ended_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- A skipped check-in field is missing, never zero: mood/energy may be NULL.
ALTER TABLE well.checkins
    ALTER COLUMN mood DROP NOT NULL,
    ALTER COLUMN energy DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS edit_version INT NOT NULL DEFAULT 0;

-- Active habit lookups for listing and logging
CREATE INDEX IF NOT EXISTS well_habits_active_idx
    ON well.habits (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active check-in lookups per user
CREATE INDEX IF NOT EXISTS well_checkins_active_idx
    ON well.checkins (tenant_id, user_id, recorded_at DESC)
    WHERE deleted_at IS NULL;

-- Append-only edit history: one row per edit holding the pre-edit snapshot
CREATE TABLE IF NOT EXISTS well.checkin_edits (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    checkin_id UUID NOT NULL REFERENCES well.checkins(id) ON DELETE CASCADE,
    mood INT,
    energy INT,
    notes TEXT NOT NULL DEFAULT '',
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    edited_by UUID NOT NULL,
    edited_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS well_checkin_edits_checkin_idx
    ON well.checkin_edits (checkin_id, edited_at DESC);

CREATE INDEX IF NOT EXISTS well_checkin_edits_tenant_idx
    ON well.checkin_edits (tenant_id);
