# Next action

## Latest: HELIXCOMMERCE-DURABILITY closed — third product through the gate

HELIXCOMMERCE-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29664024211` is all green, including
the new **HelixCommerce durability gate** job.

- Fix: `crates/helix-db/src/commerce.rs` (`cancel_order` loads items inside
  its transaction — the race proof caught a pool-exhaustion deadlock)
- Tests: `projects/helix-commerce/backend/src/main.rs`
  (`concurrent_cancels_single_winner`; oversell race already covered)
- Proof: `scripts/helix_commerce_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `commerce-durability` job (durability jobs
  now also run the ignored integration tests)
- Docs: `docs/goals/HELIXCOMMERCE_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- deadlock fix: cancel_order no longer fetches items outside its tx
- concurrency proof: 8 racing cancels → exactly one winner, inventory
  restored once
- crash proof: acknowledged order survives a forced kill of the API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-commerce` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXCOMMERCE-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 18 products.
