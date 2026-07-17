DROP TABLE IF EXISTS capital.account_balance_history;
ALTER TABLE capital.journal_lines DROP COLUMN IF EXISTS is_reversal;
DROP INDEX IF EXISTS capital_journals_status_idx;
DROP INDEX IF EXISTS capital_accounts_kind_idx;
DROP INDEX IF EXISTS capital_accounts_status_idx;
DROP INDEX IF EXISTS capital_accounts_active_idx;
ALTER TABLE capital.journals
    DROP COLUMN IF EXISTS voided_at,
    DROP COLUMN IF EXISTS void_reason;
ALTER TABLE capital.accounts
    DROP COLUMN IF EXISTS closed_at,
    DROP COLUMN IF EXISTS deleted_at;
