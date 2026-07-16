# HelixCollab deep slice

**Goal:** First product forge depth on HelixCore  
**Port:** 8101 · **Web:** 3101  

## Landed

- Durable documents (create / get / patch / delete)
- Optimistic concurrency (`base_version` → 409)
- Revision history + restore
- Presence REST + WebSocket room fan-out (NATS multi-instance)
- Document ACL grant on create + enforce on access
- **WS auth** — `?token=` / `?dev_user=` (+ headers); tenant + ACL before upgrade
- **Durable WS patch** — authenticated room writes via Postgres optimistic apply
- **Share ACL** — `POST/GET /v1/documents/{id}/share` + Invite UI
- **Optional CRDT** — `HELIX_COLLAB_CRDT=1` (yrs), `crdt_update` / `crdt_sync`
- **Console deep-link** — Catalog “Open UI” → collab web
- Domain status + graceful shutdown
- Operator web editor (list / edit / autosave / live peers / share)

## Deeper collab (e2ee / anchors / activity)

- **Server vault e2ee** — `e2ee` on create + flags; HVA4 tenant seal; API returns plaintext after decrypt
- **Client-held E2EE** — `client_e2ee` + HC1 envelopes; server never opens; keys in browser (PBKDF2 wrap + AES-GCM DEK)
- **Pin / archive** — `flags` `{ pinned, archive }`; list hides archived; pinned first
- **Anchored comments** — `anchor_start` / `anchor_end` / `anchor_quote` on comment create
- **Resolve threads** — `POST .../comments/{cid}/resolve` `{ resolved }`; WS `comment_event`
- **Activity feed** — `GET /v1/documents/{id}/activity` (`collab.activity`)
- **Typing** — ephemeral WS `typing` indicators (not durable)
- **ProseMirror** — rich editor + optional y-prosemirror CRDT; markdown storage path for e2ee

Migrations: `0020_collab_deeper.sql`, `0021_collab_client_e2ee.sql`

## Web (Yjs + workspace UI)

- `src/lib/yjs-provider.ts` — WS `crdt_sync` / `crdt_update` ↔ Y.Doc + typing + comment_event
- `YTextEditor` bound to `Y.Text("content")` with selection anchors + jump-to-range
- 3-pane workspace: workspaces/folders/docs · editor · Comments/People/Share/History/**Activity**
- Presence, ACL invite, revision restore, comments with `@mentions`, mention inbox
- Pin / E2EE / **Archive** / **Duplicate** / copy id
- Dirty state + **content autosave** (~1.6s), save status in statusbar
- Comment open/all filter, inline edit, jump-to-anchor, relative times
- Keyboard shortcuts overlay (`Ctrl+/`), focus / preview toggles
- Folder rename (✎ / double-click), list pin/e2ee badges

## Client E2EE notes

- Envelope: `HC1.<iv_b64url>.<ct_b64url>` (AES-256-GCM)
- DEK wrapped with PBKDF2(passphrase) in `localStorage` (`helix.collab.clientKeys.v1`)
- Export raw DEK for out-of-band share; import on unlock
- Server vault e2ee and client e2ee are mutually exclusive

## Sealed CRDT (stretch — multiplayer while blind)

Wire protocol (WS):

| Message | Direction | Server role |
|---------|-----------|-------------|
| `crdt_sealed_update` | client → room | Fan-out HC1 only; never decode Yjs |
| `crdt_sealed_sync` (empty) | client → server | Request last sealed full state |
| `crdt_sealed_sync` (HC1) | client → room | Cache + fan-out late-joiner state |

Also REST (ops/smoke):

- `GET/POST /v1/documents/{id}/sealed-crdt` — opaque cache put/get

Client:

- Unlocked client-e2ee docs connect WS in **sealed mode**
- Yjs updates encrypted with DEK before send; decrypt on receive
- Full sealed state republished every ~15s for late joiners
- Plain `crdt_update` rejected on client-e2ee docs

## Sovereign stack (horizons A–C)

See `SOVEREIGN_ROADMAP.md` and `THREAT_MODEL.md`.

- Device keys · key shares · classification policy · backpack export
- Durable sealed CRDT (Postgres + JetStream) · sealed presence WS
- Spaces · sealed attachment meta · offline IndexedDB · client agent
- MLS stub · threshold recovery · residency proofs · federation receipts

Migration: `0022_collab_sovereign.sql`

## Remaining depth (optional)

- Real OpenMLS wire, WebAuthn hard-bind, MinIO attachment bytes upload
- Full offline merge UI · multi-region residency enforcement

## Prove

```powershell
$env:HELIX_ENV="local"
$env:HELIX_ALLOW_DEV_HEADERS="1"
$env:HELIX_DEV_PLATFORM="1"
$env:HELIX_COLLAB_CRDT="1"   # optional
# load secrets from ~/Desktop/.keys/helixforge/.env.local
rustup run stable-x86_64-pc-windows-msvc cargo run -p helix_collab_api

powershell -File scripts/helix_collab_smoke.ps1

cd projects/helix-collab/web
pnpm install
pnpm dev   # http://127.0.0.1:3101
```

Smoke covers: full collab surface + **client_e2ee** + **sealed_crdt** blind relay.
