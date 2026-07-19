# HelixNova Labs durability proof — atomic guards, forced-kill survival, restore roundtrip
# Prereq: helix_nova_labs_api on 8120 (already built), Postgres migrated,
#         docker compose postgres running, HELIX_ALLOW_DEV_HEADERS=1.
# The script force-kills and restarts the API mid-run.
# Override the binary path with HELIX_NOVA_LABS_API_BIN if needed.

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8120"
$Bin = if ($env:HELIX_NOVA_LABS_API_BIN) { $env:HELIX_NOVA_LABS_API_BIN } elseif ($IsWindows -or $env:OS -match "Windows") { "target/debug/helix_nova_labs_api.exe" } else { "./target/debug/helix_nova_labs_api" }

# The script restarts the API mid-run; the restarted process needs its env.
# Defaults mirror scripts/dev-core.ps1 (local dev only; CI overrides via job env).
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

function InvokeApi($Method, $Uri, $Body = $null) {
    if ($Body) {
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 15
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -TimeoutSec 15
}

function Wait-Health($Seconds = 60) {
    for ($i = 1; $i -le $Seconds; $i++) {
        try {
            $r = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 2
            if ($r.StatusCode -eq 200) { return }
        } catch { }
        Start-Sleep -Seconds 2
    }
    throw "helix_nova_labs_api did not become healthy"
}

function Psql($Db, $Sql) {
    $out = $null
    try {
        $out = docker compose exec -T postgres psql -U helix -d $Db -t -A -c $Sql 2>$null
    } catch { }
    return "$out".Trim()
}

Write-Host "=== healthz ==="
Wait-Health 15
Write-Host "  healthy"

Write-Host "=== experiment + finding + confirm + conclude ==="
$suffix = Get-Random
$experiment = (InvokeApi POST "$base/v1/experiments" @{
    name = "Durability experiment $suffix"
    description = "gate proof"
}).data
InvokeApi POST "$base/v1/experiments/$($experiment.id)/start" | Out-Null
$finding = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/findings" @{
    title = "Gate finding $suffix"
    body = "survives restart"
}).data
InvokeApi POST "$base/v1/experiments/$($experiment.id)/findings/$($finding.id)/confirm" | Out-Null
$concluded = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/conclude").data
if ($concluded.status -ne "concluded") { throw "expected concluded, got $($concluded.status)" }
if (-not $concluded.concluded_at) { throw "concluded_at missing" }
Write-Host "  experiment concluded with confirmed finding"

Write-Host "=== forced-kill proof: acknowledged conclusion survives ==="
Write-Host "  concluded $($experiment.id), killing API..."
Get-Process -Name helix_nova_labs_api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$check = $null
try { $check = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 3 } catch { }
if ($check -and $check.StatusCode -eq 200) { throw "API still running after kill" }
Write-Host "  API down, restarting..."
Start-Process -FilePath $Bin -WorkingDirectory (Get-Location)
Wait-Health 60
Write-Host "  API back, verifying concluded experiment and finding..."
$survivor = (InvokeApi GET "$base/v1/experiments/$($experiment.id)").data
if ($survivor.status -ne "concluded") { throw "expected concluded after kill, got $($survivor.status)" }
if ($survivor.name -ne "Durability experiment $suffix") { throw "experiment name mismatch after kill" }
if (-not $survivor.concluded_at) { throw "concluded_at missing after kill" }
$findings = (InvokeApi GET "$base/v1/experiments/$($experiment.id)/findings").data.items
$frow = $findings | Where-Object { $_.id -eq $finding.id }
if (-not $frow) { throw "finding missing after kill" }
if ($frow.title -ne "Gate finding $suffix") { throw "finding title mismatch after kill" }
if ($frow.status -ne "confirmed") { throw "expected confirmed finding after kill, got $($frow.status)" }
if (-not $frow.confirmed_at) { throw "confirmed_at missing after kill" }
Write-Host "  experiment and finding fully present"

Write-Host "=== restore proof: nova schema roundtrip ==="
$dumpPath = "/tmp/nova_durability_dump.sql"
docker compose exec -T postgres pg_dump -U helix -d helixforge --schema=nova --no-owner --no-privileges -f $dumpPath
$dumpSize = (docker compose exec -T postgres sh -c "wc -c < $dumpPath" 2>$null).Trim()
if ([int]$dumpSize -lt 100) { throw "pg_dump produced no usable dump ($dumpSize bytes)" }

Psql "postgres" "DROP DATABASE IF EXISTS nova_restore_test" | Out-Null
Psql "postgres" "CREATE DATABASE nova_restore_test" | Out-Null
docker compose exec -T postgres psql -U helix -d nova_restore_test -f $dumpPath --quiet 2>$null | Out-Null

foreach ($table in @("experiments", "findings")) {
    $src = Psql "helixforge" "SELECT COUNT(*) FROM nova.$table"
    $dst = Psql "nova_restore_test" "SELECT COUNT(*) FROM nova.$table"
    if ($src -ne $dst) { throw "$table count mismatch: source=$src restored=$dst" }
}
$hashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || title || ':' || status), '|' ORDER BY id), 'empty')) FROM nova.findings"
$srcHash = Psql "helixforge" $hashSql
$dstHash = Psql "nova_restore_test" $hashSql
if ($srcHash -ne $dstHash) { throw "findings content hash mismatch after restore" }
Write-Host "  counts and content hashes match"

Psql "postgres" "DROP DATABASE IF EXISTS nova_restore_test" | Out-Null
docker compose exec -T postgres rm -f $dumpPath | Out-Null

Write-Host ""
Write-Host "HELIX_NOVA_LABS_DURABILITY PASS"
exit 0
