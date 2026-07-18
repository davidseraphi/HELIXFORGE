# Next action

## Latest: HELIXVITAPRIME-FULL

**Goal:** move HelixVita Prime from thin durable scaffold to full second-wave
depth.

- Migration: `crates/helix-db/migrations/0053_vita_depth.sql`
- Repo: `crates/helix-db/src/vita.rs`
- API: `projects/helix-vita-prime/backend/src/main.rs`
- Smoke: `scripts/helix_vita_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `vita-prime-smoke` job
- Docs: `docs/goals/HELIXVITAPRIME_FULL.md`, `DECISION_LOG.md`

### Scope

Study + cohort lifecycle depth:
- study update, recruit, complete (rejected while draft cohorts remain),
  terminate, soft-delete, restore
- cohort update, enroll, withdraw, soft-delete, restore
- vita summary report
- domain status with `phase: wave2_w18` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXVITAPRIME-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXVITAPRIME-FULL is the
active goal. Implement migration 0053, extend VitaRepo with study/cohort
lifecycle and vita summary; add routes and domain status planes, write unit +
integration tests, create scripts/helix_vita_prime_smoke.ps1, add the
vita-prime-smoke CI job, and prove it green on CI.
```
