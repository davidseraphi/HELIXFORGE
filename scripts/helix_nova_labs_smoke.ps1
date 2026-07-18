# HelixNova Labs smoke — experiment/finding lifecycle, conclude guard, summary
# Prereq: helix_nova_labs_api on 8120, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8120"

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
if ($st.phase -ne "wave2_w20") { throw "expected phase wave2_w20" }
if (-not $st.planes.experiment_lifecycle) { throw "expected experiment_lifecycle plane" }
if (-not $st.planes.finding_lifecycle) { throw "expected finding_lifecycle plane" }
if (-not $st.planes.conclude_guards) { throw "expected conclude_guards plane" }
if (-not $st.planes.nova_summary) { throw "expected nova_summary plane" }

Write-Host "=== create + start experiment ==="
$experiment = (InvokeApi POST "$base/v1/experiments" @{
    name = "Catalyst screen $(Get-Random)"
    description = "12-candidate panel"
}).data
if ($experiment.status -ne "draft") { throw "expected draft experiment" }
$running = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/start" $null).data
if ($running.status -ne "running") { throw "expected running status" }

Write-Host "=== create finding ==="
$finding = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/findings" @{
    title = "Candidate 4 yield"
    body = "71% at 60C"
}).data
if ($finding.status -ne "draft") { throw "expected draft finding" }

Write-Host "=== conclude blocked by draft finding ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/experiments/$($experiment.id)/conclude" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for concluding with a draft finding"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== confirm finding ==="
$confirmed = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/findings/$($finding.id)/confirm" $null).data
if ($confirmed.status -ne "confirmed") { throw "expected confirmed status" }

Write-Host "=== second finding rejected ==="
$finding2 = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/findings" @{
    title = "Candidate 9 anomaly"
}).data
$rejected = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/findings/$($finding2.id)/reject" $null).data
if ($rejected.status -ne "rejected") { throw "expected rejected status" }

Write-Host "=== summary reflects findings ==="
$summary = (InvokeApi GET "$base/v1/reports/nova-summary").data
$row = $summary | Where-Object { $_.id -eq $experiment.id }
if ($row.confirmed_findings -lt 1) { throw "expected confirmed_findings >= 1" }
if ($row.rejected_findings -lt 1) { throw "expected rejected_findings >= 1" }

Write-Host "=== conclude + reopen experiment ==="
$concluded = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/conclude" $null).data
if ($concluded.status -ne "concluded") { throw "expected concluded status" }
$reopened = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/reopen" $null).data
if ($reopened.status -ne "running") { throw "expected running status after reopen" }

Write-Host "=== update experiment + finding ==="
$upd = (InvokeApi PATCH "$base/v1/experiments/$($experiment.id)" @{
    description = "12-candidate panel (phase 2)"
}).data
if ($upd.description -ne "12-candidate panel (phase 2)") { throw "expected updated description" }
$fUpd = (InvokeApi PATCH "$base/v1/experiments/$($experiment.id)/findings/$($finding.id)" @{
    body = "73% at 60C"
}).data
if ($fUpd.body -ne "73% at 60C") { throw "expected updated finding body" }

Write-Host "=== delete + restore finding ==="
InvokeApi POST "$base/v1/experiments/$($experiment.id)/findings/$($finding2.id)/delete" $null | Out-Null
$findings = (InvokeApi GET "$base/v1/experiments/$($experiment.id)/findings").data
$gone = $findings.items | Where-Object { $_.id -eq $finding2.id }
if ($gone) { throw "deleted finding should not be listed" }
$restoredF = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/findings/$($finding2.id)/restore" $null).data
if ($restoredF.status -ne "rejected") { throw "expected restored finding to return to rejected" }

Write-Host "=== delete + restore experiment ==="
InvokeApi POST "$base/v1/experiments/$($experiment.id)/delete" $null | Out-Null
$experiments = (InvokeApi GET "$base/v1/experiments").data
$goneE = $experiments.items | Where-Object { $_.id -eq $experiment.id }
if ($goneE) { throw "deleted experiment should not be listed" }
$restored = (InvokeApi POST "$base/v1/experiments/$($experiment.id)/restore" $null).data
if ($restored.status -ne "running") { throw "expected restored experiment to return to running" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-nova-labs") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_NOVA_LABS_SMOKE PASS"
exit 0
