# Next action

## Latest: HELIXCOMMERCE-DURABILITY

**Goal:** prove the Foundation Integrity durability gate on HelixCommerce —
third product through the gate (after helix-collab, helix-capital).

- Tests: `projects/helix-commerce/backend/src/main.rs`
  (`concurrent_cancels_single_winner`)
- Proof: `scripts/helix_commerce_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `commerce-durability` job
- Docs: `docs/goals/HELIXCOMMERCE_DURABILITY.md`, `DECISION_LOG.md`

### Scope

- concurrency proof: N concurrent cancels → exactly one winner, inventory
  restored once (oversell race already proven by
  `two_buyers_cannot_oversell_last_unit`)
- crash proof: acknowledged order survives a forced kill of the API
- restore proof: `commerce` schema dump roundtrip with equal counts + hashes
- ignored-test step added to durability CI jobs so race proofs run in CI

### Active goal

`HELIXCOMMERCE-DURABILITY` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCOMMERCE-DURABILITY is
the active goal. Add the concurrent_cancels_single_winner integration test;
create scripts/helix_commerce_durability.ps1 (forced-kill + restore proofs)
and the commerce-durability CI job (with the ignored-test step, also added to
capital-durability); prove it green on CI; record helix-commerce in
durability_gate_proven_products.
```
