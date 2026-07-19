# Next action

## Active: HELIXGRIDPRIME-DURABILITY — eighteenth product through the gate

Prove the Foundation Integrity durability gate on HelixGrid Prime:
fresh crash, concurrency, and restore, verified locally and in CI.
Eighteenth product (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`,
`helix-vita-prime`).

Goal doc: `docs/goals/HELIXGRIDPRIME_DURABILITY.md`.

### Scope

`create_child` checked the parent site in a separate SELECT before the
reading INSERT; `take_offline` counted draft readings and checked
active status in separate statements from the UPDATE; energize/online
and verify/reject carry no expected-from status guard. This packet
folds the guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   site — one statement.
2. `take_offline` is a single guarded `UPDATE` (active + not deleted +
   `NOT EXISTS` draft reading).
3. `energize_site`, `bring_online`, `verify_reading`, `reject_reading`
   carry expected-from status in the `WHERE`.
4. Ignored tests `readings_rejected_on_deleted_site` and
   `concurrent_offline_single_winner` pass locally and in CI.
5. `scripts/helix_grid_prime_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
6. `grid-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
