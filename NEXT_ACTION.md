# Next action

## Active: HELIXORBITPRIME-DURABILITY — fifteenth product through the gate

Prove the Foundation Integrity durability gate on HelixOrbit Prime:
fresh crash, concurrency, and restore, verified locally and in CI.
Fifteenth product (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`).

Goal doc: `docs/goals/HELIXORBITPRIME_DURABILITY.md`.

### Scope

`create_child` checked the parent asset in a separate SELECT before the
pass INSERT; `decommission_asset` counted open passes and checked active
status in separate statements from the UPDATE; commission/recommission
and pass plan/complete/cancel carry no expected-from status guard. This
packet folds the guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   asset — one statement.
2. `decommission_asset` is a single guarded `UPDATE` (active + not
   deleted + `NOT EXISTS` draft or planned pass).
3. `commission_asset`, `recommission_asset`, `transition_pass` carry
   expected-from status in the `WHERE`.
4. Ignored tests `passes_rejected_on_deleted_asset` and
   `concurrent_decommission_single_winner` pass locally and in CI.
5. `scripts/helix_orbit_prime_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
6. `orbit-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
