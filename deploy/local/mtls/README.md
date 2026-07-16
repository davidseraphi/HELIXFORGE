# Local mTLS material (dev only)

```powershell
cd C:\Users\divin\PROJECTS\HELIXFORGE\deploy\local\mtls
powershell -File .\generate-dev-certs.ps1
```

Produces (gitignored if under `certs/`):

- `certs/ca.crt` / `ca.key` — dev CA  
- `certs/server.crt` / `server.key` — gateway/core server  
- `certs/client.crt` / `client.key` — optional client cert  

## Run gateway with TLS

```powershell
$env:HELIX_TLS_CERT_FILE = "$pwd\certs\server.crt"
$env:HELIX_TLS_KEY_FILE  = "$pwd\certs\server.key"
# optional client CA for mutual TLS:
$env:HELIX_TLS_CLIENT_CA = "$pwd\certs\ca.crt"
cargo run -p gateway
```

**Do not commit private keys.** Production uses cert-manager / SPIFFE.
