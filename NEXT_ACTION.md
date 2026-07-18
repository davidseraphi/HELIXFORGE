# Next action

## Latest: HELIXGRIDPRIME-FULL

**Goal:** move HelixGrid Prime from thin durable scaffold to full second-wave
depth.

- Migration: `crates/helix-db/migrations/0054_grid_depth.sql`
- Repo: `crates/helix-db/src/grid.rs`
- API: `projects/helix-grid-prime/backend/src/main.rs`
- Smoke: `scripts/helix_grid_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `grid-prime-smoke` job
- Docs: `docs/goals/HELIXGRIDPRIME_FULL.md`, `DECISION_LOG.md`

### Scope

Site + reading lifecycle depth:
- site update, energize, offline (rejected while draft readings remain),
  online, soft-delete, restore
- reading update, verify, reject, soft-delete, restore
- grid summary report
- domain status with `phase: wave2_w19` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXGRIDPRIME-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXGRIDPRIME-FULL is the
active goal. Implement migration 0054, extend GridRepo with site/reading
lifecycle and grid summary; add routes and domain status planes, write unit +
integration tests, create scripts/helix_grid_prime_smoke.ps1, add the
grid-prime-smoke CI job, and prove it green on CI.
```
