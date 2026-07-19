# HelixCode durability proof — guarded finishes, atomic creates, forced-kill survival, restore roundtrip
# Prereq: helix_code_api on 8102 (already built), Postgres migrated,
#         docker compose postgres running, HELIX_ALLOW_DEV_HEADERS=1, git on PATH.
# The script force-kills and restarts the API mid-run.
# Override the binary path with HELIX_CODE_API_BIN if needed.

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8102"
$Bin = if ($env:HELIX_CODE_API_BIN) { $env:HELIX_CODE_API_BIN } elseif ($IsWindows -or $env:OS -match "Windows") { "target/debug/helix_code_api.exe" } else { "./target/debug/helix_code_api" }

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
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 30
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -TimeoutSec 30
}

function Wait-Health($Seconds = 60) {
    for ($i = 1; $i -le $Seconds; $i++) {
        try {
            $r = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 2
            if ($r.StatusCode -eq 200) { return }
        } catch { }
        Start-Sleep -Seconds 2
    }
    throw "helix_code_api did not become healthy"
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

Write-Host "=== repo + workspace + pipeline + run ==="
$suffix = Get-Random
$repo = (InvokeApi POST "$base/v1/repos" @{
    name = "durability-repo-$suffix"
    description = "gate proof"
    visibility = "private"
}).data
$workspace = (InvokeApi POST "$base/v1/code/workspaces" @{
    repo_id = $repo.id
    name = "gate-ws-$suffix"
    branch = "main"
    root_path = ""
}).data
$pipe = (InvokeApi POST "$base/v1/repos/$($repo.id)/pipelines" @{
    name = "gate-ci-$suffix"
    definition = @{
        version = 1
        steps = @(
            @{ name = "hello"; run = "echo helix-code-durability" },
            @{ name = "rev"; run = "git rev-parse HEAD" }
        )
        artifacts = @("helix-ci.log")
    }
}).data
$run = (InvokeApi POST "$base/v1/pipelines/$($pipe.id)/runs" @{
    trigger_ref = "refs/heads/main"
}).data
if ($run.status -ne "succeeded") { throw "pipeline expected succeeded: $($run.log_text)" }
if (-not $run.finished_at) { throw "finished_at missing on acknowledged run" }
Write-Host "  run $($run.id) succeeded with finished_at"

Write-Host "=== forced-kill proof: acknowledged finished run survives ==="
Write-Host "  finished run $($run.id), killing API..."
Get-Process -Name helix_code_api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$check = $null
try { $check = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 3 } catch { }
if ($check -and $check.StatusCode -eq 200) { throw "API still running after kill" }
Write-Host "  API down, restarting..."
Start-Process -FilePath $Bin -WorkingDirectory (Get-Location)
Wait-Health 60
Write-Host "  API back, verifying finished run, repo, and workspace..."
$survivor = (InvokeApi GET "$base/v1/pipeline-runs/$($run.id)").data
if ($survivor.status -ne "succeeded") { throw "expected succeeded after kill, got $($survivor.status)" }
if (-not $survivor.finished_at) { throw "finished_at missing after kill" }
$srepo = (InvokeApi GET "$base/v1/repos/$($repo.id)").data
if ($srepo.name -ne "durability-repo-$suffix") { throw "repo name mismatch after kill" }
$workspaces = (InvokeApi GET "$base/v1/code/workspaces").data.items
$wrow = $workspaces | Where-Object { $_.id -eq $workspace.id }
if (-not $wrow) { throw "workspace missing after kill" }
if ($wrow.name -ne "gate-ws-$suffix") { throw "workspace name mismatch after kill" }
Write-Host "  run, repo, and workspace fully present"

Write-Host "=== restore proof: code schema roundtrip ==="
$dumpPath = "/tmp/code_durability_dump.sql"
docker compose exec -T postgres pg_dump -U helix -d helixforge --schema=code --no-owner --no-privileges -f $dumpPath
$dumpSize = (docker compose exec -T postgres sh -c "wc -c < $dumpPath" 2>$null).Trim()
if ([int]$dumpSize -lt 100) { throw "pg_dump produced no usable dump ($dumpSize bytes)" }

Psql "postgres" "DROP DATABASE IF EXISTS code_restore_test" | Out-Null
Psql "postgres" "CREATE DATABASE code_restore_test" | Out-Null
docker compose exec -T postgres psql -U helix -d code_restore_test -f $dumpPath --quiet 2>$null | Out-Null

foreach ($table in @("repos", "workspaces", "pipeline_runs")) {
    $src = Psql "helixforge" "SELECT COUNT(*) FROM code.$table"
    $dst = Psql "code_restore_test" "SELECT COUNT(*) FROM code.$table"
    if ($src -ne $dst) { throw "$table count mismatch: source=$src restored=$dst" }
}
$hashSql = "SELECT md5(COALESCE(string_agg(md5(id::text || ':' || status), '|' ORDER BY id), 'empty')) FROM code.pipeline_runs"
$srcHash = Psql "helixforge" $hashSql
$dstHash = Psql "code_restore_test" $hashSql
if ($srcHash -ne $dstHash) { throw "pipeline_runs content hash mismatch after restore" }
Write-Host "  counts and content hashes match"

Psql "postgres" "DROP DATABASE IF EXISTS code_restore_test" | Out-Null
docker compose exec -T postgres rm -f $dumpPath | Out-Null

Write-Host ""
Write-Host "HELIX_CODE_DURABILITY PASS"
exit 0
