# HelixFlow second-wave smoke — workflows, execute, step events, cancel path
# Prereq: helix_flow_api on 8103, Postgres migrated (0032), HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8103"

Write-Host "=== healthz ==="
[void](Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 8)

Write-Host "=== domain status ==="
$st = (Invoke-RestMethod "$base/v1/domain/status" -Headers $h).data
Write-Host "  phase=$($st.phase) execute=$($st.planes.in_process_execute)"
if ($st.phase -ne "wave2_w1") { throw "expected wave2_w1 phase" }

Write-Host "=== create workflow (default steps) ==="
$wf = (Invoke-RestMethod "$base/v1/workflows" -Method POST -Headers $h -Body (@{
  name = "smoke-wf-$(Get-Random)"
} | ConvertTo-Json)).data
Write-Host "  id=$($wf.id) steps=$($wf.steps)"
if ($wf.steps -lt 3) { throw "expected default multi-step definition" }

Write-Host "=== get workflow ==="
$got = (Invoke-RestMethod "$base/v1/workflows/$($wf.id)" -Headers $h).data
if ($got.id -ne $wf.id) { throw "get mismatch" }

Write-Host "=== run (in-process execute) ==="
$run = (Invoke-RestMethod "$base/v1/workflows/$($wf.id)/runs" -Method POST -Headers $h -Body "{}").data
Write-Host "  run=$($run.run.id) status=$($run.run.status) events=$($run.events.Count)"
if ($run.run.status -ne "succeeded") { throw "expected succeeded, got $($run.run.status)" }
if ($run.events.Count -lt 3) { throw "expected step events" }
if ($run.run.result.ok -ne $true) { throw "expected result.ok from set step" }

Write-Host "=== list runs + get run ==="
$runs = (Invoke-RestMethod "$base/v1/runs?workflow_id=$($wf.id)" -Headers $h).data
if ($runs.items.Count -lt 1) { throw "expected runs" }
$detail = (Invoke-RestMethod "$base/v1/runs/$($run.run.id)" -Headers $h).data
if ($detail.events.Count -lt 3) { throw "expected events on get" }

Write-Host "=== fail step path ==="
$wfFail = (Invoke-RestMethod "$base/v1/workflows" -Method POST -Headers $h -Body (@{
  name = "fail-wf"
  definition = @{
    version = 1
    steps = @(
      @{ name = "a"; type = "echo"; message = "before" }
      @{ name = "b"; type = "fail"; message = "boom" }
    )
  }
} | ConvertTo-Json -Depth 6)).data
$bad = (Invoke-RestMethod "$base/v1/workflows/run" -Method POST -Headers $h -Body (@{
  workflow_id = $wfFail.id
} | ConvertTo-Json)).data
Write-Host "  fail_status=$($bad.run.status)"
if ($bad.run.status -ne "failed") { throw "expected failed run" }

Write-Host "=== list workflows ==="
$list = (Invoke-RestMethod "$base/v1/workflows" -Headers $h).data
if ($list.items.Count -lt 2) { throw "expected workflows" }

Write-Host ""
Write-Host "HELIX_FLOW_SMOKE PASS"
exit 0
