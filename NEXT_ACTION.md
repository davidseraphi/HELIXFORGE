# Next action

## Latest: HELIXNOVALABS-FULL

**Goal:** move HelixNova Labs from thin durable scaffold to full second-wave
depth.

- Migration: `crates/helix-db/migrations/0055_nova_depth.sql`
- Repo: `crates/helix-db/src/nova.rs`
- API: `projects/helix-nova-labs/backend/src/main.rs`
- Smoke: `scripts/helix_nova_labs_smoke.ps1`
- CI: `.github/workflows/ci.yml` `nova-labs-smoke` job
- Docs: `docs/goals/HELIXNOVALABS_FULL.md`, `DECISION_LOG.md`

### Scope

Experiment + finding lifecycle depth:
- experiment update, start, conclude (rejected while draft findings remain),
  reopen, soft-delete, restore
- finding update, confirm, reject, soft-delete, restore
- nova summary report
- domain status with `phase: wave2_w20` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXNOVALABS-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXNOVALABS-FULL is the
active goal. Implement migration 0055, extend NovaRepo with experiment/finding
lifecycle and nova summary; add routes and domain status planes, write unit +
integration tests, create scripts/helix_nova_labs_smoke.ps1, add the
nova-labs-smoke CI job, and prove it green on CI.
```
