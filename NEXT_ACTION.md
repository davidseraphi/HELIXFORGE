# Next action

## Latest: HELIXCOMMERCE-FULL

**Goal:** move HelixCommerce from durable scaffold to full second-wave depth.

- Migration: `crates/helix-db/migrations/0040_commerce_depth.sql`
- Repo: `crates/helix-db/src/commerce.rs`
- API: `projects/helix-commerce/backend/src/main.rs`
- Smoke: `scripts/helix_commerce_smoke.ps1`
- CI: `.github/workflows/ci.yml` `commerce-smoke` job
- Docs: `projects/helix-commerce/README.md`, `DECISION_LOG.md`,
  `docs/goals/HELIXCOMMERCE_FULL.md`

### Scope

Exact multi-currency order truth + atomic stock/order writes:
- reject mixed-currency carts
- atomic inventory reservation on order create
- order cancel restores inventory
- two-buyer race test
- domain status planes + smoke test

### Active goal

`HELIXCOMMERCE-FULL` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCOMMERCE-FULL is the
active goal. Implement migration 0040, extend CommerceRepo with
mixed-currency guard and cancel, add routes and domain status planes,
write unit + race tests, create scripts/helix_commerce_smoke.ps1, add the
commerce-smoke CI job, and prove it green on CI.
```
