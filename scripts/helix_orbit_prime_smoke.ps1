# HelixOrbit Prime smoke — asset/pass lifecycle, decommission guard, summary
# Prereq: helix_orbit_prime_api on 8116, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8116"

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
if ($st.phase -ne "wave2_w16") { throw "expected phase wave2_w16" }
if (-not $st.planes.asset_lifecycle) { throw "expected asset_lifecycle plane" }
if (-not $st.planes.pass_lifecycle) { throw "expected pass_lifecycle plane" }
if (-not $st.planes.decommission_guards) { throw "expected decommission_guards plane" }
if (-not $st.planes.orbit_summary) { throw "expected orbit_summary plane" }

Write-Host "=== create + commission asset ==="
$asset = (InvokeApi POST "$base/v1/assets" @{
    name = "HX-1 $(Get-Random)"
    description = "earth observation cubesat"
}).data
if ($asset.status -ne "draft") { throw "expected draft asset" }
$active = (InvokeApi POST "$base/v1/assets/$($asset.id)/commission" $null).data
if ($active.status -ne "active") { throw "expected active status" }

Write-Host "=== create pass ==="
$pass = (InvokeApi POST "$base/v1/assets/$($asset.id)/passes" @{
    title = "Window 043"
    body = "ground station north"
}).data
if ($pass.status -ne "draft") { throw "expected draft pass" }

Write-Host "=== decommission blocked by draft pass ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/assets/$($asset.id)/decommission" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for decommissioning with a draft pass"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== plan pass; decommission still blocked ==="
$planned = (InvokeApi POST "$base/v1/assets/$($asset.id)/passes/$($pass.id)/plan" $null).data
if ($planned.status -ne "planned") { throw "expected planned status" }
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/assets/$($asset.id)/decommission" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for decommissioning with a planned pass"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== complete pass ==="
$completed = (InvokeApi POST "$base/v1/assets/$($asset.id)/passes/$($pass.id)/complete" $null).data
if ($completed.status -ne "completed") { throw "expected completed status" }

Write-Host "=== second pass cancelled ==="
$pass2 = (InvokeApi POST "$base/v1/assets/$($asset.id)/passes" @{
    title = "Window 044"
}).data
$cancelled = (InvokeApi POST "$base/v1/assets/$($asset.id)/passes/$($pass2.id)/cancel" $null).data
if ($cancelled.status -ne "cancelled") { throw "expected cancelled status" }

Write-Host "=== summary reflects passes ==="
$summary = (InvokeApi GET "$base/v1/reports/orbit-summary").data
$row = $summary | Where-Object { $_.id -eq $asset.id }
if ($row.completed_passes -lt 1) { throw "expected completed_passes >= 1" }
if ($row.cancelled_passes -lt 1) { throw "expected cancelled_passes >= 1" }

Write-Host "=== decommission + recommission asset ==="
$decommissioned = (InvokeApi POST "$base/v1/assets/$($asset.id)/decommission" $null).data
if ($decommissioned.status -ne "decommissioned") { throw "expected decommissioned status" }
$recommissioned = (InvokeApi POST "$base/v1/assets/$($asset.id)/recommission" $null).data
if ($recommissioned.status -ne "active") { throw "expected active status after recommission" }

Write-Host "=== update asset + pass ==="
$upd = (InvokeApi PATCH "$base/v1/assets/$($asset.id)" @{
    description = "earth observation cubesat (refit)"
}).data
if ($upd.description -ne "earth observation cubesat (refit)") { throw "expected updated description" }
$pUpd = (InvokeApi PATCH "$base/v1/assets/$($asset.id)/passes/$($pass.id)" @{
    body = "ground station north + backup"
}).data
if ($pUpd.body -ne "ground station north + backup") { throw "expected updated pass body" }

Write-Host "=== delete + restore pass ==="
InvokeApi POST "$base/v1/assets/$($asset.id)/passes/$($pass2.id)/delete" $null | Out-Null
$passes = (InvokeApi GET "$base/v1/assets/$($asset.id)/passes").data
$gone = $passes.items | Where-Object { $_.id -eq $pass2.id }
if ($gone) { throw "deleted pass should not be listed" }
$restoredP = (InvokeApi POST "$base/v1/assets/$($asset.id)/passes/$($pass2.id)/restore" $null).data
if ($restoredP.status -ne "cancelled") { throw "expected restored pass to return to cancelled" }

Write-Host "=== delete + restore asset ==="
InvokeApi POST "$base/v1/assets/$($asset.id)/delete" $null | Out-Null
$assets = (InvokeApi GET "$base/v1/assets").data
$goneA = $assets.items | Where-Object { $_.id -eq $asset.id }
if ($goneA) { throw "deleted asset should not be listed" }
$restored = (InvokeApi POST "$base/v1/assets/$($asset.id)/restore" $null).data
if ($restored.status -ne "active") { throw "expected restored asset to return to active" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-orbit-prime") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_ORBIT_PRIME_SMOKE PASS"
exit 0
