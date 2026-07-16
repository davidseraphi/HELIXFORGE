# NNN — <Feature Title> · Requirements

<!-- INSTRUCTIONS (delete before use)
  Copy this entire `_template/` directory to `docs/features/<NNN>-<slug>/`.
  NNN = next zero-padded three-digit number (000, 001, …).
  Fill every section; remove all instructional comments.
  Unresolved questions use the [NEEDS CLARIFICATION: ...] marker — resolve all
  before any implementation task begins (see tasks.md "Clarify/analyze gate").
-->

## Goal

<!-- One paragraph: what this packet delivers and why it exists now.
  Be concrete about the problem boundary — what system state changes when
  this ships. Avoid restating the feature title. -->

[NEEDS CLARIFICATION: State the goal here.]

## User / operator outcome

<!-- The concrete change a real user, operator, or downstream system experiences
  when this ships. Tie to a documented project promise (VISION.md or equivalent).
  If no direct user interaction exists, describe the observable system-level
  outcome instead. -->

[NEEDS CLARIFICATION: State the observable outcome here.]

## Non-goals

<!-- Explicitly out of scope. This is how scope creep and duplicate builds are
  prevented. Be specific — a non-goal is only useful if someone could reasonably
  mistake it for in-scope. -->

- ...
- ...

## Allowed edit paths

<!-- Globs this packet may create or modify. Implementation MUST stay inside
  these paths. Review any deviation as a scope change. -->

- `docs/features/NNN-<slug>/**`
- ...

## Forbidden edit paths

<!-- Paths this packet must never touch. List by category. -->

- `docs/features/**` (other packets — cross-packet edits need an explicit
  dependency declared in the Dependencies section below)
- Any `.env*` file inside the repo (secrets live in `.keys/<project>/.env.local`
  per the portfolio doctrine — never commit credentials)
- Production state paths (`state/`, `_meta/`, `registry/`, or project-specific
  persistent state dirs) — all test/dev invocations must use `tmp_path` or
  equivalent isolated paths per the production-state-isolation rule
- [Add project-specific forbidden paths here]

## Dependencies

<!-- Other feature packets, external services, credentials, or architectural
  decisions that must exist or be resolved before this packet can start.
  Reference Decision Journal entries by id/date. -->

- Packet prerequisites: [none | NNN-<slug>]
- External services: [none | <service name — note which credential and whether
  it is a gated path requiring Tier 3 review>]
- Decision Journal: [none | DJ-NNNN]

## Evidence

<!-- Which research, user interviews, Decision Journal entries, or prior art
  justify this feature. Cite file paths or document IDs. Research is evidence,
  not instruction. Note any claim that still needs verification ("from recall —
  not yet verified"). -->

- ...

---

## Behavior specification (Requirement / Scenario)

<!-- Substrate v2.2 (OpenSpec graft): behavior is specified as named, diffable
  Requirement/Scenario blocks — NOT a flat AC-NNN checklist. This is the
  delta/"changes" half of the specs/-vs-changes split: when this packet reaches
  status `done`, the blocks below are PROMOTED into the ratified capability
  spec(s) at specs/<capability>/spec.md.

  EARS IS PRESERVED: each `#### Requirement:` line is an EARS / RFC-2119
  normative statement (SHALL/MUST/SHOULD). Each `##### Scenario:` is the
  GIVEN/WHEN/THEN acceptance elaboration — it maps 1:1 to an evidence row in
  acceptance.md. Named (not numbered) anchors survive renumbering and produce
  clean git diffs when behavior changes.

  EARS PATTERN REFERENCE (choose the right one per Requirement line)
    Event       WHEN <trigger> THE SYSTEM SHALL <response>
    State       WHILE <system is in state> THE SYSTEM SHALL <response>
    Unwanted    IF <unwanted condition> THEN THE SYSTEM SHALL <response>
    Optional    WHERE <feature is present> THE SYSTEM SHALL <response>
    Ubiquitous  THE SYSTEM SHALL <response>   (always true)

  DELTA HEADERS (### level): group Requirements by how they change the ratified
  spec. For a NEW (greenfield) capability with no prior specs/ entry, everything
  is ADDED — keep only the `### ADDED Requirements` group.
    ### ADDED Requirements      -> promote: insert under specs/<cap>/## Requirements
    ### MODIFIED Requirements   -> promote: replace the matching ### Requirement block
    ### REMOVED Requirements    -> promote: delete the matching ### Requirement block
  (Heading levels inside this packet file are one deeper than the ratified
  spec's: groups ###, requirements ####, scenarios #####. The ratified
  specs/<cap>/spec.md flattens them to ## Requirements / ### Requirement /
  #### Scenario when promoted.)

  ANCHOR + PROMOTION: the Requirement NAME (text after `#### Requirement:`) is the
  stable match key for promotion. MODIFIED/REMOVED MUST name a Requirement that
  already exists in the target capability spec; ADDED names MUST NOT collide with
  an existing one. Names are unique per capability spec. MULTI-CAPABILITY packet:
  suffix a delta group header with the target capability, e.g. `### ADDED
  Requirements (capability: <name>)`; every named capability MUST appear in this
  packet's promotes_to.

  MARKER CONVENTION: append [NEEDS CLARIFICATION: <question>] to any Requirement
  or Scenario whose trigger/state/response boundary is not yet agreed. Resolve
  ALL of these in the Clarify/analyze gate (tasks.md T-000) before implementation.

  WORKED EXAMPLE (generic — delete before use):
    ### ADDED Requirements
    #### Requirement: Record persistence
    WHEN a user submits a validated form THE SYSTEM SHALL persist the record
    and return HTTP 201 within 500 ms.
    ##### Scenario: Valid submission
    - GIVEN a form payload that passes validation
    - WHEN the client POSTs it to /records
    - THEN the system stores the record
    - AND responds 201 with the new record id within 500 ms
-->

**Target capability spec(s):** `specs/<capability>/spec.md` _(created or updated on packet `done`; register in PROJECT_STATE.json.ratified_specs and set this packet's `promotes_to`)_

### ADDED Requirements

#### Requirement: <name>

[NEEDS CLARIFICATION: state the EARS/SHALL requirement line here.]

##### Scenario: <name>

- GIVEN ...
- WHEN ...
- THEN ...
- AND ...

<!-- Add more Requirements/Scenarios. Use ### MODIFIED Requirements /
  ### REMOVED Requirements groups only when this packet changes a capability
  already ratified in specs/. Each Scenario must be independently verifiable
  and have a matching evidence row in acceptance.md. -->
