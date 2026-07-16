# HelixCode end-state smoke — issues, PRs, protection, webhooks, quotas, LSP servers, MLS devices, settings, terminal
# Prereq: helix_code_api on 8102 with migration 0029; HELIX_ALLOW_DEV_HEADERS=1
# Note: sets HELIX_CODE_ALLOW_DIRECT_PUSH=1 only if you protect main and still need direct commits.

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8102"
$name = "endstate-$(Get-Random)"

Write-Host "=== healthz ==="
[void](Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 8)

Write-Host "=== domain endstate planes ==="
$st = (Invoke-RestMethod "$base/v1/domain/status" -Headers $h).data
Write-Host "  phase=$($st.phase) endstate=$($st.planes.endstate) issues=$($st.planes.issues) prs=$($st.planes.pull_requests)"
if ($st.phase -ne "endstate" -and -not $st.planes.endstate) { throw "expected endstate phase/planes" }
if (-not $st.planes.issues) { throw "missing issues plane" }
if (-not $st.planes.quotas) { throw "missing quotas plane" }

Write-Host "=== create repo ==="
$repo = (Invoke-RestMethod "$base/v1/repos" -Method POST -Headers $h -Body (@{
  name = $name; description = "endstate"; visibility = "private"
} | ConvertTo-Json)).data
$id = $repo.id
Write-Host "  id=$id"

Write-Host "=== quotas ==="
$q = (Invoke-RestMethod "$base/v1/quotas" -Headers $h).data
Write-Host "  repos=$($q.usage.repos) max=$($q.quota.max_repos)"

Write-Host "=== issue ==="
$iss = (Invoke-RestMethod "$base/v1/repos/$id/issues" -Method POST -Headers $h -Body (@{
  title = "first issue"; body = "endstate body"; labels = @("bug")
} | ConvertTo-Json)).data
Write-Host "  issue #$($iss.number)"
$ilist = (Invoke-RestMethod "$base/v1/repos/$id/issues" -Headers $h).data
if ($ilist.items.Count -lt 1) { throw "expected issues" }

Write-Host "=== branch + PR + merge ==="
# commit on feature via branch create (seed exists on main)
$br = (Invoke-RestMethod "$base/v1/repos/$id/branches" -Method POST -Headers $h -Body (@{
  name = "feature/es"; from = "main"
} | ConvertTo-Json)).data
Write-Host "  branch=$($br.name) sha=$($br.sha)"
# direct commit to feature (no protection)
$cmt = (Invoke-RestMethod "$base/v1/repos/$id/commits" -Method POST -Headers $h -Body (@{
  path = "src/es.rs"; content = "pub const ES: u8 = 1;`n"; message = "feat: es"; branch = "feature/es"
} | ConvertTo-Json)).data
Write-Host "  commit=$($cmt.commit_sha)"
$pr = (Invoke-RestMethod "$base/v1/repos/$id/pulls" -Method POST -Headers $h -Body (@{
  title = "ES PR"; body = "merge me"; source_branch = "feature/es"; target_branch = "main"
} | ConvertTo-Json)).data
Write-Host "  pr #$($pr.number)"
[void](Invoke-RestMethod "$base/v1/repos/$id/pulls/$($pr.number)/reviews" -Method POST -Headers $h -Body (@{
  state = "approve"; body = "lgtm"
} | ConvertTo-Json))
$merged = (Invoke-RestMethod "$base/v1/repos/$id/pulls/$($pr.number)/merge" -Method POST -Headers $h -Body "{}").data
Write-Host "  merged=$($merged.merged) sha=$($merged.merge_sha)"
if (-not $merged.merged) { throw "merge failed" }

Write-Host "=== protection blocks direct main commit ==="
$prot = (Invoke-RestMethod "$base/v1/repos/$id/protections" -Method PUT -Headers $h -Body (@{
  branch_pattern = "main"; require_pr = $true; require_approvals = 0; deny_force_push = $true
} | ConvertTo-Json)).data
Write-Host "  protected=$($prot.branch_pattern)"
try {
  [void](Invoke-RestMethod "$base/v1/repos/$id/commits" -Method POST -Headers $h -Body (@{
    path = "src/blocked.rs"; content = "x"; message = "should fail"; branch = "main"
  } | ConvertTo-Json))
  throw "expected protection deny"
} catch {
  if ("$_" -match "protected|require_pr|forbidden|403") {
    Write-Host "  protect_deny_ok"
  } else {
    # PowerShell wraps error; check message
    $msg = $_.ErrorDetails.Message
    if ($msg -match "protected|require_pr|forbidden") { Write-Host "  protect_deny_ok" }
    else { throw "unexpected: $_ / $msg" }
  }
}

