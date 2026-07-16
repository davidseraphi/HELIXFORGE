# Goal: HelixCore fully built

**Goal ID:** `HELIXCORE-FULL`  
**Status:** **SOVEREIGN-READY (local proof green)** — 2026-07-14  

**Target:** HelixCore is a complete, shippable foundation for HelixForge product forges.

## Proof (2026-07-14)

| Check | Result |
|-------|--------|
| Unit tests (auth, vault, billing, nats, audit, service_kit, …) | PASS |
| Deep smoke `scripts/helixcore_deep_smoke.ps1` | **PASS** |
| AetherID live Kratos+Hydra | PASS (`auth_mode=kratos+hydra`) |
| Vault HVA3 roundtrip | PASS |
| Payments local_sim | PASS |
| Gateway proxy `/core/auth` | PASS |
| Audit chain link verified | PASS |
| NATS JetStream | PASS (`bus=nats+jetstream`) |
| Postgres durable | PASS |
| Audit HMAC control | PASS (secret set) |
| Shared rate-limit backend | PASS (Postgres) |
| Catalog product 21 HelixPulse | PASS (scaffold, build last) |

## Order of work

```
1. BUILD HelixCore to DEEP definition of done   ✅
2. Self-verify (tests + multi-service deep smoke) ✅
3. Kimi CLI full review                         ✅ (prior NOT_COMPLETE)
4. Fix 1–11 + sovereign depth                   ✅
5. Re-proof deep smoke                          ✅
6. Optional: Kimi re-review for formal PASS stamp
```

## Deep definition of done

| # | Item | Status |
|---|------|--------|
| 1 | AetherID hybrid + fail-closed | ✅ |
| 2 | Agent Hub multi-step + timeouts + cancel | ✅ |
| 3 | Vault AES-GCM + HVA3/HVA4 + reencrypt on rotate | ✅ |
| 4 | Billing meter + idempotency + signed webhooks | ✅ |
| 5 | Audit chain + HMAC + restricted rehash | ✅ |
| 6 | Observability metrics + OTEL retries + compliance | ✅ |
| 7 | Gateway discovery + timeouts + retries + WS | ✅ |
| 8 | Postgres/NATS JetStream/MinIO fail-closed policy | ✅ |
| 9 | Dockerfile + Helm secrets fail + HPA/PDB | ✅ |
| 10 | Proof: tests + deep smoke | ✅ |

## Sovereign-ready meaning

See `docs/runbooks/sovereign-ready.md`.

**In:** fail-closed auth, durable data plane, tenant isolation, JetStream, shared rate limits, signed audit (when secret set), container/Helm path.

**Out of this goal:** multi-region active-active, Stripe production, HelixPulse cluster, deep product 1–20 UIs.

## Operator re-proof

```powershell
docker compose up -d postgres nats minio minio-init
# load .keys + HELIX_ALLOW_DEV_HEADERS=1 HELIX_DEV_PLATFORM=1
powershell -File scripts/dev-core.ps1
powershell -File scripts/helixcore_deep_smoke.ps1
```
