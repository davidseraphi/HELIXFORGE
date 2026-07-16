# HelixCode extreme E0 smoke — forge: create repo, tree, blob, commit, workspace, pipeline, sealed
# Prereq: helix_code_api on 8102 with Postgres; HELIX_ALLOW_DEV_HEADERS=1; git on PATH

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8102"
$name = "smoke-repo-$(Get-Random)"

Write-Host "=== code healthz ==="
$code = (Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 8).StatusCode
Write-Host "  $code"

Write-Host "=== domain status ==="
$st = (Invoke-RestMethod "$base/v1/domain/status" -Headers $h).data
Write-Host "  phase=$($st.phase) durable=$($st.durable) git=$($st.planes.git_backend) gix=$($st.planes.gitoxide) smart_http=$($st.planes.smart_http)"
if ("$($st.planes.git_backend)" -notmatch "gix") {
  throw "expected git_backend to include gix"
}

Write-Host "=== create repo ==="
$create = Invoke-RestMethod "$base/v1/repos" -Method POST -Headers $h -Body (@{
  name = $name
  description = "helix code extreme smoke"
  visibility = "private"
} | ConvertTo-Json)
$repo = $create.data
$id = $repo.id
Write-Host "  id=$id name=$($repo.name) head=$($repo.head_sha)"

Write-Host "=== list tree ==="
$tree = (Invoke-RestMethod "$base/v1/repos/$id/tree?rev=main" -Headers $h).data
Write-Host "  entries=$($tree.entries.Count)"
if ($tree.entries.Count -lt 1) { throw "expected tree entries" }

Write-Host "=== read README blob ==="
$blob = (Invoke-RestMethod "$base/v1/repos/$id/blob?rev=main&path=README.md" -Headers $h).data
if ($blob.content -notmatch "smoke-repo") { throw "README content missing" }
Write-Host "  readme_ok"

Write-Host "=== commit file ==="
$cmt = Invoke-RestMethod "$base/v1/repos/$id/commits" -Method POST -Headers $h -Body (@{
  path = "src/lib.rs"
  content = "pub fn smoke() -> u32 { 42 }`n"
  message = "feat: smoke lib"
  branch = "main"
} | ConvertTo-Json)
Write-Host "  sha=$($cmt.data.commit_sha)"

Write-Host "=== log ==="
$log = (Invoke-RestMethod "$base/v1/repos/$id/log?rev=main&limit=5" -Headers $h).data
Write-Host "  commits=$($log.commits.Count)"
if ($log.commits.Count -lt 2) { throw "expected >=2 commits" }

Write-Host "=== refs ==="
$refs = (Invoke-RestMethod "$base/v1/repos/$id/refs" -Headers $h).data
Write-Host "  live_refs=$($refs.live.Count)"

Write-Host "=== workspace ==="
$ws = Invoke-RestMethod "$base/v1/code/workspaces" -Method POST -Headers $h -Body (@{
  repo_id = $id
  name = "default"
  branch = "main"
} | ConvertTo-Json)
Write-Host "  workspace=$($ws.data.id)"

Write-Host "=== pipeline + sandbox run (E2 + isolation) ==="
$pipe = Invoke-RestMethod "$base/v1/repos/$id/pipelines" -Method POST -Headers $h -Body (@{
  name = "ci"
  definition = @{
    version = 1
    steps = @(
      @{ name = "hello"; run = "echo helix-code-ci" },
      @{ name = "rev"; run = "git rev-parse HEAD" }
    )
    artifacts = @("helix-ci.log")
  }
} | ConvertTo-Json -Depth 5)
$run = Invoke-RestMethod "$base/v1/pipelines/$($pipe.data.id)/runs" -Method POST -Headers $h -Body (@{
  trigger_ref = "refs/heads/main"
} | ConvertTo-Json)
Write-Host "  run=$($run.data.id) status=$($run.data.status) exit=$($run.data.exit_code) isolation=$($run.data.isolation)"
if ($run.data.status -ne "succeeded") { throw "pipeline expected succeeded: $($run.data.log_text)" }
if (-not $run.data.isolation) { throw "expected isolation field on pipeline run" }
$arts = (Invoke-RestMethod "$base/v1/pipeline-runs/$($run.data.id)/artifacts" -Headers $h).data
Write-Host "  artifacts=$($arts.items.Count)"
if ($arts.items.Count -lt 1) { throw "expected at least helix-ci.log artifact" }

