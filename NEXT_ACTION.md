# Next action

## Latest: HELIXSYNTHBIO-FULL closed and CI-proven

HELIXSYNTHBIO-FULL is complete. The implementation passed local verification
and GitHub Actions run `29644975351` is all green, including the new
**HelixSynthBio smoke** job.

- Migration: `crates/helix-db/migrations/0046_synthbio_depth.sql`
- Repo: `crates/helix-db/src/synthbio.rs`
- API: `projects/helix-synthbio/backend/src/main.rs`
- Smoke: `scripts/helix_synthbio_smoke.ps1`
- CI: `.github/workflows/ci.yml` `synthbio-smoke` job
- Docs: `docs/goals/HELIXSYNTHBIO_FULL.md`, `DECISION_LOG.md`

### What was delivered

Design + sim lifecycle depth:
- design update, submit, approve (requires a completed sim), return,
  soft-delete, restore
- sim update, start, complete, fail, soft-delete, restore
- synthbio summary report
- domain status with `phase: wave2_w11` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXSYNTHBIO-FULL is closed.

### Next action

Founder selects the next explicit named goal.
