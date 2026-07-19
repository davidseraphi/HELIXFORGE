# Next action

## Active: HELIXTERRAPRIME-DURABILITY — thirteenth product through the gate

Prove the Foundation Integrity durability gate on HelixTerra Prime:
fresh crash, concurrency, and restore, verified locally and in CI.
Thirteenth product (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`).

Goal doc: `docs/goals/HELIXTERRAPRIME_DURABILITY.md`.

### Scope

`create_child` checked the parent field in a separate SELECT before the
observation INSERT; `retire_field` counted draft observations and
checked active status in separate statements from the UPDATE;
activate/reopen and confirm/dismiss carry no expected-from status
guard. This packet folds the guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   field — one statement.
2. `retire_field` is a single guarded `UPDATE` (active + not deleted +
   `NOT EXISTS` draft observation).
3. `activate_field`, `reopen_field`, `confirm_observation`,
   `dismiss_observation` carry expected-from status in the `WHERE`.
4. Ignored tests `observations_rejected_on_deleted_field` and
   `concurrent_retire_single_winner` pass locally and in CI.
5. `scripts/helix_terra_prime_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
6. `terra-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
