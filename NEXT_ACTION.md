# Next action

## Latest: HELIXTERRAPRIME-FULL

**Goal:** move HelixTerra Prime from thin durable scaffold to full second-wave
depth.

- Migration: `crates/helix-db/migrations/0049_terra_depth.sql`
- Repo: `crates/helix-db/src/terra.rs`
- API: `projects/helix-terra-prime/backend/src/main.rs`
- Smoke: `scripts/helix_terra_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `terra-prime-smoke` job
- Docs: `docs/goals/HELIXTERRAPRIME_FULL.md`, `DECISION_LOG.md`

### Scope

Field + observation lifecycle depth:
- field update, activate, retire (rejected while draft observations remain),
  reopen, soft-delete, restore
- observation update, confirm, dismiss, soft-delete, restore
- terra summary report
- domain status with `phase: wave2_w14` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXTERRAPRIME-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXTERRAPRIME-FULL is the
active goal. Implement migration 0049, extend TerraRepo with field/observation
lifecycle and terra summary; add routes and domain status planes, write unit +
integration tests, create scripts/helix_terra_prime_smoke.ps1, add the
terra-prime-smoke CI job, and prove it green on CI.
```
