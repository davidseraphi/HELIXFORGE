# HelixCore sovereign-ready checklist

## Definition

**Sovereign-ready** means HelixCore can be operated as a self-hosted foundation with:

- Fail-closed auth outside local
- Durable Postgres + NATS JetStream
- Tenant isolation on audit/agents
- Signed audit chain (HMAC) when secret set
- Shared rate limits across replicas
- Container + Helm deploy path without default secrets
- Proof via deep smoke + unit tests

It does **not** mean multi-region active-active, Stripe marketplace, or HelixPulse cluster.

## Required env (non-local)

| Variable | Purpose |
|----------|---------|
| `HELIX_ENV` | `staging` / `prod` (not `local`) |
| `DATABASE_URL` | Required |
| `NATS_URL` | Required |
| `HELIX_VAULT_MASTER_KEY` | Required, strong |
| `HELIX_AUDIT_HMAC_SECRET` | Audit entry signatures |
| `HELIX_AUDIT_GENESIS` | Optional deployment-specific genesis |
| `HELIX_CORS_ORIGINS` | Explicit allowlist |
| `HELIX_WEBHOOK_SECRET` | Billing webhooks |
| `HELIX_HYDRA_INTROSPECT_CLIENT_ID/SECRET` | Hydra introspect auth |

**Never** set `HELIX_ALLOW_DEV_HEADERS` outside local.

## Local proof

```powershell
docker compose up -d postgres nats minio minio-init
powershell -File scripts/dev-core.ps1
# wait for healthz 8080-8085
pwsh scripts/helixcore_deep_smoke.ps1
```

## Unit proof

```powershell
rustup run stable-x86_64-pc-windows-msvc cargo test -p auth_client -p vault_client -p billing_client -p nats_client -p audit_log -p service_kit -p shared_core
```

## Deploy proof

```bash
docker build -t helixforge/helix-core:0.1.0 .
helm lint infra/helm/helix-core --set secrets.databaseUrl=... --set secrets.vaultMasterKey=...
helm template infra/helm/helix-core --set secrets.databaseUrl=... --set secrets.vaultMasterKey=...
```

## Attestation API

`GET /v1/compliance/summary` (observability-service, AuditRead) returns control flags including:

- `audit_hmac_signatures`
- `nats_jetstream` / `bus_mode`
- `shared_rate_limit`
- `postgres_durable`
- `fail_closed_auth`