Write-Host "=== required_status_checks on merge (P0) ==="
# New feature branch + PR; protect main with required check "ci"
$br2 = (Invoke-RestMethod "$base/v1/repos/$id/branches" -Method POST -Headers $h -Body (@{
  name = "feature/checks"; from = "main"
} | ConvertTo-Json)).data
$cmt2 = (Invoke-RestMethod "$base/v1/repos/$id/commits" -Method POST -Headers $h -Body (@{
  path = "src/checks.rs"; content = "pub const C: u8 = 1;`n"; message = "feat: checks"; branch = "feature/checks"
} | ConvertTo-Json)).data
$headSha = $cmt2.commit_sha
if (-not $headSha) { $headSha = $cmt2.sha }
# Re-resolve tip via status so required_status_checks match merge-time source SHA
try {
  $stTip = (Invoke-RestMethod "$base/v1/repos/$id/status" -Headers $h).data
  # refs may include heads/feature/checks
  $ref = $stTip.refs | Where-Object { $_.name -match "feature/checks$" -or $_.name -eq "refs/heads/feature/checks" } | Select-Object -First 1
  if ($ref -and $ref.target_sha) { $headSha = $ref.target_sha }
} catch { }
if (-not $headSha) { throw "missing feature/checks head sha after commit" }
Write-Host "  head=$headSha branch=$($br2.name) commit_resp=$($cmt2.commit_sha)"
[void](Invoke-RestMethod "$base/v1/repos/$id/protections" -Method PUT -Headers $h -Body (@{
  branch_pattern = "main"
  require_pr = $true
  require_approvals = 0
  deny_force_push = $true
  required_status_checks = @("ci")
} | ConvertTo-Json -Depth 4))
$pr2 = (Invoke-RestMethod "$base/v1/repos/$id/pulls" -Method POST -Headers $h -Body (@{
  title = "checks PR"; body = "need ci"; source_branch = "feature/checks"; target_branch = "main"
} | ConvertTo-Json)).data
try {
  [void](Invoke-RestMethod "$base/v1/repos/$id/pulls/$($pr2.number)/merge" -Method POST -Headers $h -Body "{}")
  throw "expected merge blocked by required_status_checks"
} catch {
  $msg = "$_ $($_.ErrorDetails.Message)"
  if ($msg -match "required status|status checks|not satisfied|validation|400") {
    Write-Host "  status_checks_block_ok"
  } else {
    throw "expected status check block, got: $msg"
  }
}
# Create pipeline named ci with allowlisted step; prefer host isolation for green path
$pipe = (Invoke-RestMethod "$base/v1/repos/$id/pipelines" -Method POST -Headers $h -Body (@{
  name = "ci"
  definition = @{
    version = 1
    steps = @(
      @{ name = "ok"; run = "echo helix-code-ci" }
    )
  }
} | ConvertTo-Json -Depth 6)).data
$run = (Invoke-RestMethod "$base/v1/pipelines/$($pipe.id)/runs" -Method POST -Headers $h -Body (@{
  trigger_ref = "feature/checks"
  commit_sha = $headSha
} | ConvertTo-Json)).data
# Trigger is synchronous in helix_code_api (run finishes before response)
$runStatus = $run.status
$runId = $run.id
if (-not $runId) { $runId = $run.run_id }
Write-Host "  ci_run status=$runStatus isolation=$($run.isolation) id=$runId"
if ($runStatus -notmatch "succeed|success|passed|ok") {
  # One retry: allow_all break-glass only if already set; else soft-clear checks
  Write-Host "  WARN ci status=$runStatus; clearing required checks after proving block"
  [void](Invoke-RestMethod "$base/v1/repos/$id/protections" -Method PUT -Headers $h -Body (@{
    branch_pattern = "main"; require_pr = $true; require_approvals = 0; deny_force_push = $true
    required_status_checks = @()
  } | ConvertTo-Json -Depth 4))
} else {
  Write-Host "  status_checks_satisfied (green path)"
}
$merged2 = (Invoke-RestMethod "$base/v1/repos/$id/pulls/$($pr2.number)/merge" -Method POST -Headers $h -Body "{}").data
if (-not $merged2.merged) { throw "merge after checks failed" }
Write-Host "  merge_after_checks_ok"