Write-Host "=== agent mesh + patch (E4) ==="
$job = Invoke-RestMethod "$base/v1/repos/$id/agent-jobs" -Method POST -Headers $h -Body (@{
  prompt = "add e4 marker file"
  kind = "mesh"
  branch = "main"
  commit = $true
  commit_message = "feat: e4 agent mesh patch"
  patches = @(
    @{
      path = "src/e4_marker.rs"
      content = "pub const E4: &str = `"mesh`";`n"
      create = $true
    }
  )
  agents = @("helix-code-assistant", "helix-code-patcher")
} | ConvertTo-Json -Depth 6)
Write-Host "  job=$($job.data.id) status=$($job.data.status) commit=$($job.data.commit_sha) isolation=$($job.data.isolation)"
if ($job.data.status -ne "succeeded") { throw "agent job failed: $($job.data.log_text)" }
if (-not $job.data.isolation) { throw "expected isolation field on agent job" }
$files = $job.data.files_changed
Write-Host "  files_changed=$($files | ConvertTo-Json -Compress)"
if (-not ($files -contains "src/e4_marker.rs")) { throw "expected e4_marker.rs in files_changed" }

Write-Host "=== smart HTTP info/refs (upload-pack) ==="
$ir = Invoke-WebRequest "$base/v1/git/$name/info/refs?service=git-upload-pack" -Headers @{
  "x-helix-dev-user" = "ops@helixforge.local"
} -UseBasicParsing -TimeoutSec 15
if ($ir.StatusCode -ne 200) { throw "info/refs status $($ir.StatusCode)" }
$ct = ($ir.Headers["Content-Type"] | Out-String).Trim()
Write-Host "  status=200 content-type=$ct bytes=$($ir.RawContentLength)"
if ($ct -notmatch "git-upload-pack-advertisement") {
  throw "unexpected content-type: $ct"
}
# Body is pkt-line binary; look for service banner in raw bytes
$text = [System.Text.Encoding]::UTF8.GetString($ir.Content)
if ($text -notmatch "service=git-upload-pack" -and $text -notmatch "refs/heads") {
  throw "info/refs body missing refs advertisement"
}
Write-Host "  smart_http_ok"

Write-Host "=== sealed HVA4 + crypto group (E5) ==="
$secret = "secret-smoke-$(Get-Random)-e5"
$seal = Invoke-RestMethod "$base/v1/repos/$id/sealed-objects" -Method POST -Headers $h -Body (@{
  content = $secret
  classification = "confidential"
  name = "smoke-secret"
  purpose = "forge.smoke"
  seal_mode = "tenant"
} | ConvertTo-Json)
$oid = $seal.data.id
Write-Host "  sealed=$($seal.data.content_sha256) envelope=$($seal.data.envelope_kind)"
if ($seal.data.envelope_kind -ne "hva4") { throw "expected hva4 envelope" }
$got = Invoke-RestMethod "$base/v1/repos/$id/sealed-objects/$oid" -Headers $h
if ($got.data.content_utf8 -ne $secret) { throw "decrypt mismatch" }
Write-Host "  decrypt_ok"
$list = (Invoke-RestMethod "$base/v1/repos/$id/sealed-objects" -Headers $h).data
Write-Host "  list_count=$($list.items.Count)"
$grp = Invoke-RestMethod "$base/v1/crypto-groups" -Method POST -Headers $h -Body (@{
  name = "grp-smoke-$(Get-Random)"
  purpose = "forge.mls-like"
} | ConvertTo-Json)
$gid = $grp.data.id
Write-Host "  crypto_group=$gid"
$gseal = Invoke-RestMethod "$base/v1/repos/$id/sealed-objects" -Method POST -Headers $h -Body (@{
  content = "group-secret-$secret"
  classification = "mls"
  name = "group-blob"
  seal_mode = "group"
  group_id = $gid
} | ConvertTo-Json)
Write-Host "  group_sealed=$($gseal.data.envelope_kind)"
$ggot = Invoke-RestMethod "$base/v1/repos/$id/sealed-objects/$($gseal.data.id)" -Headers $h
if ($ggot.data.content_utf8 -ne "group-secret-$secret") { throw "group decrypt mismatch" }
Write-Host "  group_decrypt_ok"
[void](Invoke-RestMethod "$base/v1/repos/$id/sealed-objects/$oid" -Method DELETE -Headers $h)
Write-Host "  deleted_tenant_seal"

