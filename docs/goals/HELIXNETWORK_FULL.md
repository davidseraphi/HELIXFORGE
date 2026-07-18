# HELIXNETWORK-FULL

Move HelixNetwork from durable scaffold to full second-wave depth (catalog
order 9, port `8109`).

## Scope

This packet closes the core profile + connection + opportunity lifecycle gaps
in the current scaffold: profile lifecycle management, full connection
lifecycle (decline, remove, block, re-request revival), opportunity lifecycle,
and a network summary report.

## Definition of done

1. Migration `0044_network_depth.sql` adds:
   - `deactivated_at`, `deleted_at` lifecycle columns on `network.profiles`
   - `responded_at`, `blocked_by` columns on `network.connections`
   - `closed_at`, `deleted_at` columns on `network.opportunities`
   - Partial active-profile / active-opportunity indexes, connection
     pair-status index
2. `NetworkRepo` gains:
   - `update_profile`, `deactivate_profile`, `reactivate_profile`,
     `soft_delete_profile`, `restore_profile` (owner-scoped)
   - `request_connection` rework: revives a declined/removed pair row back to
     pending, rejects pairs with a `blocked` row in either direction, and
     requires both profiles to be active
   - `decline_connection` (receiver only), `remove_connection` (either party),
     `block_connection` (either party, records `blocked_by`)
   - `update_opportunity`, `close_opportunity`, `reopen_opportunity`,
     `soft_delete_opportunity`, `restore_opportunity` (owner-scoped)
   - `get_network_summary` (per-profile accepted/pending/open-opportunity
     counts)
3. Domain APIs:
   - `PATCH /v1/profiles/{id}`
   - `POST /v1/profiles/{id}/deactivate`
   - `POST /v1/profiles/{id}/reactivate`
   - `POST /v1/profiles/{id}/delete`
   - `POST /v1/profiles/{id}/restore`
   - `POST /v1/connections/{id}/decline`
   - `POST /v1/connections/{id}/remove`
   - `POST /v1/connections/{id}/block`
   - `PATCH /v1/opportunities/{id}`
   - `POST /v1/opportunities/{id}/close`
   - `POST /v1/opportunities/{id}/reopen`
   - `POST /v1/opportunities/{id}/delete`
   - `POST /v1/opportunities/{id}/restore`
   - `GET /v1/reports/network-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w9` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for profile/opportunity transitions and
     connection revival eligibility
   - ignored Postgres integration test for the full connection lifecycle,
     profile lifecycle, opportunity lifecycle, and network summary
7. `scripts/helix_network_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29642796843`
- Smoke script: `scripts/helix_network_smoke.ps1`
- Unit tests: `3/3 PASS` (plus one ignored Postgres integration test)

## Out of scope

- Field-level audience policy, search, matching, messaging, proposals.
- Verification, moderation, reputation, federation.
- Consent/privacy enforcement at the data boundary (spec P0; belongs to the
  Foundation Integrity/category program, not this depth packet).
- Web UI changes.
