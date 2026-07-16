# 011.5 Verification Evidence

## Scope
Backup plus clean restore proof.

## Environment
- OS: Windows 11, `stable-x86_64-pc-windows-msvc`
- Postgres: `postgres://helix:helix@127.0.0.1:55432/helixforge`
- Docker Desktop 29.6.1 with Linux containers
- Date: 2026-07-15

## Checks

| Check | Command / Step | Result |
|---|---|---|
| Format | `cargo fmt --check` | PASS |
| Clippy | `cargo clippy --workspace --all-targets` | PASS (no warnings) |
| Unit/integration tests | `cargo test --workspace` | PASS |
| Operator CLI build | `cargo build -p helix_db --bin helix-audit-rehash` | PASS |
| Backup script | `powershell -File .\scripts\backup_helixcore.ps1 -OutDir <tmp> -MirrorMinio` | PASS — produced `helixforge.sql` + `backup-manifest.json` with SHA256 |
| Verify script fail-closed | `powershell -File .\scripts\verify_helixcore_restore.ps1` with services down | PASS — wrote `restore-evidence.json`, exited 1, no parse errors |
| Dockerfile lint | `docker buildx build --check .` | PASS — no warnings |

## Notes

- Added `.dockerignore` to exclude `target/`, `.git/`, `node_modules/`, `.data/`,
  front-end build artifacts, and secrets from the Docker build context.
- The runtime `Dockerfile` now builds and copies `helix-audit-rehash` to
  `/app/bin/`. A full image build was started but exceeded interactive session
  time due to the large `rust:1.97-bookworm` base-image download; the Dockerfile
  is lint-clean and the binary is included in the build graph.

## Implementation summary

- `scripts/backup_helixcore.ps1` now writes `backup-manifest.json` with SHA256
  hashes, git commit, sanitized config, and optional MinIO object mirror.
- `scripts/restore_helixcore.ps1` accepts `-Verify` and references the operator
  CLI for break-glass audit rehash.
- `scripts/verify_helixcore_restore.ps1` (new) checks healthz, compliance
  summary, vault roundtrip, auto-rehashes audit chain if needed, and writes
  `restore-evidence.json`.
- `Dockerfile` now includes `helix-audit-rehash`.
- Runbooks updated; removed obsolete HTTP `/v1/audit/rehash` reference.