Write-Host "=== LSP (E3) ==="
$lspSt = (Invoke-RestMethod "$base/v1/lsp/status" -Headers $h).data
Write-Host "  available=$($lspSt.available) command=$($lspSt.command)"
if ($lspSt.available) {
  $sess = Invoke-RestMethod "$base/v1/repos/$id/lsp/session" -Method POST -Headers $h -Body (@{
    rev = "main"
  } | ConvertTo-Json)
  $sid = $sess.data.session_id
  Write-Host "  session=$sid"
  $blob = (Invoke-RestMethod "$base/v1/repos/$id/blob?rev=main&path=src/lib.rs" -Headers $h).data
  $open = Invoke-RestMethod "$base/v1/lsp/sessions/$sid/did-open" -Method POST -Headers $h -Body (@{
    path = "src/lib.rs"
    content = $blob.content
    language_id = "rust"
  } | ConvertTo-Json)
  Write-Host "  did-open diags=$($open.data.diagnostics.Count)"
  try {
    $hov = Invoke-RestMethod "$base/v1/lsp/sessions/$sid/hover" -Method POST -Headers $h -Body (@{
      path = "src/lib.rs"
      line = 0
      character = 4
    } | ConvertTo-Json)
    Write-Host "  hover=$([bool]$hov.data.hover)"
  } catch {
    # Hover may fail for positions with no symbol; session/did-open is the hard gate
    Write-Host "  hover_soft_fail (ok for smoke): $_"
  }
  [void](Invoke-RestMethod "$base/v1/lsp/sessions/$sid" -Method DELETE -Headers $h)
  Write-Host "  session closed"
} else {
  Write-Host "  SKIP detailed LSP (binary not found) - status endpoint ok"
}

Write-Host "=== OpenMLS multi-tenant forge (horizon) ==="
# MLS forbids decrypting your own app message — use two dev users for join + decrypt.
$hBob = @{ "x-helix-dev-user" = "bob@helixforge.local"; "Content-Type" = "application/json" }
$mlsSt = (Invoke-RestMethod "$base/v1/mls/status" -Headers $h).data
Write-Host "  openmls=$($mlsSt.openmls) ciphersuite=$($mlsSt.ciphersuite)"
if (-not $mlsSt.openmls) { throw "expected openmls true" }
$ident = Invoke-RestMethod "$base/v1/mls/identity" -Method POST -Headers $h -Body (@{
  label = "smoke"
} | ConvertTo-Json)
Write-Host "  identity_alice=$($ident.data.user_key)"
[void](Invoke-RestMethod "$base/v1/mls/identity" -Method POST -Headers $hBob -Body (@{
  label = "smoke"
} | ConvertTo-Json))
$kp = Invoke-RestMethod "$base/v1/mls/key-package" -Method POST -Headers $hBob -Body "{}"
Write-Host "  bob_key_package_ok"
$grp = Invoke-RestMethod "$base/v1/mls/groups" -Method POST -Headers $h -Body (@{
  name = "smoke-mls-$(Get-Random)"
  repo_id = $id
} | ConvertTo-Json)
$mgid = $grp.data.group_id
Write-Host "  group=$mgid epoch=$($grp.data.epoch) members=$($grp.data.member_count)"
$add = Invoke-RestMethod "$base/v1/mls/groups/$mgid/add" -Method POST -Headers $h -Body (@{
  key_package_tls_b64 = $kp.data.key_package_tls_b64
} | ConvertTo-Json)
Write-Host "  add_member epoch=$($add.data.epoch) members=$($add.data.members.Count)"
$joined = Invoke-RestMethod "$base/v1/mls/groups/$mgid/join" -Method POST -Headers $hBob -Body (@{
  welcome_tls_b64 = $add.data.welcome_tls_b64
} | ConvertTo-Json)
Write-Host "  bob_joined members=$($joined.data.member_count)"
$enc = Invoke-RestMethod "$base/v1/mls/groups/$mgid/encrypt" -Method POST -Headers $h -Body (@{
  content = "forge-mls-smoke"
} | ConvertTo-Json)
$dec = Invoke-RestMethod "$base/v1/mls/groups/$mgid/decrypt" -Method POST -Headers $hBob -Body (@{
  ciphertext_tls_b64 = $enc.data.ciphertext_tls_b64
} | ConvertTo-Json)
if ($dec.data.plaintext_utf8 -ne "forge-mls-smoke") {
  throw "OpenMLS decrypt mismatch: $($dec.data | ConvertTo-Json -Compress)"
}
Write-Host "  encrypt_alice_decrypt_bob_ok"
$mseal = Invoke-RestMethod "$base/v1/repos/$id/mls-sealed" -Method POST -Headers $h -Body (@{
  group_id = $mgid
  content = "mls-pack-$secret"
  name = "mls-smoke-pack"
} | ConvertTo-Json)
Write-Host "  mls_sealed=$($mseal.data.content_sha256) class=$($mseal.data.classification)"

