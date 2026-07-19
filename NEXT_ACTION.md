# Next action

## Active: HELIXCURAPRIME-DURABILITY — twelfth product through the gate

Prove the Foundation Integrity durability gate on HelixCura Prime: fresh
crash, concurrency, and restore, verified locally and in CI. Twelfth
product (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`,
`helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`).

Goal doc: `docs/goals/HELIXCURAPRIME_DURABILITY.md`.

### Scope

`create_child` checked the parent case in a separate SELECT before the
note INSERT; `discharge_case` counted draft notes and checked active
status in separate statements from the UPDATE; `update_note` carried no
draft guard, so a racing sign let an edit overwrite a signed note; the
activate/reopen and sign/void updates carry no expected-from status
guard. This packet folds the guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   case — one statement.
2. `discharge_case` is a single guarded `UPDATE` (active + not deleted +
   `NOT EXISTS` draft note).
3. `update_note` carries `status = 'draft'` in the `WHERE` — signed
   notes stay immutable under race.
4. `activate_case`, `reopen_case`, `sign_note`, `void_note` carry
   expected-from status in the `WHERE`.
5. Ignored tests `notes_rejected_on_deleted_case` and
   `concurrent_discharge_single_winner` pass locally and in CI.
6. `scripts/helix_cura_prime_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
7. `cura-durability` CI job in `.github/workflows/ci.yml`.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
