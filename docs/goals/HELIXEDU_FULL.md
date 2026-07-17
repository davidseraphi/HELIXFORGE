# HELIXEDU-FULL

Move HelixEdu from durable scaffold to full second-wave depth (catalog
order 6, port `8106`).

## Scope

This packet closes the core course + enrollment lifecycle gaps in the current
scaffold: soft-delete course lifecycle, publish/unpublish, enrollment guards,
and durable progress history.

## Definition of done

1. Migration `0041_edu_depth.sql` adds:
   - `deleted_at` soft-delete column on `edu.courses` with partial active index
   - `edu.progress_history` side table
2. `EduRepo` enforces:
   - courses list/get exclude deleted courses
   - `update_course`, `soft_delete_course`, `restore_course`, `unpublish_course`
   - enrollments only allowed into `published` courses
   - `withdraw_enrollment`
   - `update_progress` records a history row, sets/clears `completed_at`, and
     transitions `status` between `active` and `completed`
3. Domain APIs:
   - `PATCH /v1/courses/{id}`
   - `POST /v1/courses/{id}/delete`
   - `POST /v1/courses/{id}/restore`
   - `POST /v1/courses/{id}/unpublish`
   - `GET /v1/enrollments/{id}`
   - `POST /v1/enrollments/{id}/withdraw`
4. `GET /v1/domain/status` returns `phase: wave2_w4` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - progress_pct boundary validation
   - course input validation
   - ignored data-plane test for progress history persistence
7. `scripts/helix_edu_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- Implementation commit: `ec9b01e`
- CI run: `29607668365` — all green

## Out of scope

- Lessons, modules, assessments, rubrics, submissions, feedback, credentials,
  learner UI, offline sync, mastery graph, and certification issuance.
