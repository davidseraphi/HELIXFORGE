# HelixClimatePrime

**Order:** 15 · **Tier:** frontier

Planetary-scale climate risk modeling & net-zero orchestration

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-climate-prime.*`
- Objects: MinIO bucket `helix-helix-climate-prime`

## Local development

```bash
# from monorepo root
cargo run -p helix_climate_prime_api
cd projects/helix-climate-prime/web && pnpm dev
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
