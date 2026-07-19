# Next action

## Latest: HELIXINSIGHTS-DURABILITY closed — fifth product through the gate

HELIXINSIGHTS-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29666090622` is all green, including
the new **HelixInsights durability gate** job.

- Repo: `crates/helix-db/src/insights.rs` (atomic INSERT...SELECT for
  `create_metric` and `record_point`)
- Tests: `projects/helix-insights/backend/src/main.rs`
  (`points_rejected_on_deleted_metric`, `concurrent_records_all_landed`)
- Proof: `scripts/helix_insights_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `insights-durability` job
- Docs: `docs/goals/HELIXINSIGHTS_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- check-then-insert windows closed for metric/point creation
- concurrency proof: records on a deleted metric all rejected; concurrent
  records on a live metric all land
- crash proof: acknowledged point survives a forced kill of the API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-insights` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXINSIGHTS-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 16 products.
