# ADR-0008: Vault durable AES-256-GCM envelope

## Status

Accepted — 2026-07-14

## Context

HelixCore requires durable, encrypted secrets. Per-tenant DEK rotation and HSM
are later enterprise work.

## Decision

1. Persist ciphertext in `helix_core.secrets` via `PgVault` when Postgres is up.
2. Envelope: **AES-256-GCM** with key derived as SHA-256(master || context).
   Ciphertext format: `HVA1 || nonce(12) || ciphertext+tag`.
3. Master material from `HELIX_VAULT_MASTER_KEY` (config); local default for dev only.
4. Legacy XOR rows (pre-AES) remain readable once for migration.
5. Future: per-tenant DEKs, key rotation, HSM — separate ADR.

## Consequences

- Secrets durable and encrypted at rest in Postgres.
- Operators must set a strong `HELIX_VAULT_MASTER_KEY` outside local lab.
