-- HelixSynthBio registry (Benchling-grade): accessioned designs, immutable
-- versions, bidirectional lineage, risk review with named authority.

-- Atomic accession allocation (same row-locked upsert as code.number_counters).
CREATE TABLE IF NOT EXISTS synthbio.accession_counters (
    tenant_id UUID NOT NULL,
    kind TEXT NOT NULL,
    next_value BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, kind)
);

CREATE TABLE IF NOT EXISTS synthbio.registry_designs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    accession TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    access_class TEXT NOT NULL DEFAULT 'internal',
    status TEXT NOT NULL DEFAULT 'active',
    current_version INT NOT NULL DEFAULT 1,
    created_by TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (tenant_id, accession)
);
CREATE INDEX IF NOT EXISTS synthbio_registry_designs_tenant_idx
    ON synthbio.registry_designs (tenant_id, status) WHERE deleted_at IS NULL;

-- Immutable design versions: every edit lands as a NEW version; history is
-- never rewritten (DB-enforced below).
CREATE TABLE IF NOT EXISTS synthbio.design_versions (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    design_id UUID NOT NULL REFERENCES synthbio.registry_designs(id) ON DELETE CASCADE,
    version INT NOT NULL,
    alphabet TEXT NOT NULL,
    topology TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_name TEXT NOT NULL DEFAULT '',
    sequence_length INT NOT NULL DEFAULT 0,
    sequence_text TEXT NOT NULL DEFAULT '',
    components JSONB NOT NULL DEFAULT '[]'::jsonb,
    content_hash TEXT NOT NULL,
    provenance TEXT NOT NULL DEFAULT 'depositor-claimed',
    notes TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (design_id, version)
);
CREATE INDEX IF NOT EXISTS synthbio_design_versions_design_idx
    ON synthbio.design_versions (design_id, version DESC);

-- Risk review: unknown is never safe; non-unknown decisions require a named
-- human reviewer. Transitions guarded in the repo.
CREATE TABLE IF NOT EXISTS synthbio.risk_cases (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    design_id UUID NOT NULL REFERENCES synthbio.registry_designs(id) ON DELETE CASCADE,
    design_version_id UUID REFERENCES synthbio.design_versions(id),
    state TEXT NOT NULL DEFAULT 'unknown',
    intended_use TEXT NOT NULL DEFAULT '',
    policy_version TEXT NOT NULL DEFAULT '',
    reasons JSONB NOT NULL DEFAULT '[]'::jsonb,
    conditions TEXT NOT NULL DEFAULT '',
    reviewer TEXT,
    decided_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS synthbio_risk_cases_queue_idx
    ON synthbio.risk_cases (tenant_id, state) WHERE state = 'unknown';

-- Append-only event ledger (DB-enforced) + bidirectional lineage edges.
CREATE TABLE IF NOT EXISTS synthbio.lineage_events (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    entity_kind TEXT NOT NULL,
    entity_id UUID NOT NULL,
    event_kind TEXT NOT NULL,
    actor TEXT NOT NULL DEFAULT '',
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    content_hash TEXT NOT NULL,
    prev_hash TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS synthbio_lineage_events_entity_idx
    ON synthbio.lineage_events (tenant_id, entity_kind, entity_id, created_at);

CREATE TABLE IF NOT EXISTS synthbio.lineage_edges (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_kind TEXT NOT NULL,
    parent_id UUID NOT NULL,
    child_kind TEXT NOT NULL,
    child_id UUID NOT NULL,
    relation TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, parent_kind, parent_id, child_kind, child_id, relation)
);
CREATE INDEX IF NOT EXISTS synthbio_lineage_edges_child_idx
    ON synthbio.lineage_edges (tenant_id, child_kind, child_id);

-- Immutability, DB-enforced (Benchling "irreversible versions" analogue).
CREATE OR REPLACE FUNCTION synthbio.immutable_record() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'immutable record: %', TG_TABLE_NAME;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS design_versions_immutable ON synthbio.design_versions;
CREATE TRIGGER design_versions_immutable
    BEFORE UPDATE OR DELETE ON synthbio.design_versions
    FOR EACH ROW EXECUTE FUNCTION synthbio.immutable_record();

DROP TRIGGER IF EXISTS lineage_events_immutable ON synthbio.lineage_events;
CREATE TRIGGER lineage_events_immutable
    BEFORE UPDATE OR DELETE ON synthbio.lineage_events
    FOR EACH ROW EXECUTE FUNCTION synthbio.immutable_record();
