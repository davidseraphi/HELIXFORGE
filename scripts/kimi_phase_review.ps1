#Requires -Version 5.1
<#
.SYNOPSIS
  Run a Kimi CLI phase review for HelixForge (non-interactive print mode).

.PARAMETER Phase
  Phase id: 0, A, B, C, D, E, F, G

.PARAMETER SkipInvoke
  Only write/refresh KIMI_PROMPT.md; do not call kimi.exe
#>
param(
  [Parameter(Mandatory = $true)]
  [string]$Phase,

  [switch]$SkipInvoke
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$phaseKey = $Phase.Trim()
if ($phaseKey -ne "0") {
  $phaseKey = $phaseKey.ToUpperInvariant()
}

$dir = Join-Path $Root "docs\reviews\phases\$phaseKey"
New-Item -ItemType Directory -Force -Path $dir | Out-Null

$packetPath = Join-Path $dir "PACKET.md"
$promptPath = Join-Path $dir "KIMI_PROMPT.md"
$reportPath = Join-Path $dir "KIMI_REPORT.md"

if (-not (Test-Path $packetPath)) {
  $lines = @(
    "# Phase $phaseKey implementer packet",
    "",
    "**Status:** draft",
    "**Date:** $(Get-Date -Format 'yyyy-MM-dd')",
    "**Repo:** HELIXFORGE",
    "**Goal:** HELIXCORE-0-AG",
    "",
    "## Intent",
    "",
    "(What this phase was supposed to achieve.)",
    "",
    "## Changes",
    "",
    "| Path | Why |",
    "|------|-----|",
    "| | |",
    "",
    "## Commands run (evidence)",
    "",
    '```text',
    "# paste cargo test / smoke output summaries",
    '```',
    "",
    "## Acceptance map",
    "",
    "| Criterion | Met? | Evidence |",
    "|-----------|------|----------|",
    "| | | |",
    "",
    "## Reasoning and trade-offs",
    "",
    "(Assumptions, alternatives rejected, residual risk.)",
    "",
    "## Known debt / follow-ups",
    "",
    "*",
    "",
    "## Request to Kimi",
    "",
    "Review this packet and the files listed above. Verdict: PASS / PASS_WITH_FOLLOWUPS / FAIL / BLOCKED."
  )
  Set-Content -Path $packetPath -Value $lines -Encoding utf8
  Write-Host "Created template PACKET.md at $packetPath"
  if ($SkipInvoke) { exit 0 }
}

$packetBody = Get-Content -Raw -Path $packetPath

# Build prompt without nested here-strings that break PowerShell parsing
$promptLines = @(
  "You are the independent reviewer for HelixForge HelixCore phase work.",
  "",
  "CONTEXT",
  "* Repo work-dir: $Root",
  "* Phase: $phaseKey",
  "* Program goal: finish Phase 0 plus A through G of HelixCore deep foundation",
  "* Spec: docs/features/010-helix-core-deep/requirements.md",
  "* Bootstrap (Phase 0 basis): docs/features/000-helix-core-bootstrap/",
  "* Constitution: constitution.md",
  "* Transcripts: C:\Users\divin\TRANSCRIPTS\HELIXFORGE\",
  "",
  "YOUR JOB",
  "1. Read the implementer PACKET below.",
  "2. Inspect the listed files and related tests in the work-dir.",
  "3. Critique correctness, security, multi-tenant isolation, overclaim risk, missing tests, and implementer reasoning quality.",
  "4. Produce a structured report with:",
  "   * Verdict: PASS | PASS_WITH_FOLLOWUPS | FAIL | BLOCKED",
  "   * Summary (5 to 10 lines)",
  "   * Findings: severity (blocker|major|minor|nit), file, description, suggested fix",
  "   * Reasoning audit: what the implementer got right or wrong",
  "   * Required retest commands",
  "",
  "Do NOT implement fixes. Review only.",
  "",
  "IMPLEMENTER PACKET",
  "-----",
  $packetBody
)
$prompt = $promptLines -join "`n"
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
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"

Write-Host "Invoking Kimi CLI (print mode) for phase $phaseKey ..."
$kimiArgs = @(
  "--print",
  "--final-message-only",
  "--yolo",
  "--work-dir", $Root,
  "--max-steps-per-turn", "40",
  "--prompt", $prompt
)

$report = & kimi.exe @kimiArgs 2>&1 | Out-String
$header = @(
  "# Kimi phase review: $phaseKey",
  "",
  "**Generated:** $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')",
  "**Command:** kimi --print --final-message-only --yolo --work-dir HELIXFORGE",
  "",
  "---",
  ""
) -join "`n"

Set-Content -Path $reportPath -Value ($header + $report) -Encoding utf8
Write-Host "Wrote $reportPath"
Write-Host "Next: triage findings into TRIAGE.md; only then mark phase done in status.json"
