# Next action

## Latest: HELIXPULSE-FULL

**Goal:** move HelixPulse from scaffold-only to full second-wave depth.
Deferral precondition met: products 1–20 are all at second-wave depth and
CI-proven.

- Migration: `crates/helix-db/migrations/0056_pulse_depth.sql`
- Repo: `crates/helix-db/src/pulse.rs` (new)
- API: `projects/helix-pulse/backend/src/main.rs`
- Smoke: `scripts/helix_pulse_smoke.ps1`
- CI: `.github/workflows/ci.yml` `pulse-smoke` job
- Docs: `docs/goals/HELIXPULSE_FULL.md`, `DECISION_LOG.md`

### Scope

First durable domain slice (cluster engine stays deferred):
- monitor create, update, activate, pause (rejected while open incidents
  remain), resume, soft-delete, restore
- incident create, update, acknowledge, resolve, soft-delete, restore
- pulse summary report
- domain status with `phase: wave2_w21` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXPULSE-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXPULSE-FULL is the active
goal. Implement migration 0056 creating the pulse schema, create PulseRepo
with monitor/incident lifecycle and pulse summary; add routes and domain
status planes, write unit + integration tests, create
scripts/helix_pulse_smoke.ps1, add the pulse-smoke CI job, and prove it
green on CI.
```
