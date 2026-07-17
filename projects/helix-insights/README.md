# HelixInsights

**Order:** 4 · **Tier:** standard

Predictive analytics & decision OS.

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.insights.*`
- Objects: MinIO bucket `helix-helix-insights`

## Local development

```bash
# from monorepo root
cargo run -p helix_insights_api
cd projects/helix-insights/web && pnpm dev
```

Local dev requires the HelixCore data plane:

```bash
docker compose up -d postgres nats minio minio-init
DATABASE_URL=postgres://helix:helix@127.0.0.1:55432/helixforge cargo run -p helix_db --bin helix-migrate
```

Run the smoke test against a local API:

```powershell
# in another terminal, with HELIX_ALLOW_DEV_HEADERS=1
cargo run -p helix_insights_api
pwsh -File scripts/helix_insights_smoke.ps1
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
|--------|------|-------|
| GET/POST | `/v1/datasets` | List / create datasets (Postgres when available) |
| GET/DELETE | `/v1/datasets/{id}` | Fetch / soft-delete one dataset |
| GET/POST | `/v1/datasets/{id}/metrics` | Metrics scoped to a dataset |
| GET | `/v1/metrics` | All metrics for the tenant |
| GET/DELETE | `/v1/metrics/{id}` | Fetch / soft-delete one metric |
| GET/POST | `/v1/metrics/{id}/points` | Time-series points; GET supports `from`, `to`, `dimensions`, `limit` |
| POST | `/v1/metrics/{id}/aggregate` | In-process aggregate (`sum`, `avg`, `min`, `max`, `count`) |
| GET | `/v1/domain/status` | Phase, durability flag, and capability planes |

Soft-deleted datasets and metrics remain in the database but are excluded from list/get results.

Auth (local): header `x-helix-dev-user: you@example.com`.
