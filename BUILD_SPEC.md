# HelixForge Build Spec

## Stack

| Area | Choice |
|------|--------|
| Language (backend) | Rust 1.85+ / edition 2021 |
| HTTP | Axum 0.8 + Tokio |
| Shared libs | `shared_core`, `service_kit`, `agent_framework`, `*_client` |
| Frontend | Next.js 15, React 19, TypeScript 5.7 |
| Monorepo JS | pnpm workspaces + Turborepo |
| Auth | Ory Kratos/Hydra via `auth-adapter` |
| DB | PostgreSQL 16 + TimescaleDB |
| Objects | MinIO |
| Messaging | NATS JetStream |
| Deploy | Helm + ArgoCD GitOps; Terraform modules |
| Local | Docker Compose |

## Commands

```bash
# Windows hosts where rustup defaults to GNU must select the MSVC toolchain:
#   $env:RUSTUP_TOOLCHAIN='stable-x86_64-pc-windows-msvc'   # PowerShell
#   export RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-msvc   # bash

cargo build --workspace
cargo test --workspace --all-features
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

pnpm --filter @helixforge/console typecheck
pnpm --filter @helixforge/helix-code-web typecheck
pnpm --filter @helixforge/helix-collab-web typecheck

helm lint infra/helm/helix-core
docker compose up -d postgres nats minio minio-init
```

## Current source status — 2026-07-15 after Foundation Integrity 011.1

The workspace is still a prototype, but the truthful foundation is now under
version control and the full tree is reproducible on Windows, macOS, and Linux.

- `cargo build --workspace`, `cargo test --workspace --all-features`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo fmt --all -- --check` pass on the current Windows MSVC host.
- Console, HelixCode web, and HelixCollab web type checks pass.
- The root is now a Git repository with a `main` branch and a review policy in
  `CONTRIBUTING.md`.
- `.github/workflows/ci.yml` runs native Windows, macOS, and Linux runners and
  records artifact hashes.
- The global Windows-only target pin was removed from `.cargo/config.toml`; the
  toolchain file lists all three host targets. Windows dev hosts with a GNU
  default must set `RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-msvc`.
- HelixCode and HelixCollab are real alpha prototypes. Products 3–9 are small
  backend slices. Products 10–20 are generated data-service shells. HelixPulse
  and HelixAnvil are scaffolds; HelixAnvil's canonical home is now
  `projects/helix-anvil` inside the monorepo, but it remains portfolio-last.

The active gate is Foundation Integrity child packet `011.1`. Product depth
remains paused until `011.1` closes and the founder activates `011.2`.


## Definition of done (bootstrap)

The checkmarks below mean that a structural bootstrap slice was created at some
point. They do **not** mean that its current implementation compiles, passes the
new release gates, or meets the target-state product contract.

- [x] Cargo workspace with core crates + 6 core services + 20 product APIs
- [x] Hash-chained audit log
- [x] Product catalog in `shared_core`
- [x] docker-compose data plane
- [x] Helm/Terraform/ArgoCD skeletons
- [x] Console UI with catalog
- [x] CI workflow with quality gates
- [x] Full Postgres-backed audit/meter via `helix_db` sqlx migrations (memory fallback)
- [x] HelixCollab durable documents + WebSocket presence/realtime
- [x] HelixCode + HelixFlow durable domain tables
- [x] HelixInsights durable datasets / metrics / points (`0004_insights.sql`)
- [x] Multi-instance Collab WS fan-out via NATS (`helix.collab.ws.>`)
- [x] HelixCommerce durable products / orders (`0005_commerce.sql`)
- [x] HelixEdu durable courses / enrollments (`0006_edu.sql`)
- [x] HelixCapital durable accounts / journals (`0007_capital.sql`)
- [x] HelixWell durable habits / check-ins (`0008_well.sql`)
- [x] HelixNetwork durable profiles / connections / opportunities (`0009_network.sql`)
- [x] Products 10–20 thin durable parent/child slices (`0010_products_10_20.sql`)
- [ ] gRPC service mesh (HTTP first)
- [ ] Deep product UX (pick one after widen)

## Legacy operator-only local configuration

The current prototype still reads direct environment variables. That is a
legacy bootstrap path, not the target capability contract. An operator may own
an external local configuration file, but projects and agents must not discover
its path, read its values, or copy values into chat, Git, logs, proof, or state.
Foundation Integrity must replace direct secret handling with the user-owned
capability broker. The names below document the current compatibility surface;
they are deliberately not working credentials.

```
# --- Legacy compatibility names; operator or broker supplies real values.
# Never put values in this repository, chat, logs, proof, or agent context.
DATABASE_URL=INJECT_VIA_USER_OWNED_BROKER
NATS_URL=CONFIGURE_OPERATOR_ENDPOINT
MINIO_ENDPOINT=CONFIGURE_OPERATOR_ENDPOINT
MINIO_ROOT_USER=INJECT_VIA_USER_OWNED_BROKER
MINIO_ROOT_PASSWORD=INJECT_VIA_USER_OWNED_BROKER
NEXT_PUBLIC_GATEWAY_URL=http://127.0.0.1:8080
HELIX_ENV=local
HELIX_DATA_RESIDENCY=local
```

## Semantic versioning

Workspace packages start at `0.1.0`. Breaking changes to `service_kit` public
API require a minor bump and ADR.
