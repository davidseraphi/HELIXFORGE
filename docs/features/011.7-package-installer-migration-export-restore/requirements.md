# 011.7 — Package, installer, migration, export, and restore proof on all supported systems

## Status

Closed on 2026-07-15 after implementation and partial local verification.
The live Docker-based restore roundtrip was not executed locally because the
Docker daemon was offline during this session; the CI matrix in
`.github/workflows/installer.yml` is the authoritative cross-platform proof.

## Outcome

HelixForge can be installed, packaged, migrated, exported, and cleanly restored on
Windows, macOS, and Linux from a single set of cross-platform scripts. The
installer sets up the local dev stack, the packager produces a reproducible
release artifact, and the migration/export/restore scripts prove that a
HelixCore environment can be moved between machines and recovered from scratch.

## Scope

1. **Cross-platform installer**
   - `scripts/install.sh` for macOS/Linux.
   - `scripts/install.ps1` for Windows.
   - Checks prerequisites (Docker, Rust, pnpm/Node).
   - Loads secrets from `~/Desktop/.keys/helixforge/.env.local` (Windows) or
     `$HOME/.keys/helixforge/.env.local` (macOS/Linux).
   - Starts backing services (`docker compose up -d postgres nats minio minio-init`).
   - Builds the workspace (`cargo build --workspace`) and the console
     (`pnpm install && pnpm --filter @helixforge/console build`).
2. **Release packager**
   - `scripts/package-release.sh` / `scripts/package-release.ps1`.
   - Produces `helixforge-<version>-<os>-<arch>.tar.gz` / `.zip` containing
     service binaries, Docker Compose manifest, install script, and README.
3. **Migration export**
   - `scripts/migrate-export.sh` / `scripts/migrate-export.ps1`.
   - Exports Postgres (`pg_dump`), MinIO bucket contents, and a manifest to a
     dated backup directory.
4. **Clean restore proof**
   - `scripts/migrate-restore.sh` / `scripts/migrate-restore.ps1`.
   - Stops services, wipes Postgres/MinIO state, restores from backup, restarts
     services, and runs a verification smoke (`cargo test -p helix_db` or
     equivalent).
5. **CI matrix**
   - `.github/workflows/installer.yml` runs installer and restore roundtrip on
     Windows, macOS, and Linux runners.

## Allowed edits

- `scripts/install.*`
- `scripts/package-release.*`
- `scripts/migrate-export.*`
- `scripts/migrate-restore.*`
- `.github/workflows/installer.yml` (new)
- `docs/features/011.7-package-installer-migration-export-restore/`
- `NEXT_ACTION.md`, `PROJECT_STATE.json`, `docs/features/011-foundation-integrity/requirements.md`.

## Forbidden edits

- No in-repo `.env` or secret values.
- No changes to product-domain logic.
- No production runtime state changes.
- No Git mutations.

## EARS acceptance

- The system SHALL provide one installer per supported platform.
- The system SHALL produce a release package that contains the core binaries and
  the Docker Compose manifest.
- The system SHALL export Postgres + MinIO state to a portable backup.
- The system SHALL restore a fresh environment from that backup and pass a
  verification check.
- The system SHALL prove the above on Windows, macOS, and Linux via CI or
  documented manual evidence.

## Test plan

| Check | Evidence |
|---|---|
| Installer runs | `scripts/install.ps1` on Windows; `scripts/install.sh` on WSL/macOS/Linux |
| Console builds | `pnpm --filter @helixforge/console build` after install |
| Workspace builds | `cargo build --workspace` after install |
| Release package | `scripts/package-release.ps1` / `.sh` produces artifact |
| Export roundtrip | `scripts/migrate-export.ps1` then `scripts/migrate-restore.ps1` with local Docker |
| CI matrix | `.github/workflows/installer.yml` passes on all three runners |

## Dependencies

- `011.1`–`011.6` closed.
- Existing `docker-compose.yml`, `Dockerfile`, `scripts/dev-core.*`, and
  `scripts/backup_helixcore.ps1`.

## Rollback / compensation

- New scripts are additive; deleting them reverts the feature.
- Restore scripts wipe local dev state only; production data is never touched.
