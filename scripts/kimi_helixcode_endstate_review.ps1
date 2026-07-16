#Requires -Version 5.1
param(
  [switch]$SkipInvoke
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$outDir = Join-Path $Root "docs\reviews\HELIXCODE_ENDSTATE"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$promptPath = Join-Path $outDir "KIMI_PROMPT.md"
$reportPath = Join-Path $outDir "KIMI_REPORT.md"

$prompt = @"
You are an independent senior reviewer for HelixCode (HelixForge monorepo product).

GOAL: HELIXCODE end-state + residuals — fully deep sovereign code forge (not Anvil native IDE).
Repo: $Root

Read first (required):
- projects/helix-code/docs/SOVEREIGN_ROADMAP.md
- projects/helix-code/docs/THREAT_MODEL.md
- projects/helix-code/docs/BACKUP_RESTORE.md
- projects/helix-code/docs/ELECTRON_PACKAGING.md
- docs/reviews/HELIXCODE_ENDSTATE/SELF_AUDIT_REPORT.md
- docs/reviews/HELIXCODE_ENDSTATE/PACKET.md
- AGENTS.md
- constitution.md (if present)
- crates/helix-db/migrations/0024_code_extreme.sql through 0030_code_residuals.sql
- crates/helix-db/src/code.rs, code_endstate.rs, code_residuals.rs
- projects/helix-code/backend/src/domain/ (especially smart_http.rs, dap_client.rs, collab_api.rs, endstate_api.rs, lsp_bridge.rs)
- projects/helix-code/web/src/app/page.tsx
- scripts/helix_code_smoke.ps1
- scripts/helix_code_endstate_smoke.ps1

YOUR JOB (review only — do not implement fixes):
1. Verdict on completeness vs end-state gaps 1-9 + residuals (deploy keys, sticky LSP, DAP lldb/gdb, org code-signing, web panels).
2. Separate DONE / PARTIAL / MISSING with evidence paths.
3. Security: deploy key hashing, branch protection, terminal allowlist, webhook HMAC, MLS backup opacity, secrets under Desktop/.keys not in-repo.
4. Overclaim risk (self-audit vs external).
5. Structured report:
   Verdict: PASS | PASS_WITH_FOLLOWUPS | FAIL | NOT_COMPLETE
   Executive summary
   Gap matrix 1-9 + residuals
   Findings (severity, path, issue, fix)
   Retest commands

Be harsh. Prototype scaffolding is not production-complete.
Do not use emoji characters in the report (ASCII/markdown only) so Windows console codecs do not fail.
"@

Set-Content -Path $promptPath -Value $prompt -Encoding utf8
Write-Host "Wrote $promptPath"

if ($SkipInvoke) {
  Write-Host "SkipInvoke set; not calling kimi."
  exit 0
}

if (-not (Get-Command kimi.exe -ErrorAction SilentlyContinue)) {
  Write-Error "kimi.exe not on PATH."
}

$env:PYTHONIOENCODING = "utf-8"
$env:PYTHONUTF8 = "1"
$env:PYTHONLEGACYWINDOWSSTDIO = "0"
# Avoid Windows charmap crashes on emoji in Kimi output
try { [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new() } catch {}
Write-Host "Invoking Kimi HelixCode end-state review (this can take several minutes)..."

# kimi may write progress to stderr; do not treat that as terminating under Stop.
$prevEap = $ErrorActionPreference
$ErrorActionPreference = "Continue"

$kimiArgs = @(
  "--print"
  "--final-message-only"
  "--yolo"
  "--afk"
  "--work-dir"
  $Root
  "--prompt"
  $prompt
)

$raw = & kimi.exe @kimiArgs 2>&1
$exitCode = $LASTEXITCODE
$ErrorActionPreference = $prevEap

# Flatten PowerShell error records + strings
$lines = foreach ($item in @($raw)) {
  if ($null -eq $item) { continue }
  if ($item -is [System.Management.Automation.ErrorRecord]) {
    $item.ToString()
  } else {
    "$item"
  }
}
$report = ($lines -join "`n")

$stamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$header = @"
# Kimi HelixCode end-state review

**Goal:** HELIXCODE-ENDSTATE + residuals  
**Generated:** $stamp  
**Command:** kimi --print --final-message-only --yolo --afk --work-dir HELIXFORGE  
**kimi_exit:** $exitCode

---

"@
Set-Content -Path $reportPath -Value ($header + $report) -Encoding utf8
Write-Host "Wrote $reportPath (bytes=$((Get-Item $reportPath).Length) exit=$exitCode)"
if ($report -match "Verdict:\s*(PASS|PASS_WITH_FOLLOWUPS|FAIL|NOT_COMPLETE)") {
  Write-Host "Detected verdict line in report."
} elseif ($report -match "HELLO_KIMI|error|Error|unauthorized|login") {
  Write-Host "Report written; check content for auth/errors."
}
# Non-zero kimi exit is still a written report for inspection
exit 0
