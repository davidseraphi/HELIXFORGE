# Next action

## Latest: HELIXFORGESTUDIO-DURABILITY closed — ninth product through the gate

HELIXFORGESTUDIO-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29669148679` is all green, including
the new **HelixForge Studio durability gate** job.

- Repo: `crates/helix-db/src/studio.rs` (atomic `create_child`
  INSERT...SELECT; guarded `publish_app` / `unpublish_app` /
  `archive_page` / `reopen_page`)
- Tests: `projects/helix-forge-studio/backend/src/main.rs`
  (`pages_rejected_on_deleted_app`, `concurrent_publish_single_winner`)
- Proof: `scripts/helix_forge_studio_durability.ps1` (forced-kill +
  restore)
- CI: `.github/workflows/ci.yml` `forge-studio-durability` job
- Docs: `docs/goals/HELIXFORGESTUDIO_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the page INSERT; an app
  soft-deleted mid-flight can no longer leak pages
- publish is one guarded UPDATE (draft + not deleted + EXISTS page);
  unpublish/archive/reopen carry expected-from status in the WHERE
- concurrency proof: 8 racing creates on a deleted app all rejected; 8
  racing publishes → exactly one winner
- crash proof: acknowledged published app survives a forced kill of the
  API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-forge-studio` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXFORGESTUDIO-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 12 products.
