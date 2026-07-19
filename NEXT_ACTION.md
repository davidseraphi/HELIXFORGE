# Next action

## Active: HELIXCLIMATEPRIME-DURABILITY — fourteenth product through the gate

Prove the Foundation Integrity durability gate on HelixClimate Prime:
fresh crash, concurrency, and restore, verified locally and in CI.
Fourteenth product (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`).

Goal doc: `docs/goals/HELIXCLIMATEPRIME_DURABILITY.md`.

### Scope

`create_child` checked the parent scenario in a separate SELECT before
the score INSERT; `archive_scenario` counted draft scores and checked
active status in separate statements from the UPDATE; activate/reopen
and assess/dismiss carry no expected-from status guard. This packet
folds the guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   scenario — one statement.
2. `archive_scenario` is a single guarded `UPDATE` (active + not deleted
   + `NOT EXISTS` draft score).
3. `activate_scenario`, `reopen_scenario`, `assess_score`,
   `dismiss_score` carry expected-from status in the `WHERE`.
4. Ignored tests `scores_rejected_on_deleted_scenario` and
   `concurrent_archive_single_winner` pass locally and in CI.
5. `scripts/helix_climate_prime_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
6. `climate-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
