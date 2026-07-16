-- Marketplace payment intents (local simulator provider; Stripe later)

CREATE TABLE IF NOT EXISTS helix_core.payment_intents (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    plan_id TEXT NOT NULL,
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),
    currency TEXT NOT NULL DEFAULT 'usd',
    status TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT 'local_sim',
    provider_ref TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS payment_intents_tenant_idx
    ON helix_core.payment_intents (tenant_id, created_at DESC);
