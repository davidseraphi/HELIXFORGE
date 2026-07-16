# ADR-0004: PostgreSQL + Timescale + MinIO + NATS

## Status

Accepted — 2026-07-14

## Context

Need relational OLTP, time-series meters, object storage, and messaging without
cloud lock-in.

## Decision

| Concern | Technology |
|---------|------------|
| OLTP + tenancy | PostgreSQL 16 |
| Horizontal shards (future) | Citus-compatible topology |
| Time-series meters | TimescaleDB hypertables |
| Objects | MinIO (S3 API) |
| Events | NATS JetStream |

## Consequences

- Fully self-hostable data plane.
- Citus enabled in production topology via Terraform/Helm, not required for local MVP.
