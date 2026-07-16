# HelixForge second-wave product depth

## Intent

Helix**Collab** and Helix**Code** received extreme depth. All other catalog products are durable scaffolds (~one `main.rs`). Second wave deepens them **one product at a time**, in catalog order, without underscope.

**Anvil stays last** (separate portfolio repo). **helix-pulse** stays monorepo **build-last** after waves 3–20.

## Depth bar (per product)

Minimum for a product to leave “thin scaffold”:

1. Durable schema (migration) + repo methods beyond list/create stub  
2. Real domain APIs (get, list, mutate, cancel/delete where relevant)  
3. In-process execution or side-effect path (not only “queued”)  
4. Audit + metering + NATS on key actions  
5. `GET /v1/domain/status` with planes  
6. `scripts/helix_<slug>_smoke.ps1` PASS against local Postgres  
7. Docs: product README + DECISION_LOG entry  

## Wave order (catalog 3–20; skip 1–2 done extreme)

| Wave | Product | Port | Status |
|------|---------|------|--------|
| W1 | **helix-flow** | 8103 | **PASS** (smoke) |
| W2 | helix-insights | 8104 | pending |
| W3 | helix-commerce | 8105 | pending (already has some durable catalog/orders) |
| W4 | helix-edu | 8106 | pending |
| W5 | helix-capital | 8107 | pending |
| W6 | helix-well | 8108 | pending |
| W7 | helix-network | 8109 | pending |
| W8 | helix-forge-studio | 8110 | pending |
| W9 | helix-synthbio | 8111 | pending |
| W10–W18 | primes / quantum / nova… | 8112–8120 | pending |
| W19 | **helix-pulse** | 8121 | monorepo endgame (last product) |
| — | **HELIXANVIL** | — | portfolio last (after monorepo) |

## W1 HelixFlow scope (this packet)

- Workflow CRUD get + list  
- Run list / get / cancel  
- **Execute** definition steps in-process (`echo`, `set`, `fail`, `http` blocked by default)  
- Step events persisted  
- Smoke + domain status  

## Rules

- Prefer extending `helix_db` + `service_kit` over new ad-hoc services.  
- Secrets only under Desktop `.keys`.  
- Long-running servers in user CMD, not AI background shells (prove scripts start briefly if needed).  
