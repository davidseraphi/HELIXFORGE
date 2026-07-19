# HELIXEDU-DURABILITY

Prove the Foundation Integrity durability gate on HelixEdu: fresh crash,
concurrency, and restore, verified locally and in CI. Sixth product through
the gate (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`).

## Scope

Enrollment had a check-then-insert window: the published-course guard ran in
one statement and the INSERT in another, so a course unpublished in between
would silently accept enrollments. Withdraw used an unguarded read-then-
update. This packet makes both atomic and proves the gate.

## Definition of done

1. `EduRepo::enroll` inserts with an `INSERT ... SELECT` that requires the
   course to exist, be published, and not be deleted — one statement.
2. `EduRepo::withdraw_enrollment` is a single guarded `UPDATE` with
   `status <> 'withdrawn'` and `RETURNING` — no read-then-update window.
3. New ignored Postgres integration tests (run in the `edu-durability`
   CI job):
   - `concurrent_enroll_same_learner_single_winner` — N concurrent
     enrollments of one learner in one course produce exactly one success;
     the rest conflict; exactly one enrollment row exists.
   - `enroll_rejected_when_unpublished` — N concurrent enroll attempts on
     a draft course are all rejected; no enrollment leaks in.
4. `scripts/helix_edu_durability.ps1`:
   - create course, publish, enroll, progress, verify
   - acknowledge an enrollment, force-kill the API, restart, and verify the
     enrollment and progress are fully present
   - `pg_dump` of the `edu` schema restores into a scratch database with
     equal course/enrollment/history counts and equal content hashes
5. `edu-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
6. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29667121757` (**HelixEdu durability gate** job green)
- Proof script: `scripts/helix_edu_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on edu writes.
- Durability gates for other products (each needs its own named packet).
