# Restore HelixCore Postgres dump produced by backup_helixcore.ps1
# Usage:
#   powershell -File .\scripts\restore_helixcore.ps1 -DumpFile C:\path\helixforge.sql [-Verify]
param(
  [Parameter(Mandatory = $true)][string]$DumpFile,
  [string]$Container = ""
  [string]$Db = "helixforge",
  [string]$User = "helix",
  [switch]$Verify
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path $DumpFile)) { throw "Dump not found: $DumpFile" }

if ($Container) {
  Write-Host "Restoring $DumpFile into $Container/$Db ..."
  Get-Content $DumpFile -Raw | docker exec -i $Container psql -U $User -d $Db
} else {
  Write-Host "Restoring $DumpFile into compose service postgres/$Db ..."
  Get-Content $DumpFile -Raw | docker compose exec -T postgres psql -U $User -d $Db
}
Write-Host "Postgres restore complete."

if ($Verify) {
  $verifyScript = Join-Path $PSScriptRoot "verify_helixcore_restore.ps1"
  if (Test-Path $verifyScript) {
    Write-Host "Running restore verification ..."
    & powershell -NoProfile -ExecutionPolicy Bypass -File $verifyScript
  } else {
    Write-Host "WARN: verify script not found at $verifyScript" -ForegroundColor Yellow
  }
} else {
  Write-Host "Optional: run scripts/verify_helixcore_restore.ps1 to prove the restore."
  Write-Host "Optional: if audit chain timestamps drifted, use the operator CLI:"
  Write-Host "  HELIX_AUDIT_REHASH_APPROVED=1 cargo run -q -p helix_db --bin helix-audit-rehash -- --approve"
}
