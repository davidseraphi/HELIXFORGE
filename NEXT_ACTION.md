# Next action

## Latest: HELIXORBITPRIME-FULL

**Goal:** move HelixOrbit Prime from thin durable scaffold to full second-wave
depth.

- Migration: `crates/helix-db/migrations/0051_orbit_depth.sql`
- Repo: `crates/helix-db/src/orbit.rs`
- API: `projects/helix-orbit-prime/backend/src/main.rs`
- Smoke: `scripts/helix_orbit_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `orbit-prime-smoke` job
- Docs: `docs/goals/HELIXORBITPRIME_FULL.md`, `DECISION_LOG.md`

### Scope

Asset + pass lifecycle depth:
- asset update, commission, decommission (rejected while draft or planned
  passes remain), recommission, soft-delete, restore
- pass update, plan, complete, cancel, soft-delete, restore
- orbit summary report
- domain status with `phase: wave2_w16` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXORBITPRIME-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXORBITPRIME-FULL is the
active goal. Implement migration 0051, extend OrbitRepo with asset/pass
lifecycle and orbit summary; add routes and domain status planes, write unit +
integration tests, create scripts/helix_orbit_prime_smoke.ps1, add the
orbit-prime-smoke CI job, and prove it green on CI.
```
