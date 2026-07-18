# Next action

## Latest: HELIXQUANTUMFORGE-FULL

**Goal:** move HelixQuantum Forge from thin durable scaffold to full
second-wave depth.

- Migration: `crates/helix-db/migrations/0052_quantum_depth.sql`
- Repo: `crates/helix-db/src/quantum.rs`
- API: `projects/helix-quantum-forge/backend/src/main.rs`
- Smoke: `scripts/helix_quantum_forge_smoke.ps1`
- CI: `.github/workflows/ci.yml` `quantum-forge-smoke` job
- Docs: `docs/goals/HELIXQUANTUMFORGE_FULL.md`, `DECISION_LOG.md`

### Scope

Job + circuit lifecycle depth:
- job update, submit (requires a non-deleted circuit), complete, fail,
  soft-delete, restore
- circuit update, validate, archive, soft-delete, restore
- quantum summary report
- domain status with `phase: wave2_w17` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXQUANTUMFORGE-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXQUANTUMFORGE-FULL is the
active goal. Implement migration 0052, extend QuantumRepo with job/circuit
lifecycle and quantum summary; add routes and domain status planes, write
unit + integration tests, create scripts/helix_quantum_forge_smoke.ps1, add
the quantum-forge-smoke CI job, and prove it green on CI.
```
