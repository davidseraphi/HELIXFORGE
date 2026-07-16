# Publish HelixCode Electron update feed to MinIO (or local folder).
# Requires: electron pack already built under projects/helix-code/web/dist-electron
#           mc or aws CLI optional — copies into a feed directory for static hosting.
#
# Usage:
#   .\scripts\helix_code_publish_update.ps1
#   .\scripts\helix_code_publish_update.ps1 -FeedDir "C:\feeds\helix-code" -Version 0.1.0
#   $env:HELIX_CODE_UPDATE_URL = "https://cdn.example.com/helix-code"
#
# Client: set HELIX_CODE_UPDATE_URL to the feed base (directory with latest.yml).

param(
  [string]$FeedDir = "",
  [string]$Version = "0.1.0",
  [string]$Channel = "latest"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$web = Join-Path $Root "projects\helix-code\web"
$dist = Join-Path $web "dist-electron"
if (-not $FeedDir) {
  $FeedDir = Join-Path $Root ".data\helix-code\update-feed"
}
New-Item -ItemType Directory -Force -Path $FeedDir | Out-Null

if (-not (Test-Path $dist)) {
  Write-Host "No dist-electron — running org codesign pack..."
  & (Join-Path $Root "scripts\helix_code_org_codesign.ps1") -Pack
}

$exe = Get-ChildItem -Path $dist -Recurse -Filter "HelixCode.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $exe) { throw "HelixCode.exe not found under $dist" }

$destExe = Join-Path $FeedDir "HelixCode-$Version-win-x64.exe"
Copy-Item $exe.FullName $destExe -Force

# Minimal generic latest.yml for electron-updater (path is relative to feed URL)
$sha = (Get-FileHash $destExe -Algorithm SHA512).Hash.ToLower()
$size = (Get-Item $destExe).Length
$yml = @"
version: $Version
files:
  - url: HelixCode-$Version-win-x64.exe
    sha512: $sha
    size: $size
path: HelixCode-$Version-win-x64.exe
sha512: $sha
releaseDate: '$(Get-Date -Format o)'
"@
Set-Content -Path (Join-Path $FeedDir "$Channel.yml") -Value $yml -Encoding utf8
Write-Host "Feed written to $FeedDir"
Write-Host "Set HELIX_CODE_UPDATE_URL to a URL that serves this directory (e.g. MinIO public prefix)."
Write-Host "HELIX_CODE_UPDATE_FEED_OK"
