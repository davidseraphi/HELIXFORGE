# HelixCollab durability proof — atomic create, forced-kill survival, restore roundtrip
# Prereq: helix_collab_api on 8101 (already built), Postgres migrated,
#         docker compose postgres running, HELIX_ALLOW_DEV_HEADERS=1.
# The script force-kills and restarts the API mid-run.
# Override the binary path with HELIX_COLLAB_API_BIN if needed.

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8101"
$Bin = if ($env:HELIX_COLLAB_API_BIN) { $env:HELIX_COLLAB_API_BIN } elseif ($IsWindows -or $env:OS -match "Windows") { "target/debug/helix_collab_api.exe" } else { "./target/debug/helix_collab_api" }

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
    throw "helix_collab_api did not become healthy"
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

Write-Host "=== create + patch twice (revision chain) ==="
$doc = (InvokeApi POST "$base/v1/documents" @{
    title = "Durability Doc $(Get-Random)"
    content = "v1 content"
}).data
$id = $doc.id
if ($doc.version -ne 1) { throw "expected v1" }
$p1 = (InvokeApi PATCH "$base/v1/documents/$id" @{ base_version = 1; content = "v2 content" }).data
if ($p1.version -ne 2) { throw "expected v2" }
$p2 = (InvokeApi PATCH "$base/v1/documents/$id" @{ base_version = 2; content = "v3 content" }).data
if ($p2.version -ne 3) { throw "expected v3" }
$revs = (InvokeApi GET "$base/v1/documents/$id/revisions").data
if ($revs.Count -lt 3) { throw "expected at least 3 revisions, got $($revs.Count)" }
Write-Host "  revision chain v1..v3 OK ($($revs.Count) revisions)"

Write-Host "=== forced-kill proof: acknowledged write survives ==="
$victim = (InvokeApi POST "$base/v1/documents" @{
    title = "Kill Victim $(Get-Random)"
    content = "acknowledged before kill"
}).data
Write-Host "  wrote $($victim.id), killing API..."
Get-Process -Name helix_collab_api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$check = $null
try { $check = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 3 } catch { }
if ($check -and $check.StatusCode -eq 200) { throw "API still running after kill" }
Write-Host "  API down, restarting..."
Start-Process -FilePath $Bin -WorkingDirectory (Get-Location)
Wait-Health 60
Write-Host "  API back, verifying acknowledged document..."
$survivor = (InvokeApi GET "$base/v1/documents/$($victim.id)").data
if ($survivor.title -ne $victim.title) { throw "title mismatch after kill" }
if ($survivor.content -ne "acknowledged before kill") { throw "content mismatch after kill" }
if ($survivor.version -ne 1) { throw "version mismatch after kill" }
$survivorRevs = (InvokeApi GET "$base/v1/documents/$($victim.id)/revisions").data
if ($survivorRevs.Count -ne 1) { throw "expected exactly 1 revision after kill, got $($survivorRevs.Count)" }
if ($survivorRevs[0].content -ne "acknowledged before kill") { throw "revision content mismatch after kill" }
Write-Host "  document + revision fully present"

Write-Host "=== restore proof: collab schema roundtrip ==="
# Keep the dump container-side so no encoding conversion crosses the host.
$dumpPath = "/tmp/collab_durability_dump.sql"
docker compose exec -T postgres pg_dump -U helix -d helixforge --schema=collab --no-owner --no-privileges -f $dumpPath
$dumpSize = (docker compose exec -T postgres sh -c "wc -c < $dumpPath" 2>$null).Trim()
if ([int]$dumpSize -lt 100) { throw "pg_dump produced no usable dump ($dumpSize bytes)" }

Psql "postgres" "DROP DATABASE IF EXISTS collab_restore_test" | Out-Null
Psql "postgres" "CREATE DATABASE collab_restore_test" | Out-Null
docker compose exec -T postgres psql -U helix -d collab_restore_test -f $dumpPath --quiet 2>$null | Out-Null

$srcDocs = Psql "helixforge" "SELECT COUNT(*) FROM collab.documents"
$dstDocs = Psql "collab_restore_test" "SELECT COUNT(*) FROM collab.documents"
if ($srcDocs -ne $dstDocs) { throw "documents count mismatch: source=$srcDocs restored=$dstDocs" }
$srcRevs = Psql "helixforge" "SELECT COUNT(*) FROM collab.document_revisions"
$dstRevs = Psql "collab_restore_test" "SELECT COUNT(*) FROM collab.document_revisions"
if ($srcRevs -ne $dstRevs) { throw "revisions count mismatch: source=$srcRevs restored=$dstRevs" }

$hashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || version || ':' || content), '|' ORDER BY id), 'empty')) FROM collab.documents"
$srcHash = Psql "helixforge" $hashSql
$dstHash = Psql "collab_restore_test" $hashSql
if ($srcHash -ne $dstHash) { throw "documents content hash mismatch after restore" }
$revHashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || version || ':' || content), '|' ORDER BY id), 'empty')) FROM collab.document_revisions"
$srcRevHash = Psql "helixforge" $revHashSql
$dstRevHash = Psql "collab_restore_test" $revHashSql
if ($srcRevHash -ne $dstRevHash) { throw "revisions content hash mismatch after restore" }
Write-Host "  counts and content hashes match ($srcDocs docs, $srcRevs revisions)"

Psql "postgres" "DROP DATABASE IF EXISTS collab_restore_test" | Out-Null
docker compose exec -T postgres rm -f $dumpPath | Out-Null

Write-Host ""
Write-Host "HELIX_COLLAB_DURABILITY PASS"
exit 0
