# Next action

## Latest: HELIXCODE-DURABILITY closed — all 21 products through the gate

HELIXCODE-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29687099450` is all green, including
the new **HelixCode durability gate** job. The full product catalog is
now durability-gate-proven.

- Repo: `crates/helix-db/src/code.rs` (guarded `finish_pipeline_run` /
  `finish_agent_job` with `finished_at IS NULL`; atomic
  `create_workspace` / `create_pipeline` INSERT...SELECT)
- Boot fix: `fallback_service` replaces root `.nest_service` in
  `projects/helix-code/backend/src/main.rs`
- Tests: `projects/helix-code/backend/src/main.rs` (new Postgres harness;
  `concurrent_finish_pipeline_run_single_winner`,
  `concurrent_finish_agent_job_single_winner`,
  `children_rejected_on_missing_repo`)
- Proof: `scripts/helix_code_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `code-durability` job
- Docs: `docs/goals/HELIXCODE_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- terminal finishes are single guarded UPDATEs — a concurrent finish or
  a finish racing a cancel loses with a conflict
- workspace/pipeline creates are atomic against the tenant's repo; bad
  repo ids are clean not-founds, not FK-violation 500s
- the API boots again under current axum
- concurrency proof: 8 racing finishes → exactly one winner (both
  planes)
- crash proof: acknowledged finished run survives a forced kill of the
  API
- restore proof: schema dump roundtrip with equal counts + content
  hashes
- `helix-code` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXCODE-DURABILITY is closed; all 21 catalog products hold the
durability gate.

### Next action

Founder selects the next explicit named goal.
