# ADR-0003: Hash-chained immutable audit log

## Status

Accepted — 2026-07-14

## Context

Zero-trust and regulated products (Capital, Cura, Lex) require tamper-evident
audit trails.

## Decision

- Every security-relevant action appends an audit entry.
- Entries form a **BLAKE3 hash chain** (`prev_hash` → `entry_hash`).
- Verification endpoint on observability-service fails closed on break.

## Consequences

- Append-only semantics; corrections are compensating events, never rewrites.
- Slight CPU overhead (acceptable).
