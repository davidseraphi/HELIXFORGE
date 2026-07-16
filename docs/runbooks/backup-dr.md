# HelixCore backup & disaster recovery

## What to protect

| Asset | Location | RPO guidance |
|-------|----------|--------------|
| Postgres (audit, meter, vault ciphertext, products) | Docker volume / managed PG | ≤ 1 h (enterprise target) |
| MinIO objects | MinIO volume / S3 | ≤ 1 h |
| Secrets master key | `~/Desktop/.keys/helixforge/` only | offline + HSM in prod |
| Ory Kratos/Hydra DBs | same Postgres cluster (`kratos`, `hydra`) | with PG dump |
| Helm/Git config | monorepo | Git |

## Backup (local)

```powershell
cd C:\Users\divin\PROJECTS\HELIXFORGE
powershell -File .\scripts\backup_helixcore.ps1
```

Produces:

- `helixforge.sql` — logical dump  
- `backup-manifest.json` — non-secret inventory with SHA256 hashes and git commit
- `minio-listing.txt` — object inventory when `mc` is installed

To also mirror object bytes (requires MinIO client `mc`):

```powershell
powershell -File .\scripts\backup_helixcore.ps1 -MirrorMinio
```

This adds `minio-mirror.zip` with a SHA256 hash in the manifest.

**Also copy** secrets directory offline (never into git).

## Restore

```powershell
docker compose up -d postgres nats minio minio-init
powershell -File .\scripts\restore_helixcore.ps1 -DumpFile <path>\helixforge.sql
# restart core services
```

If audit verify fails after restore, use the restricted operator CLI:

```powershell
$env:HELIX_AUDIT_REHASH_APPROVED = "1"
cargo run -q -p helix_db --bin helix-audit-rehash -- --approve
```

## Restore verification

```powershell
powershell -File .\scripts\verify_helixcore_restore.ps1 -OutFile .\restore-evidence.json
```

Checks:

- All 6 core services report `/healthz` OK.
- `GET /v1/compliance/summary` shows `audit_chain_verified=true` and
  `postgres_durable=true`.
- Vault secret roundtrip succeeds.
- Runs the operator rehash CLI automatically if the audit chain drifts.
- Writes `restore-evidence.json` with timestamps, results, and any failures.

For a combined restore + verify:

```powershell
powershell -File .\scripts\restore_helixcore.ps1 -DumpFile <path>\helixforge.sql -Verify
```

## DR runbook (enterprise)

1. **Failover region**: provision empty stack (compose/K8s) with same residency tag.  
2. **Restore latest PG dump** + MinIO `mc mirror` from cold storage.  
3. **Inject secrets** from sealed-secrets / external secrets (not from backup of repo).  
4. **Rotate vault master** only if breach suspected — record via `POST /v1/keys/rotate-meta`.  
5. **Verify**: run `verify_helixcore_restore.ps1` and confirm `passed=true`.  
6. **DNS / ingress** cutover; keep old region read-only 24h.  

## Continuous backup (local / bare metal)

```powershell
# Terminal or Task Scheduler — dumps every 60 minutes, retains 48
powershell -File .\scripts\continuous_backup.ps1 -IntervalMinutes 60 -RetainCount 48

# Object versioning (one-time)
powershell -File .\scripts\enable_minio_versioning.ps1
```

`continuous_backup.ps1` also runs `mc mirror` into each snapshot when `mc` is installed.

## Container image

The runtime Docker image includes the `helix-audit-rehash` binary at
`/app/bin/helix-audit-rehash`, so a restored K8s deployment can repair audit-chain
drift without a separate Rust build.

## Production recommendations

- Continuous WAL archiving (Postgres) or managed PITR.  
- MinIO versioning (`enable_minio_versioning.ps1`) + cross-region `mc mirror` for Frontier blobs.  
- Quarterly restore drill; document actual RTO.  
- Separate backup encryption key from vault master key.  


## Out of scope (app layer)

- Multi-region active-active write paths (later).  
- Automated Geo-DNS (infra).  
