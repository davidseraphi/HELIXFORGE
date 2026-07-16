# HelixCommerce

**Order:** 5 · **Tier:** standard

AI e-commerce & digital marketplace builder

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-commerce.*`
- Objects: MinIO bucket `helix-helix-commerce`

## Local development

```bash
# from monorepo root
cargo run -p helix_commerce_api
cd projects/helix-commerce/web && pnpm dev
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

Durable catalog/orders live in `helix_db` (`CommerceRepo`) + routes in `backend/src/main.rs`.

| Method | Path | Notes |
|--------|------|--------|
| GET/POST | `/v1/products` | List / create products (Postgres when available) |
| GET | `/v1/products/{id}` | Fetch one product |
| GET/POST | `/v1/orders` | List / create orders (inventory decrement in txn) |
| GET | `/v1/orders/{id}` | Fetch order with line items |
| GET | `/v1/domain/status` | `durable` flag from db pool |

Auth (local): header `x-helix-dev-user: you@example.com`.
