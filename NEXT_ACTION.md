# Next action

## Active: HELIXFORGESTUDIO-DURABILITY — ninth product through the gate

Prove the Foundation Integrity durability gate on HelixForge Studio:
fresh crash, concurrency, and restore, verified locally and in CI. Ninth
product (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`,
`helix-network`).

Goal doc: `docs/goals/HELIXFORGESTUDIO_DURABILITY.md`.

### Scope

`create_child` checked the parent app in a separate SELECT before the
page INSERT; `publish_app` counted pages and checked draft status in
separate statements from the UPDATE; page archive/reopen and app
unpublish carry no expected-from status guard. This packet folds the
guards into the writes and proves the gate.

### Definition of done

1. `create_child` inserts with `INSERT ... SELECT` against a non-deleted
   app — one statement.
2. `publish_app` is a single guarded `UPDATE` (draft + not deleted +
   `EXISTS` non-deleted page).
3. `unpublish_app`, `archive_page`, `reopen_page` carry expected-from
   status in the `WHERE`.
4. Ignored tests `pages_rejected_on_deleted_app` and
   `concurrent_publish_single_winner` pass locally and in CI.
5. `scripts/helix_forge_studio_durability.ps1` proves lifecycle,
   forced-kill survival, and schema restore roundtrip.
6. `forge-studio-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Implement the packet, verify locally, push, and watch CI to green.
