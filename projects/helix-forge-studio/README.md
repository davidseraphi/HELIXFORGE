# HelixForge Studio

**Order:** 10 · **Tier:** standard

No-code/low-code AI app & internal tool builder

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-forge-studio.*`
- Objects: MinIO bucket `helix-helix-forge-studio`

## Local development

```bash
# from monorepo root
cargo run -p helix_forge_studio_api
cd projects/helix-forge-studio/web && pnpm dev
```

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
