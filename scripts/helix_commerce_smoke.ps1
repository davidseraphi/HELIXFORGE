# HelixCommerce smoke — products, inventory, orders, cancel, mixed-currency guard
# Prereq: helix_commerce_api on 8105, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8105"

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
if ($st.phase -ne "wave2_w3") { throw "expected phase wave2_w3" }
if (-not $st.planes.inventory_reservation) { throw "expected inventory_reservation plane" }
if (-not $st.planes.mixed_currency_guard) { throw "expected mixed_currency_guard plane" }
if (-not $st.planes.cancel) { throw "expected cancel plane" }

Write-Host "=== create USD product ==="
$skuUsd = "smoke-usd-$(Get-Random)"
$prodUsd = (InvokeApi POST "$base/v1/products" @{
    sku = $skuUsd
    name = "Smoke USD Product"
    price_cents = 1000
    currency = "USD"
    inventory = 10
}).data
if ($prodUsd.status -ne "active") { throw "expected active product" }

Write-Host "=== create EUR product ==="
$skuEur = "smoke-eur-$(Get-Random)"
$prodEur = (InvokeApi POST "$base/v1/products" @{
    sku = $skuEur
    name = "Smoke EUR Product"
    price_cents = 900
    currency = "EUR"
    inventory = 5
}).data

Write-Host "=== update USD inventory ==="
$upd = (InvokeApi PATCH "$base/v1/products/$($prodUsd.id)" @{
    inventory_delta = 5
}).data
if ($upd.inventory -ne 15) { throw "expected inventory 15, got $($upd.inventory)" }

Write-Host "=== create order ==="
$order = (InvokeApi POST "$base/v1/orders" @{
    customer_email = "buyer@example.com"
    items = @(@{ product_id = $prodUsd.id; quantity = 3 })
}).data
if ($order.total_cents -ne 3000) { throw "expected total 3000 cents" }
if ($order.status -ne "pending") { throw "expected pending order" }

Write-Host "=== get product shows decremented inventory ==="
$prodAfter = (InvokeApi GET "$base/v1/products/$($prodUsd.id)").data
if ($prodAfter.inventory -ne 12) { throw "expected inventory 12 after order" }

Write-Host "=== mixed-currency order rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/orders" -Headers $h -Body (@{
        customer_email = "mixed@example.com"
        items = @(
            @{ product_id = $prodUsd.id; quantity = 1 }
            @{ product_id = $prodEur.id; quantity = 1 }
        )
    } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for mixed currency"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== cancel order restores inventory ==="
$cancelled = (InvokeApi POST "$base/v1/orders/$($order.id)/cancel" $null).data
if ($cancelled.status -ne "cancelled") { throw "expected cancelled status" }

$prodAfterCancel = (InvokeApi GET "$base/v1/products/$($prodUsd.id)").data
if ($prodAfterCancel.inventory -ne 15) { throw "expected inventory restored to 15" }

Write-Host "=== cancel already-cancelled order rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/orders/$($order.id)/cancel" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 422 for double cancel"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== list orders ==="
$orders = (InvokeApi GET "$base/v1/orders").data
if ($orders.items.Count -lt 1) { throw "expected orders" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product").data
if ($info.slug -ne "helix-commerce") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_COMMERCE_SMOKE PASS"
exit 0
