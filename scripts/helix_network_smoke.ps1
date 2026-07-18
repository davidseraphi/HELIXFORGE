# HelixNetwork smoke — profile/connection/opportunity lifecycle, blocking, summary
# Prereq: helix_network_api on 8109, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$hA = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$hB = @{ "x-helix-dev-user" = "bob.net@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8109"

function InvokeApi($Headers, $Method, $Uri, $Body = $null) {
    if ($Body) {
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $Headers -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 15
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $Headers -TimeoutSec 15
}

Write-Host "=== healthz ==="
$code = (Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 8).StatusCode
if ($code -ne 200) { throw "healthz failed" }

Write-Host "=== domain status ==="
$st = (InvokeApi $hA GET "$base/v1/domain/status").data
if ($st.phase -ne "wave2_w9") { throw "expected phase wave2_w9" }
if (-not $st.planes.profile_lifecycle) { throw "expected profile_lifecycle plane" }
if (-not $st.planes.connection_lifecycle) { throw "expected connection_lifecycle plane" }
if (-not $st.planes.blocking) { throw "expected blocking plane" }
if (-not $st.planes.network_summary) { throw "expected network_summary plane" }

Write-Host "=== create profiles (two users) ==="
$suffix = Get-Random
$alice = (InvokeApi $hA POST "$base/v1/profiles" @{
    display_name = "Alice $suffix"
    headline = "builder"
}).data
$bob = (InvokeApi $hB POST "$base/v1/profiles" @{
    display_name = "Bob $suffix"
    headline = "designer"
}).data
if (-not $alice.id -or -not $bob.id) { throw "profile creation failed" }

Write-Host "=== update profile ==="
$upd = (InvokeApi $hA PATCH "$base/v1/profiles/$($alice.id)" @{
    headline = "senior builder"
}).data
if ($upd.headline -ne "senior builder") { throw "expected updated headline" }

Write-Host "=== non-owner update rejected ==="
try {
    Invoke-RestMethod -Method PATCH -Uri "$base/v1/profiles/$($alice.id)" -Headers $hB -Body (@{ headline = "hijack" } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 404 for non-owner profile update"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 404) { throw "expected 404, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  404 as expected"
}

Write-Host "=== request connection A->B ==="
$conn = (InvokeApi $hA POST "$base/v1/connections" @{
    to_profile_id = $bob.id
    message = "hello"
}).data
if ($conn.status -ne "pending") { throw "expected pending connection" }

Write-Host "=== decline by requester rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/connections/$($conn.id)/decline" -Headers $hA -TimeoutSec 8 | Out-Null
    throw "expected 404 for decline by non-receiver"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 404) { throw "expected 404, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  404 as expected"
}

Write-Host "=== decline by receiver ==="
$declined = (InvokeApi $hB POST "$base/v1/connections/$($conn.id)/decline" $null).data
if ($declined.status -ne "declined") { throw "expected declined status" }
if (-not $declined.responded_at) { throw "expected responded_at set" }

Write-Host "=== re-request revives declined connection ==="
$revived = (InvokeApi $hA POST "$base/v1/connections" @{
    to_profile_id = $bob.id
    message = "second try"
}).data
if ($revived.status -ne "pending") { throw "expected revived pending status" }
if ($revived.id -ne $conn.id) { throw "expected revival of the same connection row" }

Write-Host "=== accept ==="
$accepted = (InvokeApi $hB POST "$base/v1/connections/$($conn.id)/accept" $null).data
if ($accepted.status -ne "accepted") { throw "expected accepted status" }

Write-Host "=== duplicate request while connected rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/connections" -Headers $hA -Body (@{ to_profile_id = $bob.id } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 409 for duplicate request"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 409) { throw "expected 409, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  409 as expected"
}

Write-Host "=== network summary shows accepted connection ==="
$summary = (InvokeApi $hA GET "$base/v1/reports/network-summary").data
$aliceRow = $summary | Where-Object { $_.id -eq $alice.id }
$bobRow = $summary | Where-Object { $_.id -eq $bob.id }
if ($aliceRow.accepted_count -lt 1) { throw "expected alice accepted_count >= 1" }
if ($bobRow.accepted_count -lt 1) { throw "expected bob accepted_count >= 1" }

Write-Host "=== remove connection ==="
$removed = (InvokeApi $hA POST "$base/v1/connections/$($conn.id)/remove" $null).data
if ($removed.status -ne "removed") { throw "expected removed status" }

Write-Host "=== block connection ==="
InvokeApi $hA POST "$base/v1/connections" @{ to_profile_id = $bob.id; message = "third" } | Out-Null
$blocked = (InvokeApi $hB POST "$base/v1/connections/$($conn.id)/block" $null).data
if ($blocked.status -ne "blocked") { throw "expected blocked status" }

Write-Host "=== blocked pair cannot request (both directions) ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/connections" -Headers $hA -Body (@{ to_profile_id = $bob.id } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for blocked pair A->B"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 A->B as expected"
}
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/connections" -Headers $hB -Body (@{ to_profile_id = $alice.id } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for blocked pair B->A"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 B->A as expected"
}

