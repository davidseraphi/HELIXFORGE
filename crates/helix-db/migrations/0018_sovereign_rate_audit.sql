-- Sovereign Core: shared rate-limit buckets + audit HMAC column.

CREATE TABLE IF NOT EXISTS helix_core.rate_buckets (
    bucket_key TEXT NOT NULL,
    window_epoch BIGINT NOT NULL,
    count INT NOT NULL DEFAULT 0,
    PRIMARY KEY (bucket_key, window_epoch)
);

-- Drop windows older than ~2 minutes (cleanup is best-effort in app).
CREATE INDEX IF NOT EXISTS rate_buckets_window_idx
    ON helix_core.rate_buckets (window_epoch);

ALTER TABLE audit.events
    ADD COLUMN IF NOT EXISTS hmac_signature TEXT NOT NULL DEFAULT '';
