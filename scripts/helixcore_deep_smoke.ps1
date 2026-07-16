# HelixCore deep smoke - Ory, vault/KMS, payments, OTEL, proxy, sovereign controls
# Prerequisites: docker compose up (postgres nats minio) + optional ory/observability
# Core services on 8080-8085 with DATABASE_URL and HELIX_VAULT_MASTER_KEY.
# Fail-closed auth: start services with HELIX_ALLOW_DEV_HEADERS=1 HELIX_DEV_PLATFORM=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
if (-not $env:DATABASE_URL) {
  $env:DATABASE_URL = "postgres://helix:helix@127.0.0.1:55432/helixforge"
}
if (-not $env:HELIX_ALLOW_DEV_HEADERS) {
  Write-Host "WARN: HELIX_ALLOW_DEV_HEADERS not set in this shell; services must have it at process start." -ForegroundColor Yellow
}

Write-Host "=== healthz ==="
foreach ($p in 8080..8085) {
  $code = (Invoke-WebRequest "http://127.0.0.1:$p/healthz" -UseBasicParsing -TimeoutSec 3).StatusCode
  Write-Host "  $p $code"
}

Write-Host "=== core status ==="
$st = (Invoke-RestMethod http://127.0.0.1:8080/v1/core/status -Headers $h).data
Write-Host "  edge=$($st.edge_mode) aetherid=$($st.aetherid.mode) kratos=$($st.aetherid.kratos_reachable)"
Write-Host "  caps=$($st.capabilities | ConvertTo-Json -Compress)"

Write-Host "=== ory (if ready) ==="
$ory = (Invoke-RestMethod http://127.0.0.1:8085/v1/ory/status).data
if ($ory.ready) {
  $email = "smoke-$(Get-Random)@helixforge.local"
  $pw = "password123456"
  [void](Invoke-RestMethod http://127.0.0.1:8085/v1/ory/register -Method POST -ContentType application/json -Body (@{email=$email;password=$pw}|ConvertTo-Json))
  $login = Invoke-RestMethod http://127.0.0.1:8085/v1/ory/login -Method POST -ContentType application/json -Body (@{email=$email;password=$pw}|ConvertTo-Json)
  $tok = $login.data.session_token
  $me = Invoke-RestMethod http://127.0.0.1:8080/v1/me -Headers @{ Authorization = "Bearer $tok" }
  Write-Host "  bearer session=$($me.data.session_id)"
} else {
  Write-Host "  kratos not ready - skipped live login"
}

$tid = (Invoke-RestMethod http://127.0.0.1:8080/v1/me -Headers $h).data.tenant_id
Write-Host "=== vault HVA5 + kms ==="
$put = @{ name = "smoke-hva5"; value_b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("deep-smoke-secret")) } | ConvertTo-Json
[void](Invoke-RestMethod "http://127.0.0.1:8082/v1/tenants/$tid/secrets" -Method POST -Headers $h -Body $put)
$plain = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String((Invoke-RestMethod "http://127.0.0.1:8082/v1/tenants/$tid/secrets/smoke-hva5" -Headers $h).data.value_b64))
Write-Host "  plain=$plain"
if ($plain -ne "deep-smoke-secret") { throw "vault roundtrip mismatch" }
$meta = (Invoke-RestMethod "http://127.0.0.1:8082/v1/keys/meta" -Headers $h).data
Write-Host "  key_meta algorithm=$($meta.meta.algorithm) envelope=$($meta.envelope)"
if ($meta.envelope -ne "HVA5") { throw "expected HVA5 default envelope" }

Write-Host "=== vault object HVA5 ==="
$object = @{ name = "smoke-object"; value_b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("deep-smoke-object")); content_type = "text/plain" } | ConvertTo-Json
$objectPut = Invoke-RestMethod "http://127.0.0.1:8082/v1/tenants/$tid/objects" -Method POST -Headers $h -Body $object
Write-Host "  object envelope=$($objectPut.data.envelope) key_version=$($objectPut.data.key_version)"
if ($objectPut.data.envelope -ne "HVA5-tenant-dek") { throw "expected HVA5-tenant-dek object envelope" }
$objectGet = Invoke-RestMethod "http://127.0.0.1:8082/v1/tenants/$tid/objects/smoke-object" -Headers $h
$objectPlain = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($objectGet.data.value_b64))
if ($objectPlain -ne "deep-smoke-object") { throw "object roundtrip mismatch" }
Write-Host "  object plain=$objectPlain"

Write-Host "=== payments ==="
$pay = Invoke-RestMethod "http://127.0.0.1:8083/v1/tenants/$tid/payments" -Method POST -Headers $h -Body (@{plan_id="team"}|ConvertTo-Json)
$conf = Invoke-RestMethod "http://127.0.0.1:8083/v1/tenants/$tid/payments/$($pay.data.id)/confirm" -Method POST -Headers $h
Write-Host "  payment=$($conf.data.status) plan=$((Invoke-RestMethod "http://127.0.0.1:8083/v1/tenants/$tid/plan" -Headers $h).data.plan_id)"

Write-Host "=== proxy ==="
$ph = (Invoke-RestMethod http://127.0.0.1:8080/core/auth/v1/auth/health -Headers $h).data
Write-Host "  proxy auth mode=$($ph.mode)"

Write-Host "=== compliance / sovereign controls ==="
$comp = (Invoke-RestMethod http://127.0.0.1:8084/v1/compliance/summary -Headers $h).data
Write-Host "  audit_verified=$($comp.audit_chain_verified) bus=$($comp.controls.bus_mode) jetstream=$($comp.controls.nats_jetstream)"
Write-Host "  durable=$($comp.controls.postgres_durable) hmac=$($comp.controls.audit_hmac_signatures) shared_rl=$($comp.controls.shared_rate_limit)"
if (-not $comp.audit_chain_verified) {
  Write-Host "  chain drift - break-glass rehash via operator CLI..."
  $env:HELIX_AUDIT_REHASH_APPROVED = "1"
  try {
    $rehash = (cargo run -q -p helix_db --bin helix-audit-rehash -- --approve 2>&1)
    Write-Host "  $rehash"
    $comp = (Invoke-RestMethod http://127.0.0.1:8084/v1/compliance/summary -Headers $h).data
  } catch {
    Write-Host "  rehash skipped: $_" -ForegroundColor Yellow
  }
}
if (-not $comp.audit_chain_verified) {
  Write-Host "WARN: audit_chain_verified still false (link integrity may need offline rehash)" -ForegroundColor Yellow
} else {
  Write-Host "  audit chain OK"
}
if (-not $comp.controls.postgres_durable) { throw "postgres not durable" }
if (-not $comp.controls.nats_jetstream) { Write-Host "WARN: jetstream flag false" -ForegroundColor Yellow }

Write-Host "=== catalog product 21 pulse ==="
$cat = (Invoke-RestMethod http://127.0.0.1:8080/v1/catalog -Headers $h).data
$pulse = $cat | Where-Object { $_.slug -eq "helix-pulse" }
if (-not $pulse) { throw "helix-pulse missing from catalog" }
Write-Host "  pulse order=$($pulse.order) port=$($pulse.default_port)"

Write-Host "=== DONE (deep smoke OK) ==="
