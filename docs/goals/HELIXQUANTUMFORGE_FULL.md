# HELIXQUANTUMFORGE-FULL

Move HelixQuantum Forge from thin durable scaffold to full second-wave depth
(catalog order 17, port `8117`).

## Scope

This packet closes the core job + circuit lifecycle gaps in the current
scaffold: a job submit/complete/fail lifecycle with a submit guard, a circuit
validate/archive lifecycle, and a quantum summary report.

## Definition of done

1. Migration `0052_quantum_depth.sql` adds:
   - `submitted_at`, `completed_at`, `failed_at`, `deleted_at` lifecycle
     columns on `quantum.jobs`
   - `updated_at`, `validated_at`, `archived_at`, `deleted_at` columns on
     `quantum.circuits`
   - Backfill of legacy scaffold circuit status `open` to `draft`
   - Partial active-job and active-circuit indexes
2. `QuantumRepo` gains:
   - `update_job`, `submit_job` (requires at least one non-deleted circuit),
     `complete_job`, `fail_job`, `soft_delete_job`, `restore_job`
   - `update_circuit`, `validate_circuit`, `archive_circuit`,
     `soft_delete_circuit`, `restore_circuit` (all verified against the
     parent job)
   - `get_quantum_summary` (per-job circuit counts by status)
3. Domain APIs:
   - `PATCH /v1/jobs/{id}`
   - `POST /v1/jobs/{id}/submit`
   - `POST /v1/jobs/{id}/complete`
   - `POST /v1/jobs/{id}/fail`
   - `POST /v1/jobs/{id}/delete`
   - `POST /v1/jobs/{id}/restore`
   - `PATCH /v1/jobs/{id}/circuits/{circuit_id}`
   - `POST /v1/jobs/{id}/circuits/{circuit_id}/validate`
   - `POST /v1/jobs/{id}/circuits/{circuit_id}/archive`
   - `POST /v1/jobs/{id}/circuits/{circuit_id}/delete`
   - `POST /v1/jobs/{id}/circuits/{circuit_id}/restore`
   - `GET /v1/reports/quantum-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w17` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for job/circuit status transitions
   - ignored Postgres integration test for the submit guard, job lifecycle,
     circuit lifecycle, and quantum summary
7. `scripts/helix_quantum_forge_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Quantum backends, circuit compilation, simulators, hardware queues,
  error-mitigation pipelines.
- Web UI changes.
