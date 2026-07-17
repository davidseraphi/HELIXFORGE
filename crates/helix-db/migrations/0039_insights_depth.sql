-- HelixInsights W2 depth: soft-delete lifecycle + query support
CREATE SCHEMA IF NOT EXISTS insights;

ALTER TABLE insights.datasets
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

ALTER TABLE insights.metrics
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Speed dimension-filtered and aggregate queries
CREATE INDEX IF NOT EXISTS insights_metric_points_dimensions_idx
    ON insights.metric_points USING GIN (dimensions);

-- Partial indexes for active (non-deleted) rows
CREATE INDEX IF NOT EXISTS insights_datasets_active_idx
    ON insights.datasets (tenant_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS insights_metrics_active_idx
    ON insights.metrics (tenant_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS insights_metrics_active_dataset_idx
    ON insights.metrics (dataset_id, created_at DESC)
    WHERE deleted_at IS NULL;
