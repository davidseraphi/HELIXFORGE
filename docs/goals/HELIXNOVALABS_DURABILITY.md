# HELIXNOVALABS-DURABILITY

Prove the Foundation Integrity durability gate on HelixNova Labs: fresh
crash, concurrency, and restore, verified locally and in CI. Nineteenth
product through the gate (after `helix-collab`, `helix-capital`,
`helix-commerce`, `helix-flow`, `helix-insights`, `helix-edu`,
`helix-well`, `helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`,
`helix-vita-prime`, `helix-grid-prime`).

## Scope

`create_child` checked the parent experiment in a separate SELECT before
the finding INSERT — an experiment soft-deleted in between silently
gains a finding. `conclude_experiment` counted draft findings and
checked running status in separate statements from the UPDATE — a draft
finding created, or a concurrent conclude/reopen landing in between,
breaks the "concluded means no draft findings" invariant. The
start/reopen and confirm/reject updates carry no expected-from status
in their WHERE. This packet folds the guards into the writes and proves
the gate.

## Definition of done

1. `NovaRepo::create_child` inserts with an `INSERT ... SELECT` that
   requires the experiment to exist and not be deleted — one statement.
2. `NovaRepo::conclude_experiment` is a single guarded `UPDATE`
   requiring `status = 'running'`, not deleted, and `NOT EXISTS` a
   non-deleted draft finding.
3. `NovaRepo::start_experiment`, `reopen_experiment`, `confirm_finding`,
   and `reject_finding` carry their expected-from status in the UPDATE
   `WHERE`.
4. New ignored Postgres integration tests (run in the `nova-durability`
   CI job):
   - `findings_rejected_on_deleted_experiment` — after soft-deleting an
     experiment, N concurrent finding creates are all rejected; no
     finding leaks in.
   - `concurrent_conclude_single_winner` — N concurrent concludes of one
     running experiment produce exactly one success; the rest are
     rejected; the experiment ends concluded.
5. `scripts/helix_nova_labs_durability.ps1`:
   - create experiment, start, finding, confirm, conclude, verify
   - acknowledge a concluded experiment, force-kill the API, restart,
     and verify the experiment and finding are fully present
   - `pg_dump` of the `nova` schema restores into a scratch database
     with equal experiment/finding counts and equal content hashes
6. `nova-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29685681271` (**HelixNova Labs durability gate** job green)
- Proof script: `scripts/helix_nova_labs_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on nova writes.
- Durability gates for other products (each needs its own named packet).
