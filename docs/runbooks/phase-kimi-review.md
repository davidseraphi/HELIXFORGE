# Phase gate: implement → Kimi CLI review

## Purpose

Independent review of each **phase** by **Kimi CLI** before we claim the phase done.
Applies first to HelixCore phases 0 + A–G, then the same pattern for all 21 offerings.

## Roles

| Role | Actor |
|------|--------|
| Implementer | Grok Build (or other agent) |
| Reviewer | Kimi Code CLI (`kimi.exe`) |
| Operator | You — resolve stalemates, approve WONTFIX |

## Per-phase artifacts

```
docs/reviews/phases/<PHASE>/
  PACKET.md          # implementer: work + reasoning + acceptance map
  KIMI_PROMPT.md     # generated or hand-tuned prompt for Kimi
  KIMI_REPORT.md     # Kimi final verdict (written by script / paste)
  TRIAGE.md          # implementer responses to findings
```

`<PHASE>` ∈ `0`, `A`, `B`, `C`, `D`, `E`, `F`, `G` (or later `product-N-p1`, …).

## Verdicts

| Verdict | Meaning | Next |
|---------|---------|------|
| `PASS` | Phase meets acceptance; ship | Close phase |
| `PASS_WITH_FOLLOWUPS` | OK with non-blocking debt | File follow-ups; close phase |
| `FAIL` | Blocking issues | Fix and re-run Kimi |
| `BLOCKED` | Needs operator decision | Ask user; then resume |

## Script

```powershell
cd C:\Users\divin\PROJECTS\HELIXFORGE
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\kimi_phase_review.ps1 -Phase A
```

Requires: `kimi` on PATH, network for Kimi account, work-dir = repo root.

## What Kimi must review

1. Diff / listed files for the phase  
2. Tests run and evidence  
3. PACKET reasoning (assumptions, trade-offs)  
4. Constitution: sovereignty, one-core, zero-trust, audit, secrets out of tree  
5. Overclaim risk (“completed” when only partial)  
6. Security regressions  

## Transcripts

Kimi sessions under this cwd archive to:

`C:\Users\divin\TRANSCRIPTS\HELIXFORGE\kimi\`
