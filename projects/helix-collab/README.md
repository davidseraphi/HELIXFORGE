# HelixCollab

**Order:** 1 · **Tier:** standard

Real-time collaborative workspace

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-collab.*`
- Objects: MinIO bucket `helix-helix-collab`

## Local development

```powershell
# Core optional; Postgres required for durable docs
$env:HELIX_ENV="local"
$env:HELIX_ALLOW_DEV_HEADERS="1"
rustup run stable-x86_64-pc-windows-msvc cargo run -p helix_collab_api

# smoke
powershell -File scripts/helix_collab_smoke.ps1

# web editor :3101
cd projects/helix-collab/web
pnpm install
pnpm dev
```

Deep slice notes: `docs/DEEP_SLICE.md`

## HelixCore dependencies

| Service | Use |
|---------|-----|
| gateway | Public API edge |
| auth-adapter | Identity & sessions (Ory) |
| agent-hub | AI agents |
| vault-service | Secrets |
| billing-service | Usage metering |
| observability-service | Metrics / audit |

## Domain modules

See `backend/src/domain/` for hexagonal domain logic.
