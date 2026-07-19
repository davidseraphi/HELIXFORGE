# Next action

## Latest: HELIXCLIMATEPRIME-DURABILITY closed — fourteenth product through the gate

HELIXCLIMATEPRIME-DURABILITY is complete. The implementation passed
local verification and GitHub Actions run `29671780109` is all green,
including the new **HelixClimate Prime durability gate** job.

- Repo: `crates/helix-db/src/climate.rs` (atomic `create_child`
  INSERT...SELECT; guarded `archive_scenario` / `activate_scenario` /
  `reopen_scenario` / `assess_score` / `dismiss_score`)
- Tests: `projects/helix-climate-prime/backend/src/main.rs`
  (`scores_rejected_on_deleted_scenario`,
  `concurrent_archive_single_winner`)
- Proof: `scripts/helix_climate_prime_durability.ps1` (forced-kill +
  restore)
- CI: `.github/workflows/ci.yml` `climate-durability` job
- Docs: `docs/goals/HELIXCLIMATEPRIME_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the score INSERT; a scenario
  soft-deleted mid-flight can no longer leak scores
- archive is one guarded UPDATE (active + not deleted + NOT EXISTS draft
  score); activate/reopen/assess/dismiss carry expected-from status in
  the WHERE
- concurrency proof: 8 racing creates on a deleted scenario all
  rejected; 8 racing archives → exactly one winner
- crash proof: acknowledged archived scenario survives a forced kill of
  the API
- restore proof: schema dump roundtrip with equal counts + content
  hashes
- `helix-climate-prime` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXCLIMATEPRIME-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 7 products.
