# ADR-0005: Durable audit & metering via sqlx migrations

## Status

Accepted — 2026-07-14

## Context

In-memory audit and meter sinks lose history on restart and cannot support
enterprise compliance or billing. HelixCollab and later products need durable
workspaces and documents on the same data plane.

## Decision

1. Introduce `helix_db` crate owning:
   - `sqlx` `PgPool`
   - embedded migrations under `crates/helix-db/migrations/`
   - `PgAuditSink` (implements `audit_log::AuditSink`)
   - `PgMetering` (implements `billing_client::Metering`)
   - `WorkspaceRepo`, `CollabRepo`
2. `service_kit::build_core_clients` attempts Postgres connect + migrate; on
   failure falls back to memory sinks so local boot still works without Docker.
3. Audit append uses a transaction + `pg_advisory_xact_lock` so the hash chain
   tip is serialized under concurrency.
4. Application schema is versioned by sqlx migrations (not only docker init SQL).

## Consequences

- Durable path is the default when Postgres is healthy.
- Offline/dev still boots with degraded memory mode (`/healthz` reports postgres).
- Timescale hypertable for meters remains optional (plain table in migration;
  docker init may still create hypertables when Timescale is present).
