DROP INDEX IF EXISTS cura_notes_active_idx;
DROP INDEX IF EXISTS cura_cases_active_idx;
ALTER TABLE cura.notes
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS voided_at,
    DROP COLUMN IF EXISTS signed_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE cura.care_cases
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS discharged_at,
    DROP COLUMN IF EXISTS activated_at;
