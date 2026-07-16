# HelixCode

**Order:** 2 · **Tier:** standard · **Port:** 8102  
**Posture:** sovereign **code forge** (extreme) — not a Monaco-only demo.

AI-native collaborative **forge**: git hosting, workspaces, in-forge CI, agents, sealed objects.  
Native from-scratch IDE kernel lives in sibling project **HELIXANVIL**, not here.

## Extreme roadmap

See [`docs/SOVEREIGN_ROADMAP.md`](docs/SOVEREIGN_ROADMAP.md).

| Phase | Status |
|-------|--------|
| **E0** Git forge foundation | **done** — bare repos, tree/blob/commit, smart HTTP, workspaces, pipeline runs, sealed MinIO |
| **E1** Workspace + editor | **done** — Monaco web on :3102; **gitoxide (`gix`)** object reads |
| **E2** CI runners | **done** — worktree sandbox allowlist, timeout, MinIO artifacts |
| **E3** LSP | **done** — rust-analyzer bridge; Monaco diagnostics/hover |
| **E4** Agent mesh | **done** — worktree patches, multi-agent, commit, audit |
| **E5** Sealed crypto | **done** — HVA4 + crypto groups; ciphertext-only MinIO |
| **H4** Code-OSS depth | **done** — multi-tab shell, palette, quick open, search, batch commit |
| **H5** Split + Electron | **done** — dual groups + Electron desktop chrome |
| **H6** CI Docker image | **done** — `helixforge/helix-code-ci:local` (git + cargo) |

## Architecture

- Backend: Rust Axum via `service_kit` + `helix_db`
- Git: bare repos under `HELIX_CODE_REPO_ROOT` (default `.data/helix-code/repos`)
- Dual plane: **gix** object reads + system `git` for writes/packs/smart HTTP
- Web: Next.js 15 + Monaco **Code-OSS shell** — `pnpm --filter @helixforge/helix-code-web dev` (:3102)
- Desktop: Electron loads the web UI (`pnpm --filter @helixforge/helix-code-web electron:dev` waits for :3102)
  - If `electron` fails with “failed to install correctly”, re-run `pnpm --filter @helixforge/helix-code-web rebuild electron` (or extract the cached zip under `%LOCALAPPDATA%\electron\Cache` into `node_modules/.../electron/dist`).
- CI image: `.\projects\helix-code\docker\build-ci-image.ps1` → set `HELIX_CODE_DOCKER_IMAGE=helixforge/helix-code-ci:local`
- Data: Postgres (`code.*` migrations including `0024_code_extreme.sql`)
- Objects: MinIO for sealed blobs
- Events: NATS subjects `helix.helix-code.*` (product prefix)

## Local development

```powershell
# MSVC toolchain (Windows)
$env:HELIX_ENV = "local"
$env:HELIX_ALLOW_DEV_HEADERS = "1"
$env:DATABASE_URL = "postgres://helix:helix@127.0.0.1:55432/helixforge"
# optional: $env:HELIX_CODE_REPO_ROOT = "D:\data\helix-code\repos"

rustup run stable-x86_64-pc-windows-msvc cargo run -p helix_code_api
# user-owned terminal — do not background from agent

# smoke
.\scripts\helix_code_smoke.ps1
```

```bash
# web workspace (user-owned terminal)
cd projects/helix-code/web && pnpm install && pnpm dev   # http://127.0.0.1:3102

cargo test -p helix_code_api   # prefers MSVC on Windows
```

## Key APIs (E0)

| Method | Path | Notes |
|--------|------|--------|
| GET | `/v1/domain/status` | phase + plane flags |
| POST/GET | `/v1/repos` | create bare + seed README |
| GET | `/v1/repos/{id}/tree` | `?rev=&path=` |
| GET | `/v1/repos/{id}/blob` | file content |
| POST | `/v1/repos/{id}/commits` | write file + commit |
| GET | `/v1/repos/{id}/log` | commit history |
| GET | `/v1/git/{name}/info/refs` | smart HTTP |
| POST | `/v1/git/{name}/git-upload-pack` | clone/fetch |
| POST | `/v1/git/{name}/git-receive-pack` | push |
| POST | `/v1/code/workspaces` | forge workspace records |
| POST | `/v1/repos/{id}/pipelines` + `/v1/pipelines/{id}/runs` | CI |
| POST | `/v1/repos/{id}/sealed-objects` | MinIO + index |

## HelixCore dependencies

| Service | Use |
|---------|-----|
| gateway | Public edge `/p/helix-code` |
| auth-adapter | Identity |
| agent-hub | Future E4 mesh |
| vault-service / MinIO | Sealed objects |
| billing-service | `repos.created` meter |
| observability-service | Metrics / audit chain |
