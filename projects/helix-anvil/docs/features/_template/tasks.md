# NNN — <Feature Title> · Tasks

<!-- INSTRUCTIONS (delete before use)
  Tasks must be:
    - Small: achievable in a single focused work session by a local/small model.
    - Independent when tagged [P]: tasks tagged [P] are parallel-safe — they
      may run concurrently (different agents, different worktrees, or sequential
      but order-agnostic). Tasks WITHOUT [P] must run in order.
    - Verifiable: each task has an exact verification command that a reviewer
      can run without re-interviewing the implementer.
    - Architecture-separated: never combine judgment (design, schema decisions)
      with mechanical implementation in the same local-model task. Judgment
      tasks belong to Opus tier; mechanical tasks to Sonnet or below.

  PARALLELISM MARKER [P]:
    Append [P] to the task checkbox line to signal the task is order-independent
    relative to other [P]-tagged tasks in the same group. Example:
      - [ ] T-003 [P] — Write unit test for parser
      - [ ] T-004 [P] — Write unit test for serializer
    Both can run simultaneously. Untagged tasks run sequentially after all
    prior tasks complete.

  CLARIFY/ANALYZE GATE (T-000):
    T-000 is ALWAYS the first task. It is non-bypassable.
    It resolves every [NEEDS CLARIFICATION] marker in requirements.md and
    design.md, confirms requirements<->design<->tasks are mutually consistent,
    and confirms the allowed/forbidden edit paths are correct.
    No implementation task may begin until T-000 is marked [x].

  VERIFICATION COMMANDS:
    Every verification command must be paste-ready and runnable from the
    project root. Prefer deterministic checks (lint, test, schema validation,
    hash comparison) over subjective ones ("looks right"). When the app does
    not yet exist, use substrate checks (schema validators, doc-drift checks).

  MODEL TIER PER TASK (recommended, not enforced):
    opus      — initial plan, schema design, root-cause analysis, synthesis
    sonnet    — mechanical implementation against a clear spec, test writing,
                doc updates, codex-absorption edits
    haiku     — pure read tasks, running existing scripts, status reporting
    local     — small, self-contained edits verifiable by a deterministic check
    codex     — Tier 3 cross-vendor review (not primary implementation)
-->

---

## Clarify/analyze gate

**T-000 must be complete before any implementation task begins.**

- [ ] T-000 — Clarify / analyze gate
  - Resolve every `[NEEDS CLARIFICATION: ...]` marker in requirements.md and design.md.
  - Confirm the behavior spec is well-formed: every `#### Requirement` has >=1
    `##### Scenario`, each Scenario has a matching row in acceptance.md, and the
    delta group headers (ADDED/MODIFIED/REMOVED) are correct against the target
    capability's current `specs/<capability>/spec.md` (MODIFIED/REMOVED must name
    a Requirement that already exists there).
  - Confirm requirements, design, and tasks are mutually consistent
    (no contradictions, no gaps between Scenarios and test strategy).
  - Confirm allowed/forbidden edit paths in requirements.md and design.md
    are correct and not missing any file this packet will touch.
  - Confirm dependency packets (if any) are at status `done` or confirm a
    plan to proceed in parallel with a documented assumption.
  - Verify: manually read both documents and record resolutions as inline
    comments or a brief note appended to this task.
  - Model tier: **opus**

---

## Implementation tasks

<!-- Add tasks here. Number sequentially T-001, T-002, … within this packet.
  Group into phases if the work has natural stages (e.g., data layer first,
  then API layer, then UI layer). Each group may contain [P]-tagged tasks. -->

### Phase 1 — [Phase name, e.g. "Schema / data layer"]

- [ ] T-001 — [Task description]
  - Verify: `[paste-ready command]`
  - Model tier: [opus | sonnet | haiku | local]

- [ ] T-002 [P] — [Task description — parallel-safe within this phase]
  - Verify: `[paste-ready command]`
  - Model tier: [sonnet | local]

- [ ] T-003 [P] — [Task description — parallel-safe within this phase]
  - Verify: `[paste-ready command]`
  - Model tier: [sonnet | local]

### Phase 2 — [Phase name, e.g. "API / business logic layer"]

- [ ] T-004 — [Task description — depends on Phase 1 completion]
  - Verify: `[paste-ready command]`
  - Model tier: [sonnet | local]

### Phase 3 — [Phase name, e.g. "Verification & review"]

- [ ] T-005 — Run Tier 0 deterministic gates
  - Verify: `[project Tier 0 command, e.g. make review or ruff check . && mypy src/]`
  - Model tier: haiku (script runner)

- [ ] T-006 — Run acceptance checklist (acceptance.md)
  - Verify: all AC-NNN items marked [x] in acceptance.md with evidence pasted
  - Model tier: sonnet

- [ ] T-007 — Update status.json: set status → review-ready, tasks_done, updated_at
  - Verify: `python -c "import json,sys; s=json.load(open('docs/features/NNN-<slug>/status.json')); assert s['status']=='review-ready', s['status']"`
  - Model tier: local

- [ ] T-008 — Promote behavior-spec delta into the ratified spec (on packet `done`)
  - For each capability in this packet's `promotes_to`: apply the ADDED/MODIFIED/
    REMOVED Requirement/Scenario blocks from requirements.md into
    `specs/<capability>/spec.md`, matching by exact Requirement NAME (the stable
    anchor): ADDED → insert under `## Requirements` (name must not collide);
    MODIFIED → replace the matching `### Requirement` block; REMOVED → delete it;
    flatten packet heading levels (####/##### → ###/####). Register/update the
    capability in `PROJECT_STATE.json.ratified_specs` (status `ratified`; append
    this packet id to `promoted_from` — the drift gate FAILS if a done packet's
    promotes_to capability omits its id). A NEW capability → copy
    `_spec_template/spec.md` to `specs/<capability>/spec.md` first.
  - Verify: `python tools/context/check_context_drift.py` exits clean (spec
    well-formedness + promotion-completeness checks pass).
  - Model tier: sonnet

---

## Definition of done

A task is marked `[x]` only when ALL of the following are true:

1. The code or doc change is written and committed.
2. The task's verification command passes without error.
3. The corresponding AC criterion in acceptance.md is confirmed.
4. Any touched gated path has the required review tier logged.
5. `status.json` reflects the current `tasks_done` count and `updated_at`.

A packet reaches `done` only when every task is `[x]`, every Scenario is `[x]`
in acceptance.md with evidence, all review tiers required by acceptance.md have
run, its behavior-spec delta has been promoted into `specs/<capability>/spec.md`
(T-008) for every capability in `promotes_to`, and `status.json` is set to `"done"`.
