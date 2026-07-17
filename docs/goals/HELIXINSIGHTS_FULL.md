# HELIXINSIGHTS-FULL

Move HelixInsights from **Scaffold** to full second-wave depth (catalog order 4,
port `8104`).

## Definition of done

1. Durable schema migration (`0039_insights_depth.sql`) adds soft-delete columns
   and query indexes.
2. `InsightsRepo` has methods beyond list/create stub: get, soft delete,
   tenant-wide list, filtered point list, and in-process aggregate.
3. Domain APIs:
   - `GET /v1/metrics`
   - `GET /v1/metrics/{id}`
   - `DELETE /v1/datasets/{id}`
   - `DELETE /v1/metrics/{id}`
   - `POST /v1/metrics/{id}/aggregate`
   - Filtered `GET /v1/metrics/{id}/points`
4. `GET /v1/domain/status` returns `phase: wave2_w2` and a `planes` object.
5. Audit + metering + NATS on key actions.
6. `scripts/helix_insights_smoke.ps1` passes locally and in CI.
7. Product README + DECISION_LOG entry updated.
8. `cargo test --workspace --all-features` and `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Verification

- [ ] `cargo build -p helix_insights_api`
- [ ] `cargo test -p helix_insights_api`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `scripts/helix_insights_smoke.ps1` against local Postgres
- [ ] `insights-smoke` CI job green

## Out of scope

- Decision records, alerts, reports, dashboards, forecasts, scenarios,
  federated aggregates.
- Web UI work beyond the existing scaffold.
