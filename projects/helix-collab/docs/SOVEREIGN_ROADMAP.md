# HelixCollab sovereign roadmap — implementation map

## Horizon A (landed)

| Item | API / surface |
|------|----------------|
| Device key registry | `POST/GET /v1/devices`, `POST /v1/devices/{id}/revoke` |
| Key shares / multi-device | `POST/GET /v1/documents/{id}/key-shares` |
| Classification policy | `POST/GET /v1/documents/{id}/classification` |
| Export backpack | `GET /v1/documents/{id}/export` |
| Sealed presence | WS `sealed_presence` |
| Durable sealed CRDT | `POST/GET …/sealed-crdt/durable` + JetStream subject |
| Threat model | `GET /v1/sovereign/threat-model` |

## Horizon B (landed scaffolding)

| Item | API / surface |
|------|----------------|
| Spaces tree | `POST/GET /v1/workspaces/{ws}/spaces` |
| Attachments (sealed meta) | `POST/GET /v1/documents/{id}/attachments` |
| Client agent | `POST /v1/documents/{id}/agent/suggest` (refuses HC1) |
| Offline protocol | web `offline-store.ts` (IndexedDB Y.Doc cache) |

## Horizon C + full depth (landed)

| Item | API / surface |
|------|----------------|
| **OpenMLS RFC 9420** | `POST /v1/mls/identity`, key-packages, `…/mls/group|add|join|message|process|export-secret` |
| Threshold recovery | `POST …/recovery`, `POST /v1/recovery/{id}/complete` |
| Residency hard-enforce | `POST …/required-region` + proofs |
| Federation | `POST /v1/federation/export|import` |
| **MinIO bodies** | `POST …/attachments/upload`, `GET …/attachments/{id}/body` |
| **Passkey bind** | `POST /v1/webauthn/register|authenticate/*` (ECDSA P-256, pure Rust) |
| Offline merge | web `offline-merge.ts` + UI button |

Migrations: `0022_collab_sovereign.sql`, `0023_collab_full_depth.sql`

## Edges closed (2026-07-15)

| Edge | Closure |
|------|---------|
| MLS only in-memory | `export_user_blob` / `import_user_blob` → `collab.mls_identities` + hydrate on every MLS API |
| Passkey weak bind | v2: sign `clientDataJSON` (type, challenge, origin, rpId, userId) + monotonic counter |
| Attachment no delete | `DELETE /v1/documents/{id}/attachments/{att_id}` + MinIO `delete_object` + web Download/Delete/Upload |
