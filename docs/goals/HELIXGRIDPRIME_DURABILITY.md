# HELIXGRIDPRIME-DURABILITY

Prove the Foundation Integrity durability gate on HelixGrid Prime: fresh
crash, concurrency, and restore, verified locally and in CI. Eighteenth
product through the gate (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`,
`helix-vita-prime`).

## Scope

`create_child` checked the parent site in a separate SELECT before the
reading INSERT — a site soft-deleted in between silently gains a
reading. `take_offline` counted draft readings and checked active status
in separate statements from the UPDATE — a draft reading created, or a
concurrent offline/energize landing in between, breaks the "offline
means no draft readings" invariant. The energize/online and
verify/reject updates carry no expected-from status in their WHERE.
This packet folds the guards into the writes and proves the gate.

## Definition of done

1. `GridRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the site to exist and not be deleted — one statement.
2. `GridRepo::take_offline` is a single guarded `UPDATE` requiring
   `status = 'active'`, not deleted, and `NOT EXISTS` a non-deleted draft
   reading.
3. `GridRepo::energize_site`, `bring_online`, `verify_reading`, and
   `reject_reading` carry their expected-from status in the UPDATE
   `WHERE`.
4. New ignored Postgres integration tests (run in the `grid-durability`
   CI job):
   - `readings_rejected_on_deleted_site` — after soft-deleting a site,
     N concurrent reading creates are all rejected; no reading leaks in.
   - `concurrent_offline_single_winner` — N concurrent offlines of one
     active site produce exactly one success; the rest are rejected;
     the site ends offline.
5. `scripts/helix_grid_prime_durability.ps1`:
   - create site, energize, reading, verify, offline, verify
   - acknowledge an offline site, force-kill the API, restart, and
     verify the site and reading are fully present
   - `pg_dump` of the `grid` schema restores into a scratch database
     with equal site/reading counts and equal content hashes
6. `grid-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on grid writes.
- Durability gates for other products (each needs its own named packet).
