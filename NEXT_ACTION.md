# Next action

## Latest: HELIXGRIDPRIME-DURABILITY closed — eighteenth product through the gate

HELIXGRIDPRIME-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29685116830` is all green, including
the new **HelixGrid Prime durability gate** job.

- Repo: `crates/helix-db/src/grid.rs` (atomic `create_child`
  INSERT...SELECT; guarded `take_offline` / `energize_site` /
  `bring_online` / `verify_reading` / `reject_reading`)
- Tests: `projects/helix-grid-prime/backend/src/main.rs`
  (`readings_rejected_on_deleted_site`, `concurrent_offline_single_winner`)
- Proof: `scripts/helix_grid_prime_durability.ps1` (forced-kill +
  restore)
- CI: `.github/workflows/ci.yml` `grid-durability` job
- Docs: `docs/goals/HELIXGRIDPRIME_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the reading INSERT; a site
  soft-deleted mid-flight can no longer leak readings
- offline is one guarded UPDATE (active + not deleted + NOT EXISTS draft
  reading); energize/online and verify/reject carry expected-from
  status in the WHERE
- concurrency proof: 8 racing creates on a deleted site all rejected; 8
  racing offlines → exactly one winner
- crash proof: acknowledged offline site survives a forced kill of the
  API
- restore proof: schema dump roundtrip with equal counts + content
  hashes
- `helix-grid-prime` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXGRIDPRIME-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 3 products.
