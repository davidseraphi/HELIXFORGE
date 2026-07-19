# Next action

## Active: HELIXVITAPRIME-DURABILITY — seventeenth product through the gate

Prove the Foundation Integrity durability gate on HelixVita Prime:
fresh crash, concurrency, and restore, verified locally and in CI.
Seventeenth product (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`).

Goal doc: `docs/goals/HELIXVITAPRIME_DURABILITY.md`.

### Scope

`create_child` checked the parent study in a separate SELECT before the
cohort INSERT; `complete_study` counted draft cohorts and checked
recruiting status in separate statements from the UPDATE;
recruit/terminate and enroll/withdraw carry no expected-from status
guard. This packet folds the guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   study — one statement.
2. `complete_study` is a single guarded `UPDATE` (recruiting + not
   deleted + `NOT EXISTS` draft cohort).
3. `recruit_study`, `terminate_study`, `enroll_cohort`,
   `withdraw_cohort` carry expected-from status in the `WHERE`.
4. Ignored tests `cohorts_rejected_on_deleted_study` and
   `concurrent_complete_single_winner` pass locally and in CI.
5. `scripts/helix_vita_prime_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
6. `vita-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
