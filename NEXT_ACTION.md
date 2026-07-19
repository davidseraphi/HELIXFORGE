# Next action

## Active: HELIXSYNTHBIO-DURABILITY — tenth product through the gate

Prove the Foundation Integrity durability gate on HelixSynthBio: fresh
crash, concurrency, and restore, verified locally and in CI. Tenth
product (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`,
`helix-network`, `helix-forge-studio`).

Goal doc: `docs/goals/HELIXSYNTHBIO_DURABILITY.md`.

### Scope

`create_child` checked the parent design in a separate SELECT before the
sim INSERT; `approve_design` counted completed sims and checked review
status in separate statements from the UPDATE; submit/return and the sim
transitions carry no expected-from status guard. This packet folds the
guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   design — one statement.
2. `approve_design` is a single guarded `UPDATE` (review + not deleted +
   `EXISTS` completed sim).
3. `submit_design`, `return_design`, `transition_sim` carry expected-from
   status in the `WHERE`.
4. Ignored tests `sims_rejected_on_deleted_design` and
   `concurrent_approve_single_winner` pass locally and in CI.
5. `scripts/helix_synthbio_durability.ps1` proves lifecycle, forced-kill
   survival, and schema restore roundtrip.
6. `synthbio-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
