# HELIXSYNTHBIO-FULL

Move HelixSynthBio from thin durable scaffold to full second-wave depth
(catalog order 11, port `8111`).

## Scope

This packet closes the core design + sim lifecycle gaps in the current
scaffold: a design review/approval lifecycle with an approval guard, a sim
run lifecycle, and a synthbio summary report.

## Definition of done

1. Migration `0046_synthbio_depth.sql` adds:
   - `submitted_at`, `approved_at`, `deleted_at` lifecycle columns on
     `synthbio.designs`
   - `updated_at`, `started_at`, `completed_at`, `deleted_at` columns on
     `synthbio.sims`
   - Partial active-design and active-sim indexes
2. `SynthbioRepo` gains:
   - `update_design`, `submit_design`, `approve_design` (requires at least
     one completed sim), `return_design`, `soft_delete_design`,
     `restore_design`
   - `update_sim`, `start_sim`, `complete_sim`, `fail_sim`,
     `soft_delete_sim`, `restore_sim` (all verified against the parent design)
   - `get_synthbio_summary` (per-design sim counts by status)
3. Domain APIs:
   - `PATCH /v1/designs/{id}`
   - `POST /v1/designs/{id}/submit`
   - `POST /v1/designs/{id}/approve`
   - `POST /v1/designs/{id}/return`
   - `POST /v1/designs/{id}/delete`
   - `POST /v1/designs/{id}/restore`
   - `PATCH /v1/designs/{id}/sims/{sim_id}`
   - `POST /v1/designs/{id}/sims/{sim_id}/start`
   - `POST /v1/designs/{id}/sims/{sim_id}/complete`
   - `POST /v1/designs/{id}/sims/{sim_id}/fail`
   - `POST /v1/designs/{id}/sims/{sim_id}/delete`
   - `POST /v1/designs/{id}/sims/{sim_id}/restore`
   - `GET /v1/reports/synthbio-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w11` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for design/sim status transitions
   - ignored Postgres integration test for the approval guard, design
     lifecycle, sim lifecycle, and synthbio summary
7. `scripts/helix_synthbio_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Risk engine, biological design model, lineage model, analysis engine,
  safety case.
- Web UI changes.
