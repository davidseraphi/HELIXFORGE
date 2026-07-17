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

Local dev requires the HelixCore data plane:

```bash
docker compose up -d postgres nats minio minio-init
DATABASE_URL=postgres://helix:helix@127.0.0.1:55432/helixforge cargo run -p helix_db --bin helix-migrate
```

Run the smoke test against a local API:

```powershell
# in another terminal, with HELIX_ALLOW_DEV_HEADERS=1
cargo run -p helix_commerce_api
pwsh -File scripts/helix_commerce_smoke.ps1
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
| GET/PATCH | `/v1/products/{id}` | Fetch / update product (price, inventory delta, status) |
| GET/POST | `/v1/orders` | List / create orders (atomic inventory decrement) |
| GET | `/v1/orders/{id}` | Fetch order with line items |
| POST | `/v1/orders/{id}/cancel` | Cancel pending order and restore inventory |
| GET | `/v1/domain/status` | Phase, durability flag, and capability planes |

Order creation rejects mixed-currency carts, locks product rows, and decrements
inventory in the same database transaction.

Auth (local): header `x-helix-dev-user: you@example.com`.
