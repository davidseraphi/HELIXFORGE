# HELIXCODE-REFS-CAS

Thorough fix for ref-update compare-and-swap on HelixCode. Follow-up to
HELIXCODE-DURABILITY (closed); not a re-gate — a correctness hardening
of the ref write path from the git layer up to the Postgres mirror.

## Scope

Two layers:

1. **Git layer.** Every branch write goes through `git push` to the bare
   repo, which already enforces compare-and-swap (a push onto a moved
   branch is rejected non-fast-forward) — but the rejection surfaced as
   a raw `dependency` 500. `commit_file`, `commit_files`,
   `create_branch`, and `merge_branch` now map that rejection to a clean
   `conflict` (ref moved concurrently; retry), so the caller gets a
   retryable 409 with the correct meaning.
2. **Postgres mirror.** `CodeRepoStore` gains `cas_ref(tenant, repo,
   name, expected, new_sha, ...)` — `Some(expected)` performs a
   must-match guarded UPDATE; `None` performs a must-not-exist INSERT
   (`ON CONFLICT DO NOTHING`). A mismatch loses with a conflict instead
   of silently overwriting. `upsert_ref` remains for the
   refresh-from-git-truth paths, which are convergent by construction.

## Definition of done

1. `run_git_push` helper maps non-fast-forward / stale-ref push
   rejections to `HelixError::conflict`; all four push call sites use it.
2. `CodeRepoStore::cas_ref` as above (`crates/helix-db/src/code.rs`).
3. Tests:
   - `git_store.rs`: `concurrent_commit_same_branch_cas_holds` — N
     racing commits on one branch: at least one wins, every loser is a
     clean conflict, and the branch history is exactly the seed plus one
     commit per winner (git-layer CAS proven end to end).
   - `main.rs` Postgres harness: `cas_ref_stale_expected_conflict` —
     create-on-existing conflicts, stale expected sha conflicts, current
     expected sha wins.
4. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.
5. The `code-durability` CI job (existing) runs the new ignored tests.

## Status

- **Closed / CI-proven**
- CI run: `29688633026` (**HelixCode durability gate** job green)

## Out of scope

- Force-push semantics (no caller requests them today).
- MAX+1 allocation races (their own packet: HELIXCODE-NUMBERING).
