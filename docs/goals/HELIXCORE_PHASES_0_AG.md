# Goal: Finish HelixCore Phase 0 + A–G

**Goal ID:** `HELIXCORE-0-AG`  
**Status:** active  
**Owner:** Grok Build (+ Kimi CLI reviewer)  
**Repo:** `C:\Users\divin\PROJECTS\HELIXFORGE`  
**Spec:** `docs/features/010-helix-core-deep/requirements.md`  
**Bootstrap:** `docs/features/000-helix-core-bootstrap/` (Phase 0)

## Outcome

HelixCore foundation is **verified end-to-end** through:

| Phase | Name | Meaning |
|-------|------|---------|
| **0** | Bootstrap gate | Confirm 000 bootstrap acceptance + honest gaps; Kimi reviews evidence |
| **A** | AetherID | Auth / Ory / scopes / residency |
| **B** | Audit + billing | Chain verify + meter summary APIs |
| **C** | Vault | Tenant secrets durable path |
| **D** | Agent Hub | Multi-step agents + audit + NATS |
| **E** | Observability | Metrics export / health aggregation |
| **F** | Gateway edge | Routing table or ADR defer |
| **G** | Infra | Helm / Argo / Terraform polish |

**Done when:** every phase has `status: done` (or deferred via ADR) **and** a Kimi review packet with verdict `PASS` or `PASS_WITH_FOLLOWUPS` (follow-ups filed).

This is the **template** for how we will deep-build all **21 offerings** (HelixCore + products 1–20): one phase at a time, implement → self-check → **Kimi CLI review** → fix or defer → next phase.

## Non-negotiable process (each phase)

```
1. IMPLEMENT   — code + tests + docs for this phase only
2. SELF-CHECK  — cargo test (MSVC), smoke commands from phase packet
3. PACKET      — write docs/reviews/phases/<PHASE>/PACKET.md
                 (what changed, why, acceptance map, risks, residual debt)
4. KIMI REVIEW — run scripts/kimi_phase_review.ps1 -Phase <id>
                 saves report under docs/reviews/phases/<PHASE>/
5. TRIAGE      — address FAIL/BLOCKER; note WONTFIX with rationale
6. CLOSE       — update status.json; only then start next phase
```

**Never** mark a phase complete without a Kimi report on disk for that phase.

## Phase 0 scope (bootstrap gate)

Not “rewrite bootstrap.” Instead:

- [ ] Re-run bootstrap acceptance scenarios (catalog 20, dev principal, audit chain tests)
- [ ] Document residual debt that A–G will fix (table in PACKET)
- [ ] Confirm transcript archive path for HELIXFORGE
- [ ] Kimi reviews Phase 0 packet + key files for honesty vs “completed” claims

## Phase A–G scope

See acceptance checklists in `docs/features/010-helix-core-deep/requirements.md` §6.

## Kimi invocation (canonical)

```powershell
# From repo root (UTF-8 recommended on Windows)
$env:PYTHONIOENCODING = "utf-8"
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"

powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\kimi_phase_review.ps1 -Phase 0
# or -Phase A | B | C | D | E | F | G
```

Underlying CLI pattern:

```text
kimi --print --final-message-only --work-dir <repo> --yolo `
  --prompt @docs/reviews/phases/<PHASE>/KIMI_PROMPT.md
```

Reviewer must cover: **correctness, security, sovereignty/constitution fit, missing tests, overclaim risk, and reasoning quality** in the PACKET.

## Portfolio rule (all 21)

For every offering later (Collab … NovaLabs):

1. Write or open a phased feature doc (`docs/features/…`)  
2. Implement one phase  
3. PACKET + Kimi review  
4. Close phase  
5. Repeat  

HelixCore `0 + A–G` is the first full program using this gate.

## Progress ledger

Update `docs/features/010-helix-core-deep/status.json` after each close.

| Phase | Status | Kimi report |
|-------|--------|-------------|
| 0 | pending | — |
| A | pending | — |
| B | pending | — |
| C | pending | — |
| D | pending | — |
| E | pending | — |
| F | pending | — |
| G | pending | — |
