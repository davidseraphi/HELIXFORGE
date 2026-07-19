-- HelixSynthBio e-signatures (S5): a named human signs a target; the
-- signature is append-only and the target locks against further decisions.

ALTER TABLE synthbio.risk_cases
    ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS synthbio.signatures (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    target_kind TEXT NOT NULL,
    target_id UUID NOT NULL,
    signer TEXT NOT NULL,
    meaning TEXT NOT NULL,
    statement TEXT NOT NULL DEFAULT '',
    content_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, target_kind, target_id, meaning)
);
CREATE INDEX IF NOT EXISTS synthbio_signatures_target_idx
    ON synthbio.signatures (tenant_id, target_kind, target_id);

DROP TRIGGER IF EXISTS signatures_immutable ON synthbio.signatures;
CREATE TRIGGER signatures_immutable
    BEFORE UPDATE OR DELETE ON synthbio.signatures
    FOR EACH ROW EXECUTE FUNCTION synthbio.immutable_record();
