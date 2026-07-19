# HELIXORBITPRIME-DURABILITY

Prove the Foundation Integrity durability gate on HelixOrbit Prime:
fresh crash, concurrency, and restore, verified locally and in CI.
Fifteenth product through the gate (after `helix-collab`,
`helix-capital`, `helix-commerce`, `helix-flow`, `helix-insights`,
`helix-edu`, `helix-well`, `helix-network`, `helix-forge-studio`,
`helix-synthbio`, `helix-lex-prime`, `helix-cura-prime`,
`helix-terra-prime`, `helix-climate-prime`).

## Scope

`create_child` checked the parent asset in a separate SELECT before the
pass INSERT — an asset soft-deleted in between silently gains a pass.
`decommission_asset` counted open (draft/planned) passes and checked
active status in separate statements from the UPDATE — a pass created,
or a concurrent decommission landing in between, breaks the
"decommissioned means no open passes" invariant. The
commission/recommission and pass plan/complete/cancel updates carry no
expected-from status in their WHERE. This packet folds the guards into
the writes and proves the gate.

## Definition of done

1. `OrbitRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the asset to exist and not be deleted — one statement.
2. `OrbitRepo::decommission_asset` is a single guarded `UPDATE`
   requiring `status = 'active'`, not deleted, and `NOT EXISTS` a
   non-deleted draft or planned pass.
3. `OrbitRepo::commission_asset`, `recommission_asset`, and the pass
   `transition_pass` carry their expected-from status in the UPDATE
   `WHERE`.
4. New ignored Postgres integration tests (run in the
   `orbit-durability` CI job):
   - `passes_rejected_on_deleted_asset` — after soft-deleting an asset,
     N concurrent pass creates are all rejected; no pass leaks in.
   - `concurrent_decommission_single_winner` — N concurrent
     decommissions of one active asset produce exactly one success; the
     rest are rejected; the asset ends decommissioned.
5. `scripts/helix_orbit_prime_durability.ps1`:
   - create asset, commission, pass, plan, complete, decommission,
     verify
   - acknowledge a decommissioned asset, force-kill the API, restart,
     and verify the asset and pass are fully present
   - `pg_dump` of the `orbit` schema restores into a scratch database
     with equal asset/pass counts and equal content hashes
6. `orbit-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29672257327` (**HelixOrbit Prime durability gate** job green)
- Proof script: `scripts/helix_orbit_prime_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on orbit writes.
- Durability gates for other products (each needs its own named packet).
