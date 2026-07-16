# HelixPulse

**Order:** 21 · **Tier:** frontier · **Port:** 8121  
**Build priority:** **LAST** — after HelixCore FULL and products 1–20

Sovereign distributed memory & cluster data plane — a modern, multi-tenant, residency-aware alternative to Redis-class systems (not a Redis clone).

## Status

**Scaffold only.** Do not deep-build until:

1. HelixCore sovereignty bar is closed (post–Kimi P0s + re-review)
2. Products 1–20 have durable domain slices
3. Platform consumers exist (rate limit, agent scratchpad, edge cache)

Until then Core uses **NATS + Postgres** for shared state. HelixPulse is the long-horizon product.

## Architecture (target)

- Backend: Rust (Axum) via `service_kit` + HelixCore clients
- Cluster: multi-node shard map, Raft/Paxos-class consensus (TBD), mesh mTLS
- Security: AetherID, tenant ACL, Vault DEKs, audit hash chain
- Optional: Redis protocol **subset** gateway for migration
- Frontend: operator console shell (cluster map, key browser, residency)

## Local development (scaffold)

```bash
# from monorepo root — only after you intentionally start this product
cargo run -p helix_pulse_api
# optional web shell
cd projects/helix-pulse/web && pnpm dev
```

## HelixCore dependencies

| Service | Use |
|---------|-----|
| gateway | Public API edge `/p/helix-pulse` |
| auth-adapter | Identity & sessions |
| agent-hub | Ops / diagnostics agents |
| vault-service | Envelope keys for encrypted values |
| billing-service | Meter ops + storage units |
| observability-service | Cluster audit + metrics |

## Docs

- Vision & non-goals: `VISION.md`
- Build order: `docs/BUILD_ORDER.md`
