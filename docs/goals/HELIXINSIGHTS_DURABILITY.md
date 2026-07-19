# HELIXINSIGHTS-DURABILITY

Prove the Foundation Integrity durability gate on HelixInsights: fresh
crash, concurrency, and restore, verified locally and in CI. Fifth product
through the gate (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`).

## Scope

Metric and point creation used a check-then-insert pair: the metric/dataset
existence check ran in one statement and the INSERT in another, so a metric
or dataset deleted in between would silently accept new children. This
packet makes existence enforcement atomic and proves the gate.

## Definition of done

1. `InsightsRepo::create_metric` inserts with an `INSERT ... SELECT` that
   requires the parent dataset to exist and not be deleted — one statement.
2. `InsightsRepo::record_point` inserts with an `INSERT ... SELECT` that
   requires the parent metric to exist and not be deleted — one statement.
3. New ignored Postgres integration tests (run in the `insights-durability`
   CI job):
   - `points_rejected_on_deleted_metric` — after soft-deleting a metric,
     N concurrent record attempts are all rejected and no new points land.
   - `concurrent_records_all_landed` — N concurrent point records on a live
     metric all succeed and every point is persisted.
4. `scripts/helix_insights_durability.ps1`:
   - create dataset/metric, record points, aggregate, verify
   - acknowledge a point, force-kill the API, restart, and verify the point
     is fully present
   - `pg_dump` of the `insights` schema restores into a scratch database
     with equal dataset/metric/point counts and equal content hashes
5. `insights-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
6. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29666090622` (**HelixInsights durability gate** job green; the
  capital gate job flaked on an infra port collision and passed on rerun)
- Proof script: `scripts/helix_insights_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on insights writes.
- Durability gates for other products (each needs its own named packet).
