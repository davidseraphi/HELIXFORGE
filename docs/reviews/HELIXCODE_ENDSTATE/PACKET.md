# HELIXCODE_ENDSTATE review packet

**Scope:** Close gaps 1–9 (collab, CI fleet, multi-LSP, agents, MLS devices, UI surfaces, Electron pack, quotas/ops, self-audit).

## Prove commands

```powershell
$env:HELIX_ENV="local"; $env:HELIX_ALLOW_DEV_HEADERS="1"
$env:DATABASE_URL="postgres://helix:helix@127.0.0.1:55432/helixforge"
rustup run stable-x86_64-pc-windows-msvc cargo test -p helix_code_api -- --test-threads=1
rustup run stable-x86_64-pc-windows-msvc cargo run -p helix_code_api
# other window:
.\scripts\helix_code_smoke.ps1
.\scripts\helix_code_endstate_smoke.ps1
```

## Acceptance matrix

See `SELF_AUDIT_REPORT.md`.
