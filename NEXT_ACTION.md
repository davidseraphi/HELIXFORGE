# Next action

## Latest: HELIXWELL-FULL

**Goal:** move HelixWell from durable scaffold to full second-wave depth.

- Migration: `crates/helix-db/migrations/0043_well_depth.sql`
- Repo: `crates/helix-db/src/well.rs`
- API: `projects/helix-well/backend/src/main.rs`
- Smoke: `scripts/helix_well_smoke.ps1`
- CI: `.github/workflows/ci.yml` `well-smoke` job
- Docs: `docs/goals/HELIXWELL_FULL.md`, `DECISION_LOG.md`

### Scope

Habit + check-in lifecycle depth:
- habit update, pause, resume, end, soft-delete, restore
- optional check-in fields (missing is not zero) + edit history
- habit summary report
- domain status with `phase: wave2_w8` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXWELL-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXWELL-FULL is the active
goal. Implement migration 0043, extend WellRepo with habit lifecycle, optional
check-in fields with edit history, and habit summary; add routes and domain
status planes, write unit + integration tests, create
scripts/helix_well_smoke.ps1, add the well-smoke CI job, and prove it green
on CI.
```
