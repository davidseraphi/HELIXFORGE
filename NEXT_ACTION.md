# Next action

## Latest: HELIXWELL-DURABILITY

**Goal:** prove the Foundation Integrity durability gate on HelixWell —
seventh product through the gate (after helix-collab, helix-capital,
helix-commerce, helix-flow, helix-insights, helix-edu).

- Repo: `crates/helix-db/src/well.rs` (atomic log INSERT...SELECT)
- Tests: `projects/helix-well/backend/src/main.rs`
  (`logs_rejected_on_paused_habit`, `concurrent_logs_all_landed`)
- Proof: `scripts/helix_well_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `well-durability` job
- Docs: `docs/goals/HELIXWELL_DURABILITY.md`, `DECISION_LOG.md`

### Scope

- fix: active-habit guard enforced inside the log INSERT
- concurrency proof: logs on a paused habit all rejected; concurrent logs
  on an active habit all land
- crash proof: acknowledged check-in survives a forced kill of the API
- restore proof: `well` schema dump roundtrip with equal counts + hashes

### Active goal

`HELIXWELL-DURABILITY` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXWELL-DURABILITY is the
active goal. Make log_habit an atomic INSERT...SELECT; add
logs_rejected_on_paused_habit and concurrent_logs_all_landed integration
tests; create scripts/helix_well_durability.ps1 (forced-kill + restore
proofs) and the well-durability CI job; prove it green on CI; record
helix-well in durability_gate_proven_products.
```
