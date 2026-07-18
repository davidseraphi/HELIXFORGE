# HelixClimate Prime smoke — scenario/score lifecycle, archive guard, summary
# Prereq: helix_climate_prime_api on 8115, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8115"

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
$st = (InvokeApi GET "$base/v1/domain/status").data
if ($st.phase -ne "wave2_w15") { throw "expected phase wave2_w15" }
if (-not $st.planes.scenario_lifecycle) { throw "expected scenario_lifecycle plane" }
if (-not $st.planes.score_lifecycle) { throw "expected score_lifecycle plane" }
if (-not $st.planes.archive_guards) { throw "expected archive_guards plane" }
if (-not $st.planes.climate_summary) { throw "expected climate_summary plane" }

Write-Host "=== create + activate scenario ==="
$scenario = (InvokeApi POST "$base/v1/scenarios" @{
    name = "RCP 4.5 2050 $(Get-Random)"
    description = "mid-range emissions"
}).data
if ($scenario.status -ne "draft") { throw "expected draft scenario" }
$active = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/activate" $null).data
if ($active.status -ne "active") { throw "expected active status" }

Write-Host "=== create score ==="
$score = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/risk_scores" @{
    title = "Flood exposure"
    body = "region NE lowlands"
}).data
if ($score.status -ne "draft") { throw "expected draft score" }

Write-Host "=== archive blocked by draft score ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/scenarios/$($scenario.id)/archive" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for archiving with a draft score"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== assess score ==="
$assessed = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/risk_scores/$($score.id)/assess" $null).data
if ($assessed.status -ne "assessed") { throw "expected assessed status" }
if (-not $assessed.assessed_at) { throw "expected assessed_at set" }

Write-Host "=== second score dismissed ==="
$score2 = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/risk_scores" @{
    title = "Heat stress"
}).data
$dismissed = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/risk_scores/$($score2.id)/dismiss" $null).data
if ($dismissed.status -ne "dismissed") { throw "expected dismissed status" }

Write-Host "=== summary reflects scores ==="
$summary = (InvokeApi GET "$base/v1/reports/climate-summary").data
$row = $summary | Where-Object { $_.id -eq $scenario.id }
if ($row.assessed_scores -lt 1) { throw "expected assessed_scores >= 1" }
if ($row.dismissed_scores -lt 1) { throw "expected dismissed_scores >= 1" }

Write-Host "=== archive + reopen scenario ==="
$archived = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/archive" $null).data
if ($archived.status -ne "archived") { throw "expected archived status" }
$reopened = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/reopen" $null).data
if ($reopened.status -ne "active") { throw "expected active status after reopen" }

Write-Host "=== update scenario + score ==="
$upd = (InvokeApi PATCH "$base/v1/scenarios/$($scenario.id)" @{
    description = "mid-range emissions (rev2)"
}).data
if ($upd.description -ne "mid-range emissions (rev2)") { throw "expected updated description" }
$sUpd = (InvokeApi PATCH "$base/v1/scenarios/$($scenario.id)/risk_scores/$($score.id)" @{
    body = "region NE lowlands + river delta"
}).data
if ($sUpd.body -ne "region NE lowlands + river delta") { throw "expected updated score body" }

Write-Host "=== delete + restore score ==="
InvokeApi POST "$base/v1/scenarios/$($scenario.id)/risk_scores/$($score2.id)/delete" $null | Out-Null
$scores = (InvokeApi GET "$base/v1/scenarios/$($scenario.id)/risk_scores").data
$gone = $scores.items | Where-Object { $_.id -eq $score2.id }
if ($gone) { throw "deleted score should not be listed" }
$restoredS = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/risk_scores/$($score2.id)/restore" $null).data
if ($restoredS.status -ne "dismissed") { throw "expected restored score to return to dismissed" }

Write-Host "=== delete + restore scenario ==="
InvokeApi POST "$base/v1/scenarios/$($scenario.id)/delete" $null | Out-Null
$scenarios = (InvokeApi GET "$base/v1/scenarios").data
$goneS = $scenarios.items | Where-Object { $_.id -eq $scenario.id }
if ($goneS) { throw "deleted scenario should not be listed" }
$restored = (InvokeApi POST "$base/v1/scenarios/$($scenario.id)/restore" $null).data
if ($restored.status -ne "active") { throw "expected restored scenario to return to active" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-climate-prime") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_CLIMATE_PRIME_SMOKE PASS"
exit 0