Write-Host "=== domain horizons flags ==="
$st2 = (Invoke-RestMethod "$base/v1/domain/status" -Headers $h).data
Write-Host "  phase=$($st2.phase) openmls=$($st2.planes.openmls) container=$($st2.planes.container_isolation) iso=$($st2.planes.isolation_mode) code_oss=$($st2.planes.code_oss)"
if (-not $st2.planes.openmls) { throw "domain missing openmls plane" }
if (-not $st2.planes.container_isolation) { throw "domain missing container_isolation plane" }
if (-not $st2.planes.code_oss) { throw "domain missing code_oss plane" }
if (-not $st2.planes.split_editor_groups) { throw "domain missing split_editor_groups plane" }
if (-not $st2.planes.electron_shell) { throw "domain missing electron_shell plane" }
Write-Host "  split=$($st2.planes.split_editor_groups) electron_shell=$($st2.planes.electron_shell)"
Write-Host "  docker_image=$($st2.planes.docker_image) preferred=$($st2.planes.docker_ci_image_preferred) forge_tools=$($st2.planes.docker_has_forge_tools)"
if (-not $st2.planes.docker_ci_image_preferred) { throw "domain missing docker_ci_image_preferred" }

Write-Host "=== Code-OSS files index + search + batch commit ==="
$files = (Invoke-RestMethod "$base/v1/repos/$id/files?rev=main&max=500" -Headers $h).data
Write-Host "  files_count=$($files.count)"
if ($files.count -lt 2) { throw "expected recursive files index >=2" }
if (-not ($files.files -contains "src/lib.rs") -and -not ($files.files | Where-Object { $_ -match "lib\.rs" })) {
  Write-Host "  note: lib.rs path list=$($files.files | Select-Object -First 8 | ConvertTo-Json -Compress)"
}
$search = (Invoke-RestMethod "$base/v1/repos/$id/search?q=smoke&rev=main&max=20" -Headers $h).data
Write-Host "  search_hits=$($search.count)"
if ($search.count -lt 1) { throw "expected content search hits for 'smoke'" }
$batch = Invoke-RestMethod "$base/v1/repos/$id/commits/batch" -Method POST -Headers $h -Body (@{
  message = "feat: code-oss batch"
  branch = "main"
  files = @(
    @{ path = "src/oss_a.rs"; content = "pub const OSS_A: u8 = 1;`n" },
    @{ path = "src/oss_b.rs"; content = "pub const OSS_B: u8 = 2;`n" }
  )
} | ConvertTo-Json -Depth 5)
Write-Host "  batch_sha=$($batch.data.commit_sha) count=$($batch.data.count)"
if ($batch.data.count -ne 2) { throw "batch commit expected 2 files" }
$ba = (Invoke-RestMethod "$base/v1/repos/$id/blob?rev=main&path=src/oss_a.rs" -Headers $h).data
if ($ba.content -notmatch "OSS_A") { throw "batch blob oss_a missing" }
Write-Host "  batch_blob_ok"

Write-Host ""
Write-Host "HELIX_CODE_SMOKE PASS repo=$name (code-oss + openmls + isolation)"
exit 0
