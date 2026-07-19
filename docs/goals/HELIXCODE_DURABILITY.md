# HELIXCODE-DURABILITY

Prove the Foundation Integrity durability gate on HelixCode: fresh
crash, concurrency, and restore, verified locally and in CI. Twenty-first
and final product through the gate (after `helix-collab`,
`helix-capital`, `helix-commerce`, `helix-flow`, `helix-insights`,
`helix-edu`, `helix-well`, `helix-network`, `helix-forge-studio`,
`helix-synthbio`, `helix-lex-prime`, `helix-cura-prime`,
`helix-terra-prime`, `helix-climate-prime`, `helix-orbit-prime`,
`helix-quantum-forge`, `helix-vita-prime`, `helix-grid-prime`,
`helix-nova-labs`, `helix-pulse`).

## Scope

`finish_pipeline_run` and `finish_agent_job` wrote their terminal state
with an unguarded UPDATE whose result was discarded — two concurrent
finishes both reported success (last writer wins), and a finish could
silently overwrite a cancelled run. `create_workspace` and
`create_pipeline` relied on a read-first in the handler plus the foreign
key, so a deleted or foreign-tenant repo id produced a raw FK-violation
500 instead of a clean not-found. Separately, the API did not boot at
all under the current axum (`.nest_service("/", ...)` panics — the same
class of bug the flow packet found). This packet fixes all of these and
proves the gate.

## Definition of done

1. `CodeRepoStore::finish_pipeline_run` and `finish_agent_job` are
   single guarded UPDATEs with `AND finished_at IS NULL` and
   `RETURNING` — a concurrent finish loses with a conflict instead of
   overwriting.
2. `CodeRepoStore::create_workspace` and `create_pipeline` insert with
   an `INSERT ... SELECT` that requires the repo to exist for this
   tenant — one statement, clean not-found on violation.
3. The API boots: root nest replaced with `fallback_service` in
   `projects/helix-code/backend/src/main.rs`.
4. New ignored Postgres integration tests (run in the
   `code-durability` CI job), in a new `#[cfg(test)]` harness in
   `projects/helix-code/backend/src/main.rs`:
   - `concurrent_finish_pipeline_run_single_winner` — N concurrent
     finishes of one run produce exactly one success; the rest
     conflict; the run ends finished.
   - `concurrent_finish_agent_job_single_winner` — same for agent jobs.
   - `children_rejected_on_missing_repo` — workspace and pipeline
     creates against a nonexistent repo are rejected with not-found.
5. `scripts/helix_code_durability.ps1`:
   - create repo, workspace, pipeline, trigger a run to succeeded
   - acknowledge a finished run, force-kill the API, restart, and verify
     the run, repo, and workspace are fully present
   - `pg_dump` of the `code` schema restores into a scratch database
     with equal repo/workspace/run counts and equal content hashes
6. `code-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29687099450` (**HelixCode durability gate** job green)
- Proof script: `scripts/helix_code_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Ref compare-and-swap (expected-old-sha) on `upsert_ref` — a real
  lost-update window on the git plane, but a different gate shape; its
  own named packet if wanted.
- MAX+1 numbering races on issues/PRs/agent events (backstopped by
  UNIQUE constraints today).
- Audit/metering/NATS transactionality on code writes.
