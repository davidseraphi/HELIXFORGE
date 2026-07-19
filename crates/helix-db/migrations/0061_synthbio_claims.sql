-- HelixSynthBio claims (S4): versioned statements linked to supporting,
-- conflicting, and missing evidence, with human attestation. Evidence links
-- and ELN notes are append-only (immutable triggers below).

CREATE TABLE IF NOT EXISTS synthbio.claims (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    accession TEXT NOT NULL,
    design_id UUID NOT NULL REFERENCES synthbio.registry_designs(id),
    statement TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    attested_by TEXT,
    attested_at TIMESTAMPTZ,
    created_by TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (tenant_id, accession)
);
CREATE INDEX IF NOT EXISTS synthbio_claims_design_idx
    ON synthbio.claims (design_id) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS synthbio.evidence_links (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    claim_id UUID NOT NULL REFERENCES synthbio.claims(id) ON DELETE CASCADE,
    target_kind TEXT NOT NULL,
    target_id UUID NOT NULL,
    support TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS synthbio_evidence_claim_idx
    ON synthbio.evidence_links (claim_id);

CREATE TABLE IF NOT EXISTS synthbio.notes (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    design_id UUID NOT NULL REFERENCES synthbio.registry_designs(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_by TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS synthbio_notes_design_idx
    ON synthbio.notes (design_id, created_at);

DROP TRIGGER IF EXISTS evidence_links_immutable ON synthbio.evidence_links;
CREATE TRIGGER evidence_links_immutable
    BEFORE UPDATE OR DELETE ON synthbio.evidence_links
    FOR EACH ROW EXECUTE FUNCTION synthbio.immutable_record();

DROP TRIGGER IF EXISTS notes_immutable ON synthbio.notes;
CREATE TRIGGER notes_immutable
    BEFORE UPDATE OR DELETE ON synthbio.notes
    FOR EACH ROW EXECUTE FUNCTION synthbio.immutable_record();
