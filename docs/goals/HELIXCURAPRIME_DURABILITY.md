# HELIXCURAPRIME-DURABILITY

Prove the Foundation Integrity durability gate on HelixCura Prime: fresh
crash, concurrency, and restore, verified locally and in CI. Twelfth
product through the gate (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`).

## Scope

`create_child` checked the parent case in a separate SELECT before the
note INSERT — a case soft-deleted in between silently gains a note.
`discharge_case` counted draft notes and checked active status in
separate statements from the UPDATE — a draft note created, or a
concurrent discharge landing in between, breaks the "discharged means no
draft notes" invariant. `update_note` rejected edits of non-draft notes
in a read, but the UPDATE carried no draft guard — a sign landing in
between lets an edit overwrite a signed note, breaking
`signed_immutable`. The activate/reopen and sign/void updates carry no
expected-from status in their WHERE. This packet folds the guards into
the writes and proves the gate.

## Definition of done

1. `CuraRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the case to exist and not be deleted — one statement.
2. `CuraRepo::discharge_case` is a single guarded `UPDATE` requiring
   `status = 'active'`, not deleted, and `NOT EXISTS` a non-deleted draft
   note.
3. `CuraRepo::update_note` carries `status = 'draft'` in the UPDATE
   `WHERE` — signed notes stay immutable under race.
4. `CuraRepo::activate_case`, `reopen_case`, `sign_note`, and
   `void_note` carry their expected-from status in the UPDATE `WHERE`.
5. New ignored Postgres integration tests (run in the `cura-durability`
   CI job):
   - `notes_rejected_on_deleted_case` — after soft-deleting a case, N
     concurrent note creates are all rejected; no note leaks in.
   - `concurrent_discharge_single_winner` — N concurrent discharges of
     one active case produce exactly one success; the rest are rejected;
     the case ends discharged.
6. `scripts/helix_cura_prime_durability.ps1`:
   - create case, activate, note, sign, discharge, verify
   - acknowledge a discharged case, force-kill the API, restart, and
     verify the case and signed note are fully present
   - `pg_dump` of the `cura` schema restores into a scratch database with
     equal case/note counts and equal content hashes
7. `cura-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on cura writes.
- Durability gates for other products (each needs its own named packet).
