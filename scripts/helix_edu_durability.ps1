# HelixEdu durability proof — atomic enroll, forced-kill survival, restore roundtrip
# Prereq: helix_edu_api on 8106 (already built), Postgres migrated,
#         docker compose postgres running, HELIX_ALLOW_DEV_HEADERS=1.
# The script force-kills and restarts the API mid-run.
# Override the binary path with HELIX_EDU_API_BIN if needed.

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8106"
$Bin = if ($env:HELIX_EDU_API_BIN) { $env:HELIX_EDU_API_BIN } elseif ($IsWindows -or $env:OS -match "Windows") { "target/debug/helix_edu_api.exe" } else { "./target/debug/helix_edu_api" }

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
    throw "helix_edu_api did not become healthy"
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

Write-Host "=== course + enroll + progress ==="
$course = (InvokeApi POST "$base/v1/courses" @{
    slug = "dur-course-$(Get-Random)"
    title = "Durability Course"
    description = "gate check"
    level = "beginner"
}).data
$published = (InvokeApi POST "$base/v1/courses/$($course.id)/publish" $null).data
if ($published.status -ne "published") { throw "expected published course" }
$enrollment = (InvokeApi POST "$base/v1/enrollments" @{
    course_id = $published.id
    learner_label = "dur-learner"
}).data
if ($enrollment.status -ne "active") { throw "expected active enrollment" }
$prog = (InvokeApi POST "$base/v1/enrollments/$($enrollment.id)/progress" @{ progress_pct = 50 }).data
if ($prog.progress_pct -ne 50) { throw "expected 50 progress" }

Write-Host "=== forced-kill proof: acknowledged enrollment survives ==="
Write-Host "  wrote $($enrollment.id), killing API..."
Get-Process -Name helix_edu_api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$check = $null
try { $check = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 3 } catch { }
if ($check -and $check.StatusCode -eq 200) { throw "API still running after kill" }
Write-Host "  API down, restarting..."
Start-Process -FilePath $Bin -WorkingDirectory (Get-Location)
Wait-Health 60
Write-Host "  API back, verifying acknowledged enrollment..."
$survivor = (InvokeApi GET "$base/v1/enrollments/$($enrollment.id)").data
if ($survivor.status -ne "active") { throw "expected active status after kill" }
if ($survivor.progress_pct -ne 50) { throw "expected 50 progress after kill, got $($survivor.progress_pct)" }
Write-Host "  enrollment fully present"

Write-Host "=== restore proof: edu schema roundtrip ==="
$dumpPath = "/tmp/edu_durability_dump.sql"
docker compose exec -T postgres pg_dump -U helix -d helixforge --schema=edu --no-owner --no-privileges -f $dumpPath
$dumpSize = (docker compose exec -T postgres sh -c "wc -c < $dumpPath" 2>$null).Trim()
if ([int]$dumpSize -lt 100) { throw "pg_dump produced no usable dump ($dumpSize bytes)" }

Psql "postgres" "DROP DATABASE IF EXISTS edu_restore_test" | Out-Null
Psql "postgres" "CREATE DATABASE edu_restore_test" | Out-Null
docker compose exec -T postgres psql -U helix -d edu_restore_test -f $dumpPath --quiet 2>$null | Out-Null

foreach ($table in @("courses", "enrollments", "enrollment_progress_history")) {
    $src = Psql "helixforge" "SELECT COUNT(*) FROM edu.$table"
    $dst = Psql "edu_restore_test" "SELECT COUNT(*) FROM edu.$table"
    if ($src -ne $dst) { throw "$table count mismatch: source=$src restored=$dst" }
}
$hashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || status || ':' || progress_pct), '|' ORDER BY id), 'empty')) FROM edu.enrollments"
$srcHash = Psql "helixforge" $hashSql
$dstHash = Psql "edu_restore_test" $hashSql
if ($srcHash -ne $dstHash) { throw "enrollments content hash mismatch after restore" }
Write-Host "  counts and content hashes match"

Psql "postgres" "DROP DATABASE IF EXISTS edu_restore_test" | Out-Null
docker compose exec -T postgres rm -f $dumpPath | Out-Null

Write-Host ""
Write-Host "HELIX_EDU_DURABILITY PASS"
exit 0
