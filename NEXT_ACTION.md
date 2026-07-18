# Next action

## Latest: HELIXCOLLAB-DURABILITY closed — first product through the gate

HELIXCOLLAB-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29661659103` is all green, including
the new **HelixCollab durability gate** job.

- Repo: `crates/helix-db/src/collab.rs` (atomic create),
  `crates/helix-db/src/collab_sovereign.rs` (single-INSERT attachment)
- Tests: `projects/helix-collab/backend/src/domain/documents.rs`
  (`concurrent_patches_single_winner`, `concurrent_creates_never_torn`)
- Proof: `scripts/helix_collab_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `collab-durability` job
- Docs: `docs/goals/HELIXCOLLAB_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- atomic document create (document + initial revision in one transaction)
- single-statement attachment register
- concurrency proof: 8 racing patches → exactly one winner
- crash proof: acknowledged write survives a forced kill of the API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-collab` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXCOLLAB-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 20 products; idempotency keys on collab writes; audit/NATS/
outbox transactionality on collab writes.
