# Next action

## Latest: HELIXNOVALABS-FULL closed and CI-proven

HELIXNOVALABS-FULL is complete. The implementation passed local verification
and GitHub Actions run `29658744542` is all green, including the new
**HelixNova Labs smoke** job.

- Migration: `crates/helix-db/migrations/0055_nova_depth.sql`
- Repo: `crates/helix-db/src/nova.rs`
- API: `projects/helix-nova-labs/backend/src/main.rs`
- Smoke: `scripts/helix_nova_labs_smoke.ps1`
- CI: `.github/workflows/ci.yml` `nova-labs-smoke` job
- Docs: `docs/goals/HELIXNOVALABS_FULL.md`, `DECISION_LOG.md`

### What was delivered

Experiment + finding lifecycle depth:
- experiment update, start, conclude (rejected while draft findings remain),
  reopen, soft-delete, restore
- finding update, confirm, reject, soft-delete, restore
- nova summary report
- domain status with `phase: wave2_w20` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXNOVALABS-FULL is closed.

### Next action

Founder selects the next explicit named goal. Products 1–20 are now all at
second-wave depth; HelixPulse (21) remains scaffold-only and deferred until
the founder activates it.
