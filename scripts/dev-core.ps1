# Start HelixCore services in separate Windows terminals (user-owned).
# Prerequisites: docker compose up -d postgres nats minio minio-init
# Usage: powershell -File scripts/dev-core.ps1

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

# Load secrets if present
$Keys = Join-Path $env:USERPROFILE "Desktop\.keys\helixforge\.env.local"
if (Test-Path $Keys) {
  Get-Content $Keys | ForEach-Object {
    if ($_ -match '^\s*#' -or $_ -match '^\s*$') { return }
    $pair = $_.Split('=', 2)
    if ($pair.Length -eq 2) {
      [Environment]::SetEnvironmentVariable($pair[0].Trim(), $pair[1].Trim(), "Process")
    }
  }
  Write-Host "Loaded $Keys"
}

$env:HELIX_ENV = if ($env:HELIX_ENV) { $env:HELIX_ENV } else { "local" }
$env:HELIX_LOCAL_DEV_UNSAFE = "1"
$env:HELIX_ALLOW_DEV_HEADERS = "1"
$env:HELIX_DEV_PLATFORM = "1"
$env:DATABASE_URL = if ($env:DATABASE_URL) { $env:DATABASE_URL } else { "postgres://helix:helix@127.0.0.1:55432/helixforge" }
$env:NATS_URL = if ($env:NATS_URL) { $env:NATS_URL } else { "nats://127.0.0.1:4222" }
$env:HELIX_VAULT_MASTER_KEY = if ($env:HELIX_VAULT_MASTER_KEY) { $env:HELIX_VAULT_MASTER_KEY } else { "local-dev-vault-master-key-not-for-prod" }
$env:HELIX_AUDIT_HMAC_SECRET = if ($env:HELIX_AUDIT_HMAC_SECRET) { $env:HELIX_AUDIT_HMAC_SECRET } else { "local-audit-hmac-dev-only" }
$env:HELIX_WEBHOOK_ALLOW_UNSIGNED = if ($env:HELIX_WEBHOOK_ALLOW_UNSIGNED) { $env:HELIX_WEBHOOK_ALLOW_UNSIGNED } else { "1" }

$services = @(
  @{ Name = "gateway"; Package = "gateway" },
  @{ Name = "agent-hub"; Package = "agent_hub" },
  @{ Name = "vault-service"; Package = "vault_service" },
  @{ Name = "billing-service"; Package = "billing_service" },
  @{ Name = "observability-service"; Package = "observability_service" },
  @{ Name = "auth-adapter"; Package = "auth_adapter" }
)

$msvc = "rustup run stable-x86_64-pc-windows-msvc"
foreach ($svc in $services) {
  $title = "HelixCore $($svc.Name)"
  $cmd = @"
cd /d "$Root"
set HELIX_ENV=$($env:HELIX_ENV)
set HELIX_LOCAL_DEV_UNSAFE=1
set HELIX_ALLOW_DEV_HEADERS=1
set HELIX_DEV_PLATFORM=1
set DATABASE_URL=$($env:DATABASE_URL)
set NATS_URL=$($env:NATS_URL)
set HELIX_VAULT_MASTER_KEY=$($env:HELIX_VAULT_MASTER_KEY)
set HELIX_AUDIT_HMAC_SECRET=$($env:HELIX_AUDIT_HMAC_SECRET)
set HELIX_WEBHOOK_ALLOW_UNSIGNED=$($env:HELIX_WEBHOOK_ALLOW_UNSIGNED)
set MINIO_ENDPOINT=$($env:MINIO_ENDPOINT)
title $title
$msvc cargo run -p $($svc.Package)
"@
  Start-Process -FilePath "cmd.exe" -ArgumentList "/k", $cmd -WindowStyle Normal
  Write-Host "Started window: $title"
}

Write-Host ""
Write-Host "Fail-closed local auth: HELIX_ALLOW_DEV_HEADERS=1 HELIX_DEV_PLATFORM=1"
Write-Host "Gateway:  http://127.0.0.1:8080/healthz"
Write-Host "Smoke:    pwsh scripts/helixcore_deep_smoke.ps1"
