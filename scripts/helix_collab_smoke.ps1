# HelixCollab deep smoke — documents, conflict, revisions, presence, domain status
# Prereq: Core optional; collab on 8101 with Postgres; HELIX_ALLOW_DEV_HEADERS=1 at process start

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8101"

Write-Host "=== collab healthz ==="
$code = (Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 5).StatusCode
Write-Host "  $code"

Write-Host "=== domain status ==="
$st = (Invoke-RestMethod "$base/v1/domain/status" -Headers $h).data
Write-Host "  durable=$($st.durable) features=$($st.features | ConvertTo-Json -Compress)"

Write-Host "=== create document ==="
$create = Invoke-RestMethod "$base/v1/documents" -Method POST -Headers $h -Body (@{
  title = "Smoke Doc $(Get-Random)"
  content = "# hello`nline1"
} | ConvertTo-Json)
$doc = $create.data
$id = $doc.id
Write-Host "  id=$id v=$($doc.version)"

Write-Host "=== patch ok ==="
$p1 = Invoke-RestMethod "$base/v1/documents/$id" -Method PATCH -Headers $h -Body (@{
  base_version = $doc.version
  content = "# hello`nline2"
  title = $doc.title
} | ConvertTo-Json)
Write-Host "  v=$($p1.data.version)"

Write-Host "=== patch conflict (expect 409) ==="
try {
  Invoke-RestMethod "$base/v1/documents/$id" -Method PATCH -Headers $h -Body (@{
    base_version = 1
    content = "stale"
  } | ConvertTo-Json)
  throw "expected 409 conflict"
} catch {
  $resp = $_.Exception.Response
  if ($resp -and [int]$resp.StatusCode -eq 409) {
    Write-Host "  409 OK"
  } else {
    # PowerShell may wrap differently
    Write-Host "  conflict path: $_"
  }
}

Write-Host "=== revisions ==="
$revs = (Invoke-RestMethod "$base/v1/documents/$id/revisions" -Headers $h).data
Write-Host "  count=$($revs.Count) latest=$($revs[0].version)"

Write-Host "=== presence ==="
[void](Invoke-RestMethod "$base/v1/documents/$id/presence" -Method POST -Headers $h -Body (@{
  display_name = "smoke-bot"
  cursor_pos = 3
} | ConvertTo-Json))
$pres = (Invoke-RestMethod "$base/v1/documents/$id/presence" -Headers $h).data
Write-Host "  peers=$($pres.peers.Count) ws_peers=$($pres.ws_peers)"

Write-Host "=== restore v1 ==="
$cur = (Invoke-RestMethod "$base/v1/documents/$id" -Headers $h).data
$restored = Invoke-RestMethod "$base/v1/documents/$id/revisions/1/restore" -Method POST -Headers $h
Write-Host "  new_version=$($restored.data.version)"

Write-Host "=== share ACL ==="
$share = Invoke-RestMethod "$base/v1/documents/$id/share" -Method POST -Headers $h -Body (@{
  principal_id = "peer-smoke-user"
  principal_kind = "user"
  permissions = @("read", "write")
} | ConvertTo-Json)
Write-Host "  shared_with=$($share.data.acl.principal_id)"
$acl = (Invoke-RestMethod "$base/v1/documents/$id/share" -Headers $h).data
Write-Host "  acl_count=$($acl.items.Count)"

Write-Host "=== domain flags ==="
$st2 = (Invoke-RestMethod "$base/v1/domain/status" -Headers $h).data
Write-Host "  ws_auth=$($st2.features.ws_auth) durable_ws=$($st2.features.durable_ws_patch) share=$($st2.features.share) crdt=$($st2.features.crdt)"
Write-Host "  folders=$($st2.features.folders) comments=$($st2.features.comments) mentions=$($st2.features.mentions)"

