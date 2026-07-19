# HELIXQUANTUMFORGE-DURABILITY

Prove the Foundation Integrity durability gate on HelixQuantum Forge:
fresh crash, concurrency, and restore, verified locally and in CI.
Sixteenth product through the gate (after `helix-collab`,
`helix-capital`, `helix-commerce`, `helix-flow`, `helix-insights`,
`helix-edu`, `helix-well`, `helix-network`, `helix-forge-studio`,
`helix-synthbio`, `helix-lex-prime`, `helix-cura-prime`,
`helix-terra-prime`, `helix-climate-prime`, `helix-orbit-prime`).

## Scope

`create_child` checked the parent job in a separate SELECT before the
circuit INSERT — a job soft-deleted in between silently gains a circuit.
`submit_job` counted circuits and checked draft status in separate
statements from the UPDATE — the last circuit deleted, or a concurrent
submit landing in between, breaks the "submitted means at least one
circuit" invariant. The complete/fail job transitions and the
validate/archive circuit updates carry no expected-from status in their
WHERE. This packet folds the guards into the writes and proves the gate.

## Definition of done

1. `QuantumRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the job to exist and not be deleted — one statement.
2. `QuantumRepo::submit_job` is a single guarded `UPDATE` requiring
   `status = 'draft'`, not deleted, and `EXISTS` at least one non-deleted
   circuit.
3. The job `transition_job` and `validate_circuit` / `archive_circuit`
   carry their expected-from status in the UPDATE `WHERE`.
4. New ignored Postgres integration tests (run in the
   `quantum-durability` CI job):
   - `circuits_rejected_on_deleted_job` — after soft-deleting a job, N
     concurrent circuit creates are all rejected; no circuit leaks in.
   - `concurrent_submit_single_winner` — N concurrent submits of one
     draft job produce exactly one success; the rest are rejected; the
     job ends submitted.
5. `scripts/helix_quantum_forge_durability.ps1`:
   - create job, circuit, submit, complete, verify
   - acknowledge a completed job, force-kill the API, restart, and
     verify the job and circuit are fully present
   - `pg_dump` of the `quantum` schema restores into a scratch database
     with equal job/circuit counts and equal content hashes
6. `quantum-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on quantum writes.
- Durability gates for other products (each needs its own named packet).
