# HelixNetwork

**Order:** 9 · **Tier:** standard

AI professional networking & opportunity engine

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-network.*`
- Objects: MinIO bucket `helix-helix-network`

## Local development

```bash
# from monorepo root
cargo run -p helix_network_api
cd projects/helix-network/web && pnpm dev
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

Durable networking domain lives in `helix_db` (`NetworkRepo`) + routes in `backend/src/main.rs`.

| Method | Path | Notes |
|--------|------|--------|
| GET/POST | `/v1/profiles` | List / create profile (one per user) |
| GET | `/v1/profiles/me` | Caller's profile |
| GET | `/v1/profiles/{id}` | Fetch profile |
| GET/POST | `/v1/connections` | List / request connection |
| POST | `/v1/connections/{id}/accept` | Accept (must be target profile) |
| GET/POST | `/v1/opportunities` | List / post opportunities |
| GET | `/v1/opportunities/{id}` | Fetch opportunity |
| GET | `/v1/domain/status` | `durable` flag |

Auth (local): header `x-helix-dev-user: you@example.com`.
