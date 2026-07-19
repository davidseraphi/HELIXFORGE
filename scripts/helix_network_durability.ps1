# HelixNetwork durability proof — atomic requests, forced-kill survival, restore roundtrip
# Prereq: helix_network_api on 8109 (already built), Postgres migrated,
#         docker compose postgres running, HELIX_ALLOW_DEV_HEADERS=1.
# The script force-kills and restarts the API mid-run.
# Override the binary path with HELIX_NETWORK_API_BIN if needed.

$ErrorActionPreference = "Stop"
$base = "http://127.0.0.1:8109"
$Bin = if ($env:HELIX_NETWORK_API_BIN) { $env:HELIX_NETWORK_API_BIN } elseif ($IsWindows -or $env:OS -match "Windows") { "target/debug/helix_network_api.exe" } else { "./target/debug/helix_network_api" }

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

# Fresh users per run so repeat runs never collide with leftover state.
$suffix = Get-Random
$hA = @{ "x-helix-dev-user" = "net-ops-$suffix@helixforge.local"; "Content-Type" = "application/json" }
$hB = @{ "x-helix-dev-user" = "net-bob-$suffix@helixforge.local"; "Content-Type" = "application/json" }

function InvokeApi($Headers, $Method, $Uri, $Body = $null) {
    if ($Body) {
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $Headers -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 15
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $Headers -TimeoutSec 15
}

function Wait-Health($Seconds = 60) {
    for ($i = 1; $i -le $Seconds; $i++) {
        try {
            $r = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 2
            if ($r.StatusCode -eq 200) { return }
        } catch { }
        Start-Sleep -Seconds 2
    }
    throw "helix_network_api did not become healthy"
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

Write-Host "=== profiles + request + accept ==="
$alice = (InvokeApi $hA POST "$base/v1/profiles" @{
    display_name = "Durability Ops $suffix"
    headline = "gate runner"
    location = "local"
}).data
$bob = (InvokeApi $hB POST "$base/v1/profiles" @{
    display_name = "Durability Bob $suffix"
    headline = "gate runner"
}).data
$conn = (InvokeApi $hA POST "$base/v1/connections" @{
    to_profile_id = $bob.id
    message = "durability $suffix"
}).data
if ($conn.status -ne "pending") { throw "expected pending request, got $($conn.status)" }
InvokeApi $hB POST "$base/v1/connections/$($conn.id)/accept" | Out-Null
$opp = (InvokeApi $hA POST "$base/v1/opportunities" @{
    title = "Durability role $suffix"
    description = "gate proof"
    kind = "role"
}).data
$listed = (InvokeApi $hA GET "$base/v1/connections?profile_id=$($alice.id)").data.items
$row = $listed | Where-Object { $_.id -eq $conn.id }
if (-not $row) { throw "connection missing from list after accept" }
if ($row.status -ne "accepted") { throw "expected accepted, got $($row.status)" }
Write-Host "  connection accepted, opportunity posted"

Write-Host "=== forced-kill proof: acknowledged connection survives ==="
Write-Host "  accepted $($conn.id), killing API..."
Get-Process -Name helix_network_api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$check = $null
try { $check = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 3 } catch { }
if ($check -and $check.StatusCode -eq 200) { throw "API still running after kill" }
Write-Host "  API down, restarting..."
Start-Process -FilePath $Bin -WorkingDirectory (Get-Location)
Wait-Health 60
Write-Host "  API back, verifying acknowledged connection..."
$survived = (InvokeApi $hA GET "$base/v1/connections?profile_id=$($alice.id)").data.items
$srow = $survived | Where-Object { $_.id -eq $conn.id }
if (-not $srow) { throw "connection missing after kill" }
if ($srow.status -ne "accepted") { throw "expected accepted after kill, got $($srow.status)" }
if ($srow.message -ne "durability $suffix") { throw "message mismatch after kill" }
if ($srow.from_profile_id -ne $alice.id) { throw "from_profile_id mismatch after kill" }
if ($srow.to_profile_id -ne $bob.id) { throw "to_profile_id mismatch after kill" }
$sopp = (InvokeApi $hA GET "$base/v1/opportunities/$($opp.id)").data
if ($sopp.title -ne "Durability role $suffix") { throw "opportunity title mismatch after kill" }
Write-Host "  connection and opportunity fully present"

Write-Host "=== restore proof: network schema roundtrip ==="
$dumpPath = "/tmp/network_durability_dump.sql"
docker compose exec -T postgres pg_dump -U helix -d helixforge --schema=network --no-owner --no-privileges -f $dumpPath
$dumpSize = (docker compose exec -T postgres sh -c "wc -c < $dumpPath" 2>$null).Trim()
if ([int]$dumpSize -lt 100) { throw "pg_dump produced no usable dump ($dumpSize bytes)" }

Psql "postgres" "DROP DATABASE IF EXISTS network_restore_test" | Out-Null
Psql "postgres" "CREATE DATABASE network_restore_test" | Out-Null
docker compose exec -T postgres psql -U helix -d network_restore_test -f $dumpPath --quiet 2>$null | Out-Null

foreach ($table in @("profiles", "connections", "opportunities")) {
    $src = Psql "helixforge" "SELECT COUNT(*) FROM network.$table"
    $dst = Psql "network_restore_test" "SELECT COUNT(*) FROM network.$table"
    if ($src -ne $dst) { throw "$table count mismatch: source=$src restored=$dst" }
}
$hashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || status || ':' || message), '|' ORDER BY id), 'empty')) FROM network.connections"
$srcHash = Psql "helixforge" $hashSql
$dstHash = Psql "network_restore_test" $hashSql
if ($srcHash -ne $dstHash) { throw "connections content hash mismatch after restore" }
Write-Host "  counts and content hashes match"

Psql "postgres" "DROP DATABASE IF EXISTS network_restore_test" | Out-Null
docker compose exec -T postgres rm -f $dumpPath | Out-Null

Write-Host ""
Write-Host "HELIX_NETWORK_DURABILITY PASS"
exit 0
