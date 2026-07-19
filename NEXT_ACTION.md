# Next action

## Latest: HELIXNOVALABS-DURABILITY closed — nineteenth product through the gate

HELIXNOVALABS-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29685681271` is all green, including
the new **HelixNova Labs durability gate** job.

- Repo: `crates/helix-db/src/nova.rs` (atomic `create_child`
  INSERT...SELECT; guarded `conclude_experiment` / `start_experiment` /
  `reopen_experiment` / `confirm_finding` / `reject_finding`)
- Tests: `projects/helix-nova-labs/backend/src/main.rs`
  (`findings_rejected_on_deleted_experiment`,
  `concurrent_conclude_single_winner`)
- Proof: `scripts/helix_nova_labs_durability.ps1` (forced-kill +
  restore)
- CI: `.github/workflows/ci.yml` `nova-durability` job
- Docs: `docs/goals/HELIXNOVALABS_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the finding INSERT; an
  experiment soft-deleted mid-flight can no longer leak findings
- conclude is one guarded UPDATE (running + not deleted + NOT EXISTS
  draft finding); start/reopen and confirm/reject carry expected-from
  status in the WHERE
- concurrency proof: 8 racing creates on a deleted experiment all
  rejected; 8 racing concludes → exactly one winner
- crash proof: acknowledged concluded experiment survives a forced kill
  of the API
- restore proof: schema dump roundtrip with equal counts + content
  hashes
- `helix-nova-labs` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXNOVALABS-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 2 products.
