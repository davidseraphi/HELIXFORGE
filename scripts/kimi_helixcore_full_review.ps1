#Requires -Version 5.1
param(
  [switch]$SkipInvoke
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$outDir = Join-Path $Root "docs\reviews\HELIXCORE_FULL"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$promptPath = Join-Path $outDir "KIMI_PROMPT.md"
$reportPath = Join-Path $outDir "KIMI_REPORT.md"

$prompt = @"
You are an independent senior reviewer for the HelixForge monorepo.

GOAL HELIXCORE-FULL: HelixCore must be a fully built sovereign foundation for 20 product forges.
Repo: $Root

Read first:
docs/goals/HELIXCORE_FULL.md
docs/features/010-helix-core-deep/requirements.md
docs/features/000-helix-core-bootstrap/requirements.md
constitution.md
VISION.md
AGENTS.md
BUILD_SPEC.md
crates/service-kit
crates/shared-core
crates/helix-db
crates/auth-client
crates/audit-log
crates/agent-framework
crates/vault-client
crates/billing-client
crates/observability
crates/nats-client
services/gateway
services/auth-adapter
services/agent-hub
services/vault-service
services/billing-service
services/observability-service
docker-compose.yml
infra/helm
infra/argocd
infra/terraform

YOUR JOB (review only do not implement fixes):
1. Assess how complete HelixCore is vs HELIXCORE-FULL definition of done.
2. Separate DONE PARTIAL MISSING for each capability: AetherID agents vault billing audit observability gateway data plane infra.
3. Call out overclaims security risks sovereignty gaps missing tests.
4. Prioritize the top work stream to finish HelixCore fully.
5. Produce a structured report with:
   Verdict: PASS or PASS_WITH_FOLLOWUPS or FAIL or NOT_COMPLETE
   Executive summary
   Capability matrix with evidence paths
   Findings with severity path issue fix suggestion
   Recommended build order to reach FULL
   Retest commands

Be harsh about honesty. Scaffold is not done.
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
Write-Host "Invoking Kimi full HelixCore review..."

$kimiArgs = @(
  "--print"
  "--final-message-only"
  "--yolo"
  "--work-dir"
  $Root
  "--max-steps-per-turn"
  "60"
  "--prompt"
  $prompt
)

$report = & kimi.exe @kimiArgs 2>&1 | Out-String
$stamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$header = "# Kimi full HelixCore review`n`n**Goal:** HELIXCORE-FULL`n**Generated:** $stamp`n`n---`n`n"
Set-Content -Path $reportPath -Value ($header + $report) -Encoding utf8
Write-Host "Wrote $reportPath"