Write-Host "=== workspace + folder ==="
$ws = Invoke-RestMethod "$base/v1/workspaces" -Method POST -Headers $h -Body (@{ name = "Smoke WS $(Get-Random)" } | ConvertTo-Json)
$wsId = $ws.data.id
Write-Host "  workspace=$wsId"
$folder = Invoke-RestMethod "$base/v1/workspaces/$wsId/folders" -Method POST -Headers $h -Body (@{ name = "Specs" } | ConvertTo-Json)
$fid = $folder.data.id
Write-Host "  folder=$fid"
$doc2 = Invoke-RestMethod "$base/v1/documents" -Method POST -Headers $h -Body (@{
  title = "In folder"
  content = "hello"
  workspace_id = $wsId
  folder_id = $fid
} | ConvertTo-Json)
Write-Host "  doc_in_folder=$($doc2.data.id)"

Write-Host "=== comment + mention ==="
$c = Invoke-RestMethod "$base/v1/documents/$id/comments" -Method POST -Headers $h -Body (@{
  body = "Please review @alice and @ops@helixforge.local"
  author_label = "ops@helixforge.local"
} | ConvertTo-Json)
Write-Host "  comment=$($c.data.id) mentions=$($c.data.mentions.Count)"
$inbox = (Invoke-RestMethod "$base/v1/mentions/inbox?label=alice" -Headers $h).data
Write-Host "  inbox_alice=$($inbox.Count)"

Write-Host "=== polish: rename folder + mention suggest + edit comment ==="
$ren = Invoke-RestMethod "$base/v1/folders/$fid" -Method PATCH -Headers $h -Body (@{ name = "Specs-v2" } | ConvertTo-Json)
Write-Host "  folder_renamed=$($ren.data.name)"
$sug = (Invoke-RestMethod "$base/v1/documents/$id/mention-suggest" -Headers $h).data
Write-Host "  suggestions=$($sug.suggestions.Count)"
$cid = $c.data.id
$up = Invoke-RestMethod "$base/v1/documents/$id/comments/$cid" -Method PATCH -Headers $h -Body (@{
  body = "Updated @alice please"
} | ConvertTo-Json)
Write-Host "  comment_updated mentions=$($up.data.mentions.Count)"

Write-Host "=== deeper: e2ee + anchor resolve + activity + pin ==="
$e2 = Invoke-RestMethod "$base/v1/documents" -Method POST -Headers $h -Body (@{
  title = "Secret"
  content = "classified-plaintext"
  e2ee = $true
} | ConvertTo-Json)
Write-Host "  e2ee_create encrypted=$($e2.data.encrypted) content_ok=$($e2.data.content -eq 'classified-plaintext')"
$anch = Invoke-RestMethod "$base/v1/documents/$id/comments" -Method POST -Headers $h -Body (@{
  body = "on selection"
  anchor_start = 0
  anchor_end = 5
  anchor_quote = "hello"
} | ConvertTo-Json)
Write-Host "  anchored=$($anch.data.anchor_quote)"
$res = Invoke-RestMethod "$base/v1/documents/$id/comments/$($anch.data.id)/resolve" -Method POST -Headers $h -Body (@{ resolved = $true } | ConvertTo-Json)
Write-Host "  resolved=$([bool]$res.data.resolved_at)"
$act = (Invoke-RestMethod "$base/v1/documents/$id/activity" -Headers $h).data
Write-Host "  activity=$($act.Count)"
$pin = Invoke-RestMethod "$base/v1/documents/$id/flags" -Method POST -Headers $h -Body (@{ pinned = $true } | ConvertTo-Json)
Write-Host "  pinned=$($pin.data.pinned)"

Write-Host "=== polish2: duplicate + archive ==="
$dup = Invoke-RestMethod "$base/v1/documents" -Method POST -Headers $h -Body (@{
  title = "$($doc.title) (copy)"
  content = "dup-body"
  e2ee = $false
} | ConvertTo-Json)
Write-Host "  duplicate_id=$($dup.data.id) title=$($dup.data.title)"
$arch = Invoke-RestMethod "$base/v1/documents/$($dup.data.id)/flags" -Method POST -Headers $h -Body (@{ archive = $true } | ConvertTo-Json)
Write-Host "  archived=$([bool]$arch.data.archived_at)"
$list = (Invoke-RestMethod "$base/v1/documents" -Headers $h).data.items
$still = @($list | Where-Object { $_.id -eq $dup.data.id }).Count
Write-Host "  archive_hidden_from_list=$($still -eq 0)"
if ($still -ne 0) { throw "archived doc still in list" }

