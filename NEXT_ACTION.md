# Next action

## Latest: HELIXEDU-DURABILITY

**Goal:** prove the Foundation Integrity durability gate on HelixEdu —
sixth product through the gate (after helix-collab, helix-capital,
helix-commerce, helix-flow, helix-insights).

- Repo: `crates/helix-db/src/edu.rs` (atomic enroll INSERT...SELECT,
  guarded withdraw)
- Tests: `projects/helix-edu/backend/src/main.rs`
  (`concurrent_enroll_same_learner_single_winner`,
  `enroll_rejected_when_unpublished`)
- Proof: `scripts/helix_edu_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `edu-durability` job
- Docs: `docs/goals/HELIXEDU_DURABILITY.md`, `DECISION_LOG.md`

### Scope

- fix: published-course guard enforced inside the enroll INSERT; withdraw
  is a single guarded UPDATE
- concurrency proof: N concurrent enrollments of one learner → exactly one
  wins; enroll on a draft course always rejected
- crash proof: acknowledged enrollment survives a forced kill of the API
- restore proof: `edu` schema dump roundtrip with equal counts + hashes

### Active goal

`HELIXEDU-DURABILITY` — in progress.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXEDU-DURABILITY is the
active goal. Make enroll an atomic INSERT...SELECT and withdraw a guarded
UPDATE; add the enroll race proofs; create
scripts/helix_edu_durability.ps1 (forced-kill + restore proofs) and the
edu-durability CI job; prove it green on CI; record helix-edu in
durability_gate_proven_products.
```
