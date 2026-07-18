# Next action

## Latest: HELIXCURAPRIME-FULL

**Goal:** move HelixCura Prime from thin durable scaffold to full second-wave
depth.

- Migration: `crates/helix-db/migrations/0048_cura_depth.sql`
- Repo: `crates/helix-db/src/cura.rs`
- API: `projects/helix-cura-prime/backend/src/main.rs`
- Smoke: `scripts/helix_cura_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `cura-prime-smoke` job
- Docs: `docs/goals/HELIXCURAPRIME_FULL.md`, `DECISION_LOG.md`

### Scope

Care-case + note lifecycle depth:
- case update, activate, discharge (rejected while draft notes remain),
  reopen, soft-delete, restore
- note update (draft only), sign, void, soft-delete, restore
- cura summary report
- domain status with `phase: wave2_w13` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXCURAPRIME-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCURAPRIME-FULL is the
active goal. Implement migration 0048, extend CuraRepo with case/note
lifecycle and cura summary; add routes and domain status planes, write unit +
integration tests, create scripts/helix_cura_prime_smoke.ps1, add the
cura-prime-smoke CI job, and prove it green on CI.
```
