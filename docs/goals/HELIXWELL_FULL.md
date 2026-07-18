# HELIXWELL-FULL

Move HelixWell from durable scaffold to full second-wave depth (catalog
order 8, port `8108`).

## Scope

This packet closes the core habit + check-in lifecycle gaps in the current
scaffold: habit lifecycle management, optional check-in fields (missing is not
zero), check-in edit history, and habit summary reporting.

## Definition of done

1. Migration `0043_well_depth.sql` adds:
   - `paused_at`, `ended_at`, `deleted_at` lifecycle columns on `well.habits`
   - nullable `mood` / `energy` on `well.checkins` (skipped field is missing,
     never zero), plus `updated_at`, `deleted_at`, `edit_version`
   - `well.checkin_edits` append-only edit-history side table
   - Partial active-habit and active-checkin indexes
2. `WellRepo` gains:
   - `update_habit`
   - `pause_habit` / `resume_habit`
   - `end_habit` (terminal; logs rejected)
   - `soft_delete_habit` / `restore_habit`
   - `create_checkin` accepting skipped (`NULL`) mood/energy
   - `get_checkin`, `update_checkin` (transactional snapshot into
     `well.checkin_edits` + version bump), `soft_delete_checkin`
   - `list_checkin_edits`
   - `get_habit_summary` (per-habit totals, last activity, 7-day progress)
3. Domain APIs:
   - `PATCH /v1/habits/{id}`
   - `POST /v1/habits/{id}/pause`
   - `POST /v1/habits/{id}/resume`
   - `POST /v1/habits/{id}/end`
   - `POST /v1/habits/{id}/delete`
   - `POST /v1/habits/{id}/restore`
   - `GET /v1/checkins/{id}`
   - `PATCH /v1/checkins/{id}`
   - `POST /v1/checkins/{id}/delete`
   - `GET /v1/checkins/{id}/edits`
   - `GET /v1/reports/habit-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w8` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for scale bounds, skipped fields, and status
     transitions
   - ignored Postgres integration test for habit lifecycle + check-in edit
     history + habit summary
7. `scripts/helix_well_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Consent engine, care circles, coaching agents, pattern analysis.
- Goals, routines, journals, device/clinical adapters (FHIR/Open mHealth).
- Subject-scoped privacy enforcement at the data boundary (spec P0; belongs
  to the Foundation Integrity/category program, not this depth packet).
- Permanent deletion and the 30-day recovery bin workflow.
- Web UI changes.
