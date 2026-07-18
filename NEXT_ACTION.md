# Next action

## Latest: HELIXCLIMATEPRIME-FULL closed and CI-proven

HELIXCLIMATEPRIME-FULL is complete. The implementation passed local
verification and GitHub Actions run `29650054052` is all green, including the
new **HelixClimate Prime smoke** job.

- Migration: `crates/helix-db/migrations/0050_climate_depth.sql`
- Repo: `crates/helix-db/src/climate.rs`
- API: `projects/helix-climate-prime/backend/src/main.rs`
- Smoke: `scripts/helix_climate_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `climate-prime-smoke` job
- Docs: `docs/goals/HELIXCLIMATEPRIME_FULL.md`, `DECISION_LOG.md`

### What was delivered

Scenario + risk-score lifecycle depth:
- scenario update, activate, archive (rejected while draft scores remain),
  reopen, soft-delete, restore
- score update, assess, dismiss, soft-delete, restore
- climate summary report
- domain status with `phase: wave2_w15` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXCLIMATEPRIME-FULL is closed.

### Next action

Founder selects the next explicit named goal.
