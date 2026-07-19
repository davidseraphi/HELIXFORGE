# HELIXPULSE-DURABILITY

Prove the Foundation Integrity durability gate on HelixPulse: fresh
crash, concurrency, and restore, verified locally and in CI. Twentieth
product through the gate (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`,
`helix-vita-prime`, `helix-grid-prime`, `helix-nova-labs`).

## Scope

`create_incident` checked the parent monitor in a separate SELECT before
the incident INSERT — a monitor soft-deleted in between silently gains
an incident. `pause_monitor` counted open incidents and checked active
status in separate statements from the UPDATE — an incident opened, or
a concurrent pause landing in between, breaks the "paused means no open
incidents" invariant. The activate/resume monitor updates and the
incident acknowledge/resolve transition carry no expected-from status
in their WHERE. This packet folds the guards into the writes and proves
the gate.

## Definition of done

1. `PulseRepo::create_incident` inserts with an `INSERT ... SELECT` that
   requires the monitor to exist and not be deleted — one statement.
2. `PulseRepo::pause_monitor` is a single guarded `UPDATE` requiring
   `status = 'active'`, not deleted, and `NOT EXISTS` a non-deleted open
   incident.
3. `PulseRepo::activate_monitor`, `resume_monitor`, and the incident
   `transition_incident` carry their expected-from status in the UPDATE
   `WHERE`.
4. New ignored Postgres integration tests (run in the
   `pulse-durability` CI job):
   - `incidents_rejected_on_deleted_monitor` — after soft-deleting a
     monitor, N concurrent incident creates are all rejected; no
     incident leaks in.
   - `concurrent_pause_single_winner` — N concurrent pauses of one
     active monitor produce exactly one success; the rest are rejected;
     the monitor ends paused.
5. `scripts/helix_pulse_durability.ps1`:
   - create monitor, activate, incident, resolve, pause, verify
   - acknowledge a paused monitor, force-kill the API, restart, and
     verify the monitor and incident are fully present
   - `pg_dump` of the `pulse` schema restores into a scratch database
     with equal monitor/incident counts and equal content hashes
6. `pulse-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29686421129` (**HelixPulse durability gate** job green)
- Proof script: `scripts/helix_pulse_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on pulse writes.
- The deferred Redis-class cluster engine (separate packet per the
  product sheet).
- Durability gates for other products (each needs its own named packet).
