# Build HelixCode CI Docker image (git + cargo).
# Usage (from monorepo root):
#   .\projects\helix-code\docker\build-ci-image.ps1
#   .\projects\helix-code\docker\build-ci-image.ps1 -Tag helixforge/helix-code-ci:local

param(
  [string]$Tag = "helixforge/helix-code-ci:local"
)

$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$dockerfile = Join-Path $here "Dockerfile.ci"

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
  throw "docker not on PATH"
}

Write-Host "Building $Tag from $dockerfile"
docker build -f $dockerfile -t $Tag $here
if ($LASTEXITCODE -ne 0) { throw "docker build failed" }

Write-Host "Probing image tools..."
docker run --rm $Tag sh -c "git --version && rustc --version && cargo --version && echo OK"
if ($LASTEXITCODE -ne 0) { throw "image probe failed" }

Write-Host "HELIX_CODE_CI_IMAGE_OK tag=$Tag"
Write-Host "Set: `$env:HELIX_CODE_DOCKER_IMAGE='$Tag'; `$env:HELIX_CODE_ISOLATION='docker'"
