# HelixCore smoke (local)

```powershell
cd C:\Users\divin\PROJECTS\HELIXFORGE
docker compose up -d postgres nats minio minio-init
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
$env:HELIX_ENV = "local"
$env:DATABASE_URL = "postgres://helix:helix@127.0.0.1:55432/helixforge"
$env:NATS_URL = "nats://127.0.0.1:4222"

# User-owned terminals:
cargo run -p gateway
cargo run -p auth_adapter
cargo run -p agent_hub
cargo run -p vault_service
cargo run -p billing_service
cargo run -p observability_service
```

## Checks

```powershell
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }

# Gateway
Invoke-RestMethod http://127.0.0.1:8080/healthz
(Invoke-RestMethod http://127.0.0.1:8080/v1/catalog).data.Count   # 20
Invoke-RestMethod http://127.0.0.1:8080/v1/me -Headers $h
Invoke-RestMethod http://127.0.0.1:8080/v1/core/status -Headers $h

# Auth
Invoke-RestMethod http://127.0.0.1:8085/v1/auth/health

# Audit: rehash if needed (local only), then verify
# The HTTP rehash endpoint was removed; use the restricted operator CLI.
$env:HELIX_AUDIT_REHASH_APPROVED = "1"
cargo run -q -p helix_db --bin helix-audit-rehash -- --approve
Invoke-RestMethod http://127.0.0.1:8084/v1/audit/verify -Headers $h   # verified: true

# Metrics export
Invoke-WebRequest http://127.0.0.1:8084/v1/metrics/prometheus -Headers $h

# Vault put/get
$tid = (Invoke-RestMethod http://127.0.0.1:8080/v1/me -Headers $h).data.tenant_id
$put = @{ name = "smoke"; value_b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("x")) } | ConvertTo-Json
Invoke-RestMethod "http://127.0.0.1:8082/v1/tenants/$tid/secrets" -Method POST -Headers $h -Body $put

# Agent run
$body = @{ agent = "platform-orchestrator"; input = @{ tools = @("echo"); args = @{ ping = 1 } } } | ConvertTo-Json -Depth 5
Invoke-RestMethod http://127.0.0.1:8081/v1/agents/run -Method POST -Headers $h -Body $body
```

## Deep checks (HELIXCORE-FULL)

```powershell
$env:HELIX_VAULT_MASTER_KEY = "helixforge-deep-test-master-key-32b"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }

# Scope deny: X-Helix-Dev-Scopes: read on POST agents/run -> 403
# Residency deny: service HELIX_DATA_RESIDENCY=eu-west + X-Helix-Dev-Residency: us-east -> 403
# AES vault put/get + POST /v1/tenants/{tid}/objects
# Billing: GET /v1/plans, PUT plan, GET summary (PgPlanStore survives restart)
# Agents: multi-tool run; POST /v1/agents/product/helix-collab
# Edge: /v1/core/status edge_mode=gateway_proxy; /core/auth proxy
# GET /v1/core/health bus=nats vault_crypto=postgres-aes-gcm ok=true
```

## Tests

```powershell
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
cargo test --workspace
```
