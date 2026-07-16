# 011.5 — Backup plus clean restore proof

## Status

Closed on 2026-07-15. Implementation, tests, script validation, clippy, fmt, and full-workspace tests passed. Evidence recorded in `VERIFICATION.md`.

## Outcome

HelixCore can be backed up, restored to a clean environment, and proven healthy
after restore. The proof is repeatable, recorded, and does not depend on
production secrets being stored in the backup.

## Scope

This packet covers the local/Foundation-Integrity proof only:

1. **Backup completeness**
   - Postgres logical dump of `helixforge`.
   - MinIO object mirror for the `helixforge` bucket when the MinIO client is
     installed.
   - Non-secret manifest with SHA256 hashes, timestamp, git commit, and sanitized
     config.
2. **Restore procedure**
   - Restore Postgres from a dump.
   - Optional break-glass audit-chain rehash using the restricted operator CLI.
3. **Restore verification**
   - All 6 core services report `/healthz` OK.
   - `GET /v1/compliance/summary` reports `audit_chain_verified=true` and
     `postgres_durable=true`.
   - Vault roundtrip (put + get a test secret) succeeds.
   - Evidence is written to `restore-evidence.json`.
4. **Container / operator CLI**
   - Include `helix-audit-rehash` in the runtime Docker image so restored K8s
     deployments can repair audit-chain drift without a separate build.
5. **Documentation**
   - Update `docs/runbooks/backup-dr.md` with the new scripts and verification
     steps.
   - Fix `docs/runbooks/helixcore-smoke.md` to reference the operator CLI instead
     of the removed HTTP rehash endpoint.

Out of scope for this packet: managed PITR, cross-region replication, automated
K8s CronJob/Job templates, Geo-DNS, active-active DR.

## Allowed edits

- `scripts/backup_helixcore.ps1`
- `scripts/restore_helixcore.ps1`
- `scripts/verify_helixcore_restore.ps1` (new)
- `Dockerfile`
- `docs/runbooks/backup-dr.md`
- `docs/runbooks/helixcore-smoke.md`
- This packet directory.

## Forbidden edits

- No product-domain API changes.
- No migrations that change existing data.
- No secret values in scripts or runbooks.
- No production state changes except through the documented restore scripts.

## EARS acceptance

### Backup

- The system SHALL write a Postgres dump and a non-secret manifest with SHA256
  hashes for every backup.
- The system SHALL mirror MinIO object data when `mc` is available.
- The system SHALL keep secrets (`~/Desktop/.keys/helixforge/`) out of the
  backup and document offline handling.

### Restore

- The system SHALL restore the Postgres dump into a fresh data plane.
- The system SHALL support break-glass audit-chain rehash via the operator CLI
  when timestamps drift.

### Verification

- The system SHALL verify that all 6 core services are healthy.
- The system SHALL verify `audit_chain_verified=true` and
  `postgres_durable=true` from the compliance summary.
- The system SHALL verify a vault secret roundtrip.
- The system SHALL record verification results to `restore-evidence.json`.

### Container

- The runtime Docker image SHALL include the `helix-audit-rehash` binary.

## Test plan

| Check | Evidence |
|---|---|
| Backup writes manifest with SHA256 hashes | Run `backup_helixcore.ps1`; inspect `backup-manifest.json` |
| Backup mirrors MinIO objects when `mc` present | Run with `-MirrorMinio`; inspect `minio-mirror/` directory |
| Restore runs and verifies | Run `restore_helixcore.ps1 -DumpFile <path> -Verify`; inspect `restore-evidence.json` |
| Audit rehash CLI is in Docker image | Build image; `docker run --rm helixforge/helix-core:0.1.0 helix-audit-rehash --help` |
| Runbooks are accurate | Manual review; no references to removed `/v1/audit/rehash` HTTP endpoint |

## Dependencies

- `011.1`–`011.4` closed.
- `helix-audit-rehash` operator CLI (`crates/helix-db/src/bin/helix_audit_rehash.rs`).
- `GET /v1/compliance/summary` and vault endpoints.

## Rollback / compensation

- Script changes are additive; old invocations of `backup_helixcore.ps1` and
  `restore_helixcore.ps1` continue to work.
- Dockerfile change adds one binary; rollback is removing the build/copy lines.
