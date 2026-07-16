# Enterprise code signing (OV/EV)

Local bootstrap uses a **self-signed** org cert under:

`%USERPROFILE%\Desktop\.keys\helixforge\code-signing\`

## Swap to enterprise OV/EV (Windows)

1. Obtain an OV or EV Code Signing certificate as `.pfx` from your CA.
2. Replace files (never commit):

```
Desktop/.keys/helixforge/code-signing/helix-code-org.pfx
Desktop/.keys/helixforge/code-signing/helix-code-org.password.txt
```

3. Pack:

```powershell
.\scripts\helix_code_org_codesign.ps1 -Pack
```

4. Verify:

```powershell
Get-AuthenticodeSignature projects\helix-code\web\dist-electron\win-unpacked\HelixCode.exe
```

EV certificates typically require hardware token / cloud HSM — configure `CSC_LINK` / CI secrets accordingly (same env vars as electron-builder).

## CI secrets (never in repo)

| Secret | Purpose |
|--------|---------|
| `CSC_LINK` | Path or base64 of PFX |
| `CSC_KEY_PASSWORD` | PFX password |
| `WIN_CSC_LINK` | Optional Windows-only override |

## macOS / Apple

- Developer ID Application cert + notarization (`APPLE_ID`, `APPLE_APP_SPECIFIC_PASSWORD`, `APPLE_TEAM_ID`).
