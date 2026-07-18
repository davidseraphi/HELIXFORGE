# HelixFlow durability proof — immutable terminal runs, forced-kill survival, restore roundtrip
# Prereq: helix_flow_api on 8103 (already built), Postgres migrated,
#         docker compose postgres running, HELIX_ALLOW_DEV_HEADERS=1.
# The script force-kills and restarts the API mid-run.
# Override the binary path with HELIX_FLOW_API_BIN if needed.

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8103"
$Bin = if ($env:HELIX_FLOW_API_BIN) { $env:HELIX_FLOW_API_BIN } elseif ($IsWindows -or $env:OS -match "Windows") { "target/debug/helix_flow_api.exe" } else { "./target/debug/helix_flow_api" }

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
    throw "helix_flow_api did not become healthy"
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

Write-Host "=== domain status ==="
$st = (InvokeApi GET "$base/v1/domain/status").data
if (-not $st.durable) { throw "expected durable domain" }

Write-Host "=== workflow + run lifecycle ==="
$wf = (InvokeApi POST "$base/v1/workflows" @{
    name = "durability-flow-$(Get-Random)"
    steps = 2
    definition = @{ steps = @("a", "b") }
}).data
$exec = (InvokeApi POST "$base/v1/workflows/$($wf.id)/runs" $null).data
$run = $exec.run
if (-not $run.id) { throw "expected a run" }
if ($run.status -ne "succeeded") { throw "expected succeeded run (in-process engine), got $($run.status)" }

Write-Host "=== forced-kill proof: acknowledged run survives ==="
Write-Host "  wrote $($run.id), killing API..."
Get-Process -Name helix_flow_api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$check = $null
try { $check = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 3 } catch { }
if ($check -and $check.StatusCode -eq 200) { throw "API still running after kill" }
Write-Host "  API down, restarting..."
Start-Process -FilePath $Bin -WorkingDirectory (Get-Location)
Wait-Health 60
Write-Host "  API back, verifying acknowledged run..."
$survivor = (InvokeApi GET "$base/v1/runs/$($run.id)").data
if ($survivor.run.id -ne $run.id) { throw "run id mismatch after kill" }
if ($survivor.run.workflow_id -ne $wf.id) { throw "workflow id mismatch after kill" }
if ($survivor.run.status -ne "succeeded") { throw "expected succeeded status after kill" }
Write-Host "  run fully present"

Write-Host "=== restore proof: flow schema roundtrip ==="
$dumpPath = "/tmp/flow_durability_dump.sql"
docker compose exec -T postgres pg_dump -U helix -d helixforge --schema=flow --no-owner --no-privileges -f $dumpPath
$dumpSize = (docker compose exec -T postgres sh -c "wc -c < $dumpPath" 2>$null).Trim()
if ([int]$dumpSize -lt 100) { throw "pg_dump produced no usable dump ($dumpSize bytes)" }

Psql "postgres" "DROP DATABASE IF EXISTS flow_restore_test" | Out-Null
Psql "postgres" "CREATE DATABASE flow_restore_test" | Out-Null
docker compose exec -T postgres psql -U helix -d flow_restore_test -f $dumpPath --quiet 2>$null | Out-Null

foreach ($table in @("workflows", "runs", "step_events")) {
    $src = Psql "helixforge" "SELECT COUNT(*) FROM flow.$table"
    $dst = Psql "flow_restore_test" "SELECT COUNT(*) FROM flow.$table"
    if ($src -ne $dst) { throw "$table count mismatch: source=$src restored=$dst" }
}
$hashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || status || ':' || COALESCE(finished_at::text, '-')), '|' ORDER BY id), 'empty')) FROM flow.runs"
$srcHash = Psql "helixforge" $hashSql
$dstHash = Psql "flow_restore_test" $hashSql
if ($srcHash -ne $dstHash) { throw "runs content hash mismatch after restore" }
Write-Host "  counts and content hashes match"

Psql "postgres" "DROP DATABASE IF EXISTS flow_restore_test" | Out-Null
docker compose exec -T postgres rm -f $dumpPath | Out-Null

Write-Host ""
Write-Host "HELIX_FLOW_DURABILITY PASS"
exit 0
