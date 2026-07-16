# Build a HelixForge release package for the current platform.
# Usage: powershell -File scripts\package-release.ps1 [-SkipBuild]
param(
  [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Version = (Select-String -Path "$Root\services\gateway\Cargo.toml" -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1).Matches.Groups[1].Value
$OS = "windows"
$Arch = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "x86" }
$Pkg = "helixforge-$Version-$OS-$Arch"
$Out = "$Root\target\release\packages"
New-Item -ItemType Directory -Force -Path $Out | Out-Null
$Stage = Join-Path ([System.IO.Path]::GetTempPath()) $Pkg
$BinDir = Join-Path $Stage "bin"
$ScriptDir = Join-Path $Stage "scripts"
$DeployDir = Join-Path $Stage "deploy\local"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $BinDir, $ScriptDir, $DeployDir | Out-Null

if (-not $SkipBuild) {
  Write-Host "[package] building release binaries..."
  cargo build --release --workspace
}

# Copy service binaries
$crates = @("gateway", "agent_hub", "vault_service", "billing_service", "observability_service", "auth_adapter")
foreach ($c in $crates) {
  $src = "$Root\target\release\$c.exe"
  if (Test-Path $src) { Copy-Item $src $BinDir }
}

# Copy product API binaries
Get-ChildItem "$Root\target\release\helix_*_api.exe" -ErrorAction SilentlyContinue |
  ForEach-Object { Copy-Item $_.FullName $BinDir }

Copy-Item "$Root\docker-compose.yml" $Stage
Copy-Item "$Root\deploy\local\*" $DeployDir -Recurse -Force
Copy-Item "$Root\scripts\install.ps1" $ScriptDir
Copy-Item "$Root\scripts\migrate-export.ps1" $ScriptDir
Copy-Item "$Root\scripts\migrate-restore.ps1" $ScriptDir
if (Test-Path "$Root\README.md") { Copy-Item "$Root\README.md" $Stage }

@"
# HelixForge $Version install

1. Load secrets from your key directory (not included in this package).
2. Run the installer for your platform:
   - Windows:     .\scripts\install.ps1
   - Linux/macOS: ./scripts/install.sh
3. Start core services with scripts/dev-core.* or run individual binaries from bin/.
"@ | Set-Content (Join-Path $Stage "INSTALL.md")

$Zip = "$Out\$Pkg.zip"
Compress-Archive -Path "$Stage\*" -DestinationPath $Zip -Force
$Hash = (Get-FileHash $Zip -Algorithm SHA256).Hash
"$Hash" | Set-Content "$Zip.sha256"

Write-Host "[package] created $Zip"
Write-Host "[package] sha256: $Hash"
