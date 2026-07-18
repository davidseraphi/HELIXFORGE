# HelixQuantum Forge smoke — job/circuit lifecycle, submit guard, summary
# Prereq: helix_quantum_forge_api on 8117, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8117"

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
if ($st.phase -ne "wave2_w17") { throw "expected phase wave2_w17" }
if (-not $st.planes.job_lifecycle) { throw "expected job_lifecycle plane" }
if (-not $st.planes.circuit_lifecycle) { throw "expected circuit_lifecycle plane" }
if (-not $st.planes.submit_guards) { throw "expected submit_guards plane" }
if (-not $st.planes.quantum_summary) { throw "expected quantum_summary plane" }

Write-Host "=== create job ==="
$job = (InvokeApi POST "$base/v1/jobs" @{
    name = "Bell state sweep $(Get-Random)"
    description = "2-qubit bell pairs"
}).data
if ($job.status -ne "draft") { throw "expected draft job" }

Write-Host "=== submit without circuits rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/jobs/$($job.id)/submit" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for submitting without circuits"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== create circuit ==="
$circuit = (InvokeApi POST "$base/v1/jobs/$($job.id)/circuits" @{
    title = "bell-01"
    body = "H(0) CX(0,1)"
}).data
if ($circuit.status -ne "draft") { throw "expected draft circuit" }

Write-Host "=== submit + complete job ==="
$submitted = (InvokeApi POST "$base/v1/jobs/$($job.id)/submit" $null).data
if ($submitted.status -ne "submitted") { throw "expected submitted status" }
if (-not $submitted.submitted_at) { throw "expected submitted_at set" }
$completed = (InvokeApi POST "$base/v1/jobs/$($job.id)/complete" $null).data
if ($completed.status -ne "completed") { throw "expected completed status" }

Write-Host "=== validate + archive circuit ==="
$validated = (InvokeApi POST "$base/v1/jobs/$($job.id)/circuits/$($circuit.id)/validate" $null).data
if ($validated.status -ne "validated") { throw "expected validated status" }
$circuit2 = (InvokeApi POST "$base/v1/jobs/$($job.id)/circuits" @{
    title = "bell-02"
}).data
$archived = (InvokeApi POST "$base/v1/jobs/$($job.id)/circuits/$($circuit2.id)/archive" $null).data
if ($archived.status -ne "archived") { throw "expected archived status" }

Write-Host "=== second job fails ==="
$job2 = (InvokeApi POST "$base/v1/jobs" @{
    name = "GHZ sweep $(Get-Random)"
}).data
InvokeApi POST "$base/v1/jobs/$($job2.id)/circuits" @{ title = "ghz-01" } | Out-Null
InvokeApi POST "$base/v1/jobs/$($job2.id)/submit" $null | Out-Null
$failed = (InvokeApi POST "$base/v1/jobs/$($job2.id)/fail" $null).data
if ($failed.status -ne "failed") { throw "expected failed status" }

Write-Host "=== summary reflects circuits ==="
$summary = (InvokeApi GET "$base/v1/reports/quantum-summary").data
$row = $summary | Where-Object { $_.id -eq $job.id }
if ($row.validated_circuits -lt 1) { throw "expected validated_circuits >= 1" }
if ($row.archived_circuits -lt 1) { throw "expected archived_circuits >= 1" }

Write-Host "=== update job + circuit ==="
$upd = (InvokeApi PATCH "$base/v1/jobs/$($job.id)" @{
    description = "2-qubit bell pairs v2"
}).data
if ($upd.description -ne "2-qubit bell pairs v2") { throw "expected updated description" }
$cUpd = (InvokeApi PATCH "$base/v1/jobs/$($job.id)/circuits/$($circuit.id)" @{
    body = "H(0) CX(0,1) M(0,1)"
}).data
if ($cUpd.body -ne "H(0) CX(0,1) M(0,1)") { throw "expected updated circuit body" }

Write-Host "=== delete + restore circuit ==="
InvokeApi POST "$base/v1/jobs/$($job.id)/circuits/$($circuit2.id)/delete" $null | Out-Null
$circuits = (InvokeApi GET "$base/v1/jobs/$($job.id)/circuits").data
$gone = $circuits.items | Where-Object { $_.id -eq $circuit2.id }
if ($gone) { throw "deleted circuit should not be listed" }
$restoredC = (InvokeApi POST "$base/v1/jobs/$($job.id)/circuits/$($circuit2.id)/restore" $null).data
if ($restoredC.status -ne "archived") { throw "expected restored circuit to return to archived" }

Write-Host "=== delete + restore job ==="
InvokeApi POST "$base/v1/jobs/$($job2.id)/delete" $null | Out-Null
$jobs = (InvokeApi GET "$base/v1/jobs").data
$goneJ = $jobs.items | Where-Object { $_.id -eq $job2.id }
if ($goneJ) { throw "deleted job should not be listed" }
$restored = (InvokeApi POST "$base/v1/jobs/$($job2.id)/restore" $null).data
if ($restored.status -ne "failed") { throw "expected restored job to return to failed" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-quantum-forge") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_QUANTUM_FORGE_SMOKE PASS"
exit 0
