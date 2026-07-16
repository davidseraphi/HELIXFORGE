# HelixWell

**Order:** 8 · **Tier:** standard

AI personal & team wellness platform

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-well.*`
- Objects: MinIO bucket `helix-helix-well`

## Local development

```bash
# from monorepo root
cargo run -p helix_well_api
cd projects/helix-well/web && pnpm dev
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

Durable wellness domain lives in `helix_db` (`WellRepo`) + routes in `backend/src/main.rs`.

| Method | Path | Notes |
|--------|------|--------|
| GET/POST | `/v1/habits` | List (`?mine=true`) / create habits |
| GET | `/v1/habits/{id}` | Fetch habit |
| GET/POST | `/v1/habits/{id}/logs` | Habit completion logs |
| GET/POST | `/v1/checkins` | Mood/energy check-ins (`mood`/`energy` 1..=10) |
| GET | `/v1/domain/status` | `durable` flag |

Auth (local): header `x-helix-dev-user: you@example.com`.
