# Next action

## Latest: HELIXORBITPRIME-DURABILITY closed — fifteenth product through the gate

HELIXORBITPRIME-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29672257327` is all green, including
the new **HelixOrbit Prime durability gate** job.

- Repo: `crates/helix-db/src/orbit.rs` (atomic `create_child`
  INSERT...SELECT; guarded `decommission_asset` / `commission_asset` /
  `recommission_asset` / `transition_pass`)
- Tests: `projects/helix-orbit-prime/backend/src/main.rs`
  (`passes_rejected_on_deleted_asset`,
  `concurrent_decommission_single_winner`)
- Proof: `scripts/helix_orbit_prime_durability.ps1` (forced-kill +
  restore)
- CI: `.github/workflows/ci.yml` `orbit-durability` job
- Docs: `docs/goals/HELIXORBITPRIME_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the pass INSERT; an asset
  soft-deleted mid-flight can no longer leak passes
- decommission is one guarded UPDATE (active + not deleted + NOT EXISTS
  draft or planned pass); commission/recommission and pass transitions
  carry expected-from status in the WHERE
- concurrency proof: 8 racing creates on a deleted asset all rejected; 8
  racing decommissions → exactly one winner
- crash proof: acknowledged decommissioned asset survives a forced kill
  of the API
- restore proof: schema dump roundtrip with equal counts + content
  hashes
- `helix-orbit-prime` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXORBITPRIME-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 6 products.
