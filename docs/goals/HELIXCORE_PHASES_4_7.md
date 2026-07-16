# HelixCore Phases 4–7 — Sovereign Hardening & Production Deploy Path

## Status: IMPLEMENTATION COMPLETE

All Rust code compiles and tests pass (`cargo test --workspace`).
Clippy is clean for the changed HelixCore crates; remaining warnings are pre-existing product-depth warnings outside this scope.

> Runtime deep smoke was executed in this session after the user explicitly
> asked to run it. All 6 core services started and `scripts/helixcore_deep_smoke.ps1`
> returned **DONE (deep smoke OK)** with `audit_chain_verified=true` after the
> break-glass operator rehash.

---

## Phase 4 — AetherID / auth hardening

- `auth-client`: strict-mode Hydra introspection; `principal_from_kratos`/`principal_from_oauth` fail closed outside `local` for missing tenant/scopes; residency allowlist validation.
- `service-kit` middleware: `x-helix-dev-scopes`/`x-helix-dev-residency` overrides gated on `HELIX_ENV=local && HELIX_ALLOW_DEV_HEADERS=1` plus optional `HELIX_DEV_MASTER_KEY` constant-time check.
- `auth-adapter`: `/v1/session/dev-login` restricted to local; `HELIX_DEV_LOGIN_TOKEN` enforced; Hydra admin creds required outside local for `/v1/oidc/clients`; audit events for OIDC introspect/client-create.
- Ory identity schema hardened with `org_id`, `roles`, `email_verified`, constrained `tenant_id` UUID and `residency_region` enum.
- `kratos.yml`/`hydra.yml` secrets use `${...:?required}` env substitution.

## Phase 5 — Tenant isolation / service-kit hardening

- `ServiceBuilder::base_router` refactored so global layers (request-id, rate-limit, body-limit, trace, CORS, security headers) apply to domain routes.
- All 6 core services + 21 product API `main.rs` updated to the new `base_router(...).merge(...)` pattern; product custom-serve blocks replaced with `service_kit::serve_with_shutdown`.
- CORS defaults to deny/empty origins outside local unless `HELIX_CORS_ORIGINS` is set; explicit allow-header list.
- Rate limiter skips `/healthz`, `/readyz`, `/v1/meta`, `/metrics`; key includes method + path prefix; periodic TTL cleanup.
- `TenantStatus::parse` fallback changed from `Active` to `Suspended` (fail-closed).
- `AgentRunStore::get_for_tenant` added; `agent-hub`/`product.rs` no longer falls through to in-memory runtime when DB is configured.
- `audit_recent` tenant-scoped by default; Platform global view requires `Platform + AuditRead`.
- Removed HTTP `/v1/audit/rehash`; created operator CLI `crates/helix-db/src/bin/helix_audit_rehash.rs`.

## Phase 6 — Gateway / data-plane hardening

### Gateway proxy

- `services/gateway/src/main.rs` refactored to:
  - Config-based upstream discovery (`HELIX_UPSTREAM_<SLUG>`, `HELIX_PRODUCT_HOST`, `HELIX_PRODUCT_UPSTREAM_BASE`).
  - Streaming request/response bodies via `reqwest::Body::wrap_stream` / `axum::Body::from_stream`.
  - Idempotent-only retries (`GET/HEAD/OPTIONS/DELETE/TRACE`) with `proxy_retry_count` + `proxy_retry_backoff_ms`.
  - Hop-by-hop header stripping and `x-forwarded-for`/`x-forwarded-host`/`x-forwarded-prefix` injection.
  - `reqwest` client timeouts driven from `CoreConfig` instead of a one-off env read.

### NATS TLS, credentials & resilient JetStream

- `nats-client` now builds `async_nats::ConnectOptions` from `CoreConfig`:
  - `require_tls`, `tls_first`, `retry_on_initial_connect`, `connection_timeout`, `max_reconnects`.
  - `.creds` file auth; inline JWT+NKey auth via constructed credentials string.
  - `add_root_certificates`, `add_client_certificate` for mutual TLS.
- JetStream `HELIX_CORE` stream setup uses bounded retries (`NATS_JS_RETRY_ATTEMPTS` / `NATS_JS_RETRY_BACKOFF_MS`).
- Core-NATS fallback on JetStream publish is opt-in via `HELIX_NATS_JS_FALLBACK`.

### Postgres pool resilience

- Added `DbPoolConfig` in `shared-core` with env-driven:
  - `HELIX_DB_POOL_MAX_CONNECTIONS`, `HELIX_DB_POOL_MIN_CONNECTIONS`
  - `HELIX_DB_POOL_ACQUIRE_TIMEOUT_SECS`
  - `HELIX_DB_POOL_IDLE_TIMEOUT_SECS`, `HELIX_DB_POOL_MAX_LIFETIME_SECS`
  - `HELIX_DB_POOL_TEST_BEFORE_ACQUIRE`
