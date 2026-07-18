# Next action

## Latest: HELIXCLIMATEPRIME-FULL

**Goal:** move HelixClimate Prime from thin durable scaffold to full
second-wave depth.

- Migration: `crates/helix-db/migrations/0050_climate_depth.sql`
- Repo: `crates/helix-db/src/climate.rs`
- API: `projects/helix-climate-prime/backend/src/main.rs`
- Smoke: `scripts/helix_climate_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `climate-prime-smoke` job
- Docs: `docs/goals/HELIXCLIMATEPRIME_FULL.md`, `DECISION_LOG.md`

### Scope

Scenario + risk-score lifecycle depth:
- scenario update, activate, archive (rejected while draft scores remain),
  reopen, soft-delete, restore
- score update, assess, dismiss, soft-delete, restore
- climate summary report
- domain status with `phase: wave2_w15` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXCLIMATEPRIME-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCLIMATEPRIME-FULL is the
active goal. Implement migration 0050, extend ClimateRepo with scenario/score
lifecycle and climate summary; add routes and domain status planes, write
unit + integration tests, create scripts/helix_climate_prime_smoke.ps1, add
the climate-prime-smoke CI job, and prove it green on CI.
```
