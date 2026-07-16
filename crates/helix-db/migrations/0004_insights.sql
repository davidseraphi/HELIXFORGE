-- HelixInsights durable datasets + metrics (reuse helix_core tenancy)
CREATE SCHEMA IF NOT EXISTS insights;

CREATE TABLE IF NOT EXISTS insights.datasets (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    source_type TEXT NOT NULL DEFAULT 'manual',
    schema_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS insights_datasets_tenant_idx ON insights.datasets (tenant_id);

CREATE TABLE IF NOT EXISTS insights.metrics (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    dataset_id UUID NOT NULL REFERENCES insights.datasets(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    unit TEXT NOT NULL DEFAULT 'count',
    aggregation TEXT NOT NULL DEFAULT 'sum',
    expression TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, name)
);

CREATE INDEX IF NOT EXISTS insights_metrics_tenant_idx ON insights.metrics (tenant_id);
CREATE INDEX IF NOT EXISTS insights_metrics_dataset_idx ON insights.metrics (dataset_id);

CREATE TABLE IF NOT EXISTS insights.metric_points (
    id UUID PRIMARY KEY,
    metric_id UUID NOT NULL REFERENCES insights.metrics(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    dimensions JSONB NOT NULL DEFAULT '{}'::jsonb,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS insights_metric_points_metric_idx
    ON insights.metric_points (metric_id, recorded_at DESC);
CREATE INDEX IF NOT EXISTS insights_metric_points_tenant_idx
    ON insights.metric_points (tenant_id);
