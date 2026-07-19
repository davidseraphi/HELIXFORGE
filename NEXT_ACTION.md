# Next action

## Active: HELIXNOVALABS-DURABILITY — nineteenth product through the gate

Prove the Foundation Integrity durability gate on HelixNova Labs: fresh
crash, concurrency, and restore, verified locally and in CI. Nineteenth
product (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`,
`helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`,
`helix-vita-prime`, `helix-grid-prime`).

Goal doc: `docs/goals/HELIXNOVALABS_DURABILITY.md`.

### Scope

`create_child` checked the parent experiment in a separate SELECT before
the finding INSERT; `conclude_experiment` counted draft findings and
checked running status in separate statements from the UPDATE;
start/reopen and confirm/reject carry no expected-from status guard.
This packet folds the guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   experiment — one statement.
2. `conclude_experiment` is a single guarded `UPDATE` (running + not
   deleted + `NOT EXISTS` draft finding).
3. `start_experiment`, `reopen_experiment`, `confirm_finding`,
   `reject_finding` carry expected-from status in the `WHERE`.
4. Ignored tests `findings_rejected_on_deleted_experiment` and
   `concurrent_conclude_single_winner` pass locally and in CI.
5. `scripts/helix_nova_labs_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
6. `nova-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
