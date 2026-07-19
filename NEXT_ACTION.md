# Next action

## Latest: HELIXCODE-NUMBERING + HELIXCODE-REFS-CAS closed — code hardening complete

Both HelixCode follow-up packets are complete. The joint implementation
passed local verification and GitHub Actions run `29688633026` is all
green, including the **HelixCode durability gate** job.

- Counters: `crates/helix-db/migrations/0057_code_counters.sql`
  (backfilled), `crates/helix-db/src/code_endstate.rs`
  (`allocate_number` — one row-locked upsert per allocation)
- CAS: `crates/helix-db/src/code.rs` (`cas_ref` must-match /
  must-not-exist), `projects/helix-code/backend/src/domain/git_store.rs`
  (`run_git_push` — push rejections as clean conflicts)
- Tests: `projects/helix-code/backend/src/main.rs`
  (`concurrent_issue_numbers_all_distinct`,
  `cas_ref_stale_expected_conflict`) and `git_store.rs`
  (`concurrent_commit_same_branch_cas_holds`)
- Docs: `docs/goals/HELIXCODE_NUMBERING.md`,
  `docs/goals/HELIXCODE_REFS_CAS.md`, `DECISION_LOG.md`

### What was delivered

- issue/PR numbers and agent-event seqs allocate atomically — no MAX+1
  window, no unique-violation 500s, live data backfilled
- concurrent branch writers now lose with a retryable 409 instead of a
  raw 500, at both the git layer and the Postgres mirror
- 16-way allocation races all-distinct; commit races leave exactly
  seed + winners in history

### Active goal

None. All 21 products hold the durability gate; HelixCode follow-ups
closed.

### Next action

Founder selects the next explicit named goal.
