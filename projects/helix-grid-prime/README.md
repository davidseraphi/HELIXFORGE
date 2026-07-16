# HelixGridPrime

**Order:** 19 · **Tier:** frontier

Autonomous smart energy systems & renewable optimization

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-grid-prime.*`
- Objects: MinIO bucket `helix-helix-grid-prime`

## Local development

```bash
# from monorepo root
cargo run -p helix_grid_prime_api
cd projects/helix-grid-prime/web && pnpm dev
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