- `helix_db::connect_and_migrate_with_config` applies pool tuning; `try_connect_and_migrate_with_config` used by `service-kit`.
- `helix-audit-rehash` CLI reuses the same config builder.

### Config centralization

- `CoreConfig` now owns the canonical env schema:
  - Dev controls: `HELIX_ALLOW_DEV_HEADERS`, `HELIX_DEV_LOGIN_TOKEN`, `HELIX_DEV_PLATFORM`, `HELIX_DEV_MASTER_KEY`.
  - CORS: `HELIX_CORS_ORIGINS`, `HELIX_CORS_ALLOW_CREDENTIALS`.
  - Rate limit: `HELIX_RATE_LIMIT_BACKEND`, `HELIX_RATE_LIMIT_PG_FALLBACK`.
  - Audit: `HELIX_AUDIT_HMAC_SECRET`, `HELIX_AUDIT_OPERATOR_KEY`.
  - Payment: `HELIX_PAYMENT_PROVIDER`, `STRIPE_SECRET_KEY`, `HELIX_PAYMENT_STRIPE_FORCE_SIM`.
  - Webhook: `HELIX_WEBHOOK_SECRET`, `HELIX_WEBHOOK_ALLOW_UNSIGNED`.
  - KMS: `HELIX_VAULT_KMS_MODE`, `HELIX_VAULT_KMS_URL`, `HELIX_VAULT_KMS_FALLBACK`, `HELIX_VAULT_KEK`.
  - Hydra: `HELIX_HYDRA_INTROSPECT_CLIENT_ID/SECRET`, `HELIX_HYDRA_ADMIN_CLIENT_ID/SECRET`.
- Services read these from `state.clients.config` instead of re-parsing `std::env::var`.
- `audit-log` uses a once-per-process `set_hmac_secret` driven from `CoreConfig`, eliminating split-brain HMAC config.
- `ProductService::run` no longer re-parses config; it clones the builder config before consuming it.

## Phase 7 — Production deploy path

### Dockerfile

- `Dockerfile` now forces `RUSTUP_TOOLCHAIN=stable` so the Windows-pinned `rust-toolchain.toml` (`stable-x86_64-pc-windows-msvc`) does not break Linux container builds.
- Sets `SQLX_OFFLINE=true` (compile-time query verification is not used in this codebase).
- Builds all 6 core release binaries and copies them into a `debian:bookworm-slim` runtime image with non-root user.

### Helm

- Added `infra/helm/helix-core/templates/ingress.yaml` with TLS and className support.
- Added `infra/helm/helix-core/templates/migration-job.yaml` as a Helm pre-install/pre-upgrade hook using `sqlx-cli`.
- Expanded `values.yaml` with sections for `dbPool`, `nats` TLS/creds, `cors`, `rateLimit`, `audit`, `payment`, `webhook`, `kms`, `dev`, `ingress`, and `migration`.
- Added environment-specific value files:
  - `values-prod.yaml` (replicas=3, ingress, external secrets, strict CORS)
  - `values-staging.yaml` (replicas=1, ingress, external secrets)
- Deployment template wires all new config values and supports `extraEnv`.

### ArgoCD

- `infra/argocd/applications/helix-core.yaml` remains the base app.
- Added `helix-core-staging.yaml` (tracks `develop`, namespace `helix-core-staging`).
- Added `helix-core-prod.yaml` (tracks `main`, namespace `helix-core-prod`).

### Terraform

- Added stub modules for sovereign infrastructure:
  - `infra/terraform/modules/kubernetes/main.tf`
  - `infra/terraform/modules/postgres/main.tf`
  - `infra/terraform/modules/nats/main.tf`
  - `infra/terraform/modules/minio/main.tf`
- Added `infra/terraform/environments/prod/main.tf` composing the modules.

### CI

- `.github/workflows/ci.yml` extended with:
  - `helm-template` job rendering the prod overlay.
  - `terraform-validate` job validating `dev` and `prod` environments.
  - `smoke` job that starts core infrastructure, applies migrations, builds the Docker image, and health-checks the gateway.

---

## Verification commands

```bash
# Compile & quality
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets

# Helm
helm lint infra/helm/helix-core --set secrets.databaseUrl=... --set secrets.vaultMasterKey=...
helm template helix-core-prod infra/helm/helix-core \
  -f infra/helm/helix-core/values.yaml \
  -f infra/helm/helix-core/values-prod.yaml \
  --set secrets.databaseUrl=... --set secrets.vaultMasterKey=...

# Runtime deep smoke (user-owned CMD)
docker compose up -d postgres nats minio minio-init
# start core services on 8080-8085
.\scripts\helixcore_deep_smoke.ps1
```

## Next step after smoke

Return to the **SECOND_WAVE** product-depth roadmap (`docs/SECOND_WAVE.md`):
next product is **W2 HelixInsights (8104)**.
