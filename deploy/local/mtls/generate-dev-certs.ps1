# Generate a local dev CA + server/client certs for HelixCore mTLS experiments.
# Requires: openssl on PATH
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$out = Join-Path $here "certs"
New-Item -ItemType Directory -Force -Path $out | Out-Null

function Need-OpenSsl {
  $o = Get-Command openssl -ErrorAction SilentlyContinue
  if (-not $o) { throw "openssl not found on PATH" }
}

Need-OpenSsl
Push-Location $out
try {
  openssl genrsa -out ca.key 4096 2>$null
  openssl req -x509 -new -nodes -key ca.key -sha256 -days 825 -out ca.crt -subj "/CN=HelixForge Dev CA/O=HelixForge"
  openssl genrsa -out server.key 2048 2>$null
  openssl req -new -key server.key -out server.csr -subj "/CN=localhost/O=HelixForge"
  @"
basicConstraints=CA:FALSE
keyUsage=digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:localhost,DNS:gateway,IP:127.0.0.1
"@ | Set-Content -Encoding ascii server.ext
  openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 825 -sha256 -extfile server.ext
  openssl genrsa -out client.key 2048 2>$null
  openssl req -new -key client.key -out client.csr -subj "/CN=helix-client/O=HelixForge"
  @"
basicConstraints=CA:FALSE
keyUsage=digitalSignature
extendedKeyUsage=clientAuth
"@ | Set-Content -Encoding ascii client.ext
  openssl x509 -req -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out client.crt -days 825 -sha256 -extfile client.ext
  Write-Host "Wrote certs under $out"
} finally {
  Pop-Location
}
