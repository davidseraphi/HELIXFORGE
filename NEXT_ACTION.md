# Next action

## Latest: HELIXCAPITAL-FULL

**Goal:** move HelixCapital from durable scaffold to full second-wave depth.

- Migration: `crates/helix-db/migrations/0042_capital_depth.sql`
- Repo: `crates/helix-db/src/capital.rs`
- API: `projects/helix-capital/backend/src/main.rs`
- Smoke: `scripts/helix_capital_smoke.ps1`
- CI: `.github/workflows/ci.yml` `capital-smoke` job
- Docs: `docs/goals/HELIXCAPITAL_FULL.md`, `DECISION_LOG.md`

### Scope

Account + journal lifecycle depth:
- account update, close, reopen, soft-delete
- journal void with balance reversal
- trial-balance report and durable balance snapshots
- domain status with `phase: wave2_w7` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXCAPITAL-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCAPITAL-FULL is the active
goal. Implement migration 0042, extend CapitalRepo with account lifecycle,
journal void, trial balance, and snapshots; add routes and domain status planes,
write unit + integration tests, create scripts/helix_capital_smoke.ps1, add the
capital-smoke CI job, and prove it green on CI.
```
