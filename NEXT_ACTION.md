# Next action

## Latest: HELIXCOLLAB-DURABILITY

**Goal:** prove the Foundation Integrity durability gate on HelixCollab, the
lead product — first product through the gate.

- Repo: `crates/helix-db/src/collab.rs` (atomic create),
  `crates/helix-db/src/collab_sovereign.rs` (single-INSERT attachment)
- API: `projects/helix-collab/backend/src/domain/documents.rs` (race tests),
  `projects/helix-collab/backend/src/domain/sovereign.rs` (drop UPDATE)
- Smoke: `scripts/helix_collab_durability.ps1`
- CI: `.github/workflows/ci.yml` `collab-durability` job
- Docs: `docs/goals/HELIXCOLLAB_DURABILITY.md`, `DECISION_LOG.md`

### Scope

- atomic document create (document + initial revision in one transaction)
- single-statement attachment register (`body_stored` in the INSERT)
- concurrency proof: N racing patches → exactly one winner
- crash proof: concurrent creates never torn; acknowledged write survives a
  forced kill of the API
- restore proof: `collab` schema dump restores with equal counts + hashes

### Active goal

`HELIXCOLLAB-DURABILITY` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCOLLAB-DURABILITY is the
active goal. Make create_document_full_ex transactional and register_attachment
single-statement; add concurrent_patches_single_winner and
concurrent_creates_never_torn integration tests; create
scripts/helix_collab_durability.ps1 (forced-kill + restore proofs) and the
collab-durability CI job; prove it green on CI; record helix-collab in
durability_gate_proven_products.
```
