# Next action

## Latest: HELIXINSIGHTS-FULL

**Goal:** move HelixInsights from scaffold to full second-wave depth.

- Migration: `crates/helix-db/migrations/0039_insights_depth.sql`
- Repo: `crates/helix-db/src/insights.rs`
- API: `projects/helix-insights/backend/src/main.rs`
- Smoke: `scripts/helix_insights_smoke.ps1`
- CI: `.github/workflows/ci.yml` `insights-smoke` job
- Docs: `projects/helix-insights/README.md`, `DECISION_LOG.md`,
  `docs/goals/HELIXINSIGHTS_FULL.md`

### Current status

- Backend routes added and smoke-tested locally.
- `cargo test --workspace --all-features` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- Local smoke PASS.
- Next: push to `main` and confirm the `insights-smoke` CI job passes.

### Active goal

`HELIXINSIGHTS-FULL` — awaiting CI proof.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXINSIGHTS-FULL is
implemented: migration 0039, extended InsightsRepo, new routes including
soft delete and in-process aggregate, smoke script, and CI job.
Local verification passed (cargo test workspace, clippy, smoke).
Next step is to commit, push, and confirm the insights-smoke CI job is green.
```
