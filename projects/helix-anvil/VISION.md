# HelixAnvil — vision

**Slug:** `helix-anvil` · **Stage:** substrate · **Home:** standalone portfolio project (not a HelixForge monorepo product)

## Problem

Sovereign software work still depends on **foreign IDE kernels** (VS Code / Electron, JetBrains, cloud IDEs). HelixForge’s code forge (**HelixCode**) can own git, CI, agents, and web surfaces — but if the **editor** is always someone else’s architecture, the portfolio does not own the place code is written.

## North star

A **completely native IDE built from scratch**: own document model, own command/view split, own desktop shell — not a skin over Monaco or a Code-OSS fork as identity.

Hard promise: **the text kernel and edit coordinates are ours.** Engines (tree-sitter, rust-analyzer, gitoxide) may attach; they do not *be* HelixAnvil.

## Wedge

Ship a daily-usable **native multi-file code editor** for HelixForge / Rust work on Windows first: open folder, edit, save, find, basic LSP for one language — before forge mesh or multiplayer.

## What it is not

- Not HelixCode (repos, smart HTTP, CI runners, forge web UI) — that stays in HelixForge  
- Not HelixCollab (sealed multiplayer docs)  
- Not “Electron + web editor” as the primary product  
- Not a day-one VS Code extension marketplace clone  

## Relationship to HelixForge / HelixCode

| Concern | Owner |
|---------|--------|
| Native editor kernel + desktop IDE | **HelixAnvil** (this repo) |
| Code forge (git hosting, CI, forge UX) | **HelixCode** in HelixForge |
| Identity, vault, audit spine | **HelixCore** (optional for Anvil offline kernel) |

Sequencing ratified with founder: **create this project (substrate protocol) first**, then build **HelixCode extreme** in HelixForge. Deep Anvil implementation is packet-driven here; Code extreme does not wait on Anvil feature parity.

## Success (long horizon)

- Open a real multi-crate workspace and edit without a foreign IDE  
- At least one LSP language path green  
- Agent-applied patches land as first-class edits  
- Optional HelixCore auth for remote workspaces  
