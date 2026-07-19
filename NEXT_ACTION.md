# Next action

## Latest: HELIXVITAPRIME-DURABILITY closed — seventeenth product through the gate

HELIXVITAPRIME-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29673285395` is all green, including
the new **HelixVita Prime durability gate** job.

- Repo: `crates/helix-db/src/vita.rs` (atomic `create_child`
  INSERT...SELECT; guarded `complete_study` / `recruit_study` /
  `terminate_study` / `enroll_cohort` / `withdraw_cohort`)
- Tests: `projects/helix-vita-prime/backend/src/main.rs`
  (`cohorts_rejected_on_deleted_study`,
  `concurrent_complete_single_winner`)
- Proof: `scripts/helix_vita_prime_durability.ps1` (forced-kill +
  restore)
- CI: `.github/workflows/ci.yml` `vita-durability` job
- Docs: `docs/goals/HELIXVITAPRIME_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the cohort INSERT; a study
  soft-deleted mid-flight can no longer leak cohorts
- complete is one guarded UPDATE (recruiting + not deleted + NOT EXISTS
  draft cohort); recruit/terminate and enroll/withdraw carry
  expected-from status in the WHERE
- concurrency proof: 8 racing creates on a deleted study all rejected; 8
  racing completes → exactly one winner
- crash proof: acknowledged completed study survives a forced kill of
  the API
- restore proof: schema dump roundtrip with equal counts + content
  hashes
- `helix-vita-prime` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXVITAPRIME-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 4 products.
