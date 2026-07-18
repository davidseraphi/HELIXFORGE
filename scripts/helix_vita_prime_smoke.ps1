# HelixVita Prime smoke — study/cohort lifecycle, complete guard, summary
# Prereq: helix_vita_prime_api on 8118, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8118"

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
if ($st.phase -ne "wave2_w18") { throw "expected phase wave2_w18" }
if (-not $st.planes.study_lifecycle) { throw "expected study_lifecycle plane" }
if (-not $st.planes.cohort_lifecycle) { throw "expected cohort_lifecycle plane" }
if (-not $st.planes.complete_guards) { throw "expected complete_guards plane" }
if (-not $st.planes.vita_summary) { throw "expected vita_summary plane" }

Write-Host "=== create + recruit study ==="
$study = (InvokeApi POST "$base/v1/studies" @{
    name = "Sleep & recovery $(Get-Random)"
    description = "8-week observational"
}).data
if ($study.status -ne "draft") { throw "expected draft study" }
$recruiting = (InvokeApi POST "$base/v1/studies/$($study.id)/recruit" $null).data
if ($recruiting.status -ne "recruiting") { throw "expected recruiting status" }

Write-Host "=== create cohort ==="
$cohort = (InvokeApi POST "$base/v1/studies/$($study.id)/cohorts" @{
    title = "Arm A"
    body = "wearable tracked"
}).data
if ($cohort.status -ne "draft") { throw "expected draft cohort" }

Write-Host "=== complete blocked by draft cohort ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/studies/$($study.id)/complete" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for completing with a draft cohort"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== enroll cohort ==="
$enrolled = (InvokeApi POST "$base/v1/studies/$($study.id)/cohorts/$($cohort.id)/enroll" $null).data
if ($enrolled.status -ne "enrolled") { throw "expected enrolled status" }

Write-Host "=== second cohort withdrawn ==="
$cohort2 = (InvokeApi POST "$base/v1/studies/$($study.id)/cohorts" @{
    title = "Arm B"
}).data
$withdrawn = (InvokeApi POST "$base/v1/studies/$($study.id)/cohorts/$($cohort2.id)/withdraw" $null).data
if ($withdrawn.status -ne "withdrawn") { throw "expected withdrawn status" }

Write-Host "=== summary reflects cohorts ==="
$summary = (InvokeApi GET "$base/v1/reports/vita-summary").data
$row = $summary | Where-Object { $_.id -eq $study.id }
if ($row.enrolled_cohorts -lt 1) { throw "expected enrolled_cohorts >= 1" }
if ($row.withdrawn_cohorts -lt 1) { throw "expected withdrawn_cohorts >= 1" }

Write-Host "=== complete study ==="
$completed = (InvokeApi POST "$base/v1/studies/$($study.id)/complete" $null).data
if ($completed.status -ne "completed") { throw "expected completed status" }

Write-Host "=== terminate path on second study ==="
$study2 = (InvokeApi POST "$base/v1/studies" @{
    name = "Nutrition pilot $(Get-Random)"
}).data
$terminated = (InvokeApi POST "$base/v1/studies/$($study2.id)/terminate" $null).data
if ($terminated.status -ne "terminated") { throw "expected terminated status" }

Write-Host "=== update study + cohort ==="
$upd = (InvokeApi PATCH "$base/v1/studies/$($study.id)" @{
    description = "8-week observational (extended)"
}).data
if ($upd.description -ne "8-week observational (extended)") { throw "expected updated description" }
$cUpd = (InvokeApi PATCH "$base/v1/studies/$($study.id)/cohorts/$($cohort.id)" @{
    body = "wearable + diary"
}).data
if ($cUpd.body -ne "wearable + diary") { throw "expected updated cohort body" }

Write-Host "=== delete + restore cohort ==="
InvokeApi POST "$base/v1/studies/$($study.id)/cohorts/$($cohort2.id)/delete" $null | Out-Null
$cohorts = (InvokeApi GET "$base/v1/studies/$($study.id)/cohorts").data
$gone = $cohorts.items | Where-Object { $_.id -eq $cohort2.id }
if ($gone) { throw "deleted cohort should not be listed" }
$restoredC = (InvokeApi POST "$base/v1/studies/$($study.id)/cohorts/$($cohort2.id)/restore" $null).data
if ($restoredC.status -ne "withdrawn") { throw "expected restored cohort to return to withdrawn" }

Write-Host "=== delete + restore study ==="
InvokeApi POST "$base/v1/studies/$($study2.id)/delete" $null | Out-Null
$studies = (InvokeApi GET "$base/v1/studies").data
$goneS = $studies.items | Where-Object { $_.id -eq $study2.id }
if ($goneS) { throw "deleted study should not be listed" }
$restored = (InvokeApi POST "$base/v1/studies/$($study2.id)/restore" $null).data
if ($restored.status -ne "terminated") { throw "expected restored study to return to terminated" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-vita-prime") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_VITA_PRIME_SMOKE PASS"
exit 0
