# Enable MinIO bucket versioning for PITR-ish object recovery (requires mc).
param(
  [string]$Endpoint = "http://127.0.0.1:9000",
  [string]$AccessKey = "helixminio",
  [string]$SecretKey = "helixminio_secret",
  [string]$Bucket = "helixforge"
)
$ErrorActionPreference = "Stop"
if (-not (Get-Command mc -ErrorAction SilentlyContinue)) {
  throw "mc (MinIO client) required on PATH"
}
mc alias set helixlocal $Endpoint $AccessKey $SecretKey | Out-Null
mc version enable "helixlocal/$Bucket"
Write-Host "Versioning enabled on $Bucket"
mc version info "helixlocal/$Bucket"
