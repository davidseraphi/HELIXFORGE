# Restore HelixCore Postgres + MinIO state from a backup directory.
# Usage: powershell -File scripts\migrate-restore.ps1 -BackupDir <path> [-ProjectName helixforge] [-OverrideFile <path>]
param(
  [Parameter(Mandatory)]
  [string]$BackupDir,
  [string]$ProjectName = "helixforge",
  [string]$OverrideFile = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$env:COMPOSE_PROJECT_NAME = $ProjectName
$ComposeFile = "$Root\docker-compose.yml"

$DumpFile = Join-Path $BackupDir "helixforge.sql"
$MinioDir = Join-Path $BackupDir "minio-mirror"

if (-not (Test-Path $DumpFile)) {
  throw "[restore] ERROR: missing $DumpFile"
}

if ($ProjectName -ne "helixforge") {
  Write-Host "[restore] using isolated compose project: $ProjectName"
}

# Use environment variables to shift host ports for isolated restores.
# This avoids relying on the compose `!override` tag, which older Docker Compose
# versions do not support.
$PgPort = if ($env:HELIX_POSTGRES_PORT) { $env:HELIX_POSTGRES_PORT } else { "55432" }
if ($ProjectName -ne "helixforge" -and $PgPort -eq "55432") {
  $env:HELIX_POSTGRES_PORT = "55433"
  $env:HELIX_NATS_PORT = "4223"
  $env:HELIX_NATS_MONITOR_PORT = "8223"
  $env:HELIX_MINIO_PORT = "9002"
  $env:HELIX_MINIO_CONSOLE_PORT = "9003"
  $PgPort = "55433"
}

$ComposeArgs = "-f `"$ComposeFile`""
if ($OverrideFile) {
  $ComposeArgs += " -f `"$OverrideFile`""
}

Write-Host "[restore] stopping and wiping existing project data..."
Invoke-Expression "docker compose $ComposeArgs down -v"

Write-Host "[restore] starting fresh infrastructure..."
Invoke-Expression "docker compose $ComposeArgs up -d postgres nats minio minio-init"

$PgContainer = "${ProjectName}-postgres-1"
$Ready = $false
for ($i = 1; $i -le 30; $i++) {
  try {
    docker exec $PgContainer pg_isready -U helix -d helixforge | Out-Null
    if ($LASTEXITCODE -eq 0) {
      Write-Host "[restore] postgres ready"
      $Ready = $true
      break
    }
  } catch { }
  Start-Sleep -Seconds 2
}
if (-not $Ready) {
  throw "[restore] ERROR: postgres container $PgContainer did not become ready"
}

Write-Host "[restore] restoring postgres..."
Get-Content $DumpFile | docker exec -i $PgContainer psql -U helix -d helixforge
if ($LASTEXITCODE -ne 0) {
  throw "[restore] ERROR: psql restore failed with exit code $LASTEXITCODE"
}

Write-Host "[restore] restoring minio..."
if (Test-Path $MinioDir) {
  $Network = "${ProjectName}_default"
  $MinioEndpoint = if ($PgPort -eq "55433") { "http://minio:9000" } else { "http://minio:9000" }
  foreach ($bucket in @("helixforge", "helix-collab", "helix-code")) {
    $src = "$MinioDir\$bucket"
    if (Test-Path $src) {
      docker run --rm --network $Network --entrypoint /bin/sh `
        -v "${src}:/source/$bucket" `
        minio/mc:latest `
        -c "mc alias set restore $MinioEndpoint helixminio helixminio_secret >/dev/null 2>&1 && mc mirror --overwrite /source/$bucket restore/$bucket" 2>$null | Out-Null
    }
  }
}

Write-Host "[restore] verification..."
docker exec $PgContainer pg_isready -U helix -d helixforge | Out-Null
if ($LASTEXITCODE -ne 0) {
  throw "[restore] ERROR: postgres verification failed"
}

Write-Host "[restore] done."
Write-Host ""
Write-Host "Verify with: `$env:DATABASE_URL='postgres://helix:helix@127.0.0.1:$PgPort/helixforge'; cargo test -p helix_db"
