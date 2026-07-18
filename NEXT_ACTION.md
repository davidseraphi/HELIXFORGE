# Next action

## Latest: HELIXLEXPRIME-FULL

**Goal:** move HelixLex Prime from thin durable scaffold to full second-wave
depth.

- Migration: `crates/helix-db/migrations/0047_lex_depth.sql`
- Repo: `crates/helix-db/src/lex.rs`
- API: `projects/helix-lex-prime/backend/src/main.rs`
- Smoke: `scripts/helix_lex_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `lex-prime-smoke` job
- Docs: `docs/goals/HELIXLEXPRIME_FULL.md`, `DECISION_LOG.md`

### Scope

Matter + filing lifecycle depth:
- matter update, open, close (rejected while draft filings remain), reopen,
  soft-delete, restore
- filing update, file, withdraw, soft-delete, restore
- lex summary report
- domain status with `phase: wave2_w12` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

`HELIXLEXPRIME-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXLEXPRIME-FULL is the
active goal. Implement migration 0047, extend LexRepo with matter/filing
lifecycle and lex summary; add routes and domain status planes, write unit +
integration tests, create scripts/helix_lex_prime_smoke.ps1, add the
lex-prime-smoke CI job, and prove it green on CI.
```
