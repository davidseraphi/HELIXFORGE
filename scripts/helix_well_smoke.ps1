# HelixWell smoke — habits lifecycle, optional check-in fields, edits, summary
# Prereq: helix_well_api on 8108, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8108"

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
$st = (InvokeApi GET "$base/v1/domain/status" ).data
if ($st.phase -ne "wave2_w8") { throw "expected phase wave2_w8" }
if (-not $st.planes.habit_lifecycle) { throw "expected habit_lifecycle plane" }
if (-not $st.planes.optional_checkin_fields) { throw "expected optional_checkin_fields plane" }
if (-not $st.planes.checkin_edit_history) { throw "expected checkin_edit_history plane" }
if (-not $st.planes.habit_summary) { throw "expected habit_summary plane" }

Write-Host "=== create habit ==="
$habit = (InvokeApi POST "$base/v1/habits" @{
    name = "Walk $(Get-Random)"
    description = "Daily walk"
    cadence = "daily"
    target_per_period = 1
}).data
if ($habit.status -ne "active") { throw "expected active habit" }

Write-Host "=== update habit ==="
$upd = (InvokeApi PATCH "$base/v1/habits/$($habit.id)" @{
    description = "Evening walk"
}).data
if ($upd.description -ne "Evening walk") { throw "expected updated description" }

Write-Host "=== log active habit ==="
InvokeApi POST "$base/v1/habits/$($habit.id)/logs" @{ quantity = 1; notes = "first" } | Out-Null

Write-Host "=== pause habit ==="
$paused = (InvokeApi POST "$base/v1/habits/$($habit.id)/pause" $null).data
if ($paused.status -ne "paused") { throw "expected paused status" }

Write-Host "=== log paused habit rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/habits/$($habit.id)/logs" -Headers $h -Body (@{ quantity = 1 } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for logging a paused habit"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== resume habit ==="
$resumed = (InvokeApi POST "$base/v1/habits/$($habit.id)/resume" $null).data
if ($resumed.status -ne "active") { throw "expected active status after resume" }

Write-Host "=== log resumed habit ==="
InvokeApi POST "$base/v1/habits/$($habit.id)/logs" @{ quantity = 2; notes = "second" } | Out-Null

Write-Host "=== habit summary ==="
$summary = (InvokeApi GET "$base/v1/reports/habit-summary").data
$row = $summary | Where-Object { $_.id -eq $habit.id }
if ($row.total_logs -ne 2) { throw "expected 2 total logs, got $($row.total_logs)" }
if ($row.total_quantity -ne 3) { throw "expected quantity 3, got $($row.total_quantity)" }
if ($row.logs_last_7_days -ne 2) { throw "expected 2 logs in last 7 days" }

Write-Host "=== end habit ==="
$ended = (InvokeApi POST "$base/v1/habits/$($habit.id)/end" $null).data
if ($ended.status -ne "ended") { throw "expected ended status" }

Write-Host "=== log ended habit rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/habits/$($habit.id)/logs" -Headers $h -Body (@{ quantity = 1 } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for logging an ended habit"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== create check-in with skipped energy ==="
$checkin = (InvokeApi POST "$base/v1/checkins" @{
    mood = 7
    notes = "ok day"
}).data
if ($checkin.mood -ne 7) { throw "expected mood 7" }
if ($null -ne $checkin.energy) { throw "expected skipped energy to stay missing" }

Write-Host "=== out-of-range check-in rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/checkins" -Headers $h -Body (@{ mood = 11 } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for out-of-range mood"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== edit check-in records history ==="
$edited = (InvokeApi PATCH "$base/v1/checkins/$($checkin.id)" @{
    energy = 5
    notes = "ok day, tired"
}).data
if ($edited.energy -ne 5) { throw "expected energy 5 after edit" }
if ($edited.edit_version -ne 1) { throw "expected edit_version 1" }
$edits = (InvokeApi GET "$base/v1/checkins/$($checkin.id)/edits").data
if ($edits.items.Count -ne 1) { throw "expected 1 edit snapshot" }
if ($null -ne $edits.items[0].energy) { throw "expected snapshot to hold pre-edit missing energy" }

Write-Host "=== clear mood back to missing ==="
$cleared = (InvokeApi PATCH "$base/v1/checkins/$($checkin.id)" @{
    mood = $null
}).data
if ($null -ne $cleared.mood) { throw "expected mood cleared to missing" }
if ($cleared.edit_version -ne 2) { throw "expected edit_version 2" }

Write-Host "=== delete check-in hides it ==="
InvokeApi POST "$base/v1/checkins/$($checkin.id)/delete" $null | Out-Null
$checkins = (InvokeApi GET "$base/v1/checkins?mine=true").data
$found = $checkins.items | Where-Object { $_.id -eq $checkin.id }
if ($found) { throw "deleted check-in should not be listed" }

Write-Host "=== delete + restore habit ==="
InvokeApi POST "$base/v1/habits/$($habit.id)/delete" $null | Out-Null
$habits = (InvokeApi GET "$base/v1/habits").data
$deletedHabit = $habits.items | Where-Object { $_.id -eq $habit.id }
if ($deletedHabit) { throw "deleted habit should not be listed" }
$restored = (InvokeApi POST "$base/v1/habits/$($habit.id)/restore" $null).data
if ($restored.status -ne "ended") { throw "expected restore to return pre-delete status 'ended'" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-well") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_WELL_SMOKE PASS"
exit 0
