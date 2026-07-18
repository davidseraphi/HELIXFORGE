# HELIXPULSE-FULL

Move HelixPulse from scaffold-only to full second-wave depth (catalog
order 21, port `8121`). The deferral precondition is met: products 1–20 are
all at second-wave depth and CI-proven.

## Scope

This packet gives HelixPulse its first durable domain foundation: monitors
and incidents with full lifecycles, plus a pulse summary report. The
Redis-class cluster engine stays deferred — this packet is the durable
product slice, not the cluster.

## Definition of done

1. Migration `0056_pulse_depth.sql` creates the `pulse` schema:
   - `pulse.monitors` with `activated_at`, `paused_at`, `deleted_at`
     lifecycle columns
   - `pulse.incidents` (FK to monitors) with `acknowledged_at`,
     `resolved_at`, `deleted_at` columns
   - Tenant and partial active indexes
2. New `PulseRepo` (`crates/helix-db/src/pulse.rs`):
   - `create_monitor`, `list_monitors`, `get_monitor`, `update_monitor`,
     `activate_monitor`, `pause_monitor` (rejected while open incidents
     remain), `resume_monitor`, `soft_delete_monitor`, `restore_monitor`
   - `create_incident`, `list_incidents`, `update_incident`,
     `acknowledge_incident`, `resolve_incident`, `soft_delete_incident`,
     `restore_incident` (all verified against the parent monitor)
   - `get_pulse_summary` (per-monitor incident counts by status)
3. Domain APIs:
   - `GET/POST /v1/monitors`, `GET/PATCH /v1/monitors/{id}`
   - `POST /v1/monitors/{id}/activate`
   - `POST /v1/monitors/{id}/pause`
   - `POST /v1/monitors/{id}/resume`
   - `POST /v1/monitors/{id}/delete`
   - `POST /v1/monitors/{id}/restore`
   - `GET/POST /v1/monitors/{id}/incidents`
   - `PATCH /v1/monitors/{id}/incidents/{incident_id}`
   - `POST /v1/monitors/{id}/incidents/{incident_id}/acknowledge`
   - `POST /v1/monitors/{id}/incidents/{incident_id}/resolve`
   - `POST /v1/monitors/{id}/incidents/{incident_id}/delete`
   - `POST /v1/monitors/{id}/incidents/{incident_id}/restore`
   - `GET /v1/reports/pulse-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w21` and capability planes.
   The `/v1/pulse/vision`, `/v1/pulse/cluster`, and `/v1/pulse/capabilities`
   informational endpoints stay.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for monitor/incident status transitions
   - ignored Postgres integration test for the pause guard, monitor
     lifecycle, incident lifecycle, and pulse summary
7. `scripts/helix_pulse_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- The Redis-class cluster engine (p3_cluster), embedded KV, protocol
  subsets, multi-region. Remains deferred per `docs/BUILD_ORDER.md`.
- Web UI changes.
