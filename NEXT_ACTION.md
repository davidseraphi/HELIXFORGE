# Next action

## Latest: HELIXTERRAPRIME-FULL closed and CI-proven

HELIXTERRAPRIME-FULL is complete. The implementation passed local
verification and GitHub Actions run `29648775239` is all green, including the
new **HelixTerra Prime smoke** job.

- Migration: `crates/helix-db/migrations/0049_terra_depth.sql`
- Repo: `crates/helix-db/src/terra.rs`
- API: `projects/helix-terra-prime/backend/src/main.rs`
- Smoke: `scripts/helix_terra_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `terra-prime-smoke` job
- Docs: `docs/goals/HELIXTERRAPRIME_FULL.md`, `DECISION_LOG.md`

### What was delivered

Field + observation lifecycle depth:
- field update, activate, retire (rejected while draft observations remain),
  reopen, soft-delete, restore
- observation update, confirm, dismiss, soft-delete, restore
- terra summary report
- domain status with `phase: wave2_w14` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXTERRAPRIME-FULL is closed.

### Next action

Founder selects the next explicit named goal.
