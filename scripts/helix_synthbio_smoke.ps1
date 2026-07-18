# HelixSynthBio smoke — design review lifecycle, sim runs, approval guard, summary
# Prereq: helix_synthbio_api on 8111, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8111"

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
if ($st.phase -ne "wave2_w11") { throw "expected phase wave2_w11" }
if (-not $st.planes.design_lifecycle) { throw "expected design_lifecycle plane" }
if (-not $st.planes.sim_lifecycle) { throw "expected sim_lifecycle plane" }
if (-not $st.planes.approval_guards) { throw "expected approval_guards plane" }
if (-not $st.planes.synthbio_summary) { throw "expected synthbio_summary plane" }

Write-Host "=== create design ==="
$design = (InvokeApi POST "$base/v1/designs" @{
    name = "Promoter study $(Get-Random)"
    description = "weak promoter variants"
}).data
if ($design.status -ne "draft") { throw "expected draft design" }

Write-Host "=== submit design ==="
$submitted = (InvokeApi POST "$base/v1/designs/$($design.id)/submit" $null).data
if ($submitted.status -ne "review") { throw "expected review status" }

Write-Host "=== approve without completed sim rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/designs/$($design.id)/approve" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for approving without a completed sim"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== create sim ==="
$sim = (InvokeApi POST "$base/v1/designs/$($design.id)/sims" @{
    title = "growth curve"
    body = "37C 24h"
}).data
if ($sim.status -ne "open") { throw "expected open sim" }

Write-Host "=== complete before start rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/designs/$($design.id)/sims/$($sim.id)/complete" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for completing an open sim"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== start + complete sim ==="
$running = (InvokeApi POST "$base/v1/designs/$($design.id)/sims/$($sim.id)/start" $null).data
if ($running.status -ne "running") { throw "expected running status" }
$completed = (InvokeApi POST "$base/v1/designs/$($design.id)/sims/$($sim.id)/complete" $null).data
if ($completed.status -ne "completed") { throw "expected completed status" }

Write-Host "=== second sim fails ==="
$sim2 = (InvokeApi POST "$base/v1/designs/$($design.id)/sims" @{
    title = "toxicity screen"
}).data
InvokeApi POST "$base/v1/designs/$($design.id)/sims/$($sim2.id)/start" $null | Out-Null
$failed = (InvokeApi POST "$base/v1/designs/$($design.id)/sims/$($sim2.id)/fail" $null).data
if ($failed.status -ne "failed") { throw "expected failed status" }

Write-Host "=== summary reflects outcomes ==="
$summary = (InvokeApi GET "$base/v1/reports/synthbio-summary").data
$row = $summary | Where-Object { $_.id -eq $design.id }
if ($row.completed_sims -lt 1) { throw "expected completed_sims >= 1" }
if ($row.failed_sims -lt 1) { throw "expected failed_sims >= 1" }

Write-Host "=== approve design ==="
$approved = (InvokeApi POST "$base/v1/designs/$($design.id)/approve" $null).data
if ($approved.status -ne "approved") { throw "expected approved status" }
if (-not $approved.approved_at) { throw "expected approved_at set" }

Write-Host "=== update design ==="
$upd = (InvokeApi PATCH "$base/v1/designs/$($design.id)" @{
    description = "promoter variants v2"
}).data
if ($upd.description -ne "promoter variants v2") { throw "expected updated description" }

Write-Host "=== update sim ==="
$simUpd = (InvokeApi PATCH "$base/v1/designs/$($design.id)/sims/$($sim.id)" @{
    body = "37C 24h + controls"
}).data
if ($simUpd.body -ne "37C 24h + controls") { throw "expected updated body" }

Write-Host "=== return path on second design ==="
$design2 = (InvokeApi POST "$base/v1/designs" @{
    name = "Assay panel $(Get-Random)"
}).data
InvokeApi POST "$base/v1/designs/$($design2.id)/submit" $null | Out-Null
$returned = (InvokeApi POST "$base/v1/designs/$($design2.id)/return" $null).data
if ($returned.status -ne "draft") { throw "expected draft status after return" }

Write-Host "=== delete + restore sim ==="
InvokeApi POST "$base/v1/designs/$($design.id)/sims/$($sim2.id)/delete" $null | Out-Null
$sims = (InvokeApi GET "$base/v1/designs/$($design.id)/sims").data
$gone = $sims.items | Where-Object { $_.id -eq $sim2.id }
if ($gone) { throw "deleted sim should not be listed" }
$restoredSim = (InvokeApi POST "$base/v1/designs/$($design.id)/sims/$($sim2.id)/restore" $null).data
if ($restoredSim.status -ne "failed") { throw "expected restored sim to return to failed" }

Write-Host "=== delete + restore design ==="
InvokeApi POST "$base/v1/designs/$($design.id)/delete" $null | Out-Null
$designs = (InvokeApi GET "$base/v1/designs").data
$goneD = $designs.items | Where-Object { $_.id -eq $design.id }
if ($goneD) { throw "deleted design should not be listed" }
$restored = (InvokeApi POST "$base/v1/designs/$($design.id)/restore" $null).data
if ($restored.status -ne "approved") { throw "expected restored design to return to approved" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-synthbio") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_SYNTHBIO_SMOKE PASS"
exit 0
