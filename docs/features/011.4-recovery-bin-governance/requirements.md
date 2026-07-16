# 011.4 — Recovery bin, restore, permanent-delete authority, and policy exceptions

## Status

Closed on 2026-07-15. Implementation, tests, migration, clippy, fmt, and full-workspace tests passed. Evidence recorded in `VERIFICATION.md`.

## Outcome

Deletion is safe and recoverable. A soft-delete moves data into a tenant-scoped
recovery bin with a 30-day retention window. Restore returns the item to active
use. Permanent deletion requires explicit, accountable authority and leaves an
immutable audit trail. Policy exceptions are time-bound, justified, recorded,
and approved.

## Allowed edits (after activation)

- `crates/helix-db/src/governance.rs` — retention/recovery policy logic.
- `crates/helix-db/src/acl.rs` — permanent-delete authority checks.
- `crates/helix-db/src/audit_pg.rs` / `crates/audit-log/src/lib.rs` — audit
  records for delete/restore/permanent-delete/policy-exception actions.
- `crates/helix-db/src/atomic.rs` — recover-bin writes inside the same
  transaction as domain delete/restore.
- `crates/helix-db/migrations/0038_recovery_bin_governance.sql` (+ rollback).
- `crates/service-kit/src/middleware.rs` — scope/role checks for destructive
  actions.
- Tests and fixtures: recovery-bin lifecycle, restore, purge, permanent-delete
  authorization, policy exception workflow.
- Living docs and this packet.

## Forbidden edits

- No product-domain API changes outside recovery/governance paths.
- No real destruction of production data except through the audited
  permanent-delete path.
- No bypass of tenant-scoped recovery bin.
- No raw secret values in code or tests.

## EARS acceptance

### Recovery bin

- The system SHALL move a tenant-scoped resource to a recovery bin on delete
  instead of immediately removing it.
- The system SHALL retain recovery-bin entries for 30 days by default.
- The system SHALL record resource type, resource id, original location,
  deleted by, deleted at, tenant, and retention deadline.
- The system SHALL make recovery-bin entries visible to authorized principals
  of the same tenant only.

### Restore

- The system SHALL allow an authorized principal to restore a recovery-bin
  entry to its original active state within the retention window.
- The system SHALL treat restore as a write action that is audited and
  committed atomically with the domain change.

### Permanent-delete authority

- The system SHALL require `admin` or `platform` scope for permanent deletion.
- The system SHALL require a recorded reason and, outside local mode, a
  break-glass justification for permanent deletion.
- The system SHALL append an audit event before the physical row is removed.
- The system SHALL fail closed if the audit append cannot be committed.

### Policy exceptions

- The system SHALL support time-bound retention-policy exceptions with a
  recorded justification and approving authority.
- The system SHALL enforce the exception window and revert to default policy
  when it expires.
- The system SHALL record every exception creation, extension, and revocation
  in the audit chain.

## Test plan

| Check | Evidence |
|---|---|
| Soft-delete creates recovery-bin entry | Unit/integration test: delete resource, assert bin row exists, active row is gone |
| Restore recovers resource | Test: restore within window, assert active row back and bin row marked restored |
| Expired bin purged | Test: simulate retention deadline expiry, assert purge job removes bin row and creates audit event |
| Permanent-delete authorization | Test: non-admin/platform principal is rejected; admin with reason succeeds and audit event recorded |
| Policy exception lifecycle | Test: create exception, extend, expire, revoke; each step audited |
| Atomicity | Forced rollback during delete/recover: neither active nor bin state changes |

## Dependencies

- `011.2` for stable identity, tenant separation, and per-resource access.
- `011.3` for atomic domain + audit + outbox writes and durable job execution
  (purge/orphan recovery jobs).

## Rollback / compensation

- Migration adds `helix_core.recovery_bin` and exception tables; existing rows
  are untouched.
- Rollback script drops the new tables and any default policies added by the
  migration.
- A failed permanent-delete change can be reverted without data loss because
  the row remains in the recovery bin until success is confirmed.
