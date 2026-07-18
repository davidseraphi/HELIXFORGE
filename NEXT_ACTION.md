# Next action

## Latest: HELIXNETWORK-FULL

**Goal:** move HelixNetwork from durable scaffold to full second-wave depth.

- Migration: `crates/helix-db/migrations/0044_network_depth.sql`
- Repo: `crates/helix-db/src/network.rs`
- API: `projects/helix-network/backend/src/main.rs`
- Smoke: `scripts/helix_network_smoke.ps1`
- CI: `.github/workflows/ci.yml` `network-smoke` job
- Docs: `docs/goals/HELIXNETWORK_FULL.md`, `DECISION_LOG.md`

### Scope

Profile + connection + opportunity lifecycle depth:
- profile update, deactivate, reactivate, soft-delete, restore
- connection decline, remove, block, declined/removed re-request revival
- opportunity update, close, reopen, soft-delete, restore
- network summary report
- domain status with `phase: wave2_w9` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXNETWORK-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXNETWORK-FULL is the active
goal. Implement migration 0044, extend NetworkRepo with profile/connection/
opportunity lifecycle and network summary; add routes and domain status planes,
write unit + integration tests, create scripts/helix_network_smoke.ps1, add the
network-smoke CI job, and prove it green on CI.
```
