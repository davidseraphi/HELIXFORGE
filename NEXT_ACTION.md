# Next action

## Latest: HELIXCORE-FULL P0–P2 hardening closed locally

The Kimi full HelixCore review is recorded at
`docs/reviews/HELIXCORE_FULL/KIMI_REPORT.md`.

**Verdict:** `PASS_WITH_FOLLOWUPS` locally; the new `scripts/verify_helixcore_full.ps1`
now passes end-to-end (core services → deep smoke → export → isolated restore →
restored DB tests → cleanup). Awaiting final CI re-proof of the installer /
live-restore roundtrip for a clean `PASS`.

All P0 blockers, P1 hardening items, and the P2 RLS gap identified in the review
are now addressed in code and pass local `cargo fmt`, `cargo clippy`,
`cargo test --workspace --all-features`, and `helm template`.

### Closed in this session

1. **AetherID auth parsing** — `auth-client` no longer falls back to Kratos
   `traits`; `tenant_id`, `scopes`, and `residency_region` are read only from
   `metadata_public`.
2. **Gateway proxy auth** — `/p/{slug}/*`, `/core/{service}/*`, and WebSocket
   proxy routes now require `RequireAuth` and inject
   `x-helix-tenant-id/user-id/scopes/residency` upstream.
3. **Outbox relay wired** — `service_kit::spawn_outbox_relay` is running in the
   gateway, polling `helix_core.outbox` into NATS.
4. **Durable jobs / agent-job integration** — `agent-hub` queues agent runs as
   durable `agent.run` jobs when Postgres is available; a `JobWorker` executes
   them, persists `AgentRun` records, and exposes `/v1/agents/jobs/{id}` for
   polling. Synchronous in-memory execution remains as a `?synchronous=true`
   fallback.
5. **Recovery bin HTTP surface** — gateway exposes
   `/v1/recovery-bin`, `/{id}`, `/{id}/restore`, and `/{id}/permanent-delete`
   with tenant isolation and audit.
6. **Helm migration job** — added `helix-migrate` binary to `helix_db`; the
   pre-install/pre-upgrade Job now reuses the application image and runs
   embedded migrations, eliminating the broken migration-file volume mount.
7. **Terraform deployability** — replaced placeholder modules with real
   Helm-based deployments for Postgres (Bitnami), NATS (official), and MinIO
   (Bitnami) in `infra/terraform/environments/prod`.
8. **Production-code dev defaults removed** — `shared_core::CoreConfig` now
   fails closed on missing secrets/endpoints outside local/dev. Local defaults
   are gated behind `HELIX_LOCAL_DEV_UNSAFE=1` with loud stderr warnings.
   `vault-client` no longer embeds MinIO credentials.
9. **Webhook unsigned bypass hardened** — the unsigned webhook path now requires
   `local_dev_unsafe` + `HELIX_ENV=local` + `HELIX_WEBHOOK_ALLOW_UNSIGNED=1` and
   logs a `tracing::warn` when active.
10. **Audit WORM verification implemented** — `ObjectStoreArchiveSink` lists
    archived objects via S3 ListObjectsV2 and verifies that the expected
    sequence range is present.
11. **OTLP span probe improved** — the hand-built exporter now parses W3C
    `traceparent`, continues the trace, and applies deterministic sampling via
    `HELIX_OTLP_SAMPLE_RATE`. Documented as a lightweight probe, not a full SDK.
12. **RLS tenant context pinned consistently** — a shared
    `helix_db::set_tenant_context` helper is used by `MembershipRepo`,
    `TenantRepo`, `WorkspaceRepo`, `PgMetering`, `ResourceAclRepo`, and
    `PgAuditSink`. Platform-wide audit queries bypass RLS with
    `SET LOCAL row_security = off` inside their transaction.
13. **End-to-end HELIXCORE-FULL verification script** —
    `scripts/verify_helixcore_full.ps1` orchestrates the full local proof:
    starts all 6 core services, runs `helixcore_deep_smoke.ps1`, exports a
    backup, restores to an isolated compose project via
    `deploy/local/restore.override.yml`, runs `cargo test -p helix_db` against
    the restored DB, and cleans up. All steps pass locally.
14. **Backup/restore scripts hardened** — `migrate-export.ps1` uses a fresh
    `mc alias` per mirror command and a PowerShell 5-compatible hash join;
    `migrate-restore.ps1` always includes the base compose file, uses the
    restore override for isolated ports, verifies postgres readiness before
    restoring, and fails fast on any step.

### Still open: final CI re-proof

- Pushed fixes for the active CI failures:
  - `sandbox_runs_echo_step` now forces host isolation by setting
    `HELIX_CODE_ALLOW_HOST_ISOLATION=1` in the test.
  - `deploy/docker/entrypoint.sh` now accepts `helix-migrate` and
    `helix-audit-rehash` so the core smoke job can run migrations.
  - `.github/workflows/installer.yml` starts the Docker service on Windows,
    uses env-var port shifting for isolated restore, and no longer depends on
    the Docker Compose `!override` tag.
  - `install.ps1`/`install.sh` detect `docker compose` availability and fail
    closed if Postgres does not become ready.
- Next: monitor the GitHub Actions run and address any remaining failures.

## Active goal: HELIXCORE-FULL

Do not resume product 1–20 depth work until CI re-proves the full
HELIXCORE-FULL review passes.
Do not move, implement, or activate HelixAnvil code.

## Open founder decisions

- Managed-service commercial model and final custody providers (per HelixCore
  spec) — does not block G0, but must be resolved before G1 capability broker.
- HelixAnvil canonical home is `projects/helix-anvil`; sequencing remains
  portfolio-last.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCORE-FULL P0–P2 hardening
is closed in code: auth hardening, gateway proxy auth/headers, outbox relay,
durable agent jobs, recovery bin routes, Helm migration job, Terraform modules,
production-code default removal, audit WORM verification, OTLP trace context /
sampling, and consistent RLS tenant context pinning. Local `cargo fmt`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --all-features`, `helm template`, and
`scripts/verify_helixcore_full.ps1` all pass.
Next step is final CI re-proof of the installer / live-restore roundtrip.
Do not resume product depth or activate HelixAnvil until CI passes.
```
