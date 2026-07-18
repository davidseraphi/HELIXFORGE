# Next action

## Latest: HELIXCURAPRIME-FULL closed and CI-proven

HELIXCURAPRIME-FULL is complete. The implementation passed local verification
and GitHub Actions run `29647567869` is all green, including the new
**HelixCura Prime smoke** job.

- Migration: `crates/helix-db/migrations/0048_cura_depth.sql`
- Repo: `crates/helix-db/src/cura.rs`
- API: `projects/helix-cura-prime/backend/src/main.rs`
- Smoke: `scripts/helix_cura_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `cura-prime-smoke` job
- Docs: `docs/goals/HELIXCURAPRIME_FULL.md`, `DECISION_LOG.md`

### What was delivered

Care-case + note lifecycle depth:
- case update, activate, discharge (rejected while draft notes remain),
  reopen, soft-delete, restore
- note update (draft only), sign, void, soft-delete, restore
- cura summary report
- domain status with `phase: wave2_w13` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXCURAPRIME-FULL is closed.

### Next action

Founder selects the next explicit named goal.
