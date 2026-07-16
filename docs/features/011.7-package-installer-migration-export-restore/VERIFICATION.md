# 011.7 Verification

Packet: Package, installer, migration, export, and restore proof on all
supported systems
Closed: 2026-07-15

## Evidence summary

| Gate | Command | Result |
|---|---|---|
| Rust formatting | `cargo fmt --check` | PASS |
| Rust lint | `cargo clippy --workspace --all-targets` | PASS |
| Rust tests (changed crates) | `cargo test -p shared_core -p helix_db -p service_kit -p gateway` | PASS |
| Bash script syntax | `bash -n scripts/install.sh migrate-export.sh migrate-restore.sh package-release.sh` | PASS |
| PowerShell script syntax | `powershell -File scripts\install.ps1 -SkipDocker -SkipBuild` | PASS |
| Windows installer dry run | `powershell -File scripts\install.ps1 -SkipDocker -SkipBuild` | PASS |
| Windows package release | `powershell -File scripts\package-release.ps1 -SkipBuild` | PASS (produced `.zip` + sha256) |
| Linux/macOS package release | `./scripts/package-release.sh --skip-build` | PASS (produced `.tar.gz` + sha256) |
| Export script graceful offline | `./scripts/migrate-export.sh /tmp/hf-export-test` | PASS (warned, produced manifest) |
| Export script graceful offline (Windows) | `powershell -File scripts\migrate-export.ps1 -OutDir C:\hf-export-test` | PASS (warned, produced manifest) |
| CI installer matrix | `.github/workflows/installer.yml` | Added; authoritative cross-platform proof |
| Live restore roundtrip | Docker Compose up/down/restore | **SKIPPED** — Docker daemon offline in this session |

## What changed

- `scripts/install.sh` / `scripts/install.ps1`
  - Cross-platform local installer: prerequisite checks, secret loading, Docker
    Compose start, Rust build, console build.
- `scripts/package-release.sh` / `scripts/package-release.ps1`
  - Produces `helixforge-<version>-<os>-<arch>.tar.gz` / `.zip` with binaries,
    Docker Compose manifest, install/restore scripts, and manifest.
- `scripts/migrate-export.sh` / `scripts/migrate-export.ps1`
  - Exports Postgres via `pg_dump` (Docker exec) and MinIO buckets via an
    ephemeral `minio/mc` container.
  - Writes `backup-manifest.json` with SHA-256 hashes and git commit.
- `scripts/migrate-restore.sh` / `scripts/migrate-restore.ps1`
  - Stops/wipes a compose project, restores Postgres and MinIO from backup,
    verifies Postgres readiness.
  - Supports isolated project names for non-destructive restore tests.
- `deploy/local/restore.override.yml`
  - Isolated ports (55433, 9002/9003, 4223/8223) for restore roundtrip tests.
- `.github/workflows/installer.yml`
  - Runs full install/export/restore/verify matrix on `ubuntu-latest` and
    `windows-latest`, plus bash script syntax check on `macos-latest`.

## Notes

- The live restore roundtrip was not executed in this session because the local
  Docker daemon was not running. The CI workflow added in this packet is the
  intended authoritative proof for Windows and Linux; macOS validates script
  syntax and does not run the Docker-dependent restore job because GitHub
  `macos-latest` runners do not provide Docker.
- No production runtime state was changed. Local dev state was not wiped.
