# HELIXFORGESTUDIO-FULL

Move HelixForge Studio from thin durable scaffold to full second-wave depth
(catalog order 10, port `8110`).

## Scope

This packet closes the core app + page lifecycle gaps in the current scaffold:
app lifecycle management with a publish guard, page lifecycle management, and
a studio summary report.

## Definition of done

1. Migration `0045_studio_depth.sql` adds:
   - `published_at`, `deleted_at` lifecycle columns on `studio.apps`
   - `updated_at`, `archived_at`, `deleted_at` columns on `studio.pages`
   - Partial active-app and active-page indexes
2. `StudioRepo` gains:
   - `update_app`
   - `publish_app` (requires at least one non-deleted page)
   - `unpublish_app`
   - `soft_delete_app` / `restore_app`
   - `update_page`, `archive_page`, `reopen_page`, `soft_delete_page`,
     `restore_page` (all verified against the parent app)
   - `get_studio_summary` (per-app page counts)
3. Domain APIs:
   - `PATCH /v1/apps/{id}`
   - `POST /v1/apps/{id}/publish`
   - `POST /v1/apps/{id}/unpublish`
   - `POST /v1/apps/{id}/delete`
   - `POST /v1/apps/{id}/restore`
   - `PATCH /v1/apps/{id}/pages/{page_id}`
   - `POST /v1/apps/{id}/pages/{page_id}/archive`
   - `POST /v1/apps/{id}/pages/{page_id}/reopen`
   - `POST /v1/apps/{id}/pages/{page_id}/delete`
   - `POST /v1/apps/{id}/pages/{page_id}/restore`
   - `GET /v1/reports/studio-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w10` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for app/page status transitions
   - ignored Postgres integration test for the publish guard, app lifecycle,
     page lifecycle, and studio summary
7. `scripts/helix_forge_studio_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29643838956`
- Smoke script: `scripts/helix_forge_studio_smoke.ps1`
- Unit tests: `2/2 PASS` (plus one ignored Postgres integration test)

## Out of scope

- Canvas, typed application graph, component contracts, source parser or
  generator, data model, workflow engine, sandbox, preview, test system,
  release room.
- Web UI changes.
