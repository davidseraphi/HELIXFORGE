DROP INDEX IF EXISTS studio_pages_active_idx;
DROP INDEX IF EXISTS studio_apps_active_idx;
ALTER TABLE studio.pages
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS archived_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE studio.apps
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS published_at;
