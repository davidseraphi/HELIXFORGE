# HelixLex Prime smoke — matter/filing lifecycle, close guard, summary
# Prereq: helix_lex_prime_api on 8112, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8112"

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
if ($st.phase -ne "wave2_w12") { throw "expected phase wave2_w12" }
if (-not $st.planes.matter_lifecycle) { throw "expected matter_lifecycle plane" }
if (-not $st.planes.filing_lifecycle) { throw "expected filing_lifecycle plane" }
if (-not $st.planes.close_guards) { throw "expected close_guards plane" }
if (-not $st.planes.lex_summary) { throw "expected lex_summary plane" }

Write-Host "=== create + open matter ==="
$matter = (InvokeApi POST "$base/v1/matters" @{
    name = "Acme v. Doe $(Get-Random)"
    description = "contract dispute"
}).data
if ($matter.status -ne "draft") { throw "expected draft matter" }
$opened = (InvokeApi POST "$base/v1/matters/$($matter.id)/open" $null).data
if ($opened.status -ne "open") { throw "expected open status" }

Write-Host "=== create filing ==="
$filing = (InvokeApi POST "$base/v1/matters/$($matter.id)/filings" @{
    title = "Complaint"
    body = "initial pleading"
}).data
if ($filing.status -ne "draft") { throw "expected draft filing" }

Write-Host "=== close blocked by draft filing ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/matters/$($matter.id)/close" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for closing with a draft filing"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== file the filing ==="
$filed = (InvokeApi POST "$base/v1/matters/$($matter.id)/filings/$($filing.id)/file" $null).data
if ($filed.status -ne "filed") { throw "expected filed status" }
if (-not $filed.filed_at) { throw "expected filed_at set" }

Write-Host "=== second filing withdrawn ==="
$filing2 = (InvokeApi POST "$base/v1/matters/$($matter.id)/filings" @{
    title = "Motion to compel"
}).data
$withdrawn = (InvokeApi POST "$base/v1/matters/$($matter.id)/filings/$($filing2.id)/withdraw" $null).data
if ($withdrawn.status -ne "withdrawn") { throw "expected withdrawn status" }

Write-Host "=== summary reflects filings ==="
$summary = (InvokeApi GET "$base/v1/reports/lex-summary").data
$row = $summary | Where-Object { $_.id -eq $matter.id }
if ($row.filed_filings -lt 1) { throw "expected filed_filings >= 1" }
if ($row.withdrawn_filings -lt 1) { throw "expected withdrawn_filings >= 1" }

Write-Host "=== close + reopen matter ==="
$closed = (InvokeApi POST "$base/v1/matters/$($matter.id)/close" $null).data
if ($closed.status -ne "closed") { throw "expected closed status" }
$reopened = (InvokeApi POST "$base/v1/matters/$($matter.id)/reopen" $null).data
if ($reopened.status -ne "open") { throw "expected open status after reopen" }

Write-Host "=== update matter + filing ==="
$upd = (InvokeApi PATCH "$base/v1/matters/$($matter.id)" @{
    description = "contract dispute (amended)"
}).data
if ($upd.description -ne "contract dispute (amended)") { throw "expected updated description" }
$fUpd = (InvokeApi PATCH "$base/v1/matters/$($matter.id)/filings/$($filing.id)" @{
    body = "amended pleading"
}).data
if ($fUpd.body -ne "amended pleading") { throw "expected updated filing body" }

Write-Host "=== delete + restore filing ==="
InvokeApi POST "$base/v1/matters/$($matter.id)/filings/$($filing2.id)/delete" $null | Out-Null
$filings = (InvokeApi GET "$base/v1/matters/$($matter.id)/filings").data
$gone = $filings.items | Where-Object { $_.id -eq $filing2.id }
if ($gone) { throw "deleted filing should not be listed" }
$restoredF = (InvokeApi POST "$base/v1/matters/$($matter.id)/filings/$($filing2.id)/restore" $null).data
if ($restoredF.status -ne "withdrawn") { throw "expected restored filing to return to withdrawn" }

Write-Host "=== delete + restore matter ==="
InvokeApi POST "$base/v1/matters/$($matter.id)/delete" $null | Out-Null
$matters = (InvokeApi GET "$base/v1/matters").data
$goneM = $matters.items | Where-Object { $_.id -eq $matter.id }
if ($goneM) { throw "deleted matter should not be listed" }
$restored = (InvokeApi POST "$base/v1/matters/$($matter.id)/restore" $null).data
if ($restored.status -ne "open") { throw "expected restored matter to return to open" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-lex-prime") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_LEX_PRIME_SMOKE PASS"
exit 0
