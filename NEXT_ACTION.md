# Next action

## Latest: HELIXORBITPRIME-FULL closed and CI-proven

HELIXORBITPRIME-FULL is complete. The implementation passed local
verification and GitHub Actions run `29651383990` is all green, including the
new **HelixOrbit Prime smoke** job.

- Migration: `crates/helix-db/migrations/0051_orbit_depth.sql`
- Repo: `crates/helix-db/src/orbit.rs`
- API: `projects/helix-orbit-prime/backend/src/main.rs`
- Smoke: `scripts/helix_orbit_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `orbit-prime-smoke` job
- Docs: `docs/goals/HELIXORBITPRIME_FULL.md`, `DECISION_LOG.md`

### What was delivered

Asset + pass lifecycle depth:
- asset update, commission, decommission (rejected while draft or planned
  passes remain), recommission, soft-delete, restore
- pass update, plan, complete, cancel, soft-delete, restore
- orbit summary report
- domain status with `phase: wave2_w16` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXORBITPRIME-FULL is closed.

### Next action

Founder selects the next explicit named goal.
