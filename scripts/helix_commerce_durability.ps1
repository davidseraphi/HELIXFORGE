# HelixCommerce durability proof — atomic orders, forced-kill survival, restore roundtrip
# Prereq: helix_commerce_api on 8105 (already built), Postgres migrated,
#         docker compose postgres running, HELIX_ALLOW_DEV_HEADERS=1.
# The script force-kills and restarts the API mid-run.
# Override the binary path with HELIX_COMMERCE_API_BIN if needed.

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8105"
$Bin = if ($env:HELIX_COMMERCE_API_BIN) { $env:HELIX_COMMERCE_API_BIN } elseif ($IsWindows -or $env:OS -match "Windows") { "target/debug/helix_commerce_api.exe" } else { "./target/debug/helix_commerce_api" }

# The script restarts the API mid-run; the restarted process needs its env.
# Defaults mirror scripts/dev-core.ps1 (local dev only; CI overrides via job env).
$env:HELIX_ENV = if ($env:HELIX_ENV) { $env:HELIX_ENV } else { "local" }
$env:HELIX_LOCAL_DEV_UNSAFE = if ($env:HELIX_LOCAL_DEV_UNSAFE) { $env:HELIX_LOCAL_DEV_UNSAFE } else { "1" }
$env:HELIX_ALLOW_DEV_HEADERS = if ($env:HELIX_ALLOW_DEV_HEADERS) { $env:HELIX_ALLOW_DEV_HEADERS } else { "1" }
$env:HELIX_DEV_PLATFORM = if ($env:HELIX_DEV_PLATFORM) { $env:HELIX_DEV_PLATFORM } else { "1" }
$env:DATABASE_URL = if ($env:DATABASE_URL) { $env:DATABASE_URL } else { "postgres://helix:helix@127.0.0.1:55432/helixforge" }
$env:NATS_URL = if ($env:NATS_URL) { $env:NATS_URL } else { "nats://127.0.0.1:4222" }
$env:MINIO_ENDPOINT = if ($env:MINIO_ENDPOINT) { $env:MINIO_ENDPOINT } else { "http://127.0.0.1:9000" }
$env:HELIX_VAULT_MASTER_KEY = if ($env:HELIX_VAULT_MASTER_KEY) { $env:HELIX_VAULT_MASTER_KEY } else { "local-dev-vault-master-key-not-for-prod" }
$env:HELIX_AUDIT_HMAC_SECRET = if ($env:HELIX_AUDIT_HMAC_SECRET) { $env:HELIX_AUDIT_HMAC_SECRET } else { "local-audit-hmac-dev-only" }
$env:HELIX_WEBHOOK_ALLOW_UNSIGNED = if ($env:HELIX_WEBHOOK_ALLOW_UNSIGNED) { $env:HELIX_WEBHOOK_ALLOW_UNSIGNED } else { "1" }

