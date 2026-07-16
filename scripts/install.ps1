# HelixForge local installer for Windows.
# Usage: powershell -File scripts\install.ps1
param(
  [switch]$SkipBuild,
  [switch]$SkipDocker
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Test-Command { param([string]$Name)
  $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Fail { param([string]$Message)
  Write-Host "[install] ERROR: $Message" -ForegroundColor Red
  exit 1
}

# --- Secrets ---
$Keys = Join-Path $env:USERPROFILE "Desktop\.keys\helixforge\.env.local"
if (Test-Path $Keys) {
  Get-Content $Keys | ForEach-Object {
    if ($_ -match '^\s*#' -or $_ -match '^\s*$') { return }
    $pair = $_.Split('=', 2)
    if ($pair.Length -eq 2) {
      [Environment]::SetEnvironmentVariable($pair[0].Trim(), $pair[1].Trim(), "Process")
    }
  }
  Write-Host "[install] loaded secrets from $Keys"
} else {
  Write-Host "[install] no secrets file at $Keys - continuing with defaults"
}

# --- Defaults ---
$env:HELIX_ENV = if ($env:HELIX_ENV) { $env:HELIX_ENV } else { "local" }
$env:HELIX_LOCAL_DEV_UNSAFE = if ($env:HELIX_LOCAL_DEV_UNSAFE) { $env:HELIX_LOCAL_DEV_UNSAFE } else { "1" }
$env:HELIX_ALLOW_DEV_HEADERS = if ($env:HELIX_ALLOW_DEV_HEADERS) { $env:HELIX_ALLOW_DEV_HEADERS } else { "1" }
$env:HELIX_DEV_PLATFORM = if ($env:HELIX_DEV_PLATFORM) { $env:HELIX_DEV_PLATFORM } else { "1" }
$env:DATABASE_URL = if ($env:DATABASE_URL) { $env:DATABASE_URL } else { "postgres://helix:helix@127.0.0.1:55432/helixforge" }
$env:NATS_URL = if ($env:NATS_URL) { $env:NATS_URL } else { "nats://127.0.0.1:4222" }
$env:MINIO_ENDPOINT = if ($env:MINIO_ENDPOINT) { $env:MINIO_ENDPOINT } else { "http://127.0.0.1:9000" }
$env:HELIX_VAULT_MASTER_KEY = if ($env:HELIX_VAULT_MASTER_KEY) { $env:HELIX_VAULT_MASTER_KEY } else { "local-dev-vault-master-key-not-for-prod" }
$env:HELIX_AUDIT_HMAC_SECRET = if ($env:HELIX_AUDIT_HMAC_SECRET) { $env:HELIX_AUDIT_HMAC_SECRET } else { "local-audit-hmac-dev-only" }
$env:HELIX_WEBHOOK_ALLOW_UNSIGNED = if ($env:HELIX_WEBHOOK_ALLOW_UNSIGNED) { $env:HELIX_WEBHOOK_ALLOW_UNSIGNED } else { "1" }

# --- Prerequisites ---
if (-not (Test-Command docker)) { Fail "Docker is required. Install Docker Desktop." }
if (-not (Test-Command cargo)) { Fail "Rust/Cargo is required. Install from https://rustup.rs" }
if (-not (Test-Command pnpm)) { Fail "pnpm is required. Install: npm install -g pnpm" }
if (-not (Test-Command node)) { Fail "Node.js is required. Install Node 20+." }

# Prefer "docker compose" v2
$Compose = $null
if (Test-Command "docker") {
  try {
    docker compose version | Out-Null
    $Compose = "docker compose"
  } catch {
    if (Test-Command "docker-compose") {
      $Compose = "docker-compose"
    }
  }
}
if (-not $Compose) { Fail "Docker Compose is required." }

# --- Infrastructure ---
if (-not $SkipDocker) {
  Write-Host "[install] starting backing services..."
  Invoke-Expression "$Compose up -d postgres nats minio minio-init"

  $Ready = $false
  for ($i = 1; $i -le 30; $i++) {
    try {
      Invoke-Expression "$Compose exec -T postgres pg_isready -U helix -d helixforge" | Out-Null
      if ($LASTEXITCODE -eq 0) {
        Write-Host "[install] postgres ready"
        $Ready = $true
        break
      }
    } catch { }
    Start-Sleep -Seconds 2
  }
  if (-not $Ready) { Fail "postgres did not become ready" }
}

# --- Build ---
if (-not $SkipBuild) {
  Write-Host "[install] building Rust workspace (this may take several minutes)..."
  cargo build --workspace

  Write-Host "[install] installing JS dependencies and building console..."
  pnpm install
  pnpm --filter @helixforge/console build
}

Write-Host "[install] done."
Write-Host ""
Write-Host "Next steps:"
Write-Host '  - Start core services: scripts\dev-core.ps1'
Write-Host '  - Or run gateway directly: cargo run -p gateway'
Write-Host '  - Gateway health: curl http://127.0.0.1:8080/healthz'
