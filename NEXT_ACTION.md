# Next action

## Latest: HELIXFORGESTUDIO-FULL

**Goal:** move HelixForge Studio from thin durable scaffold to full
second-wave depth.

- Migration: `crates/helix-db/migrations/0045_studio_depth.sql`
- Repo: `crates/helix-db/src/studio.rs`
- API: `projects/helix-forge-studio/backend/src/main.rs`
- Smoke: `scripts/helix_forge_studio_smoke.ps1`
- CI: `.github/workflows/ci.yml` `forge-studio-smoke` job
- Docs: `docs/goals/HELIXFORGESTUDIO_FULL.md`, `DECISION_LOG.md`

### Scope

App + page lifecycle depth:
- app update, publish (requires a non-deleted page), unpublish, soft-delete,
  restore
- page update, archive, reopen, soft-delete, restore
- studio summary report
- domain status with `phase: wave2_w10` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXFORGESTUDIO-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXFORGESTUDIO-FULL is the
active goal. Implement migration 0045, extend StudioRepo with app/page
lifecycle and studio summary; add routes and domain status planes, write
unit + integration tests, create scripts/helix_forge_studio_smoke.ps1, add
the forge-studio-smoke CI job, and prove it green on CI.
```
