# 001 — Native editor kernel design · Requirements

## Goal

Produce a design-ready specification for HelixAnvil’s first **from-scratch native
editor kernel** (document model, command surface, shell boundary) so a later
implementation packet can start without architecture thrash.

## User / operator outcome

Founder can approve a concrete design (crates, APIs, non-goals) before any IDE
binary lands. HelixCode extreme remains independent in HelixForge.

## Non-goals

- Implementing the binary (deferred to a later packet after 001 `done`)
- Full LSP / debugger / extension host
- HelixCode forge features

## Allowed edit paths

- `docs/features/001-editor-kernel-design/**`
- `docs/architecture/**` (if created by this packet)
- `BUILD_SPEC.md` (stack section only, when design ratifies)
- `DECISION_LOG.md`, `NEXT_ACTION.md`, `PROJECT_STATE.json`

## Forbidden edit paths

- Product source trees invented outside design ratification
- HelixForge monorepo

## ### ADDED Requirements

#### Requirement: Kernel boundary documented

WHEN design is complete, the packet SHALL document the buffer kernel API surface
and what the UI must not own as a second text store.

##### Scenario: Single source of text truth

- GIVEN design.md accepted
- WHEN an implementer reads design.md
- THEN document mutations are specified to go through one kernel type only

#### Requirement: Shell choice ratified or deferred with criteria

WHEN the desktop shell toolkit is chosen (or explicitly deferred), the decision
SHALL be recorded in DECISION_LOG.md with criteria (license, Windows support, no
Electron-as-identity).

##### Scenario: Decision log entry

- GIVEN 001 clarify gate complete
- WHEN shell is decided or deferred
- THEN DECISION_LOG.md has a dated entry naming the choice and reason

#### Requirement: Implementation packet spawn criteria

WHEN 001 is marked done, PROJECT_STATE SHALL name a next implementation packet
id (e.g. 002-kernel-p0) with allowed paths — or an explicit hold reason.

##### Scenario: Clear handoff

- GIVEN 001 acceptance complete
- WHEN agents read next_product_slice
- THEN they know whether code may start and under which packet

## Open questions

- [NEEDS CLARIFICATION: Preferred native shell — eframe/egui vs iced vs other?]
- [NEEDS CLARIFICATION: First language for LSP wedge — Rust only or multi?]
- [NEEDS CLARIFICATION: Workspace model — single root only for P0?]
