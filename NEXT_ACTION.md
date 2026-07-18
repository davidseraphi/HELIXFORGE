# Next action

## Latest: HELIXCAPITAL-DURABILITY closed — second product through the gate

HELIXCAPITAL-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29662883748` is all green, including
the new **HelixCapital durability gate** job.

- Tests: `projects/helix-capital/backend/src/main.rs`
  (`concurrent_voids_single_winner`, `concurrent_journals_exact_balances`)
- Proof: `scripts/helix_capital_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `capital-durability` job
- Docs: `docs/goals/HELIXCAPITAL_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- concurrency proof: 8 racing voids → exactly one winner, one reversal;
  8 concurrent journals → exact summed balances (trial balance agrees)
- crash proof: acknowledged journal survives a forced kill of the API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-capital` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXCAPITAL-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 19 products.
