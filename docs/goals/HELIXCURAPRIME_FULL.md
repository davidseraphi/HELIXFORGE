# HELIXCURAPRIME-FULL

Move HelixCura Prime from thin durable scaffold to full second-wave depth
(catalog order 13, port `8113`).

## Scope

This packet closes the core care-case + note lifecycle gaps in the current
scaffold: a case active/discharge lifecycle with a discharge guard, a note
sign/void lifecycle with signed-note immutability, and a cura summary report.

## Definition of done

1. Migration `0048_cura_depth.sql` adds:
   - `activated_at`, `discharged_at`, `deleted_at` lifecycle columns on
     `cura.care_cases`
   - `updated_at`, `signed_at`, `voided_at`, `deleted_at` columns on
     `cura.notes`
   - Backfill of legacy scaffold note status `open` to `draft`
   - Partial active-case and active-note indexes
2. `CuraRepo` gains:
   - `update_case`, `activate_case`, `discharge_case` (rejected while draft
     notes remain), `reopen_case`, `soft_delete_case`, `restore_case`
   - `update_note` (rejected once signed or voided), `sign_note`,
     `void_note`, `soft_delete_note`, `restore_note` (all verified against
     the parent case)
   - `get_cura_summary` (per-case note counts by status)
3. Domain APIs:
   - `PATCH /v1/care_cases/{id}`
   - `POST /v1/care_cases/{id}/activate`
   - `POST /v1/care_cases/{id}/discharge`
   - `POST /v1/care_cases/{id}/reopen`
   - `POST /v1/care_cases/{id}/delete`
   - `POST /v1/care_cases/{id}/restore`
   - `PATCH /v1/care_cases/{id}/notes/{note_id}`
   - `POST /v1/care_cases/{id}/notes/{note_id}/sign`
   - `POST /v1/care_cases/{id}/notes/{note_id}/void`
   - `POST /v1/care_cases/{id}/notes/{note_id}/delete`
   - `POST /v1/care_cases/{id}/notes/{note_id}/restore`
   - `GET /v1/reports/cura-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w13` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for case/note status transitions
   - ignored Postgres integration test for the discharge guard, signed-note
     immutability, case/note lifecycle, and cura summary
7. `scripts/helix_cura_prime_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29647567869`
- Smoke script: `scripts/helix_cura_prime_smoke.ps1`
- Unit tests: `2/2 PASS` (plus one ignored Postgres integration test)

## Out of scope

- Clinical data models, care plans, orders, scheduling, consent frameworks,
  regulated-record compliance.
- Web UI changes.