Write-Host "=== deactivate + reactivate profile ==="
$deact = (InvokeApi $hB POST "$base/v1/profiles/$($bob.id)/deactivate" $null).data
if ($deact.status -ne "deactivated") { throw "expected deactivated status" }
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/connections" -Headers $hA -Body (@{ to_profile_id = $bob.id } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for connect to deactivated profile"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}
$react = (InvokeApi $hB POST "$base/v1/profiles/$($bob.id)/reactivate" $null).data
if ($react.status -ne "active") { throw "expected active status after reactivate" }

Write-Host "=== opportunity lifecycle ==="
$opp = (InvokeApi $hA POST "$base/v1/opportunities" @{
    title = "Platform role $suffix"
    description = "build things"
    kind = "role"
}).data
if ($opp.status -ne "open") { throw "expected open opportunity" }
$oppUpd = (InvokeApi $hA PATCH "$base/v1/opportunities/$($opp.id)" @{
    title = "Senior platform role $suffix"
}).data
if ($oppUpd.title -ne "Senior platform role $suffix") { throw "expected updated title" }
$closed = (InvokeApi $hA POST "$base/v1/opportunities/$($opp.id)/close" $null).data
if ($closed.status -ne "closed") { throw "expected closed status" }
$reopened = (InvokeApi $hA POST "$base/v1/opportunities/$($opp.id)/reopen" $null).data
if ($reopened.status -ne "open") { throw "expected reopened status" }

Write-Host "=== summary counts open opportunity ==="
$summary2 = (InvokeApi $hA GET "$base/v1/reports/network-summary").data
$aliceRow2 = $summary2 | Where-Object { $_.id -eq $alice.id }
if ($aliceRow2.open_opportunities -lt 1) { throw "expected alice open_opportunities >= 1" }

Write-Host "=== delete + restore opportunity ==="
InvokeApi $hA POST "$base/v1/opportunities/$($opp.id)/delete" $null | Out-Null
$opps = (InvokeApi $hA GET "$base/v1/opportunities").data
$gone = $opps.items | Where-Object { $_.id -eq $opp.id }
if ($gone) { throw "deleted opportunity should not be listed" }
$restoredOpp = (InvokeApi $hA POST "$base/v1/opportunities/$($opp.id)/restore" $null).data
if ($restoredOpp.status -ne "open") { throw "expected restored opportunity to be open" }

Write-Host "=== delete + restore profile ==="
InvokeApi $hB POST "$base/v1/profiles/$($bob.id)/delete" $null | Out-Null
$profiles = (InvokeApi $hA GET "$base/v1/profiles").data
$goneP = $profiles.items | Where-Object { $_.id -eq $bob.id }
if ($goneP) { throw "deleted profile should not be listed" }
$restoredP = (InvokeApi $hB POST "$base/v1/profiles/$($bob.id)/restore" $null).data
if ($restoredP.status -ne "active") { throw "expected restored profile to be active" }

Write-Host "=== product info ==="
$info = (InvokeApi $hA GET "$base/v1/product").data
if ($info.slug -ne "helix-network") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_NETWORK_SMOKE PASS"
exit 0
