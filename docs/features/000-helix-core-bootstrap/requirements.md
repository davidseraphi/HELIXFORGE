# 000 — HelixCore bootstrap

### ADDED Requirements

#### Requirement
WHEN an operator starts the gateway with `HELIX_ENV=local` the system SHALL expose `/healthz`, `/readyz`, and `/v1/catalog` without external IdP.

##### Scenario
- GIVEN no Ory stack is running
- WHEN GET `/v1/catalog` is called
- THEN the response contains 20 product entries in catalog order

#### Requirement
WHEN a request includes `X-Helix-Dev-User` in local mode the system SHALL resolve a principal with read/write scopes.

##### Scenario
- GIVEN gateway is running locally
- WHEN GET `/v1/me` with `X-Helix-Dev-User: founder@helixforge.local`
- THEN the response includes a session_id containing the email

#### Requirement
WHEN audit events are appended the system SHALL maintain a verifiable BLAKE3 hash chain.

##### Scenario
- GIVEN an empty audit sink
- WHEN five events are appended
- THEN `verify_chain` returns true
