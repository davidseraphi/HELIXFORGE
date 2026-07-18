# HELIXNOVALABS-FULL

Move HelixNova Labs from thin durable scaffold to full second-wave depth
(catalog order 20, port `8120`).

## Scope

This packet closes the core experiment + finding lifecycle gaps in the
current scaffold: an experiment start/conclude lifecycle with a conclusion
guard, a finding confirm/reject lifecycle, and a nova summary report.

## Definition of done

1. Migration `0055_nova_depth.sql` adds:
   - `started_at`, `concluded_at`, `deleted_at` lifecycle columns on
     `nova.experiments`
   - `updated_at`, `confirmed_at`, `rejected_at`, `deleted_at` columns on
     `nova.findings`
   - Backfill of legacy scaffold finding status `open` to `draft`
   - Partial active-experiment and active-finding indexes
2. `NovaRepo` gains:
   - `update_experiment`, `start_experiment`, `conclude_experiment`
     (rejected while draft findings remain), `reopen_experiment`,
     `soft_delete_experiment`, `restore_experiment`
   - `update_finding`, `confirm_finding`, `reject_finding`,
     `soft_delete_finding`, `restore_finding` (all verified against the
     parent experiment)
   - `get_nova_summary` (per-experiment finding counts by status)
3. Domain APIs:
   - `PATCH /v1/experiments/{id}`
   - `POST /v1/experiments/{id}/start`
   - `POST /v1/experiments/{id}/conclude`
   - `POST /v1/experiments/{id}/reopen`
   - `POST /v1/experiments/{id}/delete`
   - `POST /v1/experiments/{id}/restore`
   - `PATCH /v1/experiments/{id}/findings/{finding_id}`
   - `POST /v1/experiments/{id}/findings/{finding_id}/confirm`
   - `POST /v1/experiments/{id}/findings/{finding_id}/reject`
   - `POST /v1/experiments/{id}/findings/{finding_id}/delete`
   - `POST /v1/experiments/{id}/findings/{finding_id}/restore`
   - `GET /v1/reports/nova-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w20` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for experiment/finding status transitions
   - ignored Postgres integration test for the conclusion guard, experiment
     lifecycle, finding lifecycle, and nova summary
7. `scripts/helix_nova_labs_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Lab notebook versioning, sample tracking, instrument integration,
  analysis pipelines, publication workflows.
- Web UI changes.
