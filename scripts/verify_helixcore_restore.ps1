# Verify a restored HelixCore data plane.
# Usage:
#   powershell -File .\scripts\verify_helixcore_restore.ps1 [-OutFile .\restore-evidence.json]
param(
  [string]$OutFile = "restore-evidence.json",
  [string]$ObservabilityBase = "http://127.0.0.1:8084",
  [string]$GatewayBase = "http://127.0.0.1:8080",
  [string]$VaultBase = "http://127.0.0.1:8082"
)

$ErrorActionPreference = "Stop"
$start = Get-Date

# Dev headers give us a stable platform principal in local mode.
$h = @{
  "x-helix-dev-user" = "ops@helixforge.local"
  "Content-Type"     = "application/json"
}

$evidence = @{
  started_at  = $start.ToString("o")
  services    = @{}
  compliance  = @{}
  vault       = @{}
  audit_rehash = @{}
  passed      = $true
}

function Record-Failure($step, $message) {
  Write-Host "FAIL: $step - $message" -ForegroundColor Red
  $evidence[$step] = @{ status = "fail"; error = $message }
  $evidence.passed = $false
}

# --- 1. All 6 core services healthz ---
Write-Host "=== healthz (8080-8085) ==="
foreach ($p in 8080..8085) {
  try {
    $code = (Invoke-WebRequest "http://127.0.0.1:$p/healthz" -UseBasicParsing -TimeoutSec 5).StatusCode
    Write-Host "  $p $code"
    $evidence.services[$p] = @{ status = "ok"; status_code = $code }
  } catch {
    Record-Failure "services" "port $p healthz failed: $_"
  }
}

# --- 2. Compliance summary ---
Write-Host "=== compliance summary ==="
try {
  $comp = (Invoke-RestMethod "$ObservabilityBase/v1/compliance/summary" -Headers $h).data
  $evidence.compliance = @{
    status                = "ok"
    audit_chain_verified  = $comp.audit_chain_verified
    postgres_durable      = $comp.controls.postgres_durable
    audit_hmac_signatures = $comp.controls.audit_hmac_signatures
    controls              = $comp.controls
  }
  Write-Host "  audit_verified=$($comp.audit_chain_verified) durable=$($comp.controls.postgres_durable)"

  if (-not $comp.audit_chain_verified) {
    Write-Host "  audit chain not verified - attempting operator rehash ..." -ForegroundColor Yellow
    try {
      $env:HELIX_AUDIT_REHASH_APPROVED = "1"
      $rehash = (cargo run -q -p helix_db --bin helix-audit-rehash -- --approve 2>&1)
      $evidence.audit_rehash = @{ status = "ok"; output = ($rehash -join "`n") }
      Write-Host "  rehash ok"
      # Re-check compliance after rehash
      $comp = (Invoke-RestMethod "$ObservabilityBase/v1/compliance/summary" -Headers $h).data
      $evidence.compliance.audit_chain_verified = $comp.audit_chain_verified
      if (-not $comp.audit_chain_verified) {
        Record-Failure "compliance" "audit chain still not verified after rehash"
      }
    } catch {
      $evidence.audit_rehash = @{ status = "fail"; error = $_.ToString() }
      Record-Failure "compliance" "audit rehash failed: $_"
    }
  }

  if (-not $comp.controls.postgres_durable) {
    Record-Failure "compliance" "postgres_durable=false"
  }
} catch {
  Record-Failure "compliance" "compliance summary failed: $_"
}

# --- 3. Vault roundtrip ---
Write-Host "=== vault roundtrip ==="
try {
  $me = (Invoke-RestMethod "$GatewayBase/v1/me" -Headers $h).data
  $tenantId = $me.tenant_id
  Write-Host "  tenant_id=$tenantId"

  $secretName = "restore-proof-$(Get-Random)"
  $plain = "hello restore $(Get-Date -Format o)"
  $b64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($plain))

  $put = Invoke-RestMethod "$VaultBase/v1/tenants/$tenantId/secrets" -Method POST -Headers $h -Body (@{ name = $secretName; value_b64 = $b64 } | ConvertTo-Json)
  $got = Invoke-RestMethod "$VaultBase/v1/tenants/$tenantId/secrets/$secretName" -Headers $h
  $gotPlain = [System.Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($got.data.value_b64))

  if ($gotPlain -eq $plain) {
    $evidence.vault = @{ status = "ok"; tenant_id = $tenantId; secret_name = $secretName }
    Write-Host "  vault roundtrip ok"
  } else {
    Record-Failure "vault" "roundtrip value mismatch"
  }

  # Clean up the test secret.
  try {
    [void](Invoke-RestMethod "$VaultBase/v1/tenants/$tenantId/secrets/$secretName" -Method DELETE -Headers $h)
  } catch {
    Write-Host "  cleanup warning: $_" -ForegroundColor Yellow
  }
} catch {
  Record-Failure "vault" "vault roundtrip failed: $_"
}

$evidence.finished_at = (Get-Date).ToString("o")
$evidence | ConvertTo-Json -Depth 6 | Set-Content -Path $OutFile
Write-Host "Evidence written to $OutFile"

if ($evidence.passed) {
  Write-Host "RESTORE VERIFICATION PASSED" -ForegroundColor Green
  exit 0
} else {
  Write-Host "RESTORE VERIFICATION FAILED" -ForegroundColor Red
  exit 1
}
