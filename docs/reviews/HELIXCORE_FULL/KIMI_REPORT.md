# Kimi full HelixCore review

**Goal:** HELIXCORE-FULL  
**Generated:** 2026-07-15  
**Reviewers:** independent capability-focused subagents (auth, data-plane, vault/billing/audit/observability, agents, gateway/infra)  

---

## Executive summary

HelixCore is a well-structured **local-dev scaffold** with strong crate-level
foundations, but it is **not a fully built sovereign foundation** yet. The
existing `HELIXCORE-FULL.md` status of "SOVEREIGN-READY (local proof green)"
overstates readiness: many capabilities exist only as libraries, migrations, or
partial routes and are not wired into running services or a deployable
infrastructure path.

**Verdict:** `NOT_COMPLETE` (local proof `PASS_WITH_FOLLOWUPS`).

The highest-leverage gaps are:

1. **Data-plane capabilities are write-only.** Outbox, durable jobs, and the
   recovery bin have crates/migrations but no dispatcher, worker, or HTTP
   surface.
2. **Agent execution is in-memory.** `agent-hub` does not use the durable job
   layer; runs disappear on restart.
3. **Auth trust boundary has a hole.** Kratos principal parsing falls back to
   user-controlled `traits` for `tenant_id` / `scopes`.
4. **Infrastructure is not deployable.** Terraform modules are empty placeholders;
   the Helm migration job cannot run because migrations are not mounted.
5. **Gateway proxy routes are open.** `/p/*` and `/core/*` do not enforce auth.

The honest next milestone is to close the P0 gaps below and re-run the deep
smoke plus a non-dev Ory / cluster smoke before claiming `HELIXCORE-FULL`.

---

## Capability matrix

| Capability | Status | Evidence | Notes |
|---|---|---|---|
| **AetherID / auth** | PARTIAL | `crates/auth-client`, `services/auth-adapter`, `crates/service-kit/src/middleware.rs`, `crates/shared-core/src/tenancy.rs` | Hybrid provider, dev fallback, tenant RLS, API keys, and gateway enterprise routes exist. Kratos `traits` fallback is a trust-boundary hole; production Ory is dev-only. |
| **Data plane / durability** | PARTIAL | `crates/helix-db`, `crates/nats-client`, `crates/shared-core/src/hash.rs` | Postgres, migrations, audit chain, typed IDs, NATS JetStream, health labels are real. Outbox has no dispatcher; durable jobs have no worker/service routes; RLS pinning is inconsistent; recovery bin has no HTTP surface. |
| **Vault** | DONE / PARTIAL | `crates/vault-client`, `services/vault-service`, `crates/helix-db/src/vault_pg.rs` | AES-GCM envelope, tenant DEK, lazy re-encrypt, and tenant isolation are done. `HttpKms` remote derivation is TODO; MinIO object handling swallows errors; dev defaults in operational code. |
| **Billing** | DONE / PARTIAL | `crates/billing-client`, `services/billing-service`, `crates/helix-db/src/meter_pg.rs` | Metering, idempotency, plans, tenant summary, local payments are done. Signed webhooks are a stub; Stripe confirm not wired; local bypasses in production code. |
| **Audit** | DONE / PARTIAL | `crates/audit-log`, `crates/helix-db/src/audit_pg.rs`, `crates/helix-db/src/bin/helix_audit_rehash.rs` | Hash chain, canonical JSON, HMAC, transactional append, and rehash CLI are done. WORM archive verification is a no-op TODO. |
| **Observability** | DONE / PARTIAL | `crates/observability`, `services/observability-service`, `crates/service-kit/src/health.rs` | Metrics registry, Prometheus endpoint, readiness aggregation, compliance summary are done. OTLP path is a hand-built span probe without context propagation or sampling. |
| **Agents** | PARTIAL | `crates/agent-framework`, `services/agent-hub` | Tool registry, timeouts, audit/billing of runs, tenant isolation exist. Runtime is in-memory; no durable job wiring; no mid-run checkpoints/cancellation; no service tests. |
| **Gateway** | DONE / PARTIAL | `services/gateway/src/main.rs` | Reverse proxy, WebSocket proxy, catalog/state, enterprise platform routes, rate/body limits, CORS, request ID, security headers are done. Proxy routes do not enforce auth; TLS/mTLS docs are not implemented. |
| **Infrastructure** | PARTIAL / MISSING | `Dockerfile`, `docker-compose.yml`, `infra/helm/helix-core`, `infra/argocd`, `infra/terraform` | Dockerfile, Compose, Helm shape, ArgoCD apps, and CI validation exist. Terraform modules are empty placeholders; Helm migration job cannot mount migrations; NetworkPolicy is permissive. |

