# Continuous HelixCore backup loop (schedule via Task Scheduler / cron).
# Runs backup_helixcore.ps1 every -IntervalMinutes (default 60).
param(
  [int]$IntervalMinutes = 60,
  [string]$BackupRoot = "",
  [int]$RetainCount = 48
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
if (-not $BackupRoot) {
  $BackupRoot = Join-Path $env:USERPROFILE "Desktop\helixforge-backups"
}
New-Item -ItemType Directory -Force -Path $BackupRoot | Out-Null
$script = Join-Path $root "scripts\backup_helixcore.ps1"

Write-Host "Continuous backup every ${IntervalMinutes}m -> $BackupRoot (retain $RetainCount)"
while ($true) {
  $ts = Get-Date -Format "yyyyMMdd-HHmmss"
  $out = Join-Path $BackupRoot $ts
  try {
    & powershell -NoProfile -ExecutionPolicy Bypass -File $script -OutDir $out
    # MinIO mirror if mc present
    $mc = Get-Command mc -ErrorAction SilentlyContinue
    if ($mc) {
      $mirror = Join-Path $out "minio-mirror"
      New-Item -ItemType Directory -Force -Path $mirror | Out-Null
      try {
        mc alias set helixlocal http://127.0.0.1:9000 helixminio helixminio_secret 2>$null | Out-Null
        mc mirror --overwrite "helixlocal/helixforge" $mirror 2>$null
        Write-Host "MinIO mirror ok"
      } catch {
        Write-Host "MinIO mirror skipped: $_"
      }
    }
  } catch {
    Write-Host "Backup failed: $_"
  }
  # prune old
  Get-ChildItem $BackupRoot -Directory |
    Sort-Object Name -Descending |
    Select-Object -Skip $RetainCount |
    ForEach-Object { Remove-Item $_.FullName -Recurse -Force -ErrorAction SilentlyContinue }
  Start-Sleep -Seconds ($IntervalMinutes * 60)
}
