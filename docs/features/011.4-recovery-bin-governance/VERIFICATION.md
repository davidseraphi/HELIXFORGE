# 011.4 Verification Evidence

## Scope
Recovery bin, restore, permanent-delete authority, and policy exceptions.

## Environment
- OS: Windows 11, `stable-x86_64-pc-windows-msvc`
- Postgres: `postgres://helix:helix@127.0.0.1:55432/helixforge`
- Migration applied: `0038_recovery_bin_governance.sql` (+ rollback file)
- Date: 2026-07-15

## Checks

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt --check` | PASS |
| Clippy | `cargo clippy --workspace --all-targets` | PASS (no warnings) |
| Unit/integration tests | `cargo test --workspace` | PASS — all crates, doc-tests included |
| Migration apply | `sqlx migrate!` via `connect_and_migrate` in tests | PASS — `0038` applied cleanly |

## New tests

- `crates/helix-db/src/governance.rs`
  - `recovery_bin_soft_delete_and_restore`
  - `recovery_bin_rolls_back_with_transaction`
  - `recovery_bin_expired_entries_listed`
  - `policy_exception_lifecycle`
  - `permanent_delete_requires_authority_reason_and_audit`
- `crates/shared-core/src/tenancy.rs`
  - `permanent_delete_authority_gated`

## Key implementation

- Migration `0038_recovery_bin_governance.sql` adds:
  - `helix_core.recovery_bin` with 30-day `retain_until` default, tenant FK, and
    partial expiry index.
  - `helix_core.policy_exceptions` for time-bound, justified, approved policy
    deviations.
- `GovernanceRepo` exposes:
  - `soft_delete_in_tx` / `restore_in_tx` / `permanently_delete_in_tx`
  - `list_bin_for_tenant` / `list_expired`
  - `create_policy_exception` / `revoke_policy_exception` / `is_exception_active`
  - `permanently_delete` — authority-gated, requires reason, appends a
    hash-chained audit event before physical deletion.
- `Principal::can_permanently_delete()` gates permanent delete to `admin` or
  `platform` scope.

## Notes

- The initial partial index on `policy_exceptions` used `now()` in its predicate,
  which Postgres rejected as non-IMMUTABLE. It was replaced with a composite
  index on `(tenant_id, policy_kind, revoked_at, starts_at, expires_at)`.
- No runtime services were started; no production data was modified.
