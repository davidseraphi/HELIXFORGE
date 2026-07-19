# Start all 21 product APIs as background processes (user-owned console).
# Prerequisites: docker compose up -d postgres nats minio minio-init
# Usage: powershell -File scripts/dev-products.ps1 [-Build]
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
param([switch]$Build)

$env:HELIX_ENV = if ($env:HELIX_ENV) { $env:HELIX_ENV } else { "local" }
$env:HELIX_LOCAL_DEV_UNSAFE = "1"
$env:HELIX_ALLOW_DEV_HEADERS = "1"
$env:HELIX_DEV_PLATFORM = "1"
$env:DATABASE_URL = if ($env:DATABASE_URL) { $env:DATABASE_URL } else { "postgres://helix:helix@127.0.0.1:55432/helixforge" }
$env:NATS_URL = if ($env:NATS_URL) { $env:NATS_URL } else { "nats://127.0.0.1:4222" }
$env:MINIO_ENDPOINT = if ($env:MINIO_ENDPOINT) { $env:MINIO_ENDPOINT } else { "http://127.0.0.1:9000" }
$env:HELIX_VAULT_MASTER_KEY = if ($env:HELIX_VAULT_MASTER_KEY) { $env:HELIX_VAULT_MASTER_KEY } else { "local-dev-vault-master-key-not-for-prod" }
$env:HELIX_AUDIT_HMAC_SECRET = if ($env:HELIX_AUDIT_HMAC_SECRET) { $env:HELIX_AUDIT_HMAC_SECRET } else { "local-audit-hmac-dev-only" }
$env:HELIX_WEBHOOK_ALLOW_UNSIGNED = if ($env:HELIX_WEBHOOK_ALLOW_UNSIGNED) { $env:HELIX_WEBHOOK_ALLOW_UNSIGNED } else { "1" }

$products = @(
  "helix_collab_api", "helix_code_api", "helix_flow_api", "helix_insights_api",
  "helix_commerce_api", "helix_edu_api", "helix_capital_api", "helix_well_api",
  "helix_network_api", "helix_forge_studio_api", "helix_synthbio_api",
  "helix_lex_prime_api", "helix_cura_prime_api", "helix_terra_prime_api",
  "helix_climate_prime_api", "helix_orbit_prime_api", "helix_quantum_forge_api",
  "helix_vita_prime_api", "helix_grid_prime_api", "helix_nova_labs_api",
  "helix_pulse_api"
)

foreach ($pkg in $products) {
  $bin = "target/debug/$pkg.exe"
  if ($Build -or -not (Test-Path $bin)) {
    Write-Host "Building $pkg ..."
    rustup run stable-x86_64-pc-windows-msvc cargo build -p $pkg
    if ($LASTEXITCODE -ne 0) { throw "build failed: $pkg" }
  }
  Start-Process -FilePath $bin -WorkingDirectory $Root -WindowStyle Hidden
  Write-Host "Started $pkg"
}
Write-Host ""
Write-Host "All 21 product APIs starting. Open the console at http://localhost:3000/products"
