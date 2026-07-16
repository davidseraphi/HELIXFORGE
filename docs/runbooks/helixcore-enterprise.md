# HelixCore enterprise baseline

## Controls

| Control | Implementation |
|---------|----------------|
| AuthN | Ory Kratos (live) + dev headers (local) + service API keys |
| AuthZ | Scopes: read/write/admin/platform/audit_read |
| Residency | Fail-closed when `HELIX_DATA_RESIDENCY` ≠ principal region |
| Tenant lifecycle | active/suspended; suspended tenants get 403 |
| API keys | `hk_live_*` machine principals (`X-Helix-Api-Key`) |
| Rate limit | `HELIX_RATE_LIMIT_RPS` (default 100 outside local) |
| Body limit | `HELIX_MAX_BODY_BYTES` (default 2 MiB) |
| Security headers | nosniff, DENY frame, no-referrer |
| Strong secrets | Non-local rejects default vault master key |
| Audit | BLAKE3 chain + tenant export (NDJSON) |
| Vault | HVA3 + KMS wrap; rotation meta ledger |
| Observability | OTLP/HTTP + Prometheus + core health |
| Shutdown | Ctrl+C / SIGTERM graceful drain |

## Platform APIs (gateway)

```
GET  /v1/core/inventory
GET  /v1/platform/tenants                 # Platform
POST /v1/platform/tenants                 # Platform
POST /v1/platform/tenants/{id}/suspend
POST /v1/platform/tenants/{id}/activate
POST /v1/tenants/{tid}/api-keys           # Admin
GET  /v1/tenants/{tid}/api-keys
POST /v1/tenants/{tid}/api-keys/{id}/revoke
```

## Compliance

```
GET /v1/audit/export          # observability :8084
GET /v1/compliance/summary
GET /v1/keys/meta             # vault :8082
POST /v1/keys/rotate-meta     # Platform — records rotation
```

## Production env (minimum)

```
HELIX_ENV=prod
HELIX_VAULT_MASTER_KEY=<strong 32+ char secret>
HELIX_DATA_RESIDENCY=eu-west
HELIX_RATE_LIMIT_RPS=100
DATABASE_URL=...
NATS_URL=...
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4318
KRATOS_PUBLIC_URL=...
```

## Compose profiles

```powershell
docker compose --profile ory --profile observability up -d
```
