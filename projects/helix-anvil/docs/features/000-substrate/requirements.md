# 000 — Substrate install · Requirements

## Goal

Install the portfolio vendor-neutral agent-continuation substrate into HelixAnvil
so any vendor can resume cold without chat memory. Establish project identity
as a **standalone** native-IDE repo (not a HelixForge monorepo product).

## User / operator outcome

A founder or agent opening `C:\Users\divin\PROJECTS\HELIXANVIL` has a green
Tier-0 drift gate, clear read order, and a defined next packet (editor kernel
design) — while HelixCode extreme proceeds separately in HelixForge.

## Non-goals

- Implementing the native editor binary
- Adding HelixAnvil to HelixForge `PRODUCT_CATALOG`
- Choosing final GUI toolkit (deferred to 001)
- Shipping LSP, multiplayer, or forge mesh

## Allowed edit paths

- `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`
- `VISION.md`, `constitution.md`, `BUILD_SPEC.md`, `REVIEW.md`
- `PROJECT_STATE.json`, `NEXT_ACTION.md`, `DECISION_LOG.md`, `README.md`
- `.gitignore`, `.pre-commit-config.yaml`
- `schemas/**`
- `tools/context/**`, `tools/quality/**`
- `docs/features/000-substrate/**`
- `docs/features/001-editor-kernel-design/**` (planned shell only)
- `docs/features/_template/**`
- `docs/quality/**`, `docs/bugs/**`, `docs/context/**`
- `docs/DOCUMENT_INDEX.*`, `llms.txt`, `llms-full.txt`

## Forbidden edit paths

- HelixForge monorepo (`C:\Users\divin\PROJECTS\HELIXFORGE\**`) from this packet
- Any product Rust/TS source tree under this repo (none yet — do not invent ad-hoc)
- `.env*`, secrets paths outside paste-ready docs

## ### ADDED Requirements

#### Requirement: Substrate files present

WHEN an agent inventories the HelixAnvil root, the system SHALL provide the
portable substrate set (AGENTS.md, VISION, BUILD_SPEC, PROJECT_STATE, NEXT_ACTION,
DECISION_LOG, constitution, REVIEW, schemas, feature template, context tools, Bug OS).

##### Scenario: Cold resume inventory

- GIVEN a fresh clone or folder open of HELIXANVIL
- WHEN the agent follows AGENTS.md read order
- THEN PROJECT_STATE.json and NEXT_ACTION.md exist and name current_focus 000-substrate

#### Requirement: Drift gate clean

WHEN the operator runs `python tools/context/check_context_drift.py` from repo root,
the command SHALL exit 0 (schema, packets, index, context pack, bug packets).

##### Scenario: Tier-0 pass

- GIVEN substrate files and regenerated context pack
- WHEN check_context_drift.py runs without --schema-only
- THEN exit code is 0 and no DRIFT/ERRORS are reported

#### Requirement: Standalone identity

WHERE HelixAnvil is positioned in the portfolio, the project SHALL NOT be registered
as a HelixForge monorepo PRODUCT_CATALOG entry as part of this packet.

##### Scenario: No monorepo product embed

- GIVEN this packet’s do_not_start and constitution Article VIII
- WHEN substrate install completes
- THEN HelixAnvil remains a separate PROJECTS root, not projects/helix-anvil inside HelixForge

#### Requirement: Next slice defined

WHEN packet 000 completes, PROJECT_STATE SHALL name next_product_slice
`001-editor-kernel-design` as planned design work before any editor binary.

##### Scenario: Handoff to design

- GIVEN 000 acceptance evidence
- WHEN agents read next_product_slice
- THEN packet_id is 001-editor-kernel-design and status is planned
