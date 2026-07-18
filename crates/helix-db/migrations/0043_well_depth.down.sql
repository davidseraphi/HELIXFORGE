DROP INDEX IF EXISTS well_checkin_edits_tenant_idx;
DROP INDEX IF EXISTS well_checkin_edits_checkin_idx;
DROP TABLE IF EXISTS well.checkin_edits;
DROP INDEX IF EXISTS well_checkins_active_idx;
DROP INDEX IF EXISTS well_habits_active_idx;
ALTER TABLE well.checkins
    DROP COLUMN IF EXISTS edit_version,
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS updated_at;
UPDATE well.checkins SET mood = 1 WHERE mood IS NULL;
UPDATE well.checkins SET energy = 1 WHERE energy IS NULL;
ALTER TABLE well.checkins
    ALTER COLUMN mood SET NOT NULL,
    ALTER COLUMN energy SET NOT NULL;
ALTER TABLE well.habits
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS ended_at,
    DROP COLUMN IF EXISTS paused_at;
