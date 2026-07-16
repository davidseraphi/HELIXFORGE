# HelixAnvil — Agent Operating Layer (canonical, vendor-neutral)

This file is the **single canonical instruction set** for all AI agents in this
repository. Vendor shims defer to this file; this file defers to the portfolio
doctrine (`~/shared/AGENTS.md`). North star: **resume correctly and fail loudly.**

---

## § Commands (run from repo root)

```bash
python tools/context/check_context_drift.py --schema
python tools/context/build_document_index.py
python tools/context/build_context_pack.py
python tools/context/check_context_drift.py
python tools/quality/validate_bug_packets.py
```

Product commands land in `PROJECT_STATE.json.commands` when the Rust workspace exists.
**Never background** GUI or long-running servers — user-owned terminals only.

---

## § Read order (before ANY work)

1. **`PROJECT_STATE.json`**
2. **`NEXT_ACTION.md`**
3. **`docs/context/AGENT_CONTEXT.md`**
4. **`VISION.md`** + **`constitution.md`**
5. **`BUILD_SPEC.md`**
6. **Active feature packet** (`docs/features/<NNN>-<slug>/`)
7. Evidence via `docs/DOCUMENT_INDEX.md` when needed

---

## § Source-of-truth precedence

1. Live files/tests/commands · 2. `PROJECT_STATE.json` · 3. `constitution.md`
4. `BUILD_SPEC.md` · 5. `specs/` · 6. `VISION.md` · 7. `DECISION_LOG.md`
8. Active packet · 9. `NEXT_ACTION.md` · 10. Generated context · 11. Raw evidence

---

## § Feature-packet rule

**No packet, no code.** Packets live under `docs/features/<NNN>-<slug>/` with
requirements, design, tasks, acceptance, status.json. Behavior as
Requirement/Scenario (EARS/SHALL). T-000 clarify gate before implementation.
On `done`, promote to `specs/<capability>/spec.md` when applicable.

---

## § Update + continuation

Update substrate files in the same change as the work. Compaction checklist →
`NEXT_ACTION.md` before context fills.

---

## § Boundaries and secrets

**Allowed:** repo root; active packet `allowed_edit_paths`; substrate state files.

**Ask first:** second concurrent packet; stack/architecture change; commit/push.

**NEVER:** secrets in-repo; `--no-verify`; force-push without instruction;
re-register this project as HelixForge `PRODUCT_CATALOG` product; Electron/Monaco
as product identity (constitution VII).

**Secrets:** `~/Desktop/.keys/helix-anvil/.env.local`

---

## § Review and ambition

Tier 0 = drift + bug validator (+ cargo when present). See `REVIEW.md`.
Thorough over cheap; ambition → packets and milestones, not slogans.

## § Portfolio note

HelixCode extreme is implemented in **HelixForge**, not here. This repo is the
standalone native IDE project only.
