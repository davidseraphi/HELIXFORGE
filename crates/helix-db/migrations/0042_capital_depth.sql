-- HelixCapital W7 depth: account lifecycle, journal void, trial-balance support.
CREATE SCHEMA IF NOT EXISTS capital;

ALTER TABLE capital.accounts
    ADD COLUMN IF NOT EXISTS closed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE capital.journals
    ADD COLUMN IF NOT EXISTS voided_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS void_reason TEXT NOT NULL DEFAULT '';

ALTER TABLE capital.journal_lines
    ADD COLUMN IF NOT EXISTS is_reversal BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS capital_accounts_active_idx
    ON capital.accounts (tenant_id, status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS capital_accounts_status_idx
    ON capital.accounts (tenant_id, status);
CREATE INDEX IF NOT EXISTS capital_accounts_kind_idx
    ON capital.accounts (tenant_id, kind, currency);
CREATE INDEX IF NOT EXISTS capital_journals_status_idx
    ON capital.journals (tenant_id, status);

CREATE TABLE IF NOT EXISTS capital.account_balance_history (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    account_id UUID NOT NULL REFERENCES capital.accounts(id) ON DELETE CASCADE,
    balance_cents BIGINT NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS capital_balance_history_account_idx
    ON capital.account_balance_history (account_id, captured_at DESC);
CREATE INDEX IF NOT EXISTS capital_balance_history_tenant_idx
    ON capital.account_balance_history (tenant_id, captured_at DESC);
