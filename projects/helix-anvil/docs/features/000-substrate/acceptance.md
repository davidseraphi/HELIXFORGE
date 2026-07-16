# 000 — Substrate install · Acceptance

| # | Scenario / requirement | Evidence | Status |
|---|------------------------|----------|--------|
| 1 | Substrate files present | Root: AGENTS/VISION/BUILD_SPEC/PROJECT_STATE/…; tools/context; docs/features; Bug OS | **PASS** |
| 2 | Drift gate clean | `python tools/context/check_context_drift.py` → exit 0 (2026-07-15) | **PASS** |
| 3 | Standalone identity | `C:\Users\divin\PROJECTS\HELIXANVIL`; HelixForge catalog remains 21 products (anvil monorepo embed reverted) | **PASS** |
| 4 | Next slice 001 | `PROJECT_STATE.next_product_slice.packet_id` = `001-editor-kernel-design` | **PASS** |

## Evidence log

```
[bug-packets] validated 0 bug packet(s); 0 warning(s)
== HelixAnvil substrate drift check ==
   root: C:\Users\divin\PROJECTS\HELIXANVIL
  ok: PROJECT_STATE.json validates against schema (jsonschema)
  ok: canonical_docs / historical_docs paths checked
  ok: active_feature_packets paths + files checked
  ok: next_product_slice -> packet 001-editor-kernel-design present
  ok: NEXT_ACTION.md present
  ok: bug packets valid (0 warning(s))
  ok: document index in sync
  ok: context pack in sync

All substrate checks passed.
```
