# HelixQuantumForge

**Order:** 17 · **Tier:** frontier

Hybrid quantum-classical computing platform

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-quantum-forge.*`
- Objects: MinIO bucket `helix-helix-quantum-forge`

## Local development

```bash
# from monorepo root
cargo run -p helix_quantum_forge_api
cd projects/helix-quantum-forge/web && pnpm dev
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
