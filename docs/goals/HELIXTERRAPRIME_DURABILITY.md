# HELIXTERRAPRIME-DURABILITY

Prove the Foundation Integrity durability gate on HelixTerra Prime: fresh
crash, concurrency, and restore, verified locally and in CI. Thirteenth
product through the gate (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`).

## Scope

`create_child` checked the parent field in a separate SELECT before the
observation INSERT — a field soft-deleted in between silently gains an
observation. `retire_field` counted draft observations and checked active
status in separate statements from the UPDATE — a draft observation
created, or a concurrent retire landing in between, breaks the "retired
means no draft observations" invariant. The activate/reopen and
confirm/dismiss updates carry no expected-from status in their WHERE.
This packet folds the guards into the writes and proves the gate.

## Definition of done

1. `TerraRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the field to exist and not be deleted — one statement.
2. `TerraRepo::retire_field` is a single guarded `UPDATE` requiring
   `status = 'active'`, not deleted, and `NOT EXISTS` a non-deleted draft
   observation.
3. `TerraRepo::activate_field`, `reopen_field`, `confirm_observation`,
   and `dismiss_observation` carry their expected-from status in the
   UPDATE `WHERE`.
4. New ignored Postgres integration tests (run in the
   `terra-durability` CI job):
   - `observations_rejected_on_deleted_field` — after soft-deleting a
     field, N concurrent observation creates are all rejected; no
     observation leaks in.
   - `concurrent_retire_single_winner` — N concurrent retires of one
     active field produce exactly one success; the rest are rejected;
     the field ends retired.
5. `scripts/helix_terra_prime_durability.ps1`:
   - create field, activate, observation, confirm, retire, verify
   - acknowledge a retired field, force-kill the API, restart, and verify
     the field and observation are fully present
   - `pg_dump` of the `terra` schema restores into a scratch database
     with equal field/observation counts and equal content hashes
6. `terra-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on terra writes.
- Durability gates for other products (each needs its own named packet).
