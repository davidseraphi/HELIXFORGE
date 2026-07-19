# Next action

## Active: HELIXQUANTUMFORGE-DURABILITY — sixteenth product through the gate

Prove the Foundation Integrity durability gate on HelixQuantum Forge:
fresh crash, concurrency, and restore, verified locally and in CI.
Sixteenth product (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`).

Goal doc: `docs/goals/HELIXQUANTUMFORGE_DURABILITY.md`.

### Scope

`create_child` checked the parent job in a separate SELECT before the
circuit INSERT; `submit_job` counted circuits and checked draft status
in separate statements from the UPDATE; complete/fail and
validate/archive carry no expected-from status guard. This packet folds
the guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   job — one statement.
2. `submit_job` is a single guarded `UPDATE` (draft + not deleted +
   `EXISTS` circuit).
3. `transition_job`, `validate_circuit`, `archive_circuit` carry
   expected-from status in the `WHERE`.
4. Ignored tests `circuits_rejected_on_deleted_job` and
   `concurrent_submit_single_winner` pass locally and in CI.
5. `scripts/helix_quantum_forge_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
6. `quantum-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
