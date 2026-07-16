# 002 — HelixInsights durable datasets & metrics

### ADDED Requirements

#### Requirement
WHEN Postgres is available the HelixInsights API SHALL persist datasets and
metrics via `helix_db::InsightsRepo` after migration `0004_insights.sql`.

##### Scenario
- GIVEN docker Postgres is healthy and `helix_insights_api` is running
- WHEN `POST /v1/datasets` creates a named dataset
- THEN the row exists in `insights.datasets` and audit logs `dataset.create`

#### Requirement
WHEN a metric is defined on a dataset the system SHALL reject metrics for
unknown datasets and allow recording finite metric points.

##### Scenario
- GIVEN a dataset id for the caller's tenant
- WHEN `POST /v1/datasets/{id}/metrics` succeeds
- THEN `POST /v1/metrics/{metric_id}/points` stores a point listed by GET

#### Requirement
WHEN Postgres is unavailable list endpoints SHALL report `durable: false`
and mutating endpoints SHALL return unavailable.

##### Scenario
- GIVEN no Postgres pool on AppState
- WHEN `GET /v1/datasets`
- THEN response includes `"durable": false` and empty items
