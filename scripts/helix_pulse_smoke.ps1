# HelixPulse smoke — monitor/incident lifecycle, pause guard, summary
# Prereq: helix_pulse_api on 8121, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8121"

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
if ($st.phase -ne "wave2_w21") { throw "expected phase wave2_w21" }
if (-not $st.planes.monitor_lifecycle) { throw "expected monitor_lifecycle plane" }
if (-not $st.planes.incident_lifecycle) { throw "expected incident_lifecycle plane" }
if (-not $st.planes.pause_guards) { throw "expected pause_guards plane" }
if (-not $st.planes.pulse_summary) { throw "expected pulse_summary plane" }

Write-Host "=== create + activate monitor ==="
$monitor = (InvokeApi POST "$base/v1/monitors" @{
    name = "gateway-8080 $(Get-Random)"
    description = "core gateway health"
}).data
if ($monitor.status -ne "draft") { throw "expected draft monitor" }
$active = (InvokeApi POST "$base/v1/monitors/$($monitor.id)/activate" $null).data
if ($active.status -ne "active") { throw "expected active status" }

Write-Host "=== create incident ==="
$incident = (InvokeApi POST "$base/v1/monitors/$($monitor.id)/incidents" @{
    title = "Latency spike"
    body = "p95 over 800ms"
}).data
if ($incident.status -ne "open") { throw "expected open incident" }

Write-Host "=== pause blocked by open incident ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/monitors/$($monitor.id)/pause" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for pausing with an open incident"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== acknowledge incident ==="
$acknowledged = (InvokeApi POST "$base/v1/monitors/$($monitor.id)/incidents/$($incident.id)/acknowledge" $null).data
if ($acknowledged.status -ne "acknowledged") { throw "expected acknowledged status" }

Write-Host "=== pause + resume monitor ==="
$paused = (InvokeApi POST "$base/v1/monitors/$($monitor.id)/pause" $null).data
if ($paused.status -ne "paused") { throw "expected paused status" }
$resumed = (InvokeApi POST "$base/v1/monitors/$($monitor.id)/resume" $null).data
if ($resumed.status -ne "active") { throw "expected active status after resume" }

Write-Host "=== resolve incident ==="
$resolved = (InvokeApi POST "$base/v1/monitors/$($monitor.id)/incidents/$($incident.id)/resolve" $null).data
if ($resolved.status -ne "resolved") { throw "expected resolved status" }

Write-Host "=== summary reflects incident ==="
$summary = (InvokeApi GET "$base/v1/reports/pulse-summary").data
$row = $summary | Where-Object { $_.id -eq $monitor.id }
if ($row.resolved_incidents -lt 1) { throw "expected resolved_incidents >= 1" }
if ($row.open_incidents -ne 0) { throw "expected 0 open incidents" }

Write-Host "=== update monitor + incident ==="
$upd = (InvokeApi PATCH "$base/v1/monitors/$($monitor.id)" @{
    description = "core gateway + auth health"
}).data
if ($upd.description -ne "core gateway + auth health") { throw "expected updated description" }
$iUpd = (InvokeApi PATCH "$base/v1/monitors/$($monitor.id)/incidents/$($incident.id)" @{
    body = "p95 over 800ms for 4m"
}).data
if ($iUpd.body -ne "p95 over 800ms for 4m") { throw "expected updated incident body" }

Write-Host "=== delete + restore incident ==="
InvokeApi POST "$base/v1/monitors/$($monitor.id)/incidents/$($incident.id)/delete" $null | Out-Null
$incidents = (InvokeApi GET "$base/v1/monitors/$($monitor.id)/incidents").data
$gone = $incidents.items | Where-Object { $_.id -eq $incident.id }
if ($gone) { throw "deleted incident should not be listed" }
$restoredI = (InvokeApi POST "$base/v1/monitors/$($monitor.id)/incidents/$($incident.id)/restore" $null).data
if ($restoredI.status -ne "resolved") { throw "expected restored incident to return to resolved" }

Write-Host "=== delete + restore monitor ==="
InvokeApi POST "$base/v1/monitors/$($monitor.id)/delete" $null | Out-Null
$monitors = (InvokeApi GET "$base/v1/monitors").data
$goneM = $monitors.items | Where-Object { $_.id -eq $monitor.id }
if ($goneM) { throw "deleted monitor should not be listed" }
$restored = (InvokeApi POST "$base/v1/monitors/$($monitor.id)/restore" $null).data
if ($restored.status -ne "active") { throw "expected restored monitor to return to active" }

Write-Host "=== vision endpoint still present ==="
$vis = (InvokeApi GET "$base/v1/pulse/vision").data
if ($vis.slug -ne "helix-pulse") { throw "vision slug mismatch" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-pulse") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_PULSE_SMOKE PASS"
exit 0
