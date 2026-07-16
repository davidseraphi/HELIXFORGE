# 001 — Durable HelixCore audit/meter + HelixCollab persistence

### ADDED Requirements

#### Requirement
WHEN Postgres is available at service start the system SHALL run sqlx migrations
and use `PgAuditSink` and `PgMetering` for audit and usage events.

##### Scenario
- GIVEN docker Postgres is healthy
- WHEN gateway starts
- THEN `/healthz` postgres check is ok and workspace create writes to `helix_core.workspaces`

#### Requirement
WHEN Postgres is unavailable the system SHALL fall back to in-memory sinks and
still serve `/healthz` with status degraded on the postgres check.

##### Scenario
- GIVEN Postgres is down
- WHEN any service starts
- THEN the process starts and `db.connected` in `/v1/meta` is false

#### Requirement
WHEN a HelixCollab document is created and patched the system SHALL persist
revisions and reject stale `base_version` with conflict.

##### Scenario
- GIVEN a document at version 1
- WHEN PATCH with `base_version: 1` succeeds
- THEN version becomes 2
- WHEN a second PATCH still uses `base_version: 1`
- THEN the API returns 409

#### Requirement
WHEN clients connect to the document WebSocket the system SHALL fan out
presence and snapshot messages to peers in the room.
