-- HelixCapital durable accounts + double-entry journals
CREATE SCHEMA IF NOT EXISTS capital;

CREATE TABLE IF NOT EXISTS capital.accounts (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'asset',
    currency TEXT NOT NULL DEFAULT 'USD',
    balance_cents BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, code)
);

CREATE INDEX IF NOT EXISTS capital_accounts_tenant_idx ON capital.accounts (tenant_id);

CREATE TABLE IF NOT EXISTS capital.journals (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    memo TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'posted',
    currency TEXT NOT NULL DEFAULT 'USD',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    posted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS capital_journals_tenant_idx ON capital.journals (tenant_id);

CREATE TABLE IF NOT EXISTS capital.journal_lines (
    id UUID PRIMARY KEY,
    journal_id UUID NOT NULL REFERENCES capital.journals(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    account_id UUID NOT NULL REFERENCES capital.accounts(id),
    side TEXT NOT NULL CHECK (side IN ('debit', 'credit')),
    amount_cents BIGINT NOT NULL CHECK (amount_cents > 0),
    memo TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS capital_journal_lines_journal_idx ON capital.journal_lines (journal_id);
CREATE INDEX IF NOT EXISTS capital_journal_lines_account_idx ON capital.journal_lines (account_id);
CREATE INDEX IF NOT EXISTS capital_journal_lines_tenant_idx ON capital.journal_lines (tenant_id);
