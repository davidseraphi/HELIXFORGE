# 007 — HelixNetwork durable profiles & connections

### ADDED Requirements

#### Requirement
WHEN Postgres is available HelixNetwork SHALL persist profiles via
`helix_db::NetworkRepo` after migration `0009_network.sql` (one profile per user per tenant).

##### Scenario
- GIVEN two local users sharing the dev tenant
- WHEN each `POST /v1/profiles`
- THEN both profiles are listable in the same tenant

#### Requirement
WHEN a connection is requested and accepted the status SHALL move pending → accepted
and only the target profile may accept.

##### Scenario
- GIVEN Alice and Bob profiles
- WHEN Alice requests Bob and Bob accepts
- THEN connection status is `accepted`
