# HelixTerra Prime smoke — field/observation lifecycle, retire guard, summary
# Prereq: helix_terra_prime_api on 8114, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8114"

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
if ($st.phase -ne "wave2_w14") { throw "expected phase wave2_w14" }
if (-not $st.planes.field_lifecycle) { throw "expected field_lifecycle plane" }
if (-not $st.planes.observation_lifecycle) { throw "expected observation_lifecycle plane" }
if (-not $st.planes.retire_guards) { throw "expected retire_guards plane" }
if (-not $st.planes.terra_summary) { throw "expected terra_summary plane" }

Write-Host "=== create + activate field ==="
$field = (InvokeApi POST "$base/v1/fields" @{
    name = "North 40 $(Get-Random)"
    description = "winter wheat plot"
}).data
if ($field.status -ne "draft") { throw "expected draft field" }
$active = (InvokeApi POST "$base/v1/fields/$($field.id)/activate" $null).data
if ($active.status -ne "active") { throw "expected active status" }

Write-Host "=== create observation ==="
$obs = (InvokeApi POST "$base/v1/fields/$($field.id)/observations" @{
    title = "Soil moisture"
    body = "18% at 10cm"
}).data
if ($obs.status -ne "draft") { throw "expected draft observation" }

Write-Host "=== retire blocked by draft observation ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/fields/$($field.id)/retire" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for retiring with a draft observation"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== confirm observation ==="
$confirmed = (InvokeApi POST "$base/v1/fields/$($field.id)/observations/$($obs.id)/confirm" $null).data
if ($confirmed.status -ne "confirmed") { throw "expected confirmed status" }
if (-not $confirmed.confirmed_at) { throw "expected confirmed_at set" }

Write-Host "=== second observation dismissed ==="
$obs2 = (InvokeApi POST "$base/v1/fields/$($field.id)/observations" @{
    title = "Weed pressure"
}).data
$dismissed = (InvokeApi POST "$base/v1/fields/$($field.id)/observations/$($obs2.id)/dismiss" $null).data
if ($dismissed.status -ne "dismissed") { throw "expected dismissed status" }

Write-Host "=== summary reflects observations ==="
$summary = (InvokeApi GET "$base/v1/reports/terra-summary").data
$row = $summary | Where-Object { $_.id -eq $field.id }
if ($row.confirmed_observations -lt 1) { throw "expected confirmed_observations >= 1" }
if ($row.dismissed_observations -lt 1) { throw "expected dismissed_observations >= 1" }

Write-Host "=== retire + reopen field ==="
$retired = (InvokeApi POST "$base/v1/fields/$($field.id)/retire" $null).data
if ($retired.status -ne "retired") { throw "expected retired status" }
$reopened = (InvokeApi POST "$base/v1/fields/$($field.id)/reopen" $null).data
if ($reopened.status -ne "active") { throw "expected active status after reopen" }

Write-Host "=== update field + observation ==="
$upd = (InvokeApi PATCH "$base/v1/fields/$($field.id)" @{
    description = "winter wheat plot (west half)"
}).data
if ($upd.description -ne "winter wheat plot (west half)") { throw "expected updated description" }
$oUpd = (InvokeApi PATCH "$base/v1/fields/$($field.id)/observations/$($obs.id)" @{
    body = "19% at 10cm"
}).data
if ($oUpd.body -ne "19% at 10cm") { throw "expected updated observation body" }

Write-Host "=== delete + restore observation ==="
InvokeApi POST "$base/v1/fields/$($field.id)/observations/$($obs2.id)/delete" $null | Out-Null
$observations = (InvokeApi GET "$base/v1/fields/$($field.id)/observations").data
$gone = $observations.items | Where-Object { $_.id -eq $obs2.id }
if ($gone) { throw "deleted observation should not be listed" }
$restoredO = (InvokeApi POST "$base/v1/fields/$($field.id)/observations/$($obs2.id)/restore" $null).data
if ($restoredO.status -ne "dismissed") { throw "expected restored observation to return to dismissed" }

Write-Host "=== delete + restore field ==="
InvokeApi POST "$base/v1/fields/$($field.id)/delete" $null | Out-Null
$fields = (InvokeApi GET "$base/v1/fields").data
$goneF = $fields.items | Where-Object { $_.id -eq $field.id }
if ($goneF) { throw "deleted field should not be listed" }
$restored = (InvokeApi POST "$base/v1/fields/$($field.id)/restore" $null).data
if ($restored.status -ne "active") { throw "expected restored field to return to active" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-terra-prime") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_TERRA_PRIME_SMOKE PASS"
exit 0
