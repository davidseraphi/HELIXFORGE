# Next action

## Active: HELIXLEXPRIME-DURABILITY — eleventh product through the gate

Prove the Foundation Integrity durability gate on HelixLex Prime: fresh
crash, concurrency, and restore, verified locally and in CI. Eleventh
product (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`,
`helix-network`, `helix-forge-studio`, `helix-synthbio`).

Goal doc: `docs/goals/HELIXLEXPRIME_DURABILITY.md`.

### Scope

`create_child` checked the parent matter in a separate SELECT before the
filing INSERT; `close_matter` counted draft filings and checked open
status in separate statements from the UPDATE; open/reopen and
file/withdraw carry no expected-from status guard. This packet folds the
guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   matter — one statement.
2. `close_matter` is a single guarded `UPDATE` (open + not deleted +
   `NOT EXISTS` draft filing).
3. `open_matter`, `reopen_matter`, `file_filing`, `withdraw_filing`
   carry expected-from status in the `WHERE`.
4. Ignored tests `filings_rejected_on_deleted_matter` and
   `concurrent_close_single_winner` pass locally and in CI.
5. `scripts/helix_lex_prime_durability.ps1` proves lifecycle, forced-kill
   survival, and schema restore roundtrip.
6. `lex-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
