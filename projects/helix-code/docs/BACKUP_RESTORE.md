# HelixCode backup & restore

## What to back up

| Data | Location |
|------|----------|
| Metadata | Postgres schemas `code.*`, `helix_core.*` (audit/meter/acl) |
| Git objects | `HELIX_CODE_REPO_ROOT` (default `.data/helix-code/repos`) |
| Sealed + CI artifacts | MinIO bucket keys `code/{tenant}/…` |
| MLS user blobs / key backups | Postgres `code.mls_*` |

## Backup (local docker Postgres example)

```powershell
docker exec -t <postgres> pg_dump -U helix helixforge > helixforge.sql
# MinIO: mc mirror local/helix-objects ./backup/minio
# Git: robocopy .data\helix-code\repos .\backup\repos /MIR
```

## Restore

1. Restore Postgres dump.
2. Restore MinIO prefix `code/`.
3. Restore bare repo directory tree.
4. Restart `helix_code_api` (migrations are idempotent).

## HA notes

- API is stateless for REST; LSP sessions and terminals are in-memory — use sticky sessions or accept reconnect.
- Shared Postgres + MinIO + NATS enable multi-instance API.
