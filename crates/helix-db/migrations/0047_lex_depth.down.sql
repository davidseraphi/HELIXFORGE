DROP INDEX IF EXISTS lex_filings_active_idx;
DROP INDEX IF EXISTS lex_matters_active_idx;
ALTER TABLE lex.filings
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS withdrawn_at,
    DROP COLUMN IF EXISTS filed_at,
    DROP COLUMN IF EXISTS updated_at;
ALTER TABLE lex.matters
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS closed_at,
    DROP COLUMN IF EXISTS opened_at;
