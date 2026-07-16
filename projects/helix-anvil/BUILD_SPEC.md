# HelixAnvil — Build Spec

Status: canonical · Stage: **substrate** · Updated: 2026-07-15

## What this document is

Current **implementation contract**. At substrate stage there is **no product
binary yet** — only continuation files, gates, and the first design packet.

## Stack (target — not installed until packet work)

| Area | Default | Notes |
|------|---------|--------|
| Language | Rust | cargo workspace when code lands |
| Document model | own buffer (e.g. rope-backed) | own API; not foreign editor buffer |
| Desktop shell | native GUI (eframe/egui or ratified alternative) | **not** Electron-first |
| LSP | client only | rust-analyzer etc. as external engines |
| Task runner | none yet | use `python` tools + later `cargo` / `just` |
| Secrets | `~/Desktop/.keys/helix-anvil/.env.local` | never in-repo |

Stack choices for the first binary are finalized in packet **001** design, not here by fiat.

## Architecture (target shape)

```
ui shell  →  editor commands / view  →  buffer kernel  →  files / optional LSP
```

Invariant: all document mutations go through the buffer kernel.

## Runtime boundaries

| Surface | In / out |
|---------|----------|
| HelixAnvil desktop process | in (primary) |
| HelixForge monorepo | **out** — separate product |
| HelixCode API | later consumer only |
| HelixCore (Ory, vault, audit) | optional after offline kernel works |

## Commands (substrate)

```bash
# from HELIXANVIL repo root
python tools/context/check_context_drift.py --schema
python tools/context/build_document_index.py
python tools/context/build_context_pack.py
python tools/context/check_context_drift.py
python tools/quality/validate_bug_packets.py
```

When Rust lands, add to `PROJECT_STATE.json.commands`:

```bash
cargo test --workspace
cargo run -p helix_anvil   # name TBD in packet
```

**Long-running apps:** run in a **user-owned terminal**, never agent background shells.

## Definition of done (substrate install)

- [x] Substrate files per `~/shared/substrate/new-project-prompt.md`
- [x] Bug OS ported
- [ ] `check_context_drift.py` exits 0
- [ ] First feature packet specifies next real code slice

## Definition of done (product — later)

- Native multi-file edit + save  
- One language LSP path  
- Tests for buffer kernel invariants  
- Peer review for claimed depth  

## Keys

Create when first secret is needed:

```
# ~/Desktop/.keys/helix-anvil/.env.local
# HelixAnvil local secrets — never commit
# HELIX_ANVIL_NOTE=placeholder
```
