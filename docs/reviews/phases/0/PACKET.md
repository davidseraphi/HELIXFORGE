# Phase 0 — implementer packet (bootstrap gate)

**Status:** ready for Kimi after evidence fill  
**Date:** 2026-07-14  
**Repo:** HELIXFORGE  
**Goal:** HELIXCORE-0-AG  

## Intent

Gate HelixCore **bootstrap** (feature `000`) before deep phases A–G: prove
acceptance scenarios still hold, list residual debt honestly, and get Kimi
sign-off that we are not overclaiming “foundation complete.”

## Changes (this phase)

| Path | Why |
|------|-----|
| `docs/goals/HELIXCORE_PHASES_0_AG.md` | Program goal + process |
| `docs/runbooks/phase-kimi-review.md` | Kimi gate runbook |
| `scripts/kimi_phase_review.ps1` | Automated Kimi review invoke |
| `docs/features/010-helix-core-deep/*` | Deep-core spec (A–G) |
| Prior work (reference) | Monorepo bootstrap, durable products 1–20 thin slices |

Phase 0 does **not** rewrite platform code; it **verifies and frames** debt.

## Commands run (evidence)

```text
# Mapping / archive
# TRANSCRIPTS\HELIXFORGE\grok active for this session

# Expected self-check before Kimi (operator or implementer):
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
cargo test --workspace
# smoke: docker compose up -d postgres; cargo run -p gateway
# GET /v1/catalog → 20 products
# GET /v1/me with X-Helix-Dev-User
```

*(Fill exact command output summaries when running the gate.)*

## Acceptance map (bootstrap 000)

| Criterion | Met? | Evidence |
|-----------|------|----------|
| Catalog has 20 products | yes (unit + prior smoke) | `shared_core` catalog test; PRODUCT_CATALOG len 20 |
| Local dev principal | yes | `X-Helix-Dev-User` + shared local tenant |
| Audit chain verifies | yes (unit) | `audit_log` chain tests |
| Postgres durable audit/meter when up | yes | `helix_db` migrations + prior smokes |
| Memory fallback when Postgres down | yes | service_kit path |
| Full AetherID / Ory production path | **no** | Phase A |
| Vault durable beyond memory | **no** | Phase C |
| Full OTEL / marketplace billing | **no** | Phase B/E |
| Gateway reverse-proxy products | **no** | Phase F |
| Infra production-ready | **no** | Phase G |

## Reasoning & trade-offs

- **Widen first:** products 1–20 got thin durable APIs so Core depth is the
  bottleneck, not product stubs.
- **Phase 0 as gate:** re-verify bootstrap instead of re-scaffolding; A–G own depth.
- **Kimi on every phase:** independent critique of reasoning and overclaim risk
  before portfolio-scale deep work on all 21 offerings.

## Known debt / follow-ups (feeds A–G)

1. A — Ory hybrid, scopes, residency tests  
2. B — Audit verify + billing summary APIs  
3. C — Vault durable / MinIO  
4. D — Agent Hub multi-step + NATS  
5. E — Metrics export  
6. F — Gateway routing or ADR  
7. G — Helm/Argo/TF polish  

## Request to Kimi

Review this packet, `docs/features/000-helix-core-bootstrap/`,  
`docs/features/010-helix-core-deep/requirements.md`, `constitution.md`,  
`AGENTS.md`, and `crates/service-kit` / `crates/shared-core` at a high level.

Verdict: is Phase 0 (bootstrap gate) honest and ready to proceed to A, or are
there blockers in the foundation claims?
