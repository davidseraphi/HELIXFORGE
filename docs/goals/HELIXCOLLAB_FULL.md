# Goal: HelixCollab fully built

**Goal ID:** `HELIXCOLLAB-FULL`  
**Status:** **SOVEREIGN-READY (local proof green)** — 2026-07-16

**Target:** HelixCollab is a complete, shippable product forge on top of HelixCore.

## Proof (2026-07-16)

| Check | Result |
|-------|--------|
| `cargo test -p helix_collab_api` | PASS (7/7) |
| `cargo clippy --workspace --all-targets` | PASS |
| `@helixforge/helix-collab-web` typecheck | PASS |
| `@helixforge/helix-collab-web` build | PASS |
| Deep smoke `scripts/helix_collab_smoke.ps1` | **PASS** |
| Durable documents + optimistic concurrency | PASS |
| Revisions + restore | PASS |
| Share ACL | PASS |
| Presence + WS auth | PASS |
| Server e2ee + client-held E2EE (HC1) | PASS |
| Anchored comments + resolve + activity | PASS |
| Pin / archive / duplicate | PASS |
| Sealed CRDT blind relay | PASS |
| Sovereign horizons A-C (devices, key shares, classification, backpack, sealed presence, durable sealed CRDT) | PASS |
| Spaces + attachments (sealed meta + MinIO bytes) | PASS |
| Client agent (refuses HC1) | PASS |
| Threshold recovery | PASS |
| Residency proofs + hard enforce | PASS |
| Federation export/import | PASS |
| OpenMLS group + add/join/message/process + durable secrets | PASS |
| Passkey v2 challenge | PASS |

## Order of work

```
1. BUILD HelixCollab to DEEP definition of done   ✅
2. Self-verify (tests + deep smoke)               ✅
3. Fix Axum 0.8 root-nesting runtime panic        ✅
4. Add CI smoke + web build proof                 ⏳
5. Re-proof CI                                    ⏳
```

## Deep definition of done

| # | Item | Status |
|---|------|--------|
| 1 | Documents CRUD + optimistic versioning + revisions + restore | ✅ |
| 2 | Workspace/folders + document move + list filtering | ✅ |
| 3 | Share ACL + tenant/owner enforcement | ✅ |
| 4 | Comments + anchored comments + resolve + mentions + inbox | ✅ |
| 5 | Presence REST + WS auth + durable WS patch + NATS fan-out | ✅ |
| 6 | Server e2ee (vault HVA4 seal) + client-held E2EE (HC1) | ✅ |
| 7 | Sealed CRDT blind relay cache (REST + WS) | ✅ |
| 8 | Sovereign stack: device keys, key shares, classification, backpack export, sealed presence, durable sealed CRDT | ✅ |
| 9 | Spaces + sealed attachments + MinIO bytes upload/download/delete | ✅ |
| 10 | Client agent suggest (refuses HC1) + threshold recovery + residency proofs + federation export/import | ✅ |
| 11 | OpenMLS RFC 9420 identity/group/add/join/message/process + durable blob persist | ✅ |
| 12 | Passkey v2 register challenge + required-region hard enforce | ✅ |
| 13 | Proof: tests + deep smoke + CI green | ⏳ |

## Sovereign-ready meaning

See `docs/runbooks/sovereign-ready.md` and `projects/helix-collab/docs/THREAT_MODEL.md`.

**In:** tenant-isolated documents, device-key registry, sealed CRDT/E2EE options, classification policy, threshold recovery, MinIO attachment custody, MLS group state, audited sharing and lifecycle events.

**Out of this goal:** production-scale load testing, browser e2e automation, federation against real remote homes, WebAuthn authenticator hardware attestation, mobile native clients.

## Operator re-proof

```powershell
docker compose up -d postgres nats minio minio-init
# load .keys + HELIX_ALLOW_DEV_HEADERS=1 HELIX_DEV_PLATFORM=1
helix-migrate
cargo run -p helix_collab_api
powershell -File scripts/helix_collab_smoke.ps1
```
