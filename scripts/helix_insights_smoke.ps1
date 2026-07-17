# HelixInsights smoke — datasets, metrics, points, aggregates, soft delete
# Prereq: helix_insights_api on 8104, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8104"

function InvokeApi($Method, $Uri, $Body = $null) {
    if ($Body) {
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 15
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -TimeoutSec 15
}

Write-Host "=== healthz ==="
$code = (Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 8).StatusCode
Write-Host "  $code"
if ($code -ne 200) { throw "healthz failed" }

Write-Host "=== domain status ==="
$st = (InvokeApi GET "$base/v1/domain/status" ).data
Write-Host "  phase=$($st.phase) durable=$($st.durable)"
if ($st.phase -ne "wave2_w2") { throw "expected phase wave2_w2" }
if (-not $st.durable) { throw "expected durable=true" }
if (-not $st.planes.aggregate) { throw "expected aggregate plane" }
if (-not $st.planes.soft_delete) { throw "expected soft_delete plane" }

Write-Host "=== create dataset ==="
$dsName = "smoke-dataset-$(Get-Random)"
$ds = (InvokeApi POST "$base/v1/datasets" @{
    name = $dsName
    description = "HelixInsights smoke dataset"
    source_type = "manual"
    schema_json = @{ ts = "datetime"; value = "float" }
}).data
Write-Host "  id=$($ds.id) name=$($ds.name)"
if ($ds.source_type -ne "manual") { throw "expected source_type manual" }

Write-Host "=== get dataset ==="
$got = (InvokeApi GET "$base/v1/datasets/$($ds.id)").data
if ($got.id -ne $ds.id) { throw "get dataset mismatch" }

Write-Host "=== list datasets ==="
$dss = (InvokeApi GET "$base/v1/datasets").data
if ($dss.items.Count -lt 1) { throw "expected datasets" }

Write-Host "=== create metric ==="
$metric = (InvokeApi POST "$base/v1/datasets/$($ds.id)/metrics" @{
    name = "smoke-metric-$(Get-Random)"
    unit = "count"
    aggregation = "sum"
}).data
Write-Host "  id=$($metric.id) dataset_id=$($metric.dataset_id)"
if ($metric.dataset_id -ne $ds.id) { throw "metric dataset mismatch" }

Write-Host "=== get metric ==="
$gotMetric = (InvokeApi GET "$base/v1/metrics/$($metric.id)").data
if ($gotMetric.id -ne $metric.id) { throw "get metric mismatch" }

Write-Host "=== list metrics for tenant ==="
$allMetrics = (InvokeApi GET "$base/v1/metrics").data
if ($allMetrics.items.Count -lt 1) { throw "expected tenant metrics" }

Write-Host "=== record points ==="
$point1 = (InvokeApi POST "$base/v1/metrics/$($metric.id)/points" @{
    value = 10.0
    dimensions = @{ region = "local"; smoke = $true }
}).data
$point2 = (InvokeApi POST "$base/v1/metrics/$($metric.id)/points" @{
    value = 32.0
    dimensions = @{ region = "local"; smoke = $true }
}).data
Write-Host "  ids=$($point1.id),$($point2.id)"

Write-Host "=== list points ==="
$points = (InvokeApi GET "$base/v1/metrics/$($metric.id)/points").data
Write-Host "  count=$($points.items.Count)"
if ($points.items.Count -lt 2) { throw "expected at least 2 points" }

Write-Host "=== aggregate sum ==="
$agg = (InvokeApi POST "$base/v1/metrics/$($metric.id)/aggregate" @{
    aggregation = "sum"
    dimensions = '{"region":"local"}'
}).data
Write-Host "  value=$($agg.value) count=$($agg.count)"
if ([double]$agg.value -ne 42.0) { throw "expected sum 42.0" }
if ([int]$agg.count -ne 2) { throw "expected count 2" }

Write-Host "=== aggregate count ==="
$aggCount = (InvokeApi POST "$base/v1/metrics/$($metric.id)/aggregate" @{
    aggregation = "count"
}).data
if ([int]$aggCount.count -ne 2) { throw "expected count 2" }

Write-Host "=== invalid aggregation rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/metrics/$($metric.id)/aggregate" -Headers $h -Body (@{ aggregation = "hack" } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for invalid aggregation"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) {
        throw "expected 422, got $($_.Exception.Response.StatusCode)"
    }
    Write-Host "  422 as expected"
}

Write-Host "=== soft delete metric ==="
$delMetric = (InvokeApi DELETE "$base/v1/metrics/$($metric.id)").data
if ($delMetric.id -ne $metric.id) { throw "deleted metric mismatch" }

Write-Host "=== soft delete dataset ==="
$delDs = (InvokeApi DELETE "$base/v1/datasets/$($ds.id)").data
if ($delDs.id -ne $ds.id) { throw "deleted dataset mismatch" }

Write-Host "=== verify deleted metric gone ==="
$metricsAfter = (InvokeApi GET "$base/v1/metrics").data
$stillThere = $metricsAfter.items | Where-Object { $_.id -eq $metric.id }
if ($stillThere) { throw "deleted metric still listed" }

Write-Host "=== verify deleted dataset gone ==="
$dssAfter = (InvokeApi GET "$base/v1/datasets").data
$stillThereDs = $dssAfter.items | Where-Object { $_.id -eq $ds.id }
if ($stillThereDs) { throw "deleted dataset still listed" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
Write-Host "  slug=$($info.slug)"
if ($info.slug -ne "helix-insights") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_INSIGHTS_SMOKE PASS"
exit 0
