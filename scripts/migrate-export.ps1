# Export HelixCore Postgres + MinIO state to a portable backup directory.
# Usage: powershell -File scripts\migrate-export.ps1 [-OutDir <path>]
param(
  [string]$OutDir = "",
  [string]$DatabaseUrl = "postgres://helix:helix@127.0.0.1:55432/helixforge",
  [string]$MinioEndpoint = "http://host.docker.internal:9000"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Ts = Get-Date -Format "yyyyMMdd-HHmmss"
if (-not $OutDir) {
  $OutDir = Join-Path $env:USERPROFILE "Desktop\helixforge-backups\$Ts"
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Write-Host "[export] backup dir: $OutDir"

# --- Postgres ---
$DumpFile = Join-Path $OutDir "helixforge.sql"
$DumpOk = $false
try {
  docker compose exec -T postgres pg_isready -U helix -d helixforge | Out-Null
  docker compose exec -T postgres pg_dump -U helix -d helixforge --no-owner --format=plain |
    Set-Content -Path $DumpFile -Encoding utf8
  $DumpOk = $true
  Write-Host "[export] postgres dump: $DumpFile"
} catch {
  Write-Host "WARN: docker pg_dump failed ($($_))."
}

$DumpHash = if ($DumpOk -and (Test-Path $DumpFile)) { (Get-FileHash $DumpFile -Algorithm SHA256).Hash } else { $null }

# --- MinIO via ephemeral mc container ---
$MinioDir = Join-Path $OutDir "minio-mirror"
New-Item -ItemType Directory -Force -Path $MinioDir | Out-Null
$MinioOk = $false
try {
  foreach ($bucket in @("helixforge", "helix-collab", "helix-code")) {
    docker run --rm --network host --entrypoint /bin/sh `
      -v "${MinioDir}:/mirror" `
      minio/mc:latest `
      -c "mc alias set local $MinioEndpoint helixminio helixminio_secret >/dev/null 2>&1 && mc mirror --overwrite local/$bucket /mirror/$bucket" 2>$null | Out-Null
  }
  $MinioOk = $true
  Write-Host "[export] minio mirror: $MinioDir"
} catch {
  Write-Host "WARN: minio mirror failed: $_"
}

$MinioHash = if ($MinioOk) {
  $hashes = Get-ChildItem $MinioDir -Recurse -File |
    Get-FileHash -Algorithm SHA256 |
    Select-Object -ExpandProperty Hash |
    Sort-Object
  $joined = $hashes -join ""
  (Get-FileHash -InputStream ([System.IO.MemoryStream]::new([System.Text.Encoding]::UTF8.GetBytes($joined))) -Algorithm SHA256).Hash
} else { $null }

# --- Manifest ---
$Commit = "unknown"
try { $Commit = (git -C $Root rev-parse HEAD 2>$null).Trim() } catch { }

$BackupSet = @()
if ($DumpOk) { $BackupSet += @{ kind = "postgres_dump"; path = "helixforge.sql"; sha256 = $DumpHash } }
if ($MinioOk) { $BackupSet += @{ kind = "minio_mirror"; path = "minio-mirror"; sha256 = $MinioHash } }

$Meta = @{
  timestamp = (Get-Date).ToString("o")
  git_commit = $Commit
  database_url_host = ($DatabaseUrl -replace ':[^:@/]+@', ':****@')
  minio_endpoint = $MinioEndpoint
  backup_set = $BackupSet
  note = "Secrets live outside this backup. Restore them separately from your key directory."
}
$Meta | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $OutDir "backup-manifest.json")

Write-Host "[export] manifest: $(Join-Path $OutDir backup-manifest.json)"
Write-Host "[export] done."
