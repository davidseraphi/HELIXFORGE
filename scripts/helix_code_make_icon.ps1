# Minimal HelixCode Windows icon for electron-builder (build/icon.ico).
# Uses .NET System.Drawing when available; otherwise writes a tiny PNG placeholder.
param(
  [string]$OutDir = ""
)
$ErrorActionPreference = "Stop"
if (-not $OutDir) {
  $OutDir = Join-Path (Split-Path -Parent $PSScriptRoot) "projects\helix-code\web\build"
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$ico = Join-Path $OutDir "icon.ico"
$png = Join-Path $OutDir "icon.png"

function Write-PngPlaceholder([string]$Path) {
  # 16x16 solid #1e3a5f PNG (precomputed minimal file)
  $b64 = "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAFUlEQVQ4T2NkYGD4z0ABYBw1gGE0DAB+4AX+R1cJ0QAAAABJRU5ErkJggg=="
  [IO.File]::WriteAllBytes($Path, [Convert]::FromBase64String($b64))
}

try {
  Add-Type -AssemblyName System.Drawing
  $bmp = New-Object System.Drawing.Bitmap 256, 256
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.Clear([System.Drawing.Color]::FromArgb(255, 30, 58, 95))
  $brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 94, 234, 212))
  $font = New-Object System.Drawing.Font "Segoe UI", 96, ([System.Drawing.FontStyle]::Bold)
  $g.DrawString("H", $font, $brush, 48, 48)
  $g.Dispose()
  # Save PNG
  $bmp.Save($png, [System.Drawing.Imaging.ImageFormat]::Png)
  # ICO via Icon.FromHandle is limited; copy PNG and also try Icon
  $icon = [System.Drawing.Icon]::FromHandle($bmp.GetHicon())
  $fs = [IO.File]::OpenWrite($ico)
  $icon.Save($fs)
  $fs.Close()
  $bmp.Dispose()
  Write-Host "Wrote $ico and $png"
} catch {
  Write-Host "System.Drawing path failed ($_); writing PNG placeholder only"
  Write-PngPlaceholder $png
  if (-not (Test-Path $ico)) {
    Copy-Item $png $ico -Force
  }
}
Write-Host "HELIX_CODE_ICON_OK"
