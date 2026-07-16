# 010 — HelixCore deep foundation

**Project Name:** HelixCore  
**Version target:** 0.2.0  
**Status:** Deep build (bootstrap complete; capabilities partial)

## 1. Overview

Shared **sovereign foundation** for the HelixForge ecosystem:

- **1** HelixCore platform  
- **20** product forges (Productivity 1–10 + Frontier 11–20)

HelixCore owns identity, agents, vault, billing/metering, audit, observability,
messaging, data plane access, and infra templates. Products must not reimplement
these (constitution: one core).

### 1.1 Spec shape (yes, this is correct)

A HelixCore deep-build uses a **project specification** with:

1. Overview  
2. Key features (named platform capabilities)  
3. Architecture & stack  
4. Non-functional requirements  
5. Integration points (all products)  
6. Acceptance criteria (verifiable)  
7. **Plus:** current vs target state, phased slices, and out-of-scope

Your draft is the right skeleton. This document is the **working** version
aligned to the monorepo.

### 1.2 Honest current state (do not claim “Completed”)

| Capability | Status today |
|------------|----------------|
| Gateway catalog / health / `/v1/me` / workspaces | Working (local) |
| Dev identity + scopes/residency headers + cookie session | Working |
| Ory Kratos/Hydra in compose | Optional profile; not required for first boot |
| Durable Postgres audit + meter + migrations | Working (`helix_db` through 0013) |
| Memory fallback when Postgres down | Working |
| Agent framework + agent-hub | Multi-step tools + durable runs + NATS |
| Vault service + client | AES-GCM durable + MinIO object refs |
| Billing client / meter + plans + summary | Working (payments deferred) |
| Observability | Metrics JSON + Prometheus + core health agg + OTEL runbook |
| NATS bus | Working (connect + publish/subscribe + memory fallback) |
| Infra templates (Helm/TF/Argo) | Chart probes/SA/secrets/OTEL; TF network README |
| Gateway reverse-proxy → products/core | Working `/p/{slug}` + `/core/{service}` |

Bootstrap feature `000-helix-core-bootstrap` remains **done**.  
This feature (`010`) is the **deep** program of work.

---

## 2. Key features (product names → platform modules)

Use brand names in product docs; map to crates/services in code:

| Brand / feature | Code surface | Deep-build goal |
|-----------------|--------------|-----------------|
| **AetherID** | `auth-client`, `auth-adapter`, Ory compose | Hybrid Kratos session + local dev; RBAC scopes; residency checks; session edge on gateway |
| **Agent Hub** | `agent-framework`, `agent-hub` | Multi-step tool agents, product registration, audit of agent runs, NATS job hooks |
| **Vault** | `vault-client`, `vault-service` | Tenant-scoped secrets; optional MinIO-backed object refs; never store secrets in product DBs |
| **Billing & metering** | `billing-client`, `billing-service`, `PgMetering` | Usage events, aggregates, product-tagged meters; marketplace pricing later |
| **Observability** | `observability`, `observability-service` | Structured logs, metrics registry, health aggregation; OTEL export path |
| **Audit chain** | `audit-log`, `PgAuditSink` | Append-only BLAKE3 chain; verify endpoint; all security mutations |
| **Data plane** | Postgres/Timescale, MinIO, NATS | Compose healthy; migrations on boot; JetStream subjects `helix.*` |
| **Infra templates** | `infra/helm`, `terraform`, `argocd` | Deployable core chart values; network module; GitOps app skeleton |

---

## 3. Architecture & tech stack

```
Clients (console + product webs)
        │
        ▼
   HelixCore Gateway (:8080)     ← catalog, me, future reverse proxy
        │
   ┌────┴────┬─────────┬──────────┬───────────┬────────────┐
   ▼         ▼         ▼          ▼           ▼            ▼
auth-    agent-    vault-   billing-  observability-   product APIs
adapter  hub       service  service   service          (:8101–8120)
   │         │         │          │           │            │
   └─────────┴─────────┴────┬─────┴───────────┴────────────┘
                            │
              service_kit  +  shared_core  +  helix_db
                            │
              PostgreSQL · NATS · MinIO · (Ory optional)
```

**Stack (locked):** Rust/Axum/Tokio/sqlx · Postgres 16 + Timescale · MinIO · NATS JetStream · Ory (self-hosted) · Helm + ArgoCD + Terraform skeletons · secrets only under `~/Desktop/.keys/helixforge/`.

---

## 4. Non-functional requirements

| NFR | Requirement |
|-----|-------------|
| Sovereignty | Fully self-hostable; no mandatory SaaS |
| Zero-trust | Every request has a principal; residency mismatch fails closed outside local |
| Audit | Security mutations append to hash chain; chain verifies |
| Degraded mode | Postgres/NATS down: process starts; health reports degraded; no silent “prod auth” |
| Performance | Core APIs respond for local smoke &lt; 200ms typical (no hard SLA yet) |
| Multi-region | Residency field on principal + audit; multi-region routing is **later** |
| Portability | Products only depend on `service_kit` / HelixCore clients |

