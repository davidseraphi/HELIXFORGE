# HelixForge

**Helix** = interconnected growth · **Forge** = creation

Sovereign, high-performance, Rust-first monorepo for the HelixForge ecosystem:
**HelixCore** (platform) + **21 product APIs** + Next.js consoles.
(Product 21 HelixPulse = cluster data plane, **build last**.)

## Architecture at a glance

| Layer | Stack |
|-------|--------|
| Backend | Rust 2021, Axum, Tokio, shared `service-kit` |
| Frontend | Next.js 15, React 19, pnpm + Turborepo |
| Auth | Ory Kratos/Hydra (self-hosted) + local dev headers |
| Data | PostgreSQL 16 + TimescaleDB, MinIO, NATS JetStream |
| Delivery | Docker Compose (local), Helm + ArgoCD + Terraform (prod) |

All products reuse HelixCore: **gateway · auth-adapter · agent-hub · vault · billing · observability**.

## Product catalog (1–21)

| # | Product | Port | Tier |
|---|---------|------|------|
| 1 | HelixCollab | 8101 | Standard |
| 2 | HelixCode | 8102 | Standard |
| 3 | HelixFlow | 8103 | Standard |
| 4 | HelixInsights | 8104 | Standard |
| 5 | HelixCommerce | 8105 | Standard |
| 6 | HelixEdu | 8106 | Standard |
| 7 | HelixCapital | 8107 | Standard |
| 8 | HelixWell | 8108 | Standard |
| 9 | HelixNetwork | 8109 | Standard |
| 10 | HelixForge Studio | 8110 | Standard |
| 11 | HelixSynthBio | 8111 | Frontier |
| 12 | HelixLexPrime | 8112 | Frontier |
| 13 | HelixCuraPrime | 8113 | Frontier |
| 14 | HelixTerraPrime | 8114 | Frontier |
| 15 | HelixClimatePrime | 8115 | Frontier |
| 16 | HelixOrbitPrime | 8116 | Frontier |
| 17 | HelixQuantumForge | 8117 | Frontier |
| 18 | HelixVitaPrime | 8118 | Frontier |
| 19 | HelixGridPrime | 8119 | Frontier |
| 20 | HelixNovaLabs | 8120 | Frontier |
| 21 | **HelixPulse** | 8121 | Frontier · **build last** (cluster data plane) |

## Quick start (local)

### 1. Infra

```powershell
cd C:\Users\divin\PROJECTS\HELIXFORGE
docker compose up -d postgres nats minio minio-init
```

### 2. HelixCore services

**Option A** — open one CMD window per service:

```cmd
cd C:\Users\divin\PROJECTS\HELIXFORGE
set HELIX_ENV=local
cargo run -p gateway
```

Repeat for: `agent_hub`, `vault_service`, `billing_service`, `observability_service`, `auth_adapter`.

**Option B** — spawn windows via script:

```powershell
powershell -File scripts\dev-core.ps1
```

### 3. Product API (example: Collab)

```cmd
cd C:\Users\divin\PROJECTS\HELIXFORGE
set HELIX_ENV=local
cargo run -p helix_collab_api
```

### 4. Console UI

```cmd
cd C:\Users\divin\PROJECTS\HELIXFORGE
pnpm install
cd apps\console
pnpm dev
```

Open http://127.0.0.1:3000

### Smoke checks

```cmd
curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/v1/catalog
curl -H "X-Helix-Dev-User: founder@helixforge.local" http://127.0.0.1:8080/v1/me
curl -H "X-Helix-Dev-User: founder@helixforge.local" http://127.0.0.1:8101/v1/product
```

## Repository layout

```
HELIXFORGE/
  crates/           # shared-core, service-kit, agent-framework, clients…
  services/         # HelixCore binaries
  projects/         # 21 product backends + web shells (Pulse = last)
  apps/console/     # Platform console (Next.js)
  infra/            # Terraform, Helm, ArgoCD
  deploy/local/     # docker init, Kratos config
  docs/adr/         # Architecture Decision Records
```

## Secrets

Never put credentials in the repo. Create:

`C:\Users\divin\Desktop\.keys\helixforge\.env.local`

See **Keys** section in `BUILD_SPEC.md` for the paste-ready block.

## Enterprise practices

- Clean / hexagonal architecture via `service-kit` + product domain modules
- Hash-chained immutable audit log (`audit_log` crate)
- Zero-trust principals + data residency checks
- Semantic versioning (workspace `0.1.0` → release tags)
- GitHub Actions CI: fmt, clippy, test, gitleaks, JS typecheck
- ADRs under `docs/adr/`

## License

Apache-2.0
