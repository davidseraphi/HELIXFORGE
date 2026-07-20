-- HelixSynthBio journeys: intent-first Goals decomposed into typed,
-- machine-checkable pipeline stages over the registry/inventory/measurement/
-- claims entities. The check is the teacher.

CREATE TABLE IF NOT EXISTS synthbio.journeys (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    accession TEXT NOT NULL,
    title TEXT NOT NULL,
    intent TEXT NOT NULL DEFAULT '',
    pathway_key TEXT NOT NULL DEFAULT 'plant-to-topical',
    route_choice TEXT NOT NULL DEFAULT 'undecided',
    status TEXT NOT NULL DEFAULT 'active',
    current_stage INT NOT NULL DEFAULT 0,
    created_by TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (tenant_id, accession)
);
CREATE INDEX IF NOT EXISTS synthbio_journeys_tenant_idx
    ON synthbio.journeys (tenant_id, status) WHERE deleted_at IS NULL;

-- Instantiated stages per journey. A stage completes by linking its
-- artifact (target) or by its automatic check passing on refresh.
CREATE TABLE IF NOT EXISTS synthbio.journey_stages (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    journey_id UUID NOT NULL REFERENCES synthbio.journeys(id) ON DELETE CASCADE,
    stage_index INT NOT NULL,
    stage_key TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    target_kind TEXT,
    target_id UUID,
    summary TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (journey_id, stage_index)
);
