# HelixCode end-state self-audit

**Date:** 2026-07-15  
**Verdict:** **SELF_AUDIT_PASS** (implementation + automated smoke + residual wave)  
**External Kimi:** see `KIMI_REPORT.md` when generated via `scripts/kimi_helixcode_endstate_review.ps1`

## Gap evidence

| # | Gap | Evidence |
|---|-----|----------|
| 1 | Git collab | Migration `0029`; routes issues/PRs/protections/webhooks/ACL/branches; `GitStore::merge_branch`; smoke PR merge + protect deny |
| 2 | CI fleet | `list_pipeline_runs`, cancel, artifact content, runners heartbeat; matrix columns in schema; meters on runs |
| 3 | Multi-lang LSP | `GET /v1/lsp/servers` + `list_language_servers()` |
| 4 | Agents depth | list jobs, events table/API, cancel |
| 5 | OpenMLS product | devices, key-backup, list groups |
| 6 | UI surfaces | Terminal create/write, settings, git status/diff, extensions registry, debug launch |
| 7 | Electron product | shell + `electron-builder.yml` + **org-signed pack** via `.keys` |
| 8 | Ops | `tenant_quotas`, quota checks on repo/pipeline, metering commits, THREAT_MODEL + BACKUP_RESTORE |
| 9 | Review gate | This packet + PROJECT_STATE; external Kimi CLI script |

## Residuals closed (follow-up wave)

| Residual | Evidence |
|----------|----------|
| Deploy keys | `0030_code_residuals.sql`, `POST/GET /v1/repos/{id}/deploy-keys`, smart HTTP `x-helix-deploy-key`, smoke auth + revoke |
| Sticky LSP | `code.lsp_session_registry`, `instance_id` on open/status, sticky_miss error on wrong instance |
| **Full DAP lldb/gdb** | `dap_client.rs` Content-Length DAP: initialize/launch/setBreakpoints/configurationDone/continue/next/stepIn/stepOut/pause/threads/stackTrace/scopes/variables/evaluate/disconnect; HTTP under `/v1/debug/*`; probes `lldb-dap` then `gdb --interpreter=dap` (skips Windows gdb builds without dap UI); PATH/PYTHONHOME for LLVM liblldb |
| **Org code-signing** | `scripts/helix_code_org_codesign.ps1` → `%USERPROFILE%\Desktop\.keys\helixforge\code-signing\helix-code-org.pfx`; `-Pack` signs `HelixCode.exe` with CSC_* |
| Web activity tabs | PR/collab, terminal, debug, extensions, settings (+ quotas) in Code-OSS shell |
| External review | `scripts/kimi_helixcode_endstate_review.ps1` → `KIMI_REPORT.md` |

## Host prerequisites (debug)

| Adapter | Notes |
|---------|--------|
| **lldb-dap** (preferred) | LLVM package (`scoop install llvm`). Windows `liblldb.dll` needs **Python 3.11** (`python311.dll` on PATH or beside `lldb-dap.exe`). |
| **gdb --interpreter=dap** | Only builds compiled with the DAP UI. Scoop `gdb` 17.1 from nuwen **does not** include dap (`Interpreter dap unrecognized`) — client detects and skips. |
| Override | `HELIX_CODE_DAP_COMMAND="C:\path\to\lldb-dap.exe"` |

## Remaining product polish (non-blocking)

- Production OV/EV certs replace self-signed PFX in org secret store (path unchanged).
- Multi-region: clients must honor `instance_id` / sticky_miss (HA notes).
- Anvil native IDE remains a separate portfolio project.

## Commands run

```
scripts/helix_code_org_codesign.ps1          # bootstrap org PFX
scripts/helix_code_org_codesign.ps1 -Pack    # signed Electron dir pack
# lldb-dap initialize smoke (local host)
scripts/helix_code_endstate_smoke.ps1        # includes deploy key + full DAP API surface soft probes
scripts/kimi_helixcode_endstate_review.ps1
```
