# 000 — Substrate install · Design

## Approach

Copy and fill `~/shared/substrate/` templates into `C:\Users\divin\PROJECTS\HELIXANVIL`.
Port Bug OS from `~/shared/bug-os/`. Wire bug validator into Tier-0 drift check.
Regenerate document index + context pack until drift exits clean.

## Architecture decisions (this packet only)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Project home | Standalone `PROJECTS/HELIXANVIL` | Substrate new-project protocol; not HelixForge product scaffold |
| Stage | `substrate` | No product code yet |
| First packets | 000 install + 001 design planned | Separate install from kernel design |
| Stack default | Documented as target Rust/native; not installed | Avoid premature binary in substrate stage |

## Risks

- Agents may re-attempt monorepo embed — mitigated by constitution + do_not_start + HelixForge decision log correction.
- Drift fails if context pack not regenerated after doc edits — always re-run index + pack before drift.

## Out of scope details

GUI toolkit, rope library choice, LSP — **001**.
