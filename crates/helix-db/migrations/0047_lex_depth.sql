-- HelixLex Prime W12 depth: matter lifecycle, filing lifecycle
CREATE SCHEMA IF NOT EXISTS lex;

ALTER TABLE lex.matters
    ADD COLUMN IF NOT EXISTS opened_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS closed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE lex.filings
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS filed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS withdrawn_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Normalize the scaffold status so every filing follows the new lifecycle
UPDATE lex.filings SET status = 'draft' WHERE status = 'open';
UPDATE lex.filings SET updated_at = created_at WHERE updated_at IS NULL;

-- Active matter lookups
CREATE INDEX IF NOT EXISTS lex_matters_active_idx
    ON lex.matters (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Active filing lookups per matter
CREATE INDEX IF NOT EXISTS lex_filings_active_idx
    ON lex.filings (tenant_id, parent_id, status)
    WHERE deleted_at IS NULL;
