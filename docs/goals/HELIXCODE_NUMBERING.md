# HELIXCODE-NUMBERING

Thorough fix for the MAX+1 allocation races on HelixCode issue/PR
numbers and agent-event seqs. Follow-up to HELIXCODE-DURABILITY (closed);
not a re-gate — a correctness hardening of one allocation pattern.

## Scope

`next_issue_number`, `next_pr_number`, and `append_agent_event` each ran
`SELECT COALESCE(MAX(n),0)+1` in one statement and the INSERT in another
— two concurrent allocators read the same MAX and the loser dies on the
UNIQUE constraint with a 500. The cheap fix (`INSERT ... SELECT MAX+1`)
still races when zero rows exist. The thorough fix is an allocation
counter table: one row per scope, incremented under a row lock in a
single `INSERT ... ON CONFLICT DO UPDATE ... RETURNING` statement, so
allocation is fully serialized with no window at all — including the
zero-row case.

## Definition of done

1. Migration `0057_code_counters.sql` creates `code.number_counters`
   (`tenant_id, scope_kind, scope_id, next_value`, PK on the triple) and
   **backfills** from existing issues, PRs, and agent events so live
   repos never re-allocate an in-use number. Down migration drops the
   table.
2. A private `CodeRepoStore::allocate_number` performs the atomic
   upsert-and-return; `next_issue_number`, `next_pr_number`, and
   `append_agent_event` are rewired through it (signatures unchanged).
3. New ignored Postgres integration test in
   `projects/helix-code/backend/src/main.rs` (runs in the existing
   `code-durability` CI job):
   - `concurrent_issue_numbers_all_distinct` — N concurrent issue
     creates on one repo all succeed with pairwise-distinct numbers;
     N concurrent agent-event appends on one job all succeed with
     pairwise-distinct seqs.
4. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29688633026` (**HelixCode durability gate** job green)
- Migration: `0057_code_counters.sql` (backfilled)

## Out of scope

- Number-gap compression (numbers may skip on rolled-back inserts —
  standard sequence semantics, acceptable).
- Ref compare-and-swap (its own packet: HELIXCODE-REFS-CAS).
