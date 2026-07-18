# Next action

## Latest: HELIXCAPITAL-DURABILITY

**Goal:** prove the Foundation Integrity durability gate on HelixCapital —
second product through the gate (after helix-collab).

- Tests: `projects/helix-capital/backend/src/main.rs`
  (`concurrent_voids_single_winner`, `concurrent_journals_exact_balances`)
- Proof: `scripts/helix_capital_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `capital-durability` job
- Docs: `docs/goals/HELIXCAPITAL_DURABILITY.md`, `DECISION_LOG.md`

### Scope

Journal writes are already transactional; this packet proves the gate:
- concurrency proof: N concurrent voids → exactly one winner, one reversal;
  N concurrent journals → exact summed balances
- crash proof: acknowledged journal survives a forced kill of the API
- restore proof: `capital` schema dump roundtrip with equal counts + hashes

### Active goal

`HELIXCAPITAL-DURABILITY` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCAPITAL-DURABILITY is the
active goal. Add concurrent_voids_single_winner and
concurrent_journals_exact_balances integration tests; create
scripts/helix_capital_durability.ps1 (forced-kill + restore proofs) and the
capital-durability CI job; prove it green on CI; record helix-capital in
durability_gate_proven_products.
```
