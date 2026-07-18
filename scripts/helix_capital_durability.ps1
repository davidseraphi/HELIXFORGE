# HelixCapital durability proof — atomic journals, forced-kill survival, restore roundtrip
# Prereq: helix_capital_api on 8107 (already built), Postgres migrated,
#         docker compose postgres running, HELIX_ALLOW_DEV_HEADERS=1.
# The script force-kills and restarts the API mid-run.
# Override the binary path with HELIX_CAPITAL_API_BIN if needed.

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8107"
$Bin = if ($env:HELIX_CAPITAL_API_BIN) { $env:HELIX_CAPITAL_API_BIN } elseif ($IsWindows -or $env:OS -match "Windows") { "target/debug/helix_capital_api.exe" } else { "./target/debug/helix_capital_api" }

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
    throw "helix_capital_api did not become healthy"
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

Write-Host "=== journal post + void consistency ==="
$cash = (InvokeApi POST "$base/v1/accounts" @{
    code = "dur-cash-$(Get-Random)"
    name = "Durability Cash"
    kind = "asset"
    currency = "USD"
}).data
$revenue = (InvokeApi POST "$base/v1/accounts" @{
    code = "dur-rev-$(Get-Random)"
    name = "Durability Revenue"
    kind = "revenue"
    currency = "USD"
}).data
$journal = (InvokeApi POST "$base/v1/journals" @{
    memo = "durability sale"
    currency = "USD"
    lines = @(
        @{ account_id = $cash.id; side = "debit"; amount_cents = 2500 }
        @{ account_id = $revenue.id; side = "credit"; amount_cents = 2500 }
    )
}).data
if ($journal.status -ne "posted") { throw "expected posted journal" }
$tb = (InvokeApi GET "$base/v1/reports/trial-balance").data
$cashRow = $tb | Where-Object { $_.id -eq $cash.id }
if ($cashRow.balance_cents -ne 2500) { throw "expected cash 2500, got $($cashRow.balance_cents)" }
$voided = (InvokeApi POST "$base/v1/journals/$($journal.id)/void" @{ reason = "gate check" }).data
if ($voided.status -ne "voided") { throw "expected voided journal" }
$tb2 = (InvokeApi GET "$base/v1/reports/trial-balance").data
$cashRow2 = $tb2 | Where-Object { $_.id -eq $cash.id }
if ($cashRow2.balance_cents -ne 0) { throw "expected cash 0 after void, got $($cashRow2.balance_cents)" }
Write-Host "  post/void balances consistent"

Write-Host "=== forced-kill proof: acknowledged journal survives ==="
$victim = (InvokeApi POST "$base/v1/journals" @{
    memo = "kill victim"
    currency = "USD"
    lines = @(
        @{ account_id = $cash.id; side = "debit"; amount_cents = 700 }
        @{ account_id = $revenue.id; side = "credit"; amount_cents = 700 }
    )
}).data
Write-Host "  wrote $($victim.id), killing API..."
Get-Process -Name helix_capital_api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$check = $null
try { $check = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 3 } catch { }
if ($check -and $check.StatusCode -eq 200) { throw "API still running after kill" }
Write-Host "  API down, restarting..."
Start-Process -FilePath $Bin -WorkingDirectory (Get-Location)
Wait-Health 60
Write-Host "  API back, verifying acknowledged journal..."
$survivor = (InvokeApi GET "$base/v1/journals/$($victim.id)").data
if ($survivor.status -ne "posted") { throw "expected posted status after kill" }
if ($survivor.lines.Count -ne 2) { throw "expected 2 lines after kill, got $($survivor.lines.Count)" }
$tb3 = (InvokeApi GET "$base/v1/reports/trial-balance").data
$cashRow3 = $tb3 | Where-Object { $_.id -eq $cash.id }
if ($cashRow3.balance_cents -ne 700) { throw "expected cash 700 after kill, got $($cashRow3.balance_cents)" }
Write-Host "  journal, lines, and balances fully present"

Write-Host "=== restore proof: capital schema roundtrip ==="
$dumpPath = "/tmp/capital_durability_dump.sql"
docker compose exec -T postgres pg_dump -U helix -d helixforge --schema=capital --no-owner --no-privileges -f $dumpPath
$dumpSize = (docker compose exec -T postgres sh -c "wc -c < $dumpPath" 2>$null).Trim()
if ([int]$dumpSize -lt 100) { throw "pg_dump produced no usable dump ($dumpSize bytes)" }

Psql "postgres" "DROP DATABASE IF EXISTS capital_restore_test" | Out-Null
Psql "postgres" "CREATE DATABASE capital_restore_test" | Out-Null
docker compose exec -T postgres psql -U helix -d capital_restore_test -f $dumpPath --quiet 2>$null | Out-Null

foreach ($table in @("accounts", "journals", "journal_lines")) {
    $src = Psql "helixforge" "SELECT COUNT(*) FROM capital.$table"
    $dst = Psql "capital_restore_test" "SELECT COUNT(*) FROM capital.$table"
    if ($src -ne $dst) { throw "$table count mismatch: source=$src restored=$dst" }
}
$hashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || code || ':' || balance_cents), '|' ORDER BY code), 'empty')) FROM capital.accounts"
$srcHash = Psql "helixforge" $hashSql
$dstHash = Psql "capital_restore_test" $hashSql
if ($srcHash -ne $dstHash) { throw "accounts content hash mismatch after restore" }
Write-Host "  counts and content hashes match"

Psql "postgres" "DROP DATABASE IF EXISTS capital_restore_test" | Out-Null
docker compose exec -T postgres rm -f $dumpPath | Out-Null

Write-Host ""
Write-Host "HELIX_CAPITAL_DURABILITY PASS"
exit 0