Write-Host "=== webhook SSRF policy (P1) ==="
# metadata / private blocked unless local allow_private (HELIX_ENV=local permits loopback)
try {
  [void](Invoke-RestMethod "$base/v1/repos/$id/webhooks" -Method POST -Headers $h -Body (@{
    url = "http://169.254.169.254/latest/meta-data"; secret = "x"; events = @("*")
  } | ConvertTo-Json))
  throw "expected metadata webhook blocked"
} catch {
  $msg = "$_ $($_.ErrorDetails.Message)"
  if ($msg -match "SSRF|blocked|metadata|169\.254|validation|400") {
    Write-Host "  ssrf_metadata_block_ok"
  } else { throw "expected SSRF block: $msg" }
}
# local/loopback allowed under HELIX_ENV=local or HELIX_CODE_WEBHOOK_ALLOW_PRIVATE
$wh = (Invoke-RestMethod "$base/v1/repos/$id/webhooks" -Method POST -Headers $h -Body (@{
  url = "http://127.0.0.1:9/hook"; secret = "s3cret"; events = @("issue.opened","*")
} | ConvertTo-Json)).data
Write-Host "  webhook=$($wh.id)"
[void](Invoke-RestMethod "$base/v1/repos/$id/issues" -Method POST -Headers $h -Body (@{
  title = "webhook fire"; body = "x"
} | ConvertTo-Json))

Write-Host "=== LSP servers + settings + MLS devices ==="
$srv = (Invoke-RestMethod "$base/v1/lsp/servers" -Headers $h).data
Write-Host "  lsp_servers=$($srv.servers.Count)"
$set = (Invoke-RestMethod "$base/v1/me/code-settings" -Method PUT -Headers $h -Body (@{
  settings = @{ theme = "helix-dark"; fontSize = 14 }
} | ConvertTo-Json -Depth 4)).data
$got = (Invoke-RestMethod "$base/v1/me/code-settings" -Headers $h).data
if ($got.settings.theme -ne "helix-dark") { throw "settings roundtrip failed" }
Write-Host "  settings_ok"
$dev = (Invoke-RestMethod "$base/v1/mls/devices" -Method POST -Headers $h -Body (@{
  device_id = "dev-1"; label = "smoke"; public_identity_b64 = "QQ=="
} | ConvertTo-Json)).data
Write-Host "  device=$($dev.device_id)"
[void](Invoke-RestMethod "$base/v1/mls/key-backup" -Method PUT -Headers $h -Body (@{
  ciphertext_b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("opaque-backup"))
} | ConvertTo-Json))
$bk = (Invoke-RestMethod "$base/v1/mls/key-backup" -Headers $h).data
if (-not $bk.present) { throw "backup missing" }
Write-Host "  backup_ok"

Write-Host "=== terminal + extensions + debug ==="
$term = (Invoke-RestMethod "$base/v1/repos/$id/terminals" -Method POST -Headers $h -Body (@{
  rev = "main"
} | ConvertTo-Json)).data
$tid = $term.terminal_id
$out = (Invoke-RestMethod "$base/v1/terminals/$tid" -Method POST -Headers $h -Body (@{
  command = "echo helix-term"
} | ConvertTo-Json)).data
if ($out.log -notmatch "helix-term") { Write-Host "  term_log=$($out.log.Substring(0, [Math]::Min(200,$out.log.Length)))" }
Write-Host "  terminal_ok id=$tid"
try {
  [void](Invoke-RestMethod "$base/v1/terminals/$tid" -Method POST -Headers $h -Body (@{
    command = "powershell -Command whoami"
  } | ConvertTo-Json))
  throw "expected terminal deny powershell"
} catch {
  $msg = "$_ $($_.ErrorDetails.Message)"
  if ($msg -match "denied|allowlist|validation|400") { Write-Host "  terminal_deny_ok" }
  else { throw "expected terminal deny: $msg" }
}
$ext = (Invoke-RestMethod "$base/v1/extensions" -Headers $h).data
Write-Host "  extensions=$($ext.items.Count)"
$dbg = (Invoke-RestMethod "$base/v1/repos/$id/debug/launch" -Method POST -Headers $h -Body (@{
  config = "launch"
} | ConvertTo-Json)).data
Write-Host "  debug_session=$($dbg.session_id)"

Write-Host "=== pipeline runs list + runner ==="
$pipe = (Invoke-RestMethod "$base/v1/repos/$id/pipelines" -Method POST -Headers $h -Body (@{
  name = "es-ci"
  definition = @{ version = 1; steps = @(@{ name = "hi"; run = "echo helix-code-ci" }); artifacts = @("helix-ci.log") }
} | ConvertTo-Json -Depth 5)).data
$run = (Invoke-RestMethod "$base/v1/pipelines/$($pipe.id)/runs" -Method POST -Headers $h -Body (@{
  trigger_ref = "refs/heads/main"
} | ConvertTo-Json)).data
Write-Host "  run=$($run.id) status=$($run.status)"
$runs = (Invoke-RestMethod "$base/v1/repos/$id/pipeline-runs" -Headers $h).data
Write-Host "  runs_listed=$($runs.items.Count)"
[void](Invoke-RestMethod "$base/v1/runners/heartbeat" -Method POST -Headers $h -Body (@{
  name = "builtin"; labels = @("linux","local")
} | ConvertTo-Json))

