# Next action

## Latest: HELIXCURAPRIME-DURABILITY closed — twelfth product through the gate

HELIXCURAPRIME-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29670866072` is all green, including
the new **HelixCura Prime durability gate** job.

- Repo: `crates/helix-db/src/cura.rs` (atomic `create_child`
  INSERT...SELECT; guarded `discharge_case` / `activate_case` /
  `reopen_case` / `sign_note` / `void_note`; draft guard on
  `update_note` — signed notes stay immutable under race)
- Tests: `projects/helix-cura-prime/backend/src/main.rs`
  (`notes_rejected_on_deleted_case`, `concurrent_discharge_single_winner`)
- Proof: `scripts/helix_cura_prime_durability.ps1` (forced-kill +
  restore)
- CI: `.github/workflows/ci.yml` `cura-durability` job
- Docs: `docs/goals/HELIXCURAPRIME_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the note INSERT; a case
  soft-deleted mid-flight can no longer leak notes
- discharge is one guarded UPDATE (active + not deleted + NOT EXISTS
  draft note); activate/reopen/sign/void carry expected-from status in
  the WHERE
- signed-immutable holds under race: note edits require `status = 'draft'`
  in the UPDATE itself
- concurrency proof: 8 racing creates on a deleted case all rejected; 8
  racing discharges → exactly one winner
- crash proof: acknowledged discharged case survives a forced kill of the
  API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-cura-prime` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXCURAPRIME-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 9 products.