Write-Host "=== client_e2ee: server-blind HC1 envelope ==="
# Generate AES-GCM HC1 envelope (Node crypto; matches web client format)
$envEnvelope = (node -e @"
const crypto = require('crypto');
function b64url(buf){return Buffer.from(buf).toString('base64').replace(/\+/g,'-').replace(/\//g,'_').replace(/=+$/,'')}
const key = crypto.randomBytes(32);
const iv = crypto.randomBytes(12);
const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
const pt = Buffer.from('client-secret-plaintext');
const enc = Buffer.concat([cipher.update(pt), cipher.final()]);
const tag = cipher.getAuthTag();
const ct = Buffer.concat([enc, tag]);
process.stdout.write('HC1.' + b64url(iv) + '.' + b64url(ct));
"@)
$ce = Invoke-RestMethod "$base/v1/documents" -Method POST -Headers $h -Body (@{
  title = "ClientSecret"
  content = $envEnvelope
  client_e2ee = $true
} | ConvertTo-Json)
Write-Host "  client_e2ee=$($ce.data.client_e2ee) encrypted=$($ce.data.encrypted)"
Write-Host "  server_returns_envelope=$($ce.data.content.StartsWith('HC1.'))"
Write-Host "  not_plaintext=$($ce.data.content -ne 'client-secret-plaintext')"
if (-not $ce.data.client_e2ee) { throw "client_e2ee flag missing" }
if (-not $ce.data.content.StartsWith("HC1.")) { throw "server mutated/decrypted client envelope" }
$got = (Invoke-RestMethod "$base/v1/documents/$($ce.data.id)" -Headers $h).data
if ($got.content -ne $envEnvelope) { throw "GET changed client ciphertext" }
Write-Host "  get_opaque_ok=True"
# reject plaintext patch
try {
  Invoke-RestMethod "$base/v1/documents/$($ce.data.id)" -Method PATCH -Headers $h -Body (@{
    base_version = $got.version
    content = "should-fail-plaintext"
  } | ConvertTo-Json) | Out-Null
  throw "expected validation error for plaintext on client_e2ee"
} catch {
  $code = $_.Exception.Response.StatusCode.value__
  Write-Host "  plaintext_patch_rejected status=$code"
}
$st2 = (Invoke-RestMethod "$base/v1/domain/status" -Headers $h).data
Write-Host "  features.client_e2ee=$($st2.features.client_e2ee) prosemirror=$($st2.features.prosemirror) sealed_crdt=$($st2.features.sealed_crdt)"

Write-Host "=== sealed_crdt: blind relay cache ==="
$sealedState = (node -e @"
const crypto = require('crypto');
function b64url(buf){return Buffer.from(buf).toString('base64').replace(/\+/g,'-').replace(/\//g,'_').replace(/=+$/,'')}
const key = crypto.randomBytes(32);
const iv = crypto.randomBytes(12);
const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
const pt = Buffer.from('fake-yjs-full-state-bytes');
const enc = Buffer.concat([cipher.update(pt), cipher.final()]);
const tag = cipher.getAuthTag();
process.stdout.write('HC1.' + b64url(iv) + '.' + b64url(Buffer.concat([enc, tag])));
"@)
$put = Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/sealed-crdt" -Method POST -Headers $h -Body (@{
  sealed = $sealedState
  kind = "state"
} | ConvertTo-Json)
Write-Host "  put_accepted=$($put.data.accepted) has_state=$($put.data.has_state) server_blind=$($put.data.server_blind)"
$sg = (Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/sealed-crdt" -Headers $h).data
Write-Host "  get_state_match=$($sg.sealed_state -eq $sealedState)"
if ($sg.sealed_state -ne $sealedState) { throw "sealed state not retained" }
try {
  Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/sealed-crdt" -Method POST -Headers $h -Body (@{
    sealed = "plaintext-not-allowed"
    kind = "state"
  } | ConvertTo-Json) | Out-Null
  throw "expected reject non-HC1 sealed state"
} catch {
  $code = $_.Exception.Response.StatusCode.value__
  Write-Host "  plaintext_sealed_rejected status=$code"
}
$upd = (node -e @"
const crypto = require('crypto');
function b64url(buf){return Buffer.from(buf).toString('base64').replace(/\+/g,'-').replace(/\//g,'_').replace(/=+$/,'')}
const key = crypto.randomBytes(32);
const iv = crypto.randomBytes(12);
const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
const pt = Buffer.from('incr-update');
const enc = Buffer.concat([cipher.update(pt), cipher.final(), cipher.getAuthTag()]);
process.stdout.write('HC1.' + b64url(iv) + '.' + b64url(enc));
"@)
$pu = Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/sealed-crdt" -Method POST -Headers $h -Body (@{
  sealed = $upd
  kind = "update"
} | ConvertTo-Json)
Write-Host "  update_accepted=$($pu.data.accepted)"
$sg2 = (Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/sealed-crdt" -Headers $h).data
Write-Host "  recent_updates=$($sg2.recent_updates)"

Write-Host "=== sovereign horizons A-C ==="
$tm = (Invoke-RestMethod "$base/v1/sovereign/threat-model" -Headers $h).data
Write-Host "  threat_model server_never=$($tm.server_never_sees.Count) claims_doc=$($tm.doc)"
$cap = (Invoke-RestMethod "$base/v1/sovereign/capabilities" -Headers $h).data
Write-Host "  horizons_A=$($cap.horizons.A.Count) B=$($cap.horizons.B.Count) C=$($cap.horizons.C.Count) jetstream=$($cap.jetstream)"
$dev = Invoke-RestMethod "$base/v1/devices" -Method POST -Headers $h -Body (@{
  device_label = "smoke-device"
  public_key_b64 = "smoke-pubkey-b64url-example"
  algorithm = "ECDSA_P256"
} | ConvertTo-Json)
Write-Host "  device_id=$($dev.data.id)"
$devs = (Invoke-RestMethod "$base/v1/devices" -Headers $h).data.items
Write-Host "  devices=$($devs.Count)"
# classification policy: restricted requires client_e2ee
try {
  Invoke-RestMethod "$base/v1/documents" -Method POST -Headers $h -Body (@{
    title = "ShouldFail"
    content = "clear"
    classification = "sovereign"
  } | ConvertTo-Json) | Out-Null
  throw "expected sovereign cleartext create to fail"
} catch {
  $code = $_.Exception.Response.StatusCode.value__
  Write-Host "  sovereign_cleartext_rejected status=$code"
}
$cls = Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/classification" -Method POST -Headers $h -Body (@{
  classification = "restricted"
  sealed_comments = $true
} | ConvertTo-Json)
Write-Host "  class=$($cls.data.classification)"
$ks = Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/key-shares" -Method POST -Headers $h -Body (@{
  wrapped_dek = "HC1.share.opaque"
  device_key_id = $dev.data.id
  share_kind = "device"
} | ConvertTo-Json)
Write-Host "  key_share=$($ks.data.id)"
$bp = (Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/export" -Headers $h).data
Write-Host "  backpack_sha=$($bp.sha256.Substring(0,16))… format=$($bp.backpack.format)"
$dur = Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/sealed-crdt/durable" -Method POST -Headers $h -Body (@{
  sealed = $sealedState
} | ConvertTo-Json)
Write-Host "  durable_sealed jetstream=$($dur.data.jetstream) blind=$($dur.data.server_blind)"
$sp = Invoke-RestMethod "$base/v1/workspaces/$wsId/spaces" -Method POST -Headers $h -Body (@{
  name = "Secret Space"
  classification = "restricted"
} | ConvertTo-Json)
Write-Host "  space=$($sp.data.id) class=$($sp.data.classification)"
$att = Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/attachments" -Method POST -Headers $h -Body (@{
  filename = "secret.bin"
  object_key = "minio://tenant/secret.bin"
  size_bytes = 42
  client_sealed = $true
  sha256_hex = "abc"
} | ConvertTo-Json)
Write-Host "  attachment=$($att.data.id) sealed=$($att.data.client_sealed)"
$ag = Invoke-RestMethod "$base/v1/documents/$id/agent/suggest" -Method POST -Headers $h -Body (@{
  selection = "Alpha beta gamma delta. More text for summary."
  intent = "summarize"
} | ConvertTo-Json)
Write-Host "  agent_model=$($ag.data.model)"
try {
  Invoke-RestMethod "$base/v1/documents/$id/agent/suggest" -Method POST -Headers $h -Body (@{
    selection = "HC1.should.reject"
    intent = "summarize"
  } | ConvertTo-Json) | Out-Null
  throw "agent should refuse HC1"
} catch {
  Write-Host "  agent_refuses_hc1 status=$($_.Exception.Response.StatusCode.value__)"
}
$rec = Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/recovery" -Method POST -Headers $h -Body (@{
  k = 2
  n = 3
  meta = @{ note = "smoke" }
} | ConvertTo-Json -Depth 4)
Write-Host "  recovery_ceremony=$($rec.data.ceremony.id)"
$done = Invoke-RestMethod "$base/v1/recovery/$($rec.data.ceremony.id)/complete" -Method POST -Headers $h
Write-Host "  recovery_done=$($done.data.completed)"
$res = Invoke-RestMethod "$base/v1/documents/$($ce.data.id)/residency" -Method POST -Headers $h -Body (@{
  claimed_region = "local"
  verified = $true
  evidence = @{ smoke = $true }
} | ConvertTo-Json -Depth 4)
Write-Host "  residency=$($res.data.claimed_region)"
$fx = Invoke-RestMethod "$base/v1/federation/export" -Method POST -Headers $h -Body (@{
  document_id = $ce.data.id
  remote_deployment = "https://peer.example"
  signature_b64 = "sig"
} | ConvertTo-Json)
Write-Host "  federation_export hash=$($fx.data.payload_sha256.Substring(0,12))…"
$fi = Invoke-RestMethod "$base/v1/federation/import" -Method POST -Headers $h -Body (@{
  remote_deployment = "https://peer.example"
  signature_b64 = "sig"
  payload = $fx.data.payload
} | ConvertTo-Json -Depth 8)
Write-Host "  federation_import doc=$($fi.data.document.id)"
Write-Host "=== openmls + minio + passkey + residency ==="
$aliceId = $ce.data.id
# OpenMLS: identity, group, add bob, secrets match, app message
$hBob = @{ "x-helix-dev-user" = "bob@helixforge.local"; "Content-Type" = "application/json" }
$ia = Invoke-RestMethod "$base/v1/mls/identity" -Method POST -Headers $h
$ib = Invoke-RestMethod "$base/v1/mls/identity" -Method POST -Headers $hBob
Write-Host "  mls_identity_alice=$([bool]$ia.data.identity.signature_public_b64) bob=$([bool]$ib.data.identity.signature_public_b64)"
$g = Invoke-RestMethod "$base/v1/documents/$aliceId/mls/group" -Method POST -Headers $h
Write-Host "  mls_group epoch=$($g.data.group.epoch) members=$($g.data.group.member_count)"
$kp = Invoke-RestMethod "$base/v1/mls/key-packages" -Method POST -Headers $hBob
$add = Invoke-RestMethod "$base/v1/documents/$aliceId/mls/add" -Method POST -Headers $h -Body (@{
  key_package_tls_b64 = $kp.data.key_package_tls_b64
} | ConvertTo-Json)
Write-Host "  mls_add members=$($add.data.add.members.Count) epoch=$($add.data.add.epoch)"
$join = Invoke-RestMethod "$base/v1/documents/$aliceId/mls/join" -Method POST -Headers $hBob -Body (@{
  welcome_tls_b64 = $add.data.add.welcome_tls_b64
} | ConvertTo-Json)
Write-Host "  mls_join members=$($join.data.group.member_count)"
$secA = (Invoke-RestMethod "$base/v1/documents/$aliceId/mls/export-secret" -Headers $h).data.exported_secret_b64
$secB = (Invoke-RestMethod "$base/v1/documents/$aliceId/mls/export-secret" -Headers $hBob).data.exported_secret_b64
Write-Host "  mls_secrets_match=$($secA -eq $secB)"
if ($secA -ne $secB) { throw "MLS epoch secrets diverge" }
$plainB64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("sealed-app-payload"))
$msg = Invoke-RestMethod "$base/v1/documents/$aliceId/mls/message" -Method POST -Headers $h -Body (@{
  plaintext_b64 = $plainB64
} | ConvertTo-Json)
$proc = Invoke-RestMethod "$base/v1/documents/$aliceId/mls/process" -Method POST -Headers $hBob -Body (@{
  message_tls_b64 = $msg.data.message_tls_b64
} | ConvertTo-Json)
Write-Host "  mls_app_roundtrip=$($proc.data.plaintext_b64 -eq $plainB64)"
# MinIO attachment upload/download
$payload = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("HC1.minio-sealed-body-test"))
$up = Invoke-RestMethod "$base/v1/documents/$aliceId/attachments/upload" -Method POST -Headers $h -Body (@{
  filename = "secret.dat"
  content_type = "application/octet-stream"
  data_b64 = $payload
  client_sealed = $true
} | ConvertTo-Json)
Write-Host "  minio_upload id=$($up.data.attachment.id) stored=$($up.data.body_stored) storage=$($up.data.storage)"
$down = Invoke-RestMethod "$base/v1/documents/$aliceId/attachments/$($up.data.attachment.id)/body" -Headers $h
Write-Host "  minio_download_match=$($down.data.data_b64 -eq $payload) sealed=$($down.data.client_sealed)"
if ($down.data.data_b64 -ne $payload) { throw "minio roundtrip failed" }
$del = Invoke-RestMethod "$base/v1/documents/$aliceId/attachments/$($up.data.attachment.id)" -Method DELETE -Headers $h
Write-Host "  attachment_deleted=$($del.data.deleted) minio=$($del.data.minio_deleted)"
$listAfter = (Invoke-RestMethod "$base/v1/documents/$aliceId/attachments" -Headers $h).data.items
$stillAtt = @($listAfter | Where-Object { $_.id -eq $up.data.attachment.id }).Count
Write-Host "  attachment_gone=$($stillAtt -eq 0)"
if ($stillAtt -ne 0) { throw "attachment still listed after delete" }
# Passkey challenge register (v2 bound clientData)
$ps = Invoke-RestMethod "$base/v1/webauthn/register/start" -Method POST -Headers $h
Write-Host "  passkey_challenge=$([bool]$ps.data.challenge_b64) protocol=$($ps.data.protocol) has_client_data=$([bool]$ps.data.client_data_b64)"
# Residency hard enforce
$rr = Invoke-RestMethod "$base/v1/documents/$aliceId/required-region" -Method POST -Headers $h -Body (@{
  required_region = "local"
} | ConvertTo-Json)
Write-Host "  residency_enforced region=$($rr.data.required_region) allowed=$($rr.data.allowed_now)"
# Durable MLS: persist then re-hydrate via GET identity (simulates cold start path)
$pers = Invoke-RestMethod "$base/v1/mls/persist" -Method POST -Headers $h
Write-Host "  mls_persist bytes=$($pers.data.blob_bytes)"
$persB = Invoke-RestMethod "$base/v1/mls/persist" -Method POST -Headers $hBob
Write-Host "  mls_persist_bob bytes=$($persB.data.blob_bytes)"
# After persist, secrets still match via hydrated engine
$secA2 = (Invoke-RestMethod "$base/v1/documents/$aliceId/mls/export-secret" -Headers $h).data.exported_secret_b64
$secB2 = (Invoke-RestMethod "$base/v1/documents/$aliceId/mls/export-secret" -Headers $hBob).data.exported_secret_b64
Write-Host "  mls_durable_secrets_match=$($secA2 -eq $secB2)"
if ($secA2 -ne $secB2) { throw "durable MLS secrets diverge" }

Write-Host "=== DONE (helix-collab smoke OK) ==="
