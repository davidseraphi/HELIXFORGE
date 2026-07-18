# Next action

## Latest: HELIXFLOW-DURABILITY

**Goal:** prove the Foundation Integrity durability gate on HelixFlow —
fourth product through the gate (after helix-collab, helix-capital,
helix-commerce).

- Repo: `crates/helix-db/src/flow.rs` (`update_run` terminal-state guard)
- Tests: `projects/helix-flow/backend/src/main.rs`
  (`finished_runs_are_immutable`)
- Proof: `scripts/helix_flow_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `flow-durability` job
- Docs: `docs/goals/HELIXFLOW_DURABILITY.md`, `DECISION_LOG.md`

### Scope

- fix: terminal runs are immutable (`update_run` guards on
  `finished_at IS NULL`)
- concurrency proof: after finish, N concurrent update attempts all rejected
- crash proof: acknowledged run survives a forced kill of the API
- restore proof: `flow` schema dump roundtrip with equal counts + hashes

### Active goal

`HELIXFLOW-DURABILITY` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXFLOW-DURABILITY is the
active goal. Guard FlowRepo::update_run with finished_at IS NULL; add the
finished_runs_are_immutable integration test; create
scripts/helix_flow_durability.ps1 (forced-kill + restore proofs) and the
flow-durability CI job; prove it green on CI; record helix-flow in
durability_gate_proven_products.
```
