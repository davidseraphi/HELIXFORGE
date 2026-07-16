# Backup HelixCore data plane: Postgres + MinIO object mirror + config snapshot.
# Usage:
#   powershell -File .\scripts\backup_helixcore.ps1 [-OutDir C:\backups\helixforge] [-MirrorMinio]
param(
  [string]$OutDir = "",
  [string]$DatabaseUrl = "postgres://helix:helix@127.0.0.1:55432/helixforge",
  [string]$MinioEndpoint = "http://127.0.0.1:9000",
  [string]$MinioBucket = "helixforge",
  [switch]$MirrorMinio
)

$ErrorActionPreference = "Stop"
$ts = Get-Date -Format "yyyyMMdd-HHmmss"
if (-not $OutDir) {
  $OutDir = Join-Path $env:USERPROFILE "Desktop\helixforge-backups\$ts"
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Write-Host "Backup dir: $OutDir"

# --- Postgres logical dump via docker compose exec ---
$dumpFile = Join-Path $OutDir "helixforge.sql"
$dumpOk = $false
try {
  docker compose exec -T postgres pg_isready -U helix -d helixforge | Out-Null
  docker compose exec -T postgres pg_dump -U helix -d helixforge --no-owner --format=plain |
    Set-Content -Path $dumpFile -Encoding utf8
  $dumpOk = $true
  Write-Host "Postgres dump: $dumpFile"
} catch {
  Write-Host "WARN: docker pg_dump failed ($_). Install pg_dump and use DATABASE_URL manually."
  "DATABASE_URL=$DatabaseUrl" | Set-Content (Join-Path $OutDir "postgres-manual.txt")
}

$dumpHash = $null
if ($dumpOk -and (Test-Path $dumpFile)) {
  $dumpHash = (Get-FileHash -Path $dumpFile -Algorithm SHA256).Hash
}

# --- MinIO object mirror if mc available ---
$minioMirrorOk = $false
$minioZip = Join-Path $OutDir "minio-mirror.zip"
$minioDir = Join-Path $OutDir "minio-mirror"
$mc = Get-Command mc -ErrorAction SilentlyContinue
if ($mc) {
  try {
    mc alias set helixlocal $MinioEndpoint helixminio helixminio_secret 2>$null | Out-Null
    mc ls --recursive "helixlocal/$MinioBucket" |
      Set-Content (Join-Path $OutDir "minio-listing.txt")
    Write-Host "MinIO listing written"

    if ($MirrorMinio) {
      New-Item -ItemType Directory -Force -Path $minioDir | Out-Null
      mc mirror --overwrite "helixlocal/$MinioBucket" "$minioDir/$MinioBucket" 2>$null | Out-Null
      Compress-Archive -Path "$minioDir\*" -DestinationPath $minioZip -Force
      Remove-Item -Recurse -Force $minioDir -ErrorAction SilentlyContinue
      $minioMirrorOk = $true
      Write-Host "MinIO mirror: $minioZip"
    }
  } catch {
    Write-Host "WARN: mc failed: $_"
  }
} else {
  "Install MinIO client (mc) for object inventory and mirror." |
    Set-Content (Join-Path $OutDir "minio-readme.txt")
}

$minioZipHash = $null
if ($minioMirrorOk -and (Test-Path $minioZip)) {
  $minioZipHash = (Get-FileHash -Path $minioZip -Algorithm SHA256).Hash
}

# --- Config / inventory snapshot (no secrets) ---
$commit = "unknown"
try {
  $commit = (git -C $PSScriptRoot rev-parse HEAD 2>$null).Trim()
} catch { }

$backupSet = @()
if ($dumpOk) { $backupSet += @{ kind = "postgres_dump"; path = "helixforge.sql"; sha256 = $dumpHash } }
if ($minioMirrorOk) { $backupSet += @{ kind = "minio_mirror"; path = "minio-mirror.zip"; sha256 = $minioZipHash } }

$meta = @{
  timestamp = (Get-Date).ToString("o")
  git_commit = $commit
  database_url_host = ($DatabaseUrl -replace ':[^:@/]+@', ':****@')
  minio_endpoint = $MinioEndpoint
  minio_bucket = $MinioBucket
  mirror_minio = $MirrorMinio.IsPresent
  compose_profiles = @("ory", "observability")
  backup_set = $backupSet
  note = "Secrets live under Desktop/.keys/helixforge - back those up separately offline"
}
$meta | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $OutDir "backup-manifest.json")

if (-not $dumpOk -and -not $minioMirrorOk) {
  Write-Host "WARN: backup produced no durable data artifacts. Review errors above." -ForegroundColor Yellow
}

Write-Host "DONE. Store $OutDir offline and encrypt. See docs/runbooks/backup-dr.md"
