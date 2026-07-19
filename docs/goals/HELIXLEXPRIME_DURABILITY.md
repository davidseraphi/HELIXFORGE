# HELIXLEXPRIME-DURABILITY

Prove the Foundation Integrity durability gate on HelixLex Prime: fresh
crash, concurrency, and restore, verified locally and in CI. Eleventh
product through the gate (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`).

## Scope

`create_child` checked the parent matter in a separate SELECT before the
filing INSERT — a matter soft-deleted in between silently gains a filing.
`close_matter` counted draft filings and checked open status in separate
statements from the UPDATE — a draft filing inserted, or a concurrent
close landing in between, breaks the "closed means no draft filings"
invariant. The open/reopen matter updates and the file/withdraw filing
updates carry no expected-from status in their WHERE. This packet folds
the guards into the writes and proves the gate.

## Definition of done

1. `LexRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the matter to exist and not be deleted — one statement.
2. `LexRepo::close_matter` is a single guarded `UPDATE` requiring
   `status = 'open'`, not deleted, and `NOT EXISTS` a non-deleted draft
   filing.
3. `LexRepo::open_matter`, `reopen_matter`, `file_filing`, and
   `withdraw_filing` carry their expected-from status in the UPDATE
   `WHERE`.
4. New ignored Postgres integration tests (run in the `lex-durability`
   CI job):
   - `filings_rejected_on_deleted_matter` — after soft-deleting a matter,
     N concurrent filing creates are all rejected; no filing leaks in.
   - `concurrent_close_single_winner` — N concurrent closes of one open
     matter produce exactly one success; the rest are rejected; the
     matter ends closed.
5. `scripts/helix_lex_prime_durability.ps1`:
   - create matter, open, filing, file, close, verify
   - acknowledge a closed matter, force-kill the API, restart, and verify
     the matter and filing are fully present
   - `pg_dump` of the `lex` schema restores into a scratch database with
     equal matter/filing counts and equal content hashes
6. `lex-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on lex writes.
- Durability gates for other products (each needs its own named packet).
