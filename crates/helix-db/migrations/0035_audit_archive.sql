-- Archive checkpoint table for WORM audit sink lag tracking.

CREATE TABLE IF NOT EXISTS audit.archive_checkpoints (
    id BIGSERIAL PRIMARY KEY,
    archived_up_to_seq BIGINT NOT NULL,
    archive_path TEXT NOT NULL,
    entry_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS archive_checkpoints_seq_idx
    ON audit.archive_checkpoints (archived_up_to_seq);