---

## Findings

### P0 — Data-plane capabilities are not wired to services

- **Issue:** `helix_db.outbox` has no dispatcher, `JobRepo` has no worker or
  service routes, and `GovernanceRepo` recovery-bin functions have no HTTP
  surface.
- **Path:** `crates/helix-db/src/outbox.rs`, `crates/helix-db/src/jobs.rs`,
  `crates/helix-db/src/governance.rs`, `services/gateway/src/main.rs`.
- **Fix:**
  1. Implement an `OutboxRelay` that polls `processed_at IS NULL` rows,
     publishes to NATS, and dead-letters failures.
  2. Start a `JobWorker` in `agent-hub` or a new `job-runner` service; expose
     `POST /v1/jobs`, `GET /v1/jobs/{id}`, cancel endpoints.
  3. Add gateway recovery-bin routes and integrate `soft_delete_in_tx` into
     actual resource deletes via `AtomicWork`.
- **Retest:** `cargo test -p helix_db` plus a new integration test proving
  outbox dispatch and job lifecycle end-to-end.

### P0 — Agent execution is not durable

- **Issue:** `AgentRuntime::run` is an in-process sequential loop. Runs are lost
  on restart and cannot be cancelled mid-flight.
- **Path:** `crates/agent-framework/src/lib.rs`, `services/agent-hub/src/main.rs`.
- **Fix:** Drive agent runs through `job_engine::JobWorker`/`helix_db::JobRepo`;
  emit per-step checkpoints/heartbeats to Postgres and NATS; implement
  cooperative cancellation tokens.
- **Retest:** Add agent endpoints to `scripts/helixcore_deep_smoke.ps1` and
  restart `agent-hub` mid-run to prove recovery.

### P0 — Auth trust boundary trusts user-controlled Kratos traits

- **Issue:** `principal_from_kratos` reads `tenant_id`, `scopes`, and
  `residency_region` from `traits` when `metadata_public` is absent.
- **Path:** `crates/auth-client/src/lib.rs:463-514`.
- **Fix:** Read security-critical claims **only** from `metadata_public`. Fail
  closed in strict mode when `metadata_public` is missing. Remove the `traits`
  fallback.
- **Retest:** Add a negative test where a user submits malicious traits and is
  rejected.

### P0 — Infrastructure is not deployable

- **Issue:** Terraform modules contain only variables/outputs. The Helm
  migration job does not mount `crates/helix-db/migrations`. Gateway proxy
  routes do not enforce auth.
- **Path:** `infra/terraform/modules/*`,
  `infra/helm/helix-core/templates/migration-job.yaml`,
  `services/gateway/src/main.rs:1106-1327`.
- **Fix:**
  1. Implement real Terraform modules (VPC/EKS/RDS or self-hosted k3s).
  2. Mount migrations into the Helm migration job and validate a full
     `helm install`.
  3. Add `RequireAuth` middleware to `/p/*` and `/core/*` with upstream header
     injection.
- **Retest:** `terraform plan` in CI; `helm install` against a kind/k3d cluster;
  unauthorized request to `/p/helix-flow` returns 401.

