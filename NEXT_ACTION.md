# Next action

## Latest: HELIXINSIGHTS-DURABILITY

**Goal:** prove the Foundation Integrity durability gate on HelixInsights —
fifth product through the gate (after helix-collab, helix-capital,
helix-commerce, helix-flow).

- Repo: `crates/helix-db/src/insights.rs` (atomic INSERT...SELECT for
  `create_metric` and `record_point`)
- Tests: `projects/helix-insights/backend/src/main.rs`
  (`points_rejected_on_deleted_metric`, `concurrent_records_all_landed`)
- Proof: `scripts/helix_insights_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `insights-durability` job
- Docs: `docs/goals/HELIXINSIGHTS_DURABILITY.md`, `DECISION_LOG.md`

### Scope

- fix: metric/dataset existence enforced in the INSERT itself (no
  check-then-insert window)
- concurrency proof: records on a deleted metric all rejected; concurrent
  records on a live metric all land
- crash proof: acknowledged point survives a forced kill of the API
- restore proof: `insights` schema dump roundtrip with equal counts + hashes

### Active goal

`HELIXINSIGHTS-DURABILITY` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXINSIGHTS-DURABILITY is
the active goal. Make create_metric and record_point atomic INSERT...SELECT
statements; add points_rejected_on_deleted_metric and
concurrent_records_all_landed integration tests; create
scripts/helix_insights_durability.ps1 (forced-kill + restore proofs) and the
insights-durability CI job; prove it green on CI; record helix-insights in
durability_gate_proven_products.
```
