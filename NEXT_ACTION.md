# Next action

## Latest: HELIXEDU-DURABILITY closed — sixth product through the gate

HELIXEDU-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29667121757` is all green, including
the new **HelixEdu durability gate** job.

- Repo: `crates/helix-db/src/edu.rs` (atomic enroll INSERT...SELECT,
  guarded withdraw)
- Tests: `projects/helix-edu/backend/src/main.rs`
  (`concurrent_enroll_same_learner_single_winner`,
  `enroll_rejected_when_unpublished`)
- Proof: `scripts/helix_edu_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `edu-durability` job
- Docs: `docs/goals/HELIXEDU_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- published-course guard enforced inside the enroll INSERT; withdraw is a
  single guarded UPDATE
- concurrency proof: 8 racing enrollments of one learner → exactly one
  wins; enroll on a draft course always rejected
- crash proof: acknowledged enrollment survives a forced kill of the API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-edu` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXEDU-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 15 products.
