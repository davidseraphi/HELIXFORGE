# 011.1 — Repository boundary, clean build, and native CI design

## Status

Activated for implementation on 2026-07-15 as the first child packet of the
Foundation Integrity umbrella.

## Outcome

The monorepo is under version control with protected review. The full Rust and
TypeScript workspace builds, formats, lints, and tests cleanly. The project no
longer forces a Windows-only host target. CI design covers native Windows,
macOS, and Linux runners and records artifact hashes.

## Allowed edits

- Initialize the root Git repository and `.gitignore` (founder decision already
  recorded).
- `.github/workflows/ci.yml` — add Windows, macOS, and Linux runners;
  separate jobs for format, clippy, test, helm, terraform, docker, and
  security; artifact hashing.
- `.gitignore` — exclude `target/`, `.data/`, `node_modules/`, `.env`, secrets,
  OS files, IDE files, and transcript paths.
- `.cargo/config.toml` and `rust-toolchain.toml` — remove the global Windows
  host target pin; document per-platform build paths.
- New scripts: `scripts/ci-build.ps1`, `scripts/ci-build.sh`,
  `scripts/ci-test.ps1`, `scripts/ci-test.sh`, `scripts/ci-smoke.ps1`, etc.
- Product source files may be touched **only** to resolve compiler, clippy, or
  formatting warnings; no behavioral changes.
- Living docs: `BUILD_SPEC.md`, `PROJECT_STATE.json`, `NEXT_ACTION.md`,
  `DECISION_LOG.md`, `AGENTS.md`, and this packet.

## Forbidden edits

- No product domain source changes.
- No migrations that alter existing data.
- No secret values, no `.env` files, no in-repo credentials.
- No starting long-running services or CI on production state.
- No changes to HelixAnvil code or location.

## EARS acceptance

### Repository boundary

- The system SHALL initialize a Git repository at the project root.
- The system SHALL provide a `.gitignore` that excludes build artifacts,
  runtime data, dependencies, secrets, and OS/IDE files.
- The system SHALL document a branch-protection plan requiring human review
  before merging to `main`.

### Clean build

- The system SHALL make `cargo build --workspace` succeed on a fresh clone.
- The system SHALL make `cargo test --workspace` succeed under default
  parallelism (no flaky failures).
- The system SHALL make `cargo clippy --workspace --all-targets -- -D warnings`
  succeed.
- The system SHALL make `cargo fmt --all -- --check` succeed.
- The system SHALL make `pnpm --filter @helixforge/console typecheck`,
  `@helixforge/helix-code-web typecheck`, and `@helixforge/helix-collab-web
  typecheck` succeed.

### Native CI design

- The CI workflow SHALL run on native Windows, macOS, and Linux runners.
- The CI workflow SHALL separate format, clippy, test, helm, terraform,
  docker, and security jobs.
- The CI workflow SHALL record artifact hashes and environment facts for every
  release candidate.
- The CI workflow SHALL fail closed on warnings (`RUSTFLAGS="-D warnings"`).

## Test plan

| Check | Command / evidence |
|---|---|
| Git initialized | `git status --short` shows tracked/untracked state |
| Clean build | `cargo build --workspace` |
| Serial tests | `cargo test --workspace -- --test-threads=1` |
| Parallel tests | `cargo test --workspace` (must be flake-free) |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Format | `cargo fmt --all -- --check` |
| TS checks | `pnpm --filter @helixforge/console typecheck` etc. |
| Helm | `helm lint infra/helm/helix-core` |
| Terraform | `terraform validate` in `infra/terraform/environments/{dev,prod}` |
| CI matrix | Dry-run or documented native Windows/macOS/Linux matrix |

## Known risks

- `cargo test --workspace` currently shows one flaky failure in
  `helix_code_api::domain::webhook_policy::tests::private_requires_webhook_flag_not_dev_headers`
  under default parallelism; it passes when run serially or in isolation. This
  must be root-caused and fixed before `011.1` closes.
- `cargo clippy --workspace --all-targets` currently emits warnings in
  `helix_collab_api` and `helix_code_api`; CI with `-D warnings` will fail
  until these are fixed.
- `terraform` is not installed on the current Windows dev machine; CI or another
  machine must reproduce `terraform validate`.

## Rollback / compensation

- Git history is append-only; a bad CI change can be reverted with a single
  commit revert.
- If the cross-platform matrix is unstable, the `fail-fast` flag can be disabled
  and per-platform jobs can be marked `continue-on-error` while retaining
  evidence.
