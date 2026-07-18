# HELIXCLIMATEPRIME-FULL

Move HelixClimate Prime from thin durable scaffold to full second-wave depth
(catalog order 15, port `8115`).

## Scope

This packet closes the core scenario + risk-score lifecycle gaps in the
current scaffold: a scenario active/archive lifecycle with an archive guard,
a risk-score assess/dismiss lifecycle, and a climate summary report.

## Definition of done

1. Migration `0050_climate_depth.sql` adds:
   - `activated_at`, `archived_at`, `deleted_at` lifecycle columns on
     `climate.scenarios`
   - `updated_at`, `assessed_at`, `dismissed_at`, `deleted_at` columns on
     `climate.risk_scores`
   - Backfill of legacy scaffold score status `open` to `draft`
   - Partial active-scenario and active-score indexes
2. `ClimateRepo` gains:
   - `update_scenario`, `activate_scenario`, `archive_scenario` (rejected
     while draft scores remain), `reopen_scenario`, `soft_delete_scenario`,
     `restore_scenario`
   - `update_score`, `assess_score`, `dismiss_score`, `soft_delete_score`,
     `restore_score` (all verified against the parent scenario)
   - `get_climate_summary` (per-scenario score counts by status)
3. Domain APIs:
   - `PATCH /v1/scenarios/{id}`
   - `POST /v1/scenarios/{id}/activate`
   - `POST /v1/scenarios/{id}/archive`
   - `POST /v1/scenarios/{id}/reopen`
   - `POST /v1/scenarios/{id}/delete`
   - `POST /v1/scenarios/{id}/restore`
   - `PATCH /v1/scenarios/{id}/risk_scores/{score_id}`
   - `POST /v1/scenarios/{id}/risk_scores/{score_id}/assess`
   - `POST /v1/scenarios/{id}/risk_scores/{score_id}/dismiss`
   - `POST /v1/scenarios/{id}/risk_scores/{score_id}/delete`
   - `POST /v1/scenarios/{id}/risk_scores/{score_id}/restore`
   - `GET /v1/reports/climate-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w15` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for scenario/score status transitions
   - ignored Postgres integration test for the archive guard, scenario
     lifecycle, score lifecycle, and climate summary
7. `scripts/helix_climate_prime_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Climate models, emissions accounting, geospatial data, scenario engines,
  regulatory reporting.
- Web UI changes.
