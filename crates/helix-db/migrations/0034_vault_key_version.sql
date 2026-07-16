-- Add per-row key versioning for HVA5 envelopes and default to HVA4/HVA5 algorithm.

ALTER TABLE helix_core.secrets
    ADD COLUMN IF NOT EXISTS key_version INT NOT NULL DEFAULT 1;

CREATE INDEX IF NOT EXISTS helix_core_secrets_key_version_idx
    ON helix_core.secrets (tenant_id, name, key_version);

-- Update the rotation ledger default to reflect the new preferred envelope.
INSERT INTO helix_core.vault_key_meta (id, version, algorithm, rotated_at, note)
VALUES ('default', 1, 'HVA5', now(), 'HVA5 with embedded key version')
ON CONFLICT (id) DO UPDATE SET
    algorithm = EXCLUDED.algorithm,
    note = EXCLUDED.note;
