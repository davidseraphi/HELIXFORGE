# 011.3 — Atomic writes, durable visible jobs, truthful readiness, and fresh release gates

## Status

Completed on 2026-07-15. `011.2` is closed and the verification gates below pass.

## Outcome

Domain change, audit event, and outbox entry commit together or roll back
together. Long-running work is represented as durable jobs with visible
progress, cancellation, crash recovery, and checkpoints. Readiness reports the
real state. Release gates run fresh checks and produce signed evidence.

## Allowed edits (after activation)

- `crates/helix-db/src/` — outbox table, job table, transactional helper that
  commits domain + audit + outbox together.
- `crates/audit_log/src/lib.rs` — keep hash-chained append inside the same
  transaction.
- New or extended job crate/service — durable job state machine, heartbeat,
  checkpoint, cancel, retry, orphan recovery.
- `crates/service-kit/src/health.rs` and
  `services/observability-service/src/main.rs` — readiness fails closed;
  compliance summary uses fresh checks.
- New release-gate engine or operator CLI — runs fresh checks, records
  commands/environment/hashes, treats skipped required checks as blocking.
- Migrations and tests: forced-crash write boundary, job cancel, readiness-down,
  release-gate dry-run.
- Living docs and this packet.

## Forbidden edits

- No product UI work.
- No external provider secrets or real payment providers.
- No physical-control or high-stakes product code.
- No claiming a gate passed from cached, stale, or self-reported evidence.

## EARS acceptance

### Atomic writes

- The system SHALL commit domain change, audit event, and outbox entry in one
  transaction, or roll back all three.
- The system SHALL use the same idempotency key for retries of the same
  accepted command.
- The system SHALL treat `unknown` as a real final state until evidence resolves
  it.

### Durable visible jobs

- The system SHALL represent every task longer than one second as a job with a
  stable identity.
- The system SHALL record requested outcome, owner, authority lease, stages,
  durable checkpoints, process identity, start time, last signal, elapsed time,
  and resource use.
- The system SHALL support states: queued, starting, running, waiting, blocked,
  cancelling, cancelled, failed, completed, and unknown.
- The system SHALL support safe pause, resume, retry, cancel, and recovery.
- The system SHALL show phase, last heartbeat, elapsed time, and safe next
  action; it SHALL NOT invent percentages for work with no measurable total.

### Truthful readiness

- The `/readyz` endpoint SHALL return `ready=false` when any required
  dependency is unavailable outside local mode.
- The system SHALL label degraded, skipped, unknown, and not-configured states
  honestly; it SHALL NOT call them healthy.

### Fresh release gates

- The release gate SHALL run fresh checks against the exact candidate.
- The release gate SHALL record command, environment, timestamps, artifact
  hashes, outputs, and skipped checks with reasons.
- The release gate SHALL treat any skipped required check as blocking.

## Test plan

| Check | Evidence |
|---|---|
| Atomic write boundary | `AtomicWork` commit/rollback tests: all three rows (job, audit, outbox) commit or roll back together |
| Idempotency | Outbox `enqueue_in_tx` returns existing row for duplicate tenant+key |
| Job lifecycle | `JobRepo` + `JobWorker` tests: create, claim, run, heartbeat, checkpoint, cancel, retry, fail |
| Job progress honesty | `Job` exposes phases, heartbeat, elapsed_ms, checkpoints; no percentage field exists |
| Readiness down | `/readyz` returns `ready=false` when required deps are degraded outside local; aggregate `/v1/core/ready` probes each core service |
| Release gate | `release-gate` unit tests: skipped required check blocks gate; all required passed opens gate |

## Evidence

- Migration: `crates/helix-db/migrations/0037_atomic_jobs_readiness.sql` (+ down file).
- Atomic writes: `crates/helix-db/src/atomic.rs` `AtomicWork` with shared-transaction audit/outbox/job create.
- Transactional audit: `crates/helix-db/src/audit_pg.rs` `TransactionalAuditSink::append_in_tx`.
- Outbox: `crates/helix-db/src/outbox.rs` `OutboxRepo::enqueue_in_tx` with idempotency.
- Durable jobs: `crates/helix-db/src/jobs.rs` `JobRepo` + `crates/job-engine/src/lib.rs` `JobWorker`/`JobRegistry`.
- Readiness: `crates/service-kit/src/health.rs` honest `CheckState` labels + `services/observability-service/src/main.rs` `/v1/core/ready` aggregate.
- Release gate: `tools/quality/release-gate/src/main.rs` records commands/environment/hashes/outputs.
- Verification: `cargo build --workspace`, `cargo test --workspace --all-features`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` all pass; Postgres migrations apply cleanly.

## Dependencies

- `011.1` for version control and CI that can run the gate checks.
- `011.2` for stable identity and tenant separation so jobs have real owners
  and cannot cross tenants.

## Rollback / compensation

- Migrations include rollback scripts.
- A failed gate creates a new candidate; the old candidate remains addressable.
- Incomplete jobs can be cancelled or retried without corrupting committed
  domain state.
