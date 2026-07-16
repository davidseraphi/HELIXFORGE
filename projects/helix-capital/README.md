# HelixCapital

**Order:** 7 · **Tier:** standard

AI financial operating system

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-capital.*`
- Objects: MinIO bucket `helix-helix-capital`

## Local development

```bash
# from monorepo root
cargo run -p helix_capital_api
cd projects/helix-capital/web && pnpm dev
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

Durable finance domain lives in `helix_db` (`CapitalRepo`) + routes in `backend/src/main.rs`.

| Method | Path | Notes |
|--------|------|--------|
| GET/POST | `/v1/accounts` | Chart of accounts |
| GET | `/v1/accounts/{id}` | Fetch account (includes `balance_cents`) |
| GET/POST | `/v1/journals` | List / post balanced double-entry journals |
| GET | `/v1/journals/{id}` | Fetch journal with lines |
| GET | `/v1/domain/status` | `durable` flag |

Journal lines require `side` of `debit` or `credit`; totals must balance.  
Balance rule (v1): debit **+**, credit **−** on stored `balance_cents`.

Auth (local): header `x-helix-dev-user: you@example.com`.
