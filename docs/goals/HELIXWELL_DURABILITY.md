# HELIXWELL-DURABILITY

Prove the Foundation Integrity durability gate on HelixWell: fresh crash,
concurrency, and restore, verified locally and in CI. Seventh product
through the gate (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`).

## Scope

Habit logging had a check-then-insert window: the active-habit guard ran in
one statement and the log INSERT in another, so a habit paused in between
would silently accept a log. This packet makes the guard atomic and proves
the gate.

## Definition of done

1. `WellRepo::log_habit` inserts with an `INSERT ... SELECT` that requires
   the habit to exist, be active, and not be deleted — one statement.
2. New ignored Postgres integration tests (run in the `well-durability`
   CI job):
   - `logs_rejected_on_paused_habit` — after pausing a habit, N concurrent
     log attempts are all rejected; only the baseline log exists.
   - `concurrent_logs_all_landed` — N concurrent logs on an active habit
     all persist with the exact total quantity.
3. `scripts/helix_well_durability.ps1`:
   - create habit, log, verify summary
   - acknowledge a check-in, force-kill the API, restart, and verify the
     check-in is fully present
   - `pg_dump` of the `well` schema restores into a scratch database with
     equal habit/log/checkin counts and equal content hashes
4. `well-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
5. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29667399976` (**HelixWell durability gate** job green)
- Proof script: `scripts/helix_well_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on well writes.
- Durability gates for other products (each needs its own named packet).
