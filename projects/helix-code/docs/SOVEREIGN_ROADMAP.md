# HelixCode — sovereign extreme roadmap

**Product:** HelixCode (`helix-code`, port **8102**)  
**Posture:** Sovereign **code forge** + web workspaces — **not** underscoped Monaco demo.  
**Sister project:** native from-scratch IDE = standalone **HELIXANVIL** (not this product).

## Extreme definition (ratified)

| Plane | Target | Identity rule |
|-------|--------|----------------|
| **Git object plane** | Bare repos + **gitoxide (`gix`) reads**; system `git` for init/commit | Own metadata in Postgres; objects on forge storage |
| **Smart HTTP plane** | `info/refs` + `git-upload-pack` / `git-receive-pack` | Real clone/push from `git` CLI clients |
| **Workspace plane** | Multi-file workspaces (**Monaco E1** → Code-OSS-class over time) | Web primary for forge; Anvil is separate native kernel |
| **CI / runners** | In-forge pipelines + job runners | Not “webhook to GitHub Actions only” |
| **LSP plane** | Language servers as forge sidecars | Diagnostics into workspace, not fake complete stubs |
| **Agent sandbox** | Multi-agent jobs with audit + isolation | agent-hub + sealed working trees |
| **Sealed objects** | Content-addressed sealed blobs (MLS/envelope path later) | MinIO + Postgres index; no cleartext secrets in git |

## Phases

| Phase | Name | Ships |
|-------|------|--------|
| **E0** | Git forge foundation | Bare repo create, smart HTTP, file tree API, seed commit, smoke |
| **E1** | Workspace + editor + gix | **gix** object reads; Monaco multi-file open/save/commit; branch field |
| **E2** | CI runners | Worktree sandbox + allowlist, timeout, MinIO artifacts (**done**) |
| **E3** | LSP | rust-analyzer JSON-RPC bridge + Monaco markers (**done**) |
| **E4** | Agent sandbox | Multi-agent mesh, worktree patches, commit, audit (**done**) |
| **E5** | Sealed / MLS objects | HVA4 + crypto groups (group DEK wrap), no cleartext MinIO (**done**) |
| **H1** | OpenMLS multi-tenant | RFC 9420 groups, durable blobs, mls-sealed export-DEK (**done**) |
| **H2** | Container isolation | host/docker/auto for CI + agent probe (**done**) |
| **H3** | Deeper Monaco UX | completion, definition, CI/agents/MLS panels (**done**) |
| **H4** | Code-OSS depth | Multi-tab shell, palette, quick open, search, batch commit (**done**) |
| **H5** | Split + Electron | Dual editor groups + Electron desktop shell (**done**) |
| **H6** | CI Docker image | `helixforge/helix-code-ci:local` with git + cargo (**done**) |
| **ES1–ES9** | End-state gaps 1–9 | Collab, CI fleet, multi-LSP, agents, MLS devices, UI APIs, Electron pack, quotas, self-audit (**done**) |
| **Anvil** | Native IDE kernel | **out of product** → standalone HELIXANVIL |

## Non-goals (this product)

- Replacing **HELIXANVIL** native kernel  
- Full GitHub feature parity day one  
- Soft-stub endpoints labeled “done” without tests/smoke  

## Prove bar

Every phase: unit/integration tests + `scripts/helix_code_smoke.ps1` green against local Postgres + running `helix_code_api`.
