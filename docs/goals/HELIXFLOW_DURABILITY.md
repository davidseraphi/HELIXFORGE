# HELIXFLOW-DURABILITY

Prove the Foundation Integrity durability gate on HelixFlow: fresh crash,
concurrency, and restore, verified locally and in CI. Fourth product through
the gate (after `helix-collab`, `helix-capital`, `helix-commerce`).

## Scope

Flow writes are single-statement, but `update_run` had no terminal-state
guard — a finished run could silently transition back to running. This
packet makes terminal runs immutable and proves the gate.

## Definition of done

1. `FlowRepo::update_run` guards on `finished_at IS NULL`: progress updates
   and finish writes are rejected once a run is finished (validation error).
2. New ignored Postgres integration test (run in the `flow-durability`
   CI job):
   - `finished_runs_are_immutable` — after a run finishes, N concurrent
     update attempts (progress + finish) are all rejected, and status and
     `finished_at` are unchanged.
3. `scripts/helix_flow_durability.ps1`:
   - create workflow, enqueue run, progress, finish, verify lifecycle
   - acknowledge a run, force-kill the API, restart, and verify the run is
     fully present
   - `pg_dump` of the `flow` schema restores into a scratch database with
     equal workflow/run/event counts and equal content hashes
4. `flow-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
5. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on flow writes.
- Durability gates for other products (each needs its own named packet).
