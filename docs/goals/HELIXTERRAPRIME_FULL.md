# HELIXTERRAPRIME-FULL

Move HelixTerra Prime from thin durable scaffold to full second-wave depth
(catalog order 14, port `8114`).

## Scope

This packet closes the core field + observation lifecycle gaps in the current
scaffold: a field active/retire lifecycle with a retire guard, an observation
confirm/dismiss lifecycle, and a terra summary report.

## Definition of done

1. Migration `0049_terra_depth.sql` adds:
   - `activated_at`, `retired_at`, `deleted_at` lifecycle columns on
     `terra.fields`
   - `updated_at`, `confirmed_at`, `dismissed_at`, `deleted_at` columns on
     `terra.observations`
   - Backfill of legacy scaffold observation status `open` to `draft`
   - Partial active-field and active-observation indexes
2. `TerraRepo` gains:
   - `update_field`, `activate_field`, `retire_field` (rejected while draft
     observations remain), `reopen_field`, `soft_delete_field`,
     `restore_field`
   - `update_observation`, `confirm_observation`, `dismiss_observation`,
     `soft_delete_observation`, `restore_observation` (all verified against
     the parent field)
   - `get_terra_summary` (per-field observation counts by status)
3. Domain APIs:
   - `PATCH /v1/fields/{id}`
   - `POST /v1/fields/{id}/activate`
   - `POST /v1/fields/{id}/retire`
   - `POST /v1/fields/{id}/reopen`
   - `POST /v1/fields/{id}/delete`
   - `POST /v1/fields/{id}/restore`
   - `PATCH /v1/fields/{id}/observations/{obs_id}`
   - `POST /v1/fields/{id}/observations/{obs_id}/confirm`
   - `POST /v1/fields/{id}/observations/{obs_id}/dismiss`
   - `POST /v1/fields/{id}/observations/{obs_id}/delete`
   - `POST /v1/fields/{id}/observations/{obs_id}/restore`
   - `GET /v1/reports/terra-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w14` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for field/observation status transitions
   - ignored Postgres integration test for the retire guard, field
     lifecycle, observation lifecycle, and terra summary
7. `scripts/helix_terra_prime_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Geospatial data, sensor ingestion, agronomy models, mapping, weather
  integrations.
- Web UI changes.
