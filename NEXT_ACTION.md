# Next action

## Latest: HELIXSYNTHBIO-DURABILITY closed — tenth product through the gate

HELIXSYNTHBIO-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29669804701` is all green, including
the new **HelixSynthBio durability gate** job.

- Repo: `crates/helix-db/src/synthbio.rs` (atomic `create_child`
  INSERT...SELECT; guarded `approve_design` / `submit_design` /
  `return_design` / `transition_sim`)
- Tests: `projects/helix-synthbio/backend/src/main.rs`
  (`sims_rejected_on_deleted_design`, `concurrent_approve_single_winner`)
- Proof: `scripts/helix_synthbio_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `synthbio-durability` job
- Docs: `docs/goals/HELIXSYNTHBIO_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the sim INSERT; a design
  soft-deleted mid-flight can no longer leak sims
- approve is one guarded UPDATE (review + not deleted + EXISTS completed
  sim); submit/return and sim transitions carry expected-from status in
  the WHERE
- concurrency proof: 8 racing creates on a deleted design all rejected;
  8 racing approves → exactly one winner
- crash proof: acknowledged approved design survives a forced kill of the
  API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-synthbio` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXSYNTHBIO-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 11 products.
