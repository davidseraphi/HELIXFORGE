# Next action

## Active: HELIXCODE-NUMBERING + HELIXCODE-REFS-CAS — code hardening follow-ups

Two thorough follow-up packets on HelixCode, sequenced after
HELIXCODE-DURABILITY (closed):

- `docs/goals/HELIXCODE_NUMBERING.md` — atomic allocation counters:
  `code.number_counters` (migration `0057`, backfilled) incremented under
  a row lock in one upsert statement; issue/PR numbers and agent-event
  seqs allocate with no MAX+1 window and no unique-violation 500s.
- `docs/goals/HELIXCODE_REFS_CAS.md` — ref compare-and-swap: git push
  rejections (non-fast-forward) mapped to clean conflicts across
  commit/branch/merge paths, plus `CodeRepoStore::cas_ref` with
  must-match / must-not-exist semantics.

### Proofs

- `concurrent_issue_numbers_all_distinct` — 16 racing issues and 16
  racing event appends all distinct, zero errors.
- `cas_ref_stale_expected_conflict` — create-on-existing and stale
  expected sha conflict; current expected sha wins.
- `concurrent_commit_same_branch_cas_holds` — racing commits: losers
  are clean conflicts; branch history is exactly seed + winners.

### Next action

Commit, push, watch CI (`code-durability` job + full suite) to green,
then close both packets.
