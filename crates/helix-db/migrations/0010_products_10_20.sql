-- Thin durable domain tables for products 10–20 (widen pass)
CREATE SCHEMA IF NOT EXISTS studio;
CREATE SCHEMA IF NOT EXISTS synthbio;
CREATE SCHEMA IF NOT EXISTS lex;
CREATE SCHEMA IF NOT EXISTS cura;
CREATE SCHEMA IF NOT EXISTS terra;
CREATE SCHEMA IF NOT EXISTS climate;
CREATE SCHEMA IF NOT EXISTS orbit;
CREATE SCHEMA IF NOT EXISTS quantum;
CREATE SCHEMA IF NOT EXISTS vita;
CREATE SCHEMA IF NOT EXISTS grid;
CREATE SCHEMA IF NOT EXISTS nova;


CREATE TABLE IF NOT EXISTS studio.apps (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS studio_apps_tenant_idx ON studio.apps (tenant_id);

CREATE TABLE IF NOT EXISTS studio.pages (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES studio.apps(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS studio_pages_parent_idx ON studio.pages (parent_id);
CREATE INDEX IF NOT EXISTS studio_pages_tenant_idx ON studio.pages (tenant_id);

CREATE TABLE IF NOT EXISTS synthbio.designs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS synthbio_designs_tenant_idx ON synthbio.designs (tenant_id);

CREATE TABLE IF NOT EXISTS synthbio.sims (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES synthbio.designs(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS synthbio_sims_parent_idx ON synthbio.sims (parent_id);
CREATE INDEX IF NOT EXISTS synthbio_sims_tenant_idx ON synthbio.sims (tenant_id);

CREATE TABLE IF NOT EXISTS lex.matters (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS lex_matters_tenant_idx ON lex.matters (tenant_id);

CREATE TABLE IF NOT EXISTS lex.filings (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES lex.matters(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS lex_filings_parent_idx ON lex.filings (parent_id);
CREATE INDEX IF NOT EXISTS lex_filings_tenant_idx ON lex.filings (tenant_id);

CREATE TABLE IF NOT EXISTS cura.care_cases (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS cura_care_cases_tenant_idx ON cura.care_cases (tenant_id);

CREATE TABLE IF NOT EXISTS cura.notes (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES cura.care_cases(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS cura_notes_parent_idx ON cura.notes (parent_id);
CREATE INDEX IF NOT EXISTS cura_notes_tenant_idx ON cura.notes (tenant_id);

CREATE TABLE IF NOT EXISTS terra.fields (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS terra_fields_tenant_idx ON terra.fields (tenant_id);

CREATE TABLE IF NOT EXISTS terra.observations (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES terra.fields(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS terra_observations_parent_idx ON terra.observations (parent_id);
CREATE INDEX IF NOT EXISTS terra_observations_tenant_idx ON terra.observations (tenant_id);

CREATE TABLE IF NOT EXISTS climate.scenarios (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS climate_scenarios_tenant_idx ON climate.scenarios (tenant_id);

CREATE TABLE IF NOT EXISTS climate.risk_scores (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES climate.scenarios(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS climate_risk_scores_parent_idx ON climate.risk_scores (parent_id);
CREATE INDEX IF NOT EXISTS climate_risk_scores_tenant_idx ON climate.risk_scores (tenant_id);

CREATE TABLE IF NOT EXISTS orbit.assets (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS orbit_assets_tenant_idx ON orbit.assets (tenant_id);

CREATE TABLE IF NOT EXISTS orbit.passes (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES orbit.assets(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS orbit_passes_parent_idx ON orbit.passes (parent_id);
CREATE INDEX IF NOT EXISTS orbit_passes_tenant_idx ON orbit.passes (tenant_id);

CREATE TABLE IF NOT EXISTS quantum.jobs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS quantum_jobs_tenant_idx ON quantum.jobs (tenant_id);

CREATE TABLE IF NOT EXISTS quantum.circuits (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES quantum.jobs(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS quantum_circuits_parent_idx ON quantum.circuits (parent_id);
CREATE INDEX IF NOT EXISTS quantum_circuits_tenant_idx ON quantum.circuits (tenant_id);

CREATE TABLE IF NOT EXISTS vita.studies (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS vita_studies_tenant_idx ON vita.studies (tenant_id);

CREATE TABLE IF NOT EXISTS vita.cohorts (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES vita.studies(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS vita_cohorts_parent_idx ON vita.cohorts (parent_id);
CREATE INDEX IF NOT EXISTS vita_cohorts_tenant_idx ON vita.cohorts (tenant_id);

CREATE TABLE IF NOT EXISTS grid.sites (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS grid_sites_tenant_idx ON grid.sites (tenant_id);

CREATE TABLE IF NOT EXISTS grid.readings (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES grid.sites(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS grid_readings_parent_idx ON grid.readings (parent_id);
CREATE INDEX IF NOT EXISTS grid_readings_tenant_idx ON grid.readings (tenant_id);

CREATE TABLE IF NOT EXISTS nova.experiments (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS nova_experiments_tenant_idx ON nova.experiments (tenant_id);

CREATE TABLE IF NOT EXISTS nova.findings (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    parent_id UUID NOT NULL REFERENCES nova.experiments(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS nova_findings_parent_idx ON nova.findings (parent_id);
CREATE INDEX IF NOT EXISTS nova_findings_tenant_idx ON nova.findings (tenant_id);
