# Next action

## Active: HELIXCODE-DURABILITY — twenty-first and final product through the gate

Prove the Foundation Integrity durability gate on HelixCode: fresh
crash, concurrency, and restore, verified locally and in CI. Final
product (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`,
`helix-network`, `helix-forge-studio`, `helix-synthbio`,
`helix-lex-prime`, `helix-cura-prime`, `helix-terra-prime`,
`helix-climate-prime`, `helix-orbit-prime`, `helix-quantum-forge`,
`helix-vita-prime`, `helix-grid-prime`, `helix-nova-labs`,
`helix-pulse`).

Goal doc: `docs/goals/HELIXCODE_DURABILITY.md`.

### Scope

`finish_pipeline_run` / `finish_agent_job` wrote terminal state with
unguarded discarded UPDATEs — concurrent finishes both "won" and a
finish could overwrite a cancel. `create_workspace` / `create_pipeline`
relied on handler read-first + FK, producing 500s instead of clean
not-founds. The API also did not boot under current axum
(`.nest_service("/", ...)` panic). All fixed and proven.

### Definition of done

1. `finish_pipeline_run` / `finish_agent_job` are guarded UPDATEs with
   `AND finished_at IS NULL` + `RETURNING`; concurrent finish →
   conflict.
2. `create_workspace` / `create_pipeline` insert with
   `INSERT ... SELECT` against the tenant's repo.
3. API boots via `fallback_service` in
   `projects/helix-code/backend/src/main.rs`.
4. Ignored tests `concurrent_finish_pipeline_run_single_winner`,
   `concurrent_finish_agent_job_single_winner`,
   `children_rejected_on_missing_repo` pass locally and in CI.
5. `scripts/helix_code_durability.ps1` proves lifecycle, forced-kill
   survival, and schema restore roundtrip.
6. `code-durability` CI job in `.github/workflows/ci.yml`.
7. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Push the implementation and watch CI to green.
