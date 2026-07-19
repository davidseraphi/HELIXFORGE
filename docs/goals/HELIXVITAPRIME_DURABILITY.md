# HELIXVITAPRIME-DURABILITY

Prove the Foundation Integrity durability gate on HelixVita Prime: fresh
crash, concurrency, and restore, verified locally and in CI. Seventeenth
product through the gate (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`).

## Scope

`create_child` checked the parent study in a separate SELECT before the
cohort INSERT — a study soft-deleted in between silently gains a cohort.
`complete_study` counted draft cohorts and checked recruiting status in
separate statements from the UPDATE — a draft cohort created, or a
concurrent complete/terminate landing in between, breaks the "completed
means no draft cohorts" invariant. The recruit/terminate and
enroll/withdraw updates carry no expected-from status in their WHERE.
This packet folds the guards into the writes and proves the gate.

## Definition of done

1. `VitaRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the study to exist and not be deleted — one statement.
2. `VitaRepo::complete_study` is a single guarded `UPDATE` requiring
   `status = 'recruiting'`, not deleted, and `NOT EXISTS` a non-deleted
   draft cohort.
3. `VitaRepo::recruit_study`, `terminate_study`, `enroll_cohort`, and
   `withdraw_cohort` carry their expected-from status in the UPDATE
   `WHERE`.
4. New ignored Postgres integration tests (run in the `vita-durability`
   CI job):
   - `cohorts_rejected_on_deleted_study` — after soft-deleting a study,
     N concurrent cohort creates are all rejected; no cohort leaks in.
   - `concurrent_complete_single_winner` — N concurrent completes of one
     recruiting study produce exactly one success; the rest are
     rejected; the study ends completed.
5. `scripts/helix_vita_prime_durability.ps1`:
   - create study, recruit, cohort, enroll, complete, verify
   - acknowledge a completed study, force-kill the API, restart, and
     verify the study and cohort are fully present
   - `pg_dump` of the `vita` schema restores into a scratch database
     with equal study/cohort counts and equal content hashes
6. `vita-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Active**

## Out of scope

- Audit/metering/NATS transactionality on vita writes.
- Durability gates for other products (each needs its own named packet).
