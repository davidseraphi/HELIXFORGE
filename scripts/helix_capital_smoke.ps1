# HelixCapital smoke — accounts, journals, void, trial balance, snapshots
# Prereq: helix_capital_api on 8107, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8107"

function InvokeApi($Method, $Uri, $Body = $null) {
    if ($Body) {
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 15
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -TimeoutSec 15
}

Write-Host "=== healthz ==="
$code = (Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 8).StatusCode
if ($code -ne 200) { throw "healthz failed" }

Write-Host "=== domain status ==="
$st = (InvokeApi GET "$base/v1/domain/status" ).data
if ($st.phase -ne "wave2_w7") { throw "expected phase wave2_w7" }
if (-not $st.planes.account_lifecycle) { throw "expected account_lifecycle plane" }
if (-not $st.planes.journal_void) { throw "expected journal_void plane" }
if (-not $st.planes.trial_balance) { throw "expected trial_balance plane" }
if (-not $st.planes.balance_snapshots) { throw "expected balance_snapshots plane" }

Write-Host "=== create accounts ==="
$cash = (InvokeApi POST "$base/v1/accounts" @{
    code = "cash-$(Get-Random)"
    name = "Cash"
    kind = "asset"
    currency = "USD"
}).data
$revenue = (InvokeApi POST "$base/v1/accounts" @{
    code = "revenue-$(Get-Random)"
    name = "Revenue"
    kind = "revenue"
    currency = "USD"
}).data
$expense = (InvokeApi POST "$base/v1/accounts" @{
    code = "expense-$(Get-Random)"
    name = "Expense"
    kind = "expense"
    currency = "USD"
}).data

Write-Host "=== update cash name ==="
$upd = (InvokeApi PATCH "$base/v1/accounts/$($cash.id)" @{
    name = "Cash on Hand"
}).data
if ($upd.name -ne "Cash on Hand") { throw "expected updated name" }

Write-Host "=== post initial journal ==="
$journal1 = (InvokeApi POST "$base/v1/journals" @{
    memo = "sale"
    currency = "USD"
    lines = @(
        @{ account_id = $cash.id; side = "debit"; amount_cents = 5000 }
        @{ account_id = $revenue.id; side = "credit"; amount_cents = 5000 }
    )
}).data
if ($journal1.status -ne "posted") { throw "expected posted journal" }

Write-Host "=== trial balance ==="
$tb = (InvokeApi GET "$base/v1/reports/trial-balance").data
$cashRow = $tb | Where-Object { $_.id -eq $cash.id }
$revRow = $tb | Where-Object { $_.id -eq $revenue.id }
if ($cashRow.balance_cents -ne 5000) { throw "expected cash 5000" }
if ($revRow.balance_cents -ne -5000) { throw "expected revenue -5000" }

Write-Host "=== close revenue rejected with non-zero balance ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/accounts/$($revenue.id)/close" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for closing non-zero account"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== post zeroing journal ==="
$journal2 = (InvokeApi POST "$base/v1/journals" @{
    memo = "zero revenue"
    currency = "USD"
    lines = @(
        @{ account_id = $revenue.id; side = "debit"; amount_cents = 5000 }
        @{ account_id = $cash.id; side = "credit"; amount_cents = 5000 }
    )
}).data

Write-Host "=== void zeroing journal restores balances ==="
$voided = (InvokeApi POST "$base/v1/journals/$($journal2.id)/void" @{ reason = "correction" }).data
if ($voided.status -ne "voided") { throw "expected voided status" }

$tb2 = (InvokeApi GET "$base/v1/reports/trial-balance").data
$cashRow2 = $tb2 | Where-Object { $_.id -eq $cash.id }
$revRow2 = $tb2 | Where-Object { $_.id -eq $revenue.id }
if ($cashRow2.balance_cents -ne 5000) { throw "expected cash restored to 5000" }
if ($revRow2.balance_cents -ne -5000) { throw "expected revenue restored to -5000" }

Write-Host "=== close zero-balance expense succeeds ==="
$closed = (InvokeApi POST "$base/v1/accounts/$($expense.id)/close" $null).data
if ($closed.status -ne "closed") { throw "expected closed status" }

Write-Host "=== post to closed account rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/journals" -Headers $h -Body (@{
        memo = "bad"
        currency = "USD"
        lines = @(
            @{ account_id = $cash.id; side = "debit"; amount_cents = 100 }
            @{ account_id = $expense.id; side = "credit"; amount_cents = 100 }
        )
    } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for posting to closed account"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== balance snapshot ==="
$snap = (InvokeApi POST "$base/v1/reports/balance-snapshot" $null).data
if ($snap.accounts -lt 1) { throw "expected snapshot rows" }

Write-Host "=== soft-delete empty account ==="
InvokeApi POST "$base/v1/accounts/$($expense.id)/delete" $null | Out-Null
$accounts = (InvokeApi GET "$base/v1/accounts").data
$deleted = $accounts.items | Where-Object { $_.id -eq $expense.id }
if ($deleted) { throw "deleted account should not be listed" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-capital") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_CAPITAL_SMOKE PASS"
exit 0
