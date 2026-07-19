# HELIXSYNTHBIO-DURABILITY

Prove the Foundation Integrity durability gate on HelixSynthBio: fresh
crash, concurrency, and restore, verified locally and in CI. Tenth product
through the gate (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`,
`helix-network`, `helix-forge-studio`).

## Scope

`create_child` checked the parent design in a separate SELECT before the
sim INSERT — a design soft-deleted in between silently gains a sim.
`approve_design` counted completed sims and checked review status in
separate statements from the UPDATE — the last completed sim deleted, or
a concurrent approve/return landing in between, breaks the "approved
means at least one completed sim" invariant. The submit/return design
updates and the sim start/complete/fail transition carry no expected-from
status in their WHERE. This packet folds the guards into the writes and
proves the gate.

## Definition of done

1. `SynthbioRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the design to exist and not be deleted — one statement.
2. `SynthbioRepo::approve_design` is a single guarded `UPDATE` requiring
   `status = 'review'`, not deleted, and `EXISTS` at least one non-deleted
   completed sim.
3. `SynthbioRepo::submit_design`, `return_design`, and the sim
   `transition_sim` carry their expected-from status in the UPDATE
   `WHERE`.
4. New ignored Postgres integration tests (run in the
   `synthbio-durability` CI job):
   - `sims_rejected_on_deleted_design` — after soft-deleting a design, N
     concurrent sim creates are all rejected; no sim leaks in.
   - `concurrent_approve_single_winner` — N concurrent approves of one
     in-review design produce exactly one success; the rest are rejected;
     the design ends approved.
5. `scripts/helix_synthbio_durability.ps1`:
   - create design, submit, sim, start, complete, approve, verify
   - acknowledge an approved design, force-kill the API, restart, and
     verify the design and sim are fully present
   - `pg_dump` of the `synthbio` schema restores into a scratch database
     with equal design/sim counts and equal content hashes
6. `synthbio-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on synthbio writes.
- Durability gates for other products (each needs its own named packet).
