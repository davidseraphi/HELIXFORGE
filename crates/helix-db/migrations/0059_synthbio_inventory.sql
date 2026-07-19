-- HelixSynthBio inventory (S2): accessioned samples with append-only
-- custody, DB-enforced, linked to designs and parents by lineage edges.

CREATE TABLE IF NOT EXISTS synthbio.samples (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    accession TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'other',
    design_id UUID REFERENCES synthbio.registry_designs(id),
    status TEXT NOT NULL DEFAULT 'active',
    location TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (tenant_id, accession)
);
CREATE INDEX IF NOT EXISTS synthbio_samples_tenant_idx
    ON synthbio.samples (tenant_id, status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS synthbio_samples_design_idx
    ON synthbio.samples (design_id) WHERE design_id IS NOT NULL;

-- Append-only custody ledger (immutable trigger below). The sample's
-- current location is recomputed inside the same transaction as the event
-- that moves it — custody and location can never disagree.
CREATE TABLE IF NOT EXISTS synthbio.custody_events (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    sample_id UUID NOT NULL REFERENCES synthbio.samples(id) ON DELETE CASCADE,
    event TEXT NOT NULL,
    from_location TEXT NOT NULL DEFAULT '',
    to_location TEXT NOT NULL DEFAULT '',
    actor TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS synthbio_custody_sample_idx
    ON synthbio.custody_events (sample_id, created_at);

DROP TRIGGER IF EXISTS custody_events_immutable ON synthbio.custody_events;
CREATE TRIGGER custody_events_immutable
    BEFORE UPDATE OR DELETE ON synthbio.custody_events
    FOR EACH ROW EXECUTE FUNCTION synthbio.immutable_record();
