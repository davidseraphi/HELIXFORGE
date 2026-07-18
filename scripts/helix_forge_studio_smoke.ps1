# HelixForge Studio smoke — app/page lifecycle, publish guard, summary
# Prereq: helix_forge_studio_api on 8110, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8110"

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
if ($st.phase -ne "wave2_w10") { throw "expected phase wave2_w10" }
if (-not $st.planes.app_lifecycle) { throw "expected app_lifecycle plane" }
if (-not $st.planes.page_lifecycle) { throw "expected page_lifecycle plane" }
if (-not $st.planes.publish_guards) { throw "expected publish_guards plane" }
if (-not $st.planes.studio_summary) { throw "expected studio_summary plane" }

Write-Host "=== create app ==="
$app = (InvokeApi POST "$base/v1/apps" @{
    name = "Shopfront $(Get-Random)"
    description = "store app"
}).data
if ($app.status -ne "draft") { throw "expected draft app" }

Write-Host "=== publish without pages rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/apps/$($app.id)/publish" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for publishing an app with no pages"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== create page ==="
$page = (InvokeApi POST "$base/v1/apps/$($app.id)/pages" @{
    title = "Home"
    body = "hero + grid"
}).data
if ($page.status -ne "open") { throw "expected open page" }

Write-Host "=== publish app ==="
$published = (InvokeApi POST "$base/v1/apps/$($app.id)/publish" $null).data
if ($published.status -ne "published") { throw "expected published status" }
if (-not $published.published_at) { throw "expected published_at set" }

Write-Host "=== unpublish app ==="
$unpublished = (InvokeApi POST "$base/v1/apps/$($app.id)/unpublish" $null).data
if ($unpublished.status -ne "draft") { throw "expected draft status after unpublish" }

Write-Host "=== update app ==="
$upd = (InvokeApi PATCH "$base/v1/apps/$($app.id)" @{
    description = "store app v2"
}).data
if ($upd.description -ne "store app v2") { throw "expected updated description" }

Write-Host "=== update page ==="
$pageUpd = (InvokeApi PATCH "$base/v1/apps/$($app.id)/pages/$($page.id)" @{
    body = "hero + grid + footer"
}).data
if ($pageUpd.body -ne "hero + grid + footer") { throw "expected updated body" }

Write-Host "=== archive + reopen page ==="
$archived = (InvokeApi POST "$base/v1/apps/$($app.id)/pages/$($page.id)/archive" $null).data
if ($archived.status -ne "archived") { throw "expected archived status" }

Write-Host "=== summary reflects archived page ==="
$summary = (InvokeApi GET "$base/v1/reports/studio-summary").data
$row = $summary | Where-Object { $_.id -eq $app.id }
if ($row.archived_pages -lt 1) { throw "expected archived_pages >= 1" }

$reopened = (InvokeApi POST "$base/v1/apps/$($app.id)/pages/$($page.id)/reopen" $null).data
if ($reopened.status -ne "open") { throw "expected open status after reopen" }

Write-Host "=== delete + restore page ==="
InvokeApi POST "$base/v1/apps/$($app.id)/pages/$($page.id)/delete" $null | Out-Null
$pages = (InvokeApi GET "$base/v1/apps/$($app.id)/pages").data
$gone = $pages.items | Where-Object { $_.id -eq $page.id }
if ($gone) { throw "deleted page should not be listed" }
$restoredPage = (InvokeApi POST "$base/v1/apps/$($app.id)/pages/$($page.id)/restore" $null).data
if ($restoredPage.status -ne "open") { throw "expected restored page to be open" }

Write-Host "=== delete + restore app ==="
InvokeApi POST "$base/v1/apps/$($app.id)/publish" $null | Out-Null
InvokeApi POST "$base/v1/apps/$($app.id)/delete" $null | Out-Null
$apps = (InvokeApi GET "$base/v1/apps").data
$goneApp = $apps.items | Where-Object { $_.id -eq $app.id }
if ($goneApp) { throw "deleted app should not be listed" }
$restoredApp = (InvokeApi POST "$base/v1/apps/$($app.id)/restore" $null).data
if ($restoredApp.status -ne "published") { throw "expected restored app to return to published" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-forge-studio") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_FORGE_STUDIO_SMOKE PASS"
exit 0
