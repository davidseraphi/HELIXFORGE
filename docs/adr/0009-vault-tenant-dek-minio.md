# ADR-0009: Per-tenant vault DEK + MinIO object plane

## Status

Accepted — 2026-07-14

## Context

HelixCore secrets need stronger isolation than a single global AES key.
Large blobs belong in MinIO (S3), not Postgres rows.

## Decision

1. **HVA2 envelope**: DEK = SHA-256(master || `helixforge-vault-tenant-dek-v1` || tenant_id).
2. New puts use HVA2; open tries HVA2 then HVA1 then legacy XOR.
3. Vault-service can **PUT sealed bytes to MinIO** when `value_b64` is supplied;
   metadata remains in `helix_core.vault_objects`.
4. Credentials: `MINIO_ACCESS_KEY` / `MINIO_SECRET_KEY` (compose defaults for local).
5. HSM / external KMS is **later** (not this ADR).

## Consequences

- Tenant A ciphertext cannot open with tenant B DEK.
- Object path is real S3 SigV4 against MinIO, not metadata-only.
