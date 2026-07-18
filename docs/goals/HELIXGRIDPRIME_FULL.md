# HELIXGRIDPRIME-FULL

Move HelixGrid Prime from thin durable scaffold to full second-wave depth
(catalog order 19, port `8119`).

## Scope

This packet closes the core site + reading lifecycle gaps in the current
scaffold: a site energize/offline lifecycle with an offline guard, a reading
verify/reject lifecycle, and a grid summary report.

## Definition of done

1. Migration `0054_grid_depth.sql` adds:
   - `energized_at`, `offline_at`, `deleted_at` lifecycle columns on
     `grid.sites`
   - `updated_at`, `verified_at`, `rejected_at`, `deleted_at` columns on
     `grid.readings`
   - Backfill of legacy scaffold reading status `open` to `draft`
   - Partial active-site and active-reading indexes
2. `GridRepo` gains:
   - `update_site`, `energize_site`, `take_offline` (rejected while draft
     readings remain), `bring_online`, `soft_delete_site`, `restore_site`
   - `update_reading`, `verify_reading`, `reject_reading`,
     `soft_delete_reading`, `restore_reading` (all verified against the
     parent site)
   - `get_grid_summary` (per-site reading counts by status)
3. Domain APIs:
   - `PATCH /v1/sites/{id}`
   - `POST /v1/sites/{id}/energize`
   - `POST /v1/sites/{id}/offline`
   - `POST /v1/sites/{id}/online`
   - `POST /v1/sites/{id}/delete`
   - `POST /v1/sites/{id}/restore`
   - `PATCH /v1/sites/{id}/readings/{reading_id}`
   - `POST /v1/sites/{id}/readings/{reading_id}/verify`
   - `POST /v1/sites/{id}/readings/{reading_id}/reject`
   - `POST /v1/sites/{id}/readings/{reading_id}/delete`
   - `POST /v1/sites/{id}/readings/{reading_id}/restore`
   - `GET /v1/reports/grid-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w19` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for site/reading status transitions
   - ignored Postgres integration test for the offline guard, site
     lifecycle, reading lifecycle, and grid summary
7. `scripts/helix_grid_prime_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- SCADA ingestion, telemetry streaming, grid balancing, forecasting, market
  integration.
- Web UI changes.
