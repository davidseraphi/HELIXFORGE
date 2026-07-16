# HelixCode org code-signing material under ~/Desktop/.keys/helixforge (never in-repo).
# Creates a self-signed org code-signing cert for local/CI when none exists.
# Usage:
#   .\scripts\helix_code_org_codesign.ps1
#   .\scripts\helix_code_org_codesign.ps1 -Pack

param(
  [switch]$Pack,
  [string]$KeysRoot = ""
)

$ErrorActionPreference = "Stop"
if (-not $KeysRoot) {
  $KeysRoot = Join-Path $env:USERPROFILE "Desktop\.keys\helixforge\code-signing"
}
New-Item -ItemType Directory -Force -Path $KeysRoot | Out-Null

$pfxPath = Join-Path $KeysRoot "helix-code-org.pfx"
$passPath = Join-Path $KeysRoot "helix-code-org.password.txt"
$readme = Join-Path $KeysRoot "README.md"

if (-not (Test-Path $readme)) {
  @"
# HelixCode org code-signing (local)

- Place enterprise cert as ``helix-code-org.pfx`` here OR let the script generate a self-signed org cert.
- Password file: ``helix-code-org.password.txt`` (gitignored via .keys location).
- Never commit this directory.

Pack signed build:
``````powershell
.\scripts\helix_code_org_codesign.ps1 -Pack
``````
"@ | Set-Content -Path $readme -Encoding utf8
}

if (-not (Test-Path $pfxPath)) {
  Write-Host "Generating self-signed org code-signing certificate at $pfxPath"
  $plain = -join ((48..57 + 65..90 + 97..122 | Get-Random -Count 24 | ForEach-Object { [char]$_ }))
  Set-Content -Path $passPath -Value $plain -Encoding ascii -NoNewline
  $secure = ConvertTo-SecureString -String $plain -Force -AsPlainText
  $cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject "CN=HelixForge Org Code Signing, O=HelixForge, C=US" `
    -KeyAlgorithm RSA `
    -KeyLength 4096 `
    -HashAlgorithm SHA256 `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -NotAfter (Get-Date).AddYears(3)
  Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $secure | Out-Null
  # remove from store after export (file is source of truth)
  Remove-Item "Cert:\CurrentUser\My\$($cert.Thumbprint)" -Force -ErrorAction SilentlyContinue
  Write-Host "Created self-signed PFX (thumbprint was $($cert.Thumbprint))"
} else {
  Write-Host "Using existing PFX: $pfxPath"
  if (-not (Test-Path $passPath)) {
    throw "Password file missing: $passPath (create it with the PFX password)"
  }
}

$pass = (Get-Content $passPath -Raw).Trim()
$env:CSC_LINK = $pfxPath
$env:CSC_KEY_PASSWORD = $pass
$env:WIN_CSC_LINK = $pfxPath
Write-Host "CSC_LINK=$env:CSC_LINK"

if ($Pack) {
  $web = Join-Path (Split-Path -Parent $PSScriptRoot) "projects\helix-code\web"
  Set-Location $web
  if (-not (Test-Path (Join-Path $web "node_modules\electron-builder"))) {
    Write-Host "Installing web deps (electron-builder)..."
    pnpm install
    if ($LASTEXITCODE -ne 0) { throw "pnpm install failed in helix-code/web" }
  }
  # Ensure Electron binary path.txt exists (electron postinstall can be incomplete on Windows)
  $electronDir = Join-Path $web "node_modules\electron"
  $pathTxt = Join-Path $electronDir "path.txt"
  if (-not (Test-Path $pathTxt)) {
    Write-Host "electron path.txt missing - running electron install script..."
    pnpm exec node node_modules\electron\install.js
  }
  Write-Host "Running electron-builder (signed)..."
  # Prefer local binary over PATH-less pnpm exec on some Windows setups
  $eb = Join-Path $web "node_modules\.bin\electron-builder.cmd"
  if (Test-Path $eb) {
    & $eb --config electron-builder.yml --win dir
  } else {
    pnpm exec electron-builder --config electron-builder.yml --win dir
  }
  if ($LASTEXITCODE -ne 0) { throw "electron-builder failed" }
  $dist = Join-Path $web "dist-electron"
  Write-Host "Signed pack output under $dist"
  Get-ChildItem -Path $dist -Recurse -Filter "*.exe" -ErrorAction SilentlyContinue | ForEach-Object {
    try {
      $sig = Get-AuthenticodeSignature $_.FullName
      Write-Host "  signature $($_.Name): $($sig.Status) ($($sig.SignerCertificate.Subject))"
    } catch {
      Write-Host "  signature check skipped for $($_.Name): $_"
    }
  }
}

Write-Host "HELIX_CODE_ORG_CODESIGN_OK"
