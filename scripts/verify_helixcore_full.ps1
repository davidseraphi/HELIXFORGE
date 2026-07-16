# Full HelixCore local verification: start services, deep smoke, export/restore.
# Usage: powershell -File scripts\verify_helixcore_full.ps1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$env:HELIX_ENV = "local"
$env:HELIX_LOCAL_DEV_UNSAFE = "1"
$env:HELIX_ALLOW_DEV_HEADERS = "1"
$env:HELIX_DEV_PLATFORM = "1"
$env:DATABASE_URL = "postgres://helix:helix@127.0.0.1:55432/helixforge"
$env:NATS_URL = "nats://127.0.0.1:4222"
$env:HELIX_VAULT_MASTER_KEY = "local-dev-vault-master-key-not-for-prod"
$env:HELIX_AUDIT_HMAC_SECRET = "local-audit-hmac-dev-only"
$env:MINIO_ENDPOINT = "http://127.0.0.1:9000"

$LogDir = "$Root\target\verify-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

function Start-HelixService {
    param([string]$Name, [string]$Package)
    $log = "$LogDir\$Name.log"
    $cmd = "cd /d `"$Root`" && title HelixCore $Name && rustup run stable-x86_64-pc-windows-msvc cargo run -p $Package > `"$log`" 2>&1"
    Write-Host "[verify] starting $Name (log: $log)"
    Start-Process -FilePath "cmd.exe" -ArgumentList "/c", $cmd -WindowStyle Hidden
}

function Wait-Health {
    param([int]$Port, [string]$Name)
    for ($i = 1; $i -le 60; $i++) {
        try {
            $r = Invoke-WebRequest "http://127.0.0.1:$Port/healthz" -UseBasicParsing -TimeoutSec 2
            if ($r.StatusCode -eq 200) {
                Write-Host "[verify] $Name on $Port healthy"
                return
            }
        } catch { }
        Start-Sleep -Seconds 2
    }
    throw "$Name on $Port did not become healthy"
}

function Stop-HelixServices {
    Get-Process | Where-Object {
        $_.Name -in @("gateway", "agent_hub", "vault_service", "billing_service", "observability_service", "auth_adapter")
    } | Stop-Process -Force -ErrorAction SilentlyContinue
}

function Assert-LastExitCode {
    param([string]$Step)
    if ($LASTEXITCODE -ne 0) {
        throw "[verify] ERROR: $Step failed (exit code $LASTEXITCODE)"
    }
}

# --- Start services ---
Stop-HelixServices
Start-HelixService "gateway" "gateway"
Start-HelixService "agent-hub" "agent_hub"
Start-HelixService "vault" "vault_service"
Start-HelixService "billing" "billing_service"
Start-HelixService "observability" "observability_service"
Start-HelixService "auth-adapter" "auth_adapter"

Wait-Health 8080 "gateway"
Wait-Health 8081 "agent-hub"
Wait-Health 8082 "vault"
Wait-Health 8083 "billing"
Wait-Health 8084 "observability"
Wait-Health 8085 "auth-adapter"

# --- Deep smoke ---
Write-Host "[verify] running deep smoke..."
powershell -File "$Root\scripts\helixcore_deep_smoke.ps1"
Assert-LastExitCode "deep smoke"

# --- Export / restore roundtrip ---
$BackupDir = "C:\helixforge-backup-test"
if (Test-Path $BackupDir) { Remove-Item -Recurse -Force $BackupDir }
Write-Host "[verify] exporting backup..."
powershell -File "$Root\scripts\migrate-export.ps1" -OutDir $BackupDir
Assert-LastExitCode "backup export"

Write-Host "[verify] restoring to isolated project..."
powershell -File "$Root\scripts\migrate-restore.ps1" -BackupDir $BackupDir -ProjectName "helixforge-restore-test" -OverrideFile "$Root\deploy\local\restore.override.yml"
Assert-LastExitCode "isolated restore"

Write-Host "[verify] testing restored database..."
$env:DATABASE_URL = "postgres://helix:helix@127.0.0.1:55433/helixforge"
cargo test -p helix_db
Assert-LastExitCode "restored database tests"

# --- Cleanup ---
Write-Host "[verify] cleaning up isolated restore project..."
$env:COMPOSE_PROJECT_NAME = "helixforge-restore-test"
docker compose -f "$Root\docker-compose.yml" -f "$Root\deploy\local\restore.override.yml" down -v | Out-Null

Write-Host "[verify] stopping core services..."
Stop-HelixServices

Write-Host "[verify] DONE - HelixCore full local proof passed."