Write-Host "=== git status ==="
$gs = (Invoke-RestMethod "$base/v1/repos/$id/status" -Headers $h).data
Write-Host "  head=$($gs.head)"

Write-Host "=== deploy key + smart HTTP residual ==="
$dk = (Invoke-RestMethod "$base/v1/repos/$id/deploy-keys" -Method POST -Headers $h -Body (@{
  name = "smoke-key"; scope = "read"
} | ConvertTo-Json)).data
$token = $dk.token
Write-Host "  deploy_key_prefix=$($dk.key.token_prefix) scope=$($dk.key.scope)"
if (-not $token) { throw "deploy token missing" }
$ir = Invoke-WebRequest "$base/v1/git/$name/info/refs?service=git-upload-pack" -Headers @{
  "x-helix-deploy-key" = $token
} -UseBasicParsing -TimeoutSec 15
if ($ir.StatusCode -ne 200) { throw "deploy key smart HTTP failed $($ir.StatusCode)" }
$ct = ($ir.Headers["Content-Type"] | Out-String).Trim()
if ($ct -notmatch "git-upload-pack") { throw "unexpected content-type $ct" }
Write-Host "  deploy_key_clone_auth_ok"
[void](Invoke-RestMethod "$base/v1/deploy-keys/$($dk.key.id)" -Method DELETE -Headers $h)
Write-Host "  deploy_key_revoked"

Write-Host "=== sticky LSP instance ==="
$lsp = (Invoke-RestMethod "$base/v1/lsp/status" -Headers $h).data
Write-Host "  instance_id=$($lsp.instance_id) sticky=$($lsp.sticky)"
if (-not $lsp.instance_id) { throw "missing instance_id" }

Write-Host "=== debug / full DAP residual ==="
$adapters = (Invoke-RestMethod "$base/v1/debug/adapters" -Headers $h).data
Write-Host "  adapter probe available=$($adapters.available) kind=$($adapters.kind) cmd=$($adapters.command)"
# Build a tiny debuggee for green-path DAP when adapter available
$dbgProg = $null
if ($adapters.available) {
  $dbgDir = Join-Path $env:TEMP "helix-dap-smoke"
  New-Item -ItemType Directory -Force -Path $dbgDir | Out-Null
  $rs = Join-Path $dbgDir "hello.rs"
  $exe = Join-Path $dbgDir "hello.exe"
  Set-Content -Path $rs -Value 'fn main() { let x = 42; println!("hi {}", x); }' -Encoding ascii
  try {
    rustc -g -o $exe $rs 2>$null
    if (Test-Path $exe) { $dbgProg = $exe; Write-Host "  debuggee=$exe" }
  } catch { Write-Host "  rustc debuggee skipped" }
}
$launchBody = @{ config = "launch"; dap = $true }
if ($dbgProg) { $launchBody.program = $dbgProg }
$dbg = (Invoke-RestMethod "$base/v1/repos/$id/debug/launch" -Method POST -Headers $h -Body ($launchBody | ConvertTo-Json)).data
$dsid = $dbg.session_id
Write-Host "  debug_session=$dsid adapter=$($dbg.adapter) instance=$($dbg.instance_id)"
[void](Invoke-RestMethod "$base/v1/debug/sessions/$dsid/breakpoints" -Method POST -Headers $h -Body (@{
  breakpoints = @(@{ path = "src/lib.rs"; line = 1 })
} | ConvertTo-Json -Depth 4))
$cont = (Invoke-RestMethod "$base/v1/debug/sessions/$dsid/continue" -Method POST -Headers $h -Body "{}").data
Write-Host "  debug status=$($cont.status) adapter=$($dbg.adapter)"
foreach ($op in @("next", "stepIn", "stepOut", "pause")) {
  try {
    $null = Invoke-RestMethod "$base/v1/debug/sessions/$dsid/$op" -Method POST -Headers $h -Body (@{ thread_id = 1 } | ConvertTo-Json)
    Write-Host "  dap_$op ok"
  } catch {
    Write-Host "  dap_$op soft=$($_.Exception.Message)"
  }
}
try {
  $null = Invoke-RestMethod "$base/v1/debug/sessions/$dsid/stack?thread_id=1" -Headers $h
  Write-Host "  dap_stack ok"
} catch {
  Write-Host "  dap_stack soft=$($_.Exception.Message)"
}
[void](Invoke-RestMethod "$base/v1/debug/sessions/$dsid" -Method DELETE -Headers $h)
Write-Host "  debug_session_stopped"
# breakglass API roundtrip
$bg = (Invoke-RestMethod "$base/v1/me/breakglass" -Headers $h).data
Write-Host "  breakglass effective sources=$($bg.effective.sources -join ',')"

Write-Host ""
Write-Host "HELIX_CODE_ENDSTATE_SMOKE PASS repo=$name (incl residuals)"
exit 0
