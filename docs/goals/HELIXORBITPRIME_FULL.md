# HELIXORBITPRIME-FULL

Move HelixOrbit Prime from thin durable scaffold to full second-wave depth
(catalog order 16, port `8116`).

## Scope

This packet closes the core asset + pass lifecycle gaps in the current
scaffold: an asset commission/decommission lifecycle with a decommission
guard, a pass plan/complete/cancel lifecycle, and an orbit summary report.

## Definition of done

1. Migration `0051_orbit_depth.sql` adds:
   - `commissioned_at`, `decommissioned_at`, `deleted_at` lifecycle columns
     on `orbit.assets`
   - `updated_at`, `planned_at`, `completed_at`, `cancelled_at`, `deleted_at`
     columns on `orbit.passes`
   - Backfill of legacy scaffold pass status `open` to `draft`
   - Partial active-asset and active-pass indexes
2. `OrbitRepo` gains:
   - `update_asset`, `commission_asset`, `decommission_asset` (rejected
     while draft or planned passes remain), `recommission_asset`,
     `soft_delete_asset`, `restore_asset`
   - `update_pass`, `plan_pass`, `complete_pass`, `cancel_pass`,
     `soft_delete_pass`, `restore_pass` (all verified against the parent
     asset)
   - `get_orbit_summary` (per-asset pass counts by status)
3. Domain APIs:
   - `PATCH /v1/assets/{id}`
   - `POST /v1/assets/{id}/commission`
   - `POST /v1/assets/{id}/decommission`
   - `POST /v1/assets/{id}/recommission`
   - `POST /v1/assets/{id}/delete`
   - `POST /v1/assets/{id}/restore`
   - `PATCH /v1/assets/{id}/passes/{pass_id}`
   - `POST /v1/assets/{id}/passes/{pass_id}/plan`
   - `POST /v1/assets/{id}/passes/{pass_id}/complete`
   - `POST /v1/assets/{id}/passes/{pass_id}/cancel`
   - `POST /v1/assets/{id}/passes/{pass_id}/delete`
   - `POST /v1/assets/{id}/passes/{pass_id}/restore`
   - `GET /v1/reports/orbit-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w16` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for asset/pass status transitions
   - ignored Postgres integration test for the decommission guard, asset
     lifecycle, pass lifecycle, and orbit summary
7. `scripts/helix_orbit_prime_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Orbital mechanics, conjunction screening, ground-station scheduling,
  telemetry ingestion, space-domain awareness feeds.
- Web UI changes.
