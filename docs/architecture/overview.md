# HelixForge architecture overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Clients (Next.js UIs)                    │
│              apps/console · projects/*/web                   │
└───────────────────────────┬─────────────────────────────────┘
                            │ HTTPS
┌───────────────────────────▼─────────────────────────────────┐
│                     HelixCore Gateway                        │
│              catalog · routing · session edge                │
└─┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┘
  │         │         │         │         │         │
  ▼         ▼         ▼         ▼         ▼         ▼
auth-    agent-    vault-   billing-  observ-   product
adapter  hub       service  service   ability   APIs
  │         │         │         │         │         │
  └─────────┴─────────┴────┬────┴─────────┴─────────┘
                           │
              NATS JetStream · PostgreSQL · MinIO
```

## Layers (hexagonal)

1. **Domain** — pure product logic (`projects/*/backend/src/domain`)
2. **Application** — use-cases / agents
3. **Adapters** — HTTP (Axum), NATS, SQL, MinIO
4. **HelixCore** — shared platform services via `service-kit`

## Sovereignty controls

- Data residency region on every principal and audit entry
- Self-hosted IdP (Ory)
- Secrets outside repo (`~/Desktop/.keys/helixforge/`)
- Immutable audit chain
