# Next action

## Latest: HELIXLEXPRIME-FULL closed and CI-proven

HELIXLEXPRIME-FULL is complete. The implementation passed local verification
and GitHub Actions run `29646308966` is all green, including the new
**HelixLex Prime smoke** job.

- Migration: `crates/helix-db/migrations/0047_lex_depth.sql`
- Repo: `crates/helix-db/src/lex.rs`
- API: `projects/helix-lex-prime/backend/src/main.rs`
- Smoke: `scripts/helix_lex_prime_smoke.ps1`
- CI: `.github/workflows/ci.yml` `lex-prime-smoke` job
- Docs: `docs/goals/HELIXLEXPRIME_FULL.md`, `DECISION_LOG.md`

### What was delivered

Matter + filing lifecycle depth:
- matter update, open, close (rejected while draft filings remain), reopen,
  soft-delete, restore
- filing update, file, withdraw, soft-delete, restore
- lex summary report
- domain status with `phase: wave2_w12` and capability planes
- in-process validation tests + ignored Postgres integration test
- PowerShell smoke and CI job

### Active goal

None. HELIXLEXPRIME-FULL is closed.

### Next action

Founder selects the next explicit named goal.
