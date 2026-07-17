# Next action

## Latest: HELIXINSIGHTS-FULL (CI-proven)

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
- CI run `29597119407` all green, including the new `HelixInsights smoke` job.

### Active goal

`HELIXINSIGHTS-FULL` is complete. Awaiting the next explicit product/program goal.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXINSIGHTS-FULL is
CI-proven (run 29597119407). The wave-2 depth packet added soft delete,
in-process aggregates, filtered point queries, a PowerShell smoke script,
and a CI job. HELIXCORE-FULL, HELIXCOLLAB-FULL, and HELIXINSIGHTS-FULL are
all green.
Next step is to set the next explicit product/program goal.
```
