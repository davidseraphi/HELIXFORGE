-- HelixSynthBio measurements (S3): instrument observations with method,
-- unit, and uncertainty, attached to samples and optionally pinned to a
-- design version. Accept/reject is a guarded single-winner transition.

CREATE TABLE IF NOT EXISTS synthbio.measurements (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    accession TEXT NOT NULL,
    sample_id UUID NOT NULL REFERENCES synthbio.samples(id),
    design_version_id UUID REFERENCES synthbio.design_versions(id),
    kind TEXT NOT NULL DEFAULT 'other',
    method TEXT NOT NULL DEFAULT '',
    value DOUBLE PRECISION,
    unit TEXT NOT NULL DEFAULT '',
    uncertainty DOUBLE PRECISION,
    raw JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'draft',
    analyst TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (tenant_id, accession)
);
CREATE INDEX IF NOT EXISTS synthbio_measurements_sample_idx
    ON synthbio.measurements (sample_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS synthbio_measurements_tenant_idx
    ON synthbio.measurements (tenant_id, status);
