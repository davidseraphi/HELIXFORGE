# Next action

## Latest: HELIXTERRAPRIME-DURABILITY closed — thirteenth product through the gate

HELIXTERRAPRIME-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29671334631` is all green, including
the new **HelixTerra Prime durability gate** job.

- Repo: `crates/helix-db/src/terra.rs` (atomic `create_child`
  INSERT...SELECT; guarded `retire_field` / `activate_field` /
  `reopen_field` / `confirm_observation` / `dismiss_observation`)
- Tests: `projects/helix-terra-prime/backend/src/main.rs`
  (`observations_rejected_on_deleted_field`,
  `concurrent_retire_single_winner`)
- Proof: `scripts/helix_terra_prime_durability.ps1` (forced-kill +
  restore)
- CI: `.github/workflows/ci.yml` `terra-durability` job
- Docs: `docs/goals/HELIXTERRAPRIME_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the observation INSERT; a
  field soft-deleted mid-flight can no longer leak observations
- retire is one guarded UPDATE (active + not deleted + NOT EXISTS draft
  observation); activate/reopen/confirm/dismiss carry expected-from
  status in the WHERE
- concurrency proof: 8 racing creates on a deleted field all rejected; 8
  racing retires → exactly one winner
- crash proof: acknowledged retired field survives a forced kill of the
  API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-terra-prime` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXTERRAPRIME-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 8 products.
