-- Reverse HelixInsights W2 depth migration
DROP INDEX IF EXISTS insights_metrics_active_dataset_idx;
DROP INDEX IF EXISTS insights_metrics_active_idx;
DROP INDEX IF EXISTS insights_datasets_active_idx;
DROP INDEX IF EXISTS insights_metric_points_dimensions_idx;

ALTER TABLE insights.metrics DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE insights.datasets DROP COLUMN IF EXISTS deleted_at;
