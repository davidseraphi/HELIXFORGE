# HelixCore as foundation for 20 products — enterprise fit

**Date:** 2026-07-14  
**Question:** Is HelixCore the best possible foundation for the whole platform, or are there still gaps?

## Short answer

HelixCore is a **strong sovereign enterprise foundation** for all 20 product forges.
It is the right architecture: one core, many products via `service_kit`.

It is **not** “done forever.” Remaining gaps are mostly **product-domain** and
**scale/region** work — not missing identity/audit/billing primitives.

---

## What every product already gets for free

| Need | HelixCore surface |
|------|-------------------|
| AuthN / session | Kratos sessions + Hydra OIDC + API keys + local dev |
| AuthZ scopes | read / write / admin / platform / audit_read |
| Multi-tenant isolation | tenant id on principal; suspend gate |
| Secrets | Vault HVA3 + KMS + MinIO objects |
| Usage / commercial | Meter + plans + payment intents (local_sim) |
| Audit / compliance | Hash chain, export, compliance summary |
| Events | NATS `helix.*` subjects |
| Agents | Agent hub + durable runs + tools |
| Edge | Gateway catalog, proxy `/p/{slug}`, inventory |
| Ops | Health, Prometheus, OTEL/HTTP, rate limit, graceful shutdown |
| Deploy | Helm + Argo + NetworkPolicy + mesh annotations |

Products **must not reimplement** these. They add domain tables + routes only.

---

## What each product does at enterprise level

### Productivity (1–10)

| # | Product | Enterprise product work | Relies on Core for |
|---|---------|-------------------------|--------------------|
| 1 | **Collab** | CRDT/WS scale-out, doc ACLs, offline, e2ee optional | identity, audit, residency, NATS fan-out, vault for keys |
| 2 | **Code** | Repo storage, CI hooks, agent coding policies | agents, vault (SSH tokens), meter (compute), audit |
| 3 | **Flow** | Workflow engine durability, retries, human-in-loop | agents, NATS jobs, audit, meter |
| 4 | **Insights** | Timescale pipelines, PII redaction, dashboards | meter, residency, audit, MinIO datasets |
| 5 | **Commerce** | Catalog, checkout, tax, fraud, webhooks | payments intent API, vault (PSP keys), audit, meter |
| 6 | **Edu** | Courses, proctoring signals, credentials | identity, audit, meter, vault |
| 7 | **Capital** | Double-entry depth, SOX-ish controls, reporting | audit immutability, residency, vault |
| 8 | **Well** | PHI-adjacent privacy, consent, retention | residency, audit export, vault |
| 9 | **Network** | Graph privacy, anti-abuse, messaging | identity, tenant isolation, rate limit |
| 10 | **Forge Studio** | App sandboxing, secrets per app, publish | vault, agents, gateway catalog |

### Frontier (11–20)

| # | Product | Enterprise product work | Relies on Core for |
|---|---------|-------------------------|--------------------|
| 11 | **SynthBio** | Design IP isolation, large sim artifacts | MinIO, meter (GPU), audit, residency |
| 12 | **Lex Prime** | Legal hold, privilege, matter ACLs | audit chain, export, vault |
| 13 | **Cura Prime** | **HIPAA-class** controls, BAAs, PHI vault | residency hard fail, audit, encryption, DR |
| 14 | **Terra Prime** | Field data retention, offline edge nodes | MinIO, meter, residency |
| 15 | **Climate Prime** | Model provenance, public data licensing | audit, MinIO, meter |
| 16 | **Orbit Prime** | ITAR/export control metadata | residency, platform scopes, audit |
| 17 | **Quantum Forge** | Queue fairness, expensive job billing | meter, agents, vault API keys |
| 18 | **Vita Prime** | Clinical trial privacy, IRB workflows | same as Cura-class controls |
| 19 | **Grid Prime** | OT/ICS integration, high-reliability ingest | NATS, rate limit, DR |
| 20 | **Nova Labs** | Experiment isolation, multi-tenant labs | tenants, API keys, agents, vault |

---

## Gaps — closed in Core (migration 0016+)

| Former gap | Status |
|------------|--------|
| **Resource ACL** | ✅ `ResourceAclRepo` + `/v1/acl/*` |
| **Retention / legal hold** | ✅ `GovernanceRepo` + `/v1/governance/*` |
| **Purpose / consent binding** | ✅ purpose bindings + purpose_allows |
| **Multi-region registry + write affinity** | ✅ `RegionRepo` + `/v1/regions` (active-active DB still deploy ops) |
| **Gateway WS proxy** | ✅ `/p/{slug}/ws/**` with auth header forward |
| **Continuous backup** | ✅ `continuous_backup.ps1` + MinIO versioning helper |

### Still product- or deploy-owned (not missing Core APIs)

| Item | Owner |
|------|--------|
| Field-level PHI encryption UX | Product + vault keys |
| Live multi-region Postgres replication | Infra (Core exposes region registry) |
| Mesh mTLS in cluster | Linkerd/Istio (ADR-0011) |
| Stripe PSP | Deferred by product choice |
| Hardware HSM | Optional HTTP KMS |
| Console OIDC login UI | Console app |

### Not gaps — correct deferrals

- Product domain depth (commerce checkout, quantum scheduling) → **product crates**
- Console UX polish → **apps/console**
- Vendor SaaS lock-in → intentionally avoided

---

## Is this the best foundation possible?

**Yes, for HelixForge’s stated doctrine** (sovereign, self-hosted, one core, twenty forges):

1. **Architectural fit is excellent** — `service_kit` + shared clients prevent 20× reimplementation.  
2. **Security spine is enterprise-grade for B2B SaaS / internal platforms**: identity (Kratos+Hydra+API keys), audit chain, residency, rate limits, tenant suspend, vault/KMS.  
3. **Regulated verticals (Cura/Vita/Lex)** need **additional Core modules** later (retention, ACL resource model, consent) before claiming HIPAA/FedRAMP-class completeness — but the **spine is right**.  
4. **Frontier compute products** will push **metering + job queues + MinIO** harder; Core already has the hooks.  
5. A “better” foundation would only mean buying a cloud-only platform (AWS Control Tower + Cognito + …) and abandoning sovereignty — that conflicts with the project goal.

### Recommendation

- Treat HelixCore **enterprise baseline as ready** to deep-build products 1–20 on top.  
- Schedule Core epics only when a product hits a P0 gap (especially **resource ACL** + **retention** when Cura/Vita go deep).  
- Run **Kimi full review** when you want an external pass on this foundation before product deep-dives.

---

## Product integration checklist (for each forge)

When deepening product N:

1. Domain migrations under `helix_db` (tenant-scoped FKs).  
2. Routes via `ProductService` + domain router.  
3. Every security mutation → `clients.audit`.  
4. Billable actions → `clients.billing.record_usage`.  
5. Events → `helix.{slug}.*` on NATS.  
6. Secrets → vault / object store — never product tables.  
7. Agents register via hub or product assistant pattern.  
8. Catalog already lists slug/port — no Core change unless new capability.
