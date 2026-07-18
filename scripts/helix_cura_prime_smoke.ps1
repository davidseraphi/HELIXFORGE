# HelixCura Prime smoke — case/note lifecycle, discharge guard, signed immutability
# Prereq: helix_cura_prime_api on 8113, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8113"

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
if ($st.phase -ne "wave2_w13") { throw "expected phase wave2_w13" }
if (-not $st.planes.case_lifecycle) { throw "expected case_lifecycle plane" }
if (-not $st.planes.note_lifecycle) { throw "expected note_lifecycle plane" }
if (-not $st.planes.signed_immutable) { throw "expected signed_immutable plane" }
if (-not $st.planes.discharge_guards) { throw "expected discharge_guards plane" }

Write-Host "=== create + activate case ==="
$case = (InvokeApi POST "$base/v1/care_cases" @{
    name = "Case Rivera $(Get-Random)"
    description = "post-op followup"
}).data
if ($case.status -ne "draft") { throw "expected draft case" }
$active = (InvokeApi POST "$base/v1/care_cases/$($case.id)/activate" $null).data
if ($active.status -ne "active") { throw "expected active status" }

Write-Host "=== create note ==="
$note = (InvokeApi POST "$base/v1/care_cases/$($case.id)/notes" @{
    title = "Round 1"
    body = "vitals stable"
}).data
if ($note.status -ne "draft") { throw "expected draft note" }

Write-Host "=== discharge blocked by draft note ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/care_cases/$($case.id)/discharge" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for discharging with a draft note"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== sign note ==="
$signed = (InvokeApi POST "$base/v1/care_cases/$($case.id)/notes/$($note.id)/sign" $null).data
if ($signed.status -ne "signed") { throw "expected signed status" }
if (-not $signed.signed_at) { throw "expected signed_at set" }

Write-Host "=== signed note is immutable ==="
try {
    Invoke-RestMethod -Method PATCH -Uri "$base/v1/care_cases/$($case.id)/notes/$($note.id)" -Headers $h -Body (@{ body = "edited" } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for editing a signed note"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== second note voided ==="
$note2 = (InvokeApi POST "$base/v1/care_cases/$($case.id)/notes" @{
    title = "Round 2"
}).data
$voided = (InvokeApi POST "$base/v1/care_cases/$($case.id)/notes/$($note2.id)/void" $null).data
if ($voided.status -ne "voided") { throw "expected voided status" }

Write-Host "=== summary reflects notes ==="
$summary = (InvokeApi GET "$base/v1/reports/cura-summary").data
$row = $summary | Where-Object { $_.id -eq $case.id }
if ($row.signed_notes -lt 1) { throw "expected signed_notes >= 1" }
if ($row.voided_notes -lt 1) { throw "expected voided_notes >= 1" }

Write-Host "=== discharge + reopen case ==="
$discharged = (InvokeApi POST "$base/v1/care_cases/$($case.id)/discharge" $null).data
if ($discharged.status -ne "discharged") { throw "expected discharged status" }
$reopened = (InvokeApi POST "$base/v1/care_cases/$($case.id)/reopen" $null).data
if ($reopened.status -ne "active") { throw "expected active status after reopen" }

Write-Host "=== update case + draft note ==="
$upd = (InvokeApi PATCH "$base/v1/care_cases/$($case.id)" @{
    description = "post-op followup (revised)"
}).data
if ($upd.description -ne "post-op followup (revised)") { throw "expected updated description" }
$note3 = (InvokeApi POST "$base/v1/care_cases/$($case.id)/notes" @{
    title = "Round 3"
    body = "initial"
}).data
$nUpd = (InvokeApi PATCH "$base/v1/care_cases/$($case.id)/notes/$($note3.id)" @{
    body = "revised"
}).data
if ($nUpd.body -ne "revised") { throw "expected updated draft note body" }

Write-Host "=== delete + restore note ==="
InvokeApi POST "$base/v1/care_cases/$($case.id)/notes/$($note2.id)/delete" $null | Out-Null
$notes = (InvokeApi GET "$base/v1/care_cases/$($case.id)/notes").data
$gone = $notes.items | Where-Object { $_.id -eq $note2.id }
if ($gone) { throw "deleted note should not be listed" }
$restoredN = (InvokeApi POST "$base/v1/care_cases/$($case.id)/notes/$($note2.id)/restore" $null).data
if ($restoredN.status -ne "voided") { throw "expected restored note to return to voided" }

Write-Host "=== delete + restore case ==="
InvokeApi POST "$base/v1/care_cases/$($case.id)/delete" $null | Out-Null
$cases = (InvokeApi GET "$base/v1/care_cases").data
$goneC = $cases.items | Where-Object { $_.id -eq $case.id }
if ($goneC) { throw "deleted case should not be listed" }
$restored = (InvokeApi POST "$base/v1/care_cases/$($case.id)/restore" $null).data
if ($restored.status -ne "active") { throw "expected restored case to return to active" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-cura-prime") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_CURA_PRIME_SMOKE PASS"
exit 0
