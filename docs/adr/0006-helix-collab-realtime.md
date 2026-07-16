# ADR-0006: HelixCollab persistence + real-time model

## Status

Accepted — 2026-07-14

## Context

HelixCollab needs multi-user co-editing with durable history, presence, and
live updates without vendor CRDT SaaS lock-in.

## Decision

1. **Persistence**: Postgres tables `collab.documents`, `collab.document_revisions`,
   `collab.presence` via `helix_db::CollabRepo`.
2. **Concurrency**: optimistic versioning — `PATCH` supplies `base_version`;
   mismatch → HTTP 409 Conflict.
3. **Real-time**: in-process `RealtimeHub` (broadcast channels) + WebSocket
   `/v1/ws/documents/{id}` for presence/patch fan-out.
4. **Durable writes** prefer REST patch (authenticated + audited); WebSocket
   mirrors live state to peers.
5. **Multi-instance fan-out**: each publish also emits
   `helix.collab.ws.{document_id}` with a `FanoutEnvelope { origin, document_id, message }`.
   Replicas subscribe to `helix.collab.ws.>` and inject remote messages into the
   local hub, skipping envelopes whose `origin` matches this process.
6. Future: optional CRDT (yrs/automerge) layer on top of the same wire protocol.

## Consequences

- Local/dev still works with in-memory bus (wildcard bridge is NATS-oriented;
  single instance remains correct via local broadcast).
- Multi-replica Collab requires shared NATS; sticky sessions are optional.
- Revisions enable point-in-time recovery without rewriting history.
