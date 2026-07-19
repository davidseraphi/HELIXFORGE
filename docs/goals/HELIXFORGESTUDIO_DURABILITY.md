# HELIXFORGESTUDIO-DURABILITY

Prove the Foundation Integrity durability gate on HelixForge Studio: fresh
crash, concurrency, and restore, verified locally and in CI. Ninth product
through the gate (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`, `helix-network`).

## Scope

`create_child` checked the parent app in a separate SELECT before the page
INSERT — an app soft-deleted in between silently gains a page.
`publish_app` counted pages and checked draft status in separate
statements from the UPDATE — the last page deleted, or a concurrent
publish landing in between, breaks the "published means at least one
page" invariant. The page archive/reopen and app unpublish updates carry
no expected-from status in their WHERE. This packet folds the guards into
the writes and proves the gate.

## Definition of done

1. `StudioRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the app to exist and not be deleted — one statement.
2. `StudioRepo::publish_app` is a single guarded `UPDATE` requiring
   `status = 'draft'`, not deleted, and `EXISTS` at least one non-deleted
   page.
3. `StudioRepo::unpublish_app`, `archive_page`, and `reopen_page` carry
   their expected-from status in the UPDATE `WHERE`.
4. New ignored Postgres integration tests (run in the
   `forge-studio-durability` CI job):
   - `pages_rejected_on_deleted_app` — after soft-deleting an app, N
     concurrent page creates are all rejected; no page leaks in.
   - `concurrent_publish_single_winner` — N concurrent publishes of one
     draft app produce exactly one success; the rest are rejected; the
     app ends published.
5. `scripts/helix_forge_studio_durability.ps1`:
   - create app, create page, publish, verify
   - acknowledge a published app, force-kill the API, restart, and verify
     the app and page are fully present
   - `pg_dump` of the `studio` schema restores into a scratch database
     with equal app/page counts and equal content hashes
6. `forge-studio-durability` CI job in `.github/workflows/ci.yml` running
   the ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29669148679` (**HelixForge Studio durability gate** job green)
- Proof script: `scripts/helix_forge_studio_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on studio writes.
- Durability gates for other products (each needs its own named packet).
