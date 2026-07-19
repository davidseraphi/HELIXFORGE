# Next action

## Active: HELIXPULSE-DURABILITY — twentieth product through the gate

Prove the Foundation Integrity durability gate on HelixPulse: fresh
crash, concurrency, and restore, verified locally and in CI. Twentieth
product (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`,
`helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`,
`helix-vita-prime`, `helix-grid-prime`, `helix-nova-labs`).

Goal doc: `docs/goals/HELIXPULSE_DURABILITY.md`.

### Scope

`create_incident` checked the parent monitor in a separate SELECT before
the incident INSERT; `pause_monitor` counted open incidents and checked
active status in separate statements from the UPDATE; activate/resume
and acknowledge/resolve carry no expected-from status guard. This
packet folds the guards into the writes and proves the gate.

### Definition of done

1. `create_incident` inserts with `INSERT ... SELECT` against a
   non-deleted monitor — one statement.
2. `pause_monitor` is a single guarded `UPDATE` (active + not deleted +
   `NOT EXISTS` open incident).
3. `activate_monitor`, `resume_monitor`, `transition_incident` carry
   expected-from status in the `WHERE`.
4. Ignored tests `incidents_rejected_on_deleted_monitor` and
   `concurrent_pause_single_winner` pass locally and in CI.
5. `scripts/helix_pulse_durability.ps1` proves lifecycle, forced-kill
   survival, and schema restore roundtrip.
6. `pulse-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
