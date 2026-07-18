# HELIXVITAPRIME-FULL

Move HelixVita Prime from thin durable scaffold to full second-wave depth
(catalog order 18, port `8118`).

## Scope

This packet closes the core study + cohort lifecycle gaps in the current
scaffold: a study recruit/complete/terminate lifecycle with a completion
guard, a cohort enroll/withdraw lifecycle, and a vita summary report.

## Definition of done

1. Migration `0053_vita_depth.sql` adds:
   - `recruiting_at`, `completed_at`, `terminated_at`, `deleted_at` lifecycle
     columns on `vita.studies`
   - `updated_at`, `enrolled_at`, `withdrawn_at`, `deleted_at` columns on
     `vita.cohorts`
   - Backfill of legacy scaffold cohort status `open` to `draft`
   - Partial active-study and active-cohort indexes
2. `VitaRepo` gains:
   - `update_study`, `recruit_study`, `complete_study` (rejected while draft
     cohorts remain), `terminate_study`, `soft_delete_study`,
     `restore_study`
   - `update_cohort`, `enroll_cohort`, `withdraw_cohort`,
     `soft_delete_cohort`, `restore_cohort` (all verified against the parent
     study)
   - `get_vita_summary` (per-study cohort counts by status)
3. Domain APIs:
   - `PATCH /v1/studies/{id}`
   - `POST /v1/studies/{id}/recruit`
   - `POST /v1/studies/{id}/complete`
   - `POST /v1/studies/{id}/terminate`
   - `POST /v1/studies/{id}/delete`
   - `POST /v1/studies/{id}/restore`
   - `PATCH /v1/studies/{id}/cohorts/{cohort_id}`
   - `POST /v1/studies/{id}/cohorts/{cohort_id}/enroll`
   - `POST /v1/studies/{id}/cohorts/{cohort_id}/withdraw`
   - `POST /v1/studies/{id}/cohorts/{cohort_id}/delete`
   - `POST /v1/studies/{id}/cohorts/{cohort_id}/restore`
   - `GET /v1/reports/vita-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w18` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for study/cohort status transitions
   - ignored Postgres integration test for the completion guard, study
     lifecycle, cohort lifecycle, and vita summary
7. `scripts/helix_vita_prime_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Trial protocols, consent management, participant tracking, outcomes
  analysis, regulatory submissions.
- Web UI changes.