### P1 — Production-code dev defaults and bypasses

- **Issue:** Hardcoded MinIO credentials, local vault master key defaults, and
  `HELIX_WEBHOOK_ALLOW_UNSIGNED` bypasses live in code paths that could be used
  in production.
- **Path:** `crates/vault-client/src/minio.rs`, `services/billing-service/src/main.rs`.
- **Fix:** Remove all production-code defaults; gate local bypasses behind
  explicit `local_dev_unsafe_*` flags that log loud warnings.
- **Retest:** Run services without any `.env.local` and confirm they fail
  closed.

### P1 — Audit WORM archive verification is unimplemented

- **Issue:** `ObjectStoreArchiveSink::verify_archive` and
  `latest_archived_seq` are no-op TODOs.
- **Path:** `crates/helix-db/src/audit_archive.rs:52-63`.
- **Fix:** Implement object-listing verification against a checkpoint sequence
  table and surface archive integrity in the compliance summary.
- **Retest:** `cargo test` for archive verification plus operator rehash.

### P1 — OTLP observability is a manual span probe

- **Issue:** `export_span` hand-builds OTLP/HTTP JSON without trace context,
  metrics, or sampling.
- **Path:** `crates/observability/src/lib.rs:101-187`.
- **Fix:** Integrate the official `opentelemetry-otlp` SDK or demote the claim
  to "OTLP span probe" in goal docs.
- **Retest:** Verify distributed trace context propagation across gateway and
  a product service.

### P2 — RLS tenant context is not pinned consistently

- **Issue:** Only `MembershipRepo` calls `set_tenant_context`; other repos rely
  on application filters.
- **Path:** `crates/helix-db/src/*`.
- **Fix:** Call `set_tenant_context` at the start of every tenant-scoped
  transaction or remove the RLS claim.
- **Retest:** Test that a query issued with the wrong tenant context returns
  zero rows at the DB level.

---

## Recommended build order to reach FULL

1. **Close the auth trust-boundary hole.** Remove Kratos `traits` fallback and
   enforce `metadata_public` claims. This is a security blocker.
2. **Wire the data plane.** Outbox relay → job worker → recovery-bin HTTP
   surface. Prove each with integration tests.
3. **Make agents durable.** Convert `agent-hub` runs to `JobWorker` jobs with
   checkpoints and cancellation.
4. **Make infrastructure real.** Implement Terraform modules, fix Helm
   migrations, enforce auth on proxy routes, and validate in a real cluster.
5. **Harden operational defaults.** Remove hardcoded dev credentials/bypasses;
   implement audit WORM verification; replace or clarify OTLP path.
6. **Run the full re-proof.** Local deep smoke, non-dev Ory smoke, cluster
   install smoke, and restore roundtrip CI.

---

## Retest commands

```bash
# Local quality gates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
pnpm --filter @helixforge/console typecheck

# Local data-plane proof
docker compose up -d postgres nats minio minio-init
cargo test -p helix_db  # includes atomic/outbox/job/governance tests

# Deep smoke (after starting core services)
powershell -File scripts/helixcore_deep_smoke.ps1

# Non-dev Ory smoke (after auth hardening)
docker compose --profile ory up -d
# run auth-adapter integration tests / deep smoke with auth_mode=kratos+hydra

# Cluster proof (after infra fixes)
helm lint infra/helm/helix-core
terraform -chdir=infra/terraform/environments/dev plan
kind create cluster
helm install helix-core infra/helm/helix-core -f infra/helm/helix-core/values.yaml
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=gateway

# Installer/restore roundtrip (CI)
# See .github/workflows/installer.yml
```

---

## Bottom line

HelixCore has a solid foundation of crates, migrations, and local proofs, but
`HELIXCORE-FULL` is not yet earned. The path forward is to stop treating
library code and dev-only configuration as "done" and prove each capability
through running services, integration tests, and a real cluster deployment.
