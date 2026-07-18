# HELIXCOLLAB-DURABILITY

Prove the Foundation Integrity durability gate on HelixCollab, the lead
product: fresh crash, concurrency, and restore, verified locally and in CI.
This is the first product through the gate (`durability_gate_proven_products`
was empty).

## Scope

Close the crash windows the write paths had, then prove the gate end to end:

1. **Atomic create** — document creation previously inserted the document row
   and the initial revision row in two separate, non-transactional INSERTs; a
   crash between them left a document with no revision. Now one transaction:
   both or neither.
2. **Single-statement attachment register** — attachment upload previously
   inserted the metadata row and then ran a separate `body_stored = true`
   UPDATE. The INSERT now stores the flag directly (the object bytes are
   already in MinIO by that point). An orphaned MinIO object on INSERT
   failure remains acceptable garbage.
3. **Concurrency proof** — a true race: N concurrent patches with the same
   `base_version` on one document produce exactly one winner; the rest get a
   conflict; the unique `(document_id, version)` revision constraint holds.
4. **Crash proofs** — concurrent creates never produce a torn document/
   revision pair; an acknowledged write survives an immediate forced kill of
   the API process.
5. **Restore proof** — `pg_dump` of the `collab` schema restores into a
   scratch database with identical row counts and content hashes.

## Definition of done

1. `CollabRepo::create_document_full_ex` writes the document and its initial
   revision in one transaction.
2. `SovereignCollabRepo::register_attachment` inserts `body_stored = true` in
   the single INSERT; the separate UPDATE in the upload handler is removed.
3. Ignored Postgres integration tests (run in the existing `collab-smoke`
   CI job):
   - `concurrent_patches_single_winner`
   - `concurrent_creates_never_torn`
4. `scripts/helix_collab_durability.ps1`:
   - create doc, patch twice, verify revision chain
   - POST a doc, force-kill the API (`Stop-Process -Force`), restart it,
     and verify the acknowledged document and its revision are fully present
   - pg_dump the `collab` schema, load it into a scratch database, and assert
     equal document/revision counts and equal ordered content hashes
5. `collab-durability` CI job in `.github/workflows/ci.yml` running the
   durability script; existing `collab-smoke` job untouched.
6. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Idempotency keys on collab writes (retry dedupe) — tracked as follow-up
  Foundation Integrity work, not part of this gate proof.
- Audit/NATS/outbox transactionality on collab writes (post-commit steps stay
  post-commit; the gate proves domain rows are never torn).
- Durability gates for other products (each needs its own named packet).
