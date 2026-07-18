# HelixGrid Prime smoke — site/reading lifecycle, offline guard, summary
# Prereq: helix_grid_prime_api on 8119, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8119"

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
if ($st.phase -ne "wave2_w19") { throw "expected phase wave2_w19" }
if (-not $st.planes.site_lifecycle) { throw "expected site_lifecycle plane" }
if (-not $st.planes.reading_lifecycle) { throw "expected reading_lifecycle plane" }
if (-not $st.planes.offline_guards) { throw "expected offline_guards plane" }
if (-not $st.planes.grid_summary) { throw "expected grid_summary plane" }

Write-Host "=== create + energize site ==="
$site = (InvokeApi POST "$base/v1/sites" @{
    name = "Substation 7 $(Get-Random)"
    description = "north feeder"
}).data
if ($site.status -ne "draft") { throw "expected draft site" }
$active = (InvokeApi POST "$base/v1/sites/$($site.id)/energize" $null).data
if ($active.status -ne "active") { throw "expected active status" }

Write-Host "=== create reading ==="
$reading = (InvokeApi POST "$base/v1/sites/$($site.id)/readings" @{
    title = "Load 10:00"
    body = "4.2 MW"
}).data
if ($reading.status -ne "draft") { throw "expected draft reading" }

Write-Host "=== offline blocked by draft reading ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/sites/$($site.id)/offline" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for going offline with a draft reading"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== verify reading ==="
$verified = (InvokeApi POST "$base/v1/sites/$($site.id)/readings/$($reading.id)/verify" $null).data
if ($verified.status -ne "verified") { throw "expected verified status" }

Write-Host "=== second reading rejected ==="
$reading2 = (InvokeApi POST "$base/v1/sites/$($site.id)/readings" @{
    title = "Load 10:15"
}).data
$rejected = (InvokeApi POST "$base/v1/sites/$($site.id)/readings/$($reading2.id)/reject" $null).data
if ($rejected.status -ne "rejected") { throw "expected rejected status" }

Write-Host "=== summary reflects readings ==="
$summary = (InvokeApi GET "$base/v1/reports/grid-summary").data
$row = $summary | Where-Object { $_.id -eq $site.id }
if ($row.verified_readings -lt 1) { throw "expected verified_readings >= 1" }
if ($row.rejected_readings -lt 1) { throw "expected rejected_readings >= 1" }

Write-Host "=== offline + online site ==="
$offline = (InvokeApi POST "$base/v1/sites/$($site.id)/offline" $null).data
if ($offline.status -ne "offline") { throw "expected offline status" }
$online = (InvokeApi POST "$base/v1/sites/$($site.id)/online" $null).data
if ($online.status -ne "active") { throw "expected active status after online" }

Write-Host "=== update site + reading ==="
$upd = (InvokeApi PATCH "$base/v1/sites/$($site.id)" @{
    description = "north feeder (upgraded)"
}).data
if ($upd.description -ne "north feeder (upgraded)") { throw "expected updated description" }
$rUpd = (InvokeApi PATCH "$base/v1/sites/$($site.id)/readings/$($reading.id)" @{
    body = "4.3 MW"
}).data
if ($rUpd.body -ne "4.3 MW") { throw "expected updated reading body" }

Write-Host "=== delete + restore reading ==="
InvokeApi POST "$base/v1/sites/$($site.id)/readings/$($reading2.id)/delete" $null | Out-Null
$readings = (InvokeApi GET "$base/v1/sites/$($site.id)/readings").data
$gone = $readings.items | Where-Object { $_.id -eq $reading2.id }
if ($gone) { throw "deleted reading should not be listed" }
$restoredR = (InvokeApi POST "$base/v1/sites/$($site.id)/readings/$($reading2.id)/restore" $null).data
if ($restoredR.status -ne "rejected") { throw "expected restored reading to return to rejected" }

Write-Host "=== delete + restore site ==="
InvokeApi POST "$base/v1/sites/$($site.id)/delete" $null | Out-Null
$sites = (InvokeApi GET "$base/v1/sites").data
$goneS = $sites.items | Where-Object { $_.id -eq $site.id }
if ($goneS) { throw "deleted site should not be listed" }
$restored = (InvokeApi POST "$base/v1/sites/$($site.id)/restore" $null).data
if ($restored.status -ne "active") { throw "expected restored site to return to active" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-grid-prime") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_GRID_PRIME_SMOKE PASS"
exit 0
