# HELIXCLIMATEPRIME-DURABILITY

Prove the Foundation Integrity durability gate on HelixClimate Prime:
fresh crash, concurrency, and restore, verified locally and in CI.
Fourteenth product through the gate (after `helix-collab`,
`helix-capital`, `helix-commerce`, `helix-flow`, `helix-insights`,
`helix-edu`, `helix-well`, `helix-network`, `helix-forge-studio`,
`helix-synthbio`, `helix-lex-prime`, `helix-cura-prime`,
`helix-terra-prime`).

## Scope

`create_child` checked the parent scenario in a separate SELECT before
the score INSERT — a scenario soft-deleted in between silently gains a
score. `archive_scenario` counted draft scores and checked active status
in separate statements from the UPDATE — a draft score created, or a
concurrent archive landing in between, breaks the "archived means no
draft scores" invariant. The activate/reopen and assess/dismiss updates
carry no expected-from status in their WHERE. This packet folds the
guards into the writes and proves the gate.

## Definition of done

1. `ClimateRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the scenario to exist and not be deleted — one statement.
2. `ClimateRepo::archive_scenario` is a single guarded `UPDATE`
   requiring `status = 'active'`, not deleted, and `NOT EXISTS` a
   non-deleted draft score.
3. `ClimateRepo::activate_scenario`, `reopen_scenario`, `assess_score`,
   and `dismiss_score` carry their expected-from status in the UPDATE
   `WHERE`.
4. New ignored Postgres integration tests (run in the
   `climate-durability` CI job):
   - `scores_rejected_on_deleted_scenario` — after soft-deleting a
     scenario, N concurrent score creates are all rejected; no score
     leaks in.
   - `concurrent_archive_single_winner` — N concurrent archives of one
     active scenario produce exactly one success; the rest are rejected;
     the scenario ends archived.
5. `scripts/helix_climate_prime_durability.ps1`:
   - create scenario, activate, score, assess, archive, verify
   - acknowledge an archived scenario, force-kill the API, restart, and
     verify the scenario and score are fully present
   - `pg_dump` of the `climate` schema restores into a scratch database
     with equal scenario/score counts and equal content hashes
6. `climate-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on climate writes.
- Durability gates for other products (each needs its own named packet).
