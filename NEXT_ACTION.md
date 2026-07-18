# Next action

## Latest: HELIXNETWORK-FULL closed and CI-proven

HELIXNETWORK-FULL is complete. The implementation passed local verification
and GitHub Actions run `29642796843` is all green, including the new
**HelixNetwork smoke** job.

- Migration: `crates/helix-db/migrations/0044_network_depth.sql`
- Repo: `crates/helix-db/src/network.rs`
- API: `projects/helix-network/backend/src/main.rs`
- Smoke: `scripts/helix_network_smoke.ps1`
- CI: `.github/workflows/ci.yml` `network-smoke` job
- Docs: `docs/goals/HELIXNETWORK_FULL.md`, `DECISION_LOG.md`

### What was delivered

Profile + connection + opportunity lifecycle depth:
- profile update, deactivate, reactivate, soft-delete, restore (owner-scoped)
- connection decline, remove, block; declined/removed re-request revival;
  blocked pairs rejected in both directions
- opportunity update, close, reopen, soft-delete, restore (owner-scoped)
- network summary report
- domain status with `phase: wave2_w9` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXNETWORK-FULL is closed.

### Next action

Founder selects the next explicit named goal.
