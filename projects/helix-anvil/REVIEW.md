# HelixAnvil — Review Configuration

Status: canonical · Last updated: 2026-07-15

Four-tier review doctrine per `~/shared/AGENTS.md`.

## Tier 0 — Deterministic gates (always on, blocking)

```bash
python tools/context/check_context_drift.py
python tools/quality/validate_bug_packets.py
# when Rust lands:
# cargo test --workspace
# cargo clippy --workspace --all-targets
```

## Tier 1 — Single-pass review

- `/review` on push  
- `/security-review` on PR  

## Tier 2 — Multi-specialist

- `/review-pr` on PRs with large diffs  

## Tier 3 — Deep review (gated paths)

Mandatory highest-tier before user-facing release or when diff touches:

| Path pattern | Reason |
|---|---|
| `**/buffer/**`, future kernel crates | Document model correctness |
| Future crypto / signed-release paths | Trust |
| `constitution.md`, `PROJECT_STATE.json` schema shape | Substrate integrity |
| Secrets or auth wiring | Credential boundary |

## Threshold

Tier 3 also when diff exceeds ~400 LOC of product code (not doc-only substrate).
