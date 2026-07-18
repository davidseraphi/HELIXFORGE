-- HelixForge Studio W10 depth: app lifecycle, page lifecycle
CREATE SCHEMA IF NOT EXISTS studio;

ALTER TABLE studio.apps
    ADD COLUMN IF NOT EXISTS published_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE studio.pages
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Backfill page updated_at so existing rows carry a sane value
UPDATE studio.pages SET updated_at = created_at WHERE updated_at IS NULL;

-- Active app lookups
CREATE INDEX IF NOT EXISTS studio_apps_active_idx
    ON studio.apps (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active page lookups per app
CREATE INDEX IF NOT EXISTS studio_pages_active_idx
    ON studio.pages (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
