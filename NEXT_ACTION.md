# Next action

## Latest: HELIXSYNTHBIO-FULL

**Goal:** move HelixSynthBio from thin durable scaffold to full second-wave
depth.

- Migration: `crates/helix-db/migrations/0046_synthbio_depth.sql`
- Repo: `crates/helix-db/src/synthbio.rs`
- API: `projects/helix-synthbio/backend/src/main.rs`
- Smoke: `scripts/helix_synthbio_smoke.ps1`
- CI: `.github/workflows/ci.yml` `synthbio-smoke` job
- Docs: `docs/goals/HELIXSYNTHBIO_FULL.md`, `DECISION_LOG.md`

### Scope

Design + sim lifecycle depth:
- design update, submit, approve (requires a completed sim), return,
  soft-delete, restore
- sim update, start, complete, fail, soft-delete, restore
- synthbio summary report
- domain status with `phase: wave2_w11` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXSYNTHBIO-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXSYNTHBIO-FULL is the
active goal. Implement migration 0046, extend SynthbioRepo with design/sim
lifecycle and synthbio summary; add routes and domain status planes, write
unit + integration tests, create scripts/helix_synthbio_smoke.ps1, add the
synthbio-smoke CI job, and prove it green on CI.
```