---

## 5. Integration points

**All 20 products** (and console) integrate HelixCore via:

1. `ServiceBuilder::new(slug, port)` — shared clients  
2. `ProductService::router` — workspaces / product meta where applicable  
3. `RequireAuth` — principal + scopes  
4. `clients.audit` / `clients.billing` / `clients.bus` / `clients.vault` / `clients.agents`  
5. Gateway catalog discovery (`/v1/catalog`)  
6. Shared env: `DATABASE_URL`, `NATS_URL`, `HELIX_ENV`, residency  

Adding product N must remain: domain tables + routes + catalog entry (already true for 1–20 thin slices).

---

## 6. Acceptance criteria (deep program)

### 6.0 Bootstrap (already met)

- [x] Core crates + 6 core services compile  
- [x] `cargo test` green (MSVC toolchain on Windows)  
- [x] Catalog returns 20 products  
- [x] Local dev identity works  
- [x] Audit chain unit tests pass  
- [x] Postgres migrations + durable audit/meter when Docker up  

### 6.1 AetherID (slice A)

- [x] Kratos optional: `auth-adapter` reports `kratos_reachable` + mode (dev-fallback when down)  
- [x] Gateway `/v1/me` works with Bearer/cookie session **or** dev header in local  
- [x] Scope enforcement: `X-Helix-Dev-Scopes: read` → 403 on agent run (Write)  
- [x] Residency mismatch outside local → 403 (`X-Helix-Dev-Residency` vs `HELIX_DATA_RESIDENCY`)  

### 6.2 Audit + billing (slice B)

- [x] `GET` audit verify returns chain ok after rehash  
- [x] Meter summary + plans + invoice-style tenant summary  
- [x] Security actions (workspace create, secret put, agent run) go through PgAuditSink when DB up  

### 6.3 Vault (slice C)

- [x] Vault-service put/get/list/delete with tenant isolation + AES-GCM  
- [x] MinIO object binding: metadata + in-process SigV4 put/get of HVA2-sealed bytes  

- [x] Tenant isolation on secret paths  

### 6.4 Agent Hub (slice D)

- [x] Register product agent via hub API (`POST /v1/agents/product/{product}`)  
- [x] Run multi-step agent with tools; durable Postgres runs  
- [x] NATS publish on agent completion (`helix.core.agent.completed`)  

### 6.5 Observability (slice E)

- [x] `/healthz` checks postgres, bus mode, auth (fast Kratos probe)  
- [x] Prometheus + metrics JSON from observability-service  
- [x] Documented OTEL exporter config (`docs/runbooks/otel.md`)  

### 6.6 Gateway edge (slice F)

- [x] Reverse-proxy `/p/{slug}` + `/core/{service}` (ADR-0007)  
- [x] Catalog includes `gateway_prefix`; direct ports still allowed  

### 6.7 Infra (slice G)

- [x] Helm values for core services with health probes + SA/secrets/OTEL  
- [x] ArgoCD Application points at chart  
- [x] Terraform network module documented (dry-run README)  

### 6.8 Program done when

- [~] All slices A–G checked (residuals: Ory live optional, external HSM, full OTEL SDK)  
- [x] `cargo test --workspace` green (MSVC)  
- [x] Runbook: smoke + OTEL  
- [ ] Status of this feature set to `done` in `status.json` — **after Kimi** (not yet)  

---

## 7. Implementation phases (recommended order)

| Phase | Focus | Outcome |
|-------|--------|---------|
| **A** | AetherID hardening | Real Ory path + clear local/dev split |
| **B** | Audit verify + billing APIs | Operator-visible chain + usage |
| **C** | Vault durable backend | Tenant secrets beyond memory |
| **D** | Agent Hub depth | Portable agents for all products |
| **E** | Observability export | Scrape/export, not only logs |
| **F** | Gateway routing | Single entry for products (or ADR defer) |
| **G** | Infra polish | Deployable core, not skeleton-only |

Do **not** start product-deep UI work until A–C land (auth, audit, vault are the spine).

---

## 8. Out of scope (this feature)

- Full product UIs / domain depth for 1–20  
- Multi-region active-active  
- Production marketplace payments  
- gRPC service mesh  
- Replacing NATS or Postgres  

---

## 9. Naming note

“AetherID”, “Agent Hub”, etc. are **product brand names** for HelixCore capabilities.  
Code keeps neutral crate names (`auth-client`, `agent-hub`, …). Docs may use brands; APIs stay `helix.*` / service names.
