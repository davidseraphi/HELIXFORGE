# Next action

## Latest: HELIXVITAPRIME-FULL closed and CI-proven

HELIXVITAPRIME-FULL is complete. The implementation passed local verification
and GitHub Actions run `29655268193` is all green, including the new
**HelixVita Prime smoke** job.

- Migration: `crates/helix-db/migrations/0053_vita_depth.sql`
- Repo: `crates/helix-db/src/vita.rs`
- API: `projects/helix-vita-prime/backend/src/main.rs`
- Smoke: `scripts/helix_vita_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `vita-prime-smoke` job
- Docs: `docs/goals/HELIXVITAPRIME_FULL.md`, `DECISION_LOG.md`

### What was delivered

Study + cohort lifecycle depth:
- study update, recruit, complete (rejected while draft cohorts remain),
  terminate, soft-delete, restore
- cohort update, enroll, withdraw, soft-delete, restore
- vita summary report
- domain status with `phase: wave2_w18` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXVITAPRIME-FULL is closed.

### Next action

Founder selects the next explicit named goal.
