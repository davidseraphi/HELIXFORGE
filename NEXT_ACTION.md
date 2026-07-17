# Next action

## Latest: HELIXCAPITAL-FULL closed and CI-proven

HELIXCAPITAL-FULL is complete. The implementation passed local verification and
GitHub Actions run `29621350739` is all green, including the new
**HelixCapital smoke** job.

- Migration: `crates/helix-db/migrations/0042_capital_depth.sql`
- Repo: `crates/helix-db/src/capital.rs`
- API: `projects/helix-capital/backend/src/main.rs`
- Smoke: `scripts/helix_capital_smoke.ps1`
- CI: `.github/workflows/ci.yml` `capital-smoke` job
- Docs: `docs/goals/HELIXCAPITAL_FULL.md`, `DECISION_LOG.md`

### What was delivered

Account + journal lifecycle depth:
- account update, close, reopen, soft-delete
- journal void with balance reversal
- trial-balance report and durable balance snapshots
- domain status with `phase: wave2_w7` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXCAPITAL-FULL is closed.

### Next action

Founder selects the next explicit named goal.
