# HelixCode multi-instance sticky routing (HA)

HelixCode keeps some state **process-local**:

| State | Sticky key | Failure mode without sticky |
|-------|------------|-----------------------------|
| LSP sessions | `HELIX_CODE_INSTANCE_ID` + `code.lsp_session_registry` | `sticky_miss` if request hits another node |
| DAP debug sessions | process memory + `code.process_sessions` (kind=`dap`) | `sticky_miss` if wrong instance |
| Terminal sessions | process memory + `code.process_sessions` (kind=`terminal`) | `sticky_miss` if wrong instance |

## Required env per API instance

```powershell
$env:HELIX_CODE_INSTANCE_ID = "api-1"   # unique per replica
$env:HELIX_ENV = "local"                # or production
$env:DATABASE_URL = "postgres://..."
```

On open LSP / create terminal / launch debug, clients should:

1. Read `instance_id` from open/launch response or `GET /v1/lsp/status`.
2. Route subsequent requests for that session to the same replica.

## Load balancer

- Prefer **cookie or header affinity** on `x-helix-instance-id` (client echoes registry value).
- Alternative: consistent hash of `session_id` only works if sessions are in shared store (not yet for DAP/terminal).

## Health

- `GET /healthz` — liveness
- `GET /v1/domain/status` — planes + isolation
- Sticky registry: wrong instance returns clear `sticky_miss` / not found — clients should re-open.

## Production checklist

- [ ] Unique `HELIX_CODE_INSTANCE_ID` per pod
- [ ] LB sticky for LSP/debug/terminal routes under `/v1/lsp/*`, `/v1/terminals/*`, `/v1/debug/*`
- [ ] Shared Postgres + MinIO + NATS (already durable for repos/CI/MLS)
- [ ] `HELIX_ALLOW_DEV_HEADERS=0` outside local
- [ ] Webhooks: set `HELIX_CODE_WEBHOOK_ALLOW_HOSTS` allowlist; HTTPS required when not local
