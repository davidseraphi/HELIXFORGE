-- Meter idempotency + API key hash lookup (Kimi P1).

ALTER TABLE helix_core.meter_events
    ADD COLUMN IF NOT EXISTS idempotency_key TEXT;

-- Unique when present: same key cannot double-bill.
CREATE UNIQUE INDEX IF NOT EXISTS meter_events_idempotency_uidx
    ON helix_core.meter_events (tenant_id, product, idempotency_key)
    WHERE idempotency_key IS NOT NULL AND idempotency_key <> '';

CREATE INDEX IF NOT EXISTS service_api_keys_key_hash_idx
    ON helix_core.service_api_keys (key_hash)
    WHERE revoked_at IS NULL;