function InvokeApi($Method, $Uri, $Body = $null) {
    if ($Body) {
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 15
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -TimeoutSec 15
}

function Wait-Health($Seconds = 60) {
    for ($i = 1; $i -le $Seconds; $i++) {
        try {
            $r = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 2
            if ($r.StatusCode -eq 200) { return }
        } catch { }
        Start-Sleep -Seconds 2
    }
    throw "helix_commerce_api did not become healthy"
}

function Psql($Db, $Sql) {
    $out = $null
    try {
        $out = docker compose exec -T postgres psql -U helix -d $Db -t -A -c $Sql 2>$null
    } catch { }
    return "$out".Trim()
}

Write-Host "=== healthz ==="
Wait-Health 15
Write-Host "  healthy"

Write-Host "=== order + cancel consistency ==="
$product = (InvokeApi POST "$base/v1/products" @{
    sku = "dur-sku-$(Get-Random)"
    name = "Durability Widget"
    price_cents = 900
    currency = "USD"
    inventory = 3
}).data
$order = (InvokeApi POST "$base/v1/orders" @{
    customer_email = "dur@example.com"
    items = @(@{ product_id = $product.id; quantity = 2 })
}).data
if ($order.status -ne "pending") { throw "expected pending order" }
$afterOrder = (InvokeApi GET "$base/v1/products/$($product.id)").data
if ($afterOrder.inventory -ne 1) { throw "expected inventory 1 after order, got $($afterOrder.inventory)" }
$cancelled = (InvokeApi POST "$base/v1/orders/$($order.id)/cancel" $null).data
if ($cancelled.status -ne "cancelled") { throw "expected cancelled order" }
$afterCancel = (InvokeApi GET "$base/v1/products/$($product.id)").data
if ($afterCancel.inventory -ne 3) { throw "expected inventory 3 after cancel, got $($afterCancel.inventory)" }
Write-Host "  reservation and restoration consistent"

Write-Host "=== forced-kill proof: acknowledged order survives ==="
$victim = (InvokeApi POST "$base/v1/orders" @{
    customer_email = "victim@example.com"
    items = @(@{ product_id = $product.id; quantity = 1 })
}).data
Write-Host "  wrote $($victim.id), killing API..."
Get-Process -Name helix_commerce_api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$check = $null
try { $check = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 3 } catch { }
if ($check -and $check.StatusCode -eq 200) { throw "API still running after kill" }
Write-Host "  API down, restarting..."
Start-Process -FilePath $Bin -WorkingDirectory (Get-Location)
Wait-Health 60
Write-Host "  API back, verifying acknowledged order..."
$survivor = (InvokeApi GET "$base/v1/orders/$($victim.id)").data
if ($survivor.status -ne "pending") { throw "expected pending status after kill" }
if ($survivor.total_cents -ne 900) { throw "expected total 900 after kill, got $($survivor.total_cents)" }
$afterKill = (InvokeApi GET "$base/v1/products/$($product.id)").data
if ($afterKill.inventory -ne 2) { throw "expected inventory 2 after kill, got $($afterKill.inventory)" }
Write-Host "  order and reservation fully present"

Write-Host "=== restore proof: commerce schema roundtrip ==="
$dumpPath = "/tmp/commerce_durability_dump.sql"
docker compose exec -T postgres pg_dump -U helix -d helixforge --schema=commerce --no-owner --no-privileges -f $dumpPath
$dumpSize = (docker compose exec -T postgres sh -c "wc -c < $dumpPath" 2>$null).Trim()
if ([int]$dumpSize -lt 100) { throw "pg_dump produced no usable dump ($dumpSize bytes)" }

Psql "postgres" "DROP DATABASE IF EXISTS commerce_restore_test" | Out-Null
Psql "postgres" "CREATE DATABASE commerce_restore_test" | Out-Null
docker compose exec -T postgres psql -U helix -d commerce_restore_test -f $dumpPath --quiet 2>$null | Out-Null

foreach ($table in @("products", "orders", "order_items")) {
    $src = Psql "helixforge" "SELECT COUNT(*) FROM commerce.$table"
    $dst = Psql "commerce_restore_test" "SELECT COUNT(*) FROM commerce.$table"
    if ($src -ne $dst) { throw "$table count mismatch: source=$src restored=$dst" }
}
$hashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || sku || ':' || inventory), '|' ORDER BY sku), 'empty')) FROM commerce.products"
$srcHash = Psql "helixforge" $hashSql
$dstHash = Psql "commerce_restore_test" $hashSql
if ($srcHash -ne $dstHash) { throw "products content hash mismatch after restore" }
Write-Host "  counts and content hashes match"

Psql "postgres" "DROP DATABASE IF EXISTS commerce_restore_test" | Out-Null
docker compose exec -T postgres rm -f $dumpPath | Out-Null

Write-Host ""
Write-Host "HELIX_COMMERCE_DURABILITY PASS"
exit 0
