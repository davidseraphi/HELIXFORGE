# Next action

## Latest: HELIXEDU-FULL

**Goal:** move HelixEdu from durable scaffold to full second-wave depth.

- Migration: `crates/helix-db/migrations/0041_edu_depth.sql`
- Repo: `crates/helix-db/src/edu.rs`
- API: `projects/helix-edu/backend/src/main.rs`
- Smoke: `scripts/helix_edu_smoke.ps1`
- CI: `.github/workflows/ci.yml` `edu-smoke` job
- Docs: `projects/helix-edu/README.md`, `DECISION_LOG.md`,
  `docs/goals/HELIXEDU_FULL.md`

### Scope

Course + enrollment lifecycle depth:
- soft-delete and restore courses
- update course metadata
- publish / unpublish course
- enroll only into published courses
- withdraw enrollment
- progress history side table
- progress 0..=100 validation and completion transitions
- domain status planes + smoke test

### Active goal

`HELIXEDU-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXEDU-FULL is the active
goal. Implement migration 0041, extend EduRepo with soft-delete, course update,
unpublish, withdraw enrollment, and progress history; add routes and domain
status planes, write unit + integration tests, create scripts/helix_edu_smoke.ps1,
add the edu-smoke CI job, and prove it green on CI.
```
