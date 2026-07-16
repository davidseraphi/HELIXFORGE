# HelixInsights

**Order:** 4 · **Tier:** standard

Predictive analytics & decision OS

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-insights.*`
- Objects: MinIO bucket `helix-helix-insights`

## Local development

```bash
# from monorepo root
cargo run -p helix_insights_api
cd projects/helix-insights/web && pnpm dev
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

Durable analytics domain lives in `helix_db` (`InsightsRepo`) + routes in `backend/src/main.rs`.

| Method | Path | Notes |
|--------|------|--------|
| GET/POST | `/v1/datasets` | List / create datasets (Postgres when available) |
| GET | `/v1/datasets/{id}` | Fetch one dataset |
| GET/POST | `/v1/datasets/{id}/metrics` | Metrics on a dataset |
| GET/POST | `/v1/metrics/{id}/points` | Time-series points (`limit` query on GET) |
| GET | `/v1/domain/status` | `durable` flag from db pool |

Auth (local): header `x-helix-dev-user: you@example.com`.
