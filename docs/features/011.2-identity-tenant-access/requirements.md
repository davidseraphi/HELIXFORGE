# 011.2 — Stable identity, safe registration, tenant separation, and per-resource access

## Status

Completed on 2026-07-15. `011.1` is closed and the verification gates below pass.

## Outcome

Identity never depends on a folder path. Public registration cannot claim an
existing tenant or receive admin scope by default. Database tenant isolation is
enforced by policy. Per-resource access checks are wired into the shared
service kit and auth-adapter.

## Allowed edits (after activation)

- `crates/shared-core/src/ids.rs` — stable opaque identity types.
- `crates/shared-core/src/tenancy.rs` — principal, scopes, role bindings.
- `crates/helix-db/src/` — tenant, membership, resource ACL repositories and
  migrations.
- `services/auth-adapter/src/main.rs` and `crates/auth-client/src/lib.rs` —
  registration/invite flow, least-privilege default scopes, removal of
  caller-chosen tenant authority.
- `crates/service-kit/src/middleware.rs` — `RequireAuth`, scope/ACL enforcement,
  fail-closed tenant status.
- Tests and fixtures: identity roundtrip, registration rejection, RLS
  adversarial, cross-tenant access attempts.
- Living docs and this packet.

## Forbidden edits

- No product-domain API changes outside auth/identity.
- No deletion or re-identification of existing tenants/users.
- No enabling real payments, signing, or physical-control paths.
- No reuse of existing product capability IDs with new meanings.
- No raw secret values in code or tests.

## EARS acceptance

### Stable identity

- The system SHALL generate stable opaque IDs for tenant, project, person,
  workload, job, artifact, and evidence bundle.
- The system SHALL treat path, URL, display name, email, device name, and
  provider ID as bindings, not as the identity.
- The system SHALL keep a project’s identity when it is moved to a new folder
  or machine.

### Safe registration and membership

- The system SHALL reject a public registration that supplies an existing
  `tenant_id` as authority unless a valid invitation or accountable admin
  process is provided.
- The system SHALL assign least-privilege scopes (`read`, `write`, `audit_read`)
  to newly registered users.
- The system SHALL require explicit operator action for `admin` or `platform`
  scopes.
- The system SHALL support joining an organization only through invitation,
  verified domain process, or accountable administrator approval.

### Tenant separation

- The database SHALL enforce tenant isolation via row-level security or an
  equivalent storage policy.
- Application-layer tenant checks SHALL be an additional layer, not the only
  layer.
- Cross-tenant links SHALL be impossible at the storage level.

### Per-resource access

- The system SHALL include tenant, resource, action, purpose, role, time, and
  device/workload identity in access decisions.
- The system SHALL fail closed when the policy service is unavailable.

## Test plan

| Check | Evidence |
|---|---|
| Identity roundtrip | `shared_core::ids` unit tests; `ProjectId` added |
| Registration rejection | `services/auth-adapter/src/main.rs`: public `tenant_id` rejected; only fresh tenant created |
| Least-privilege default scopes | `auth-adapter` registers with `read`/`write`/`audit_read`; `admin` requires invite/ops elevation |
| Scope escalation gating | `HELIX_DEV_PLATFORM=1` or `ops@` prefix required for `Platform`/`Admin` in dev identity |
| RLS adversarial | `crates/helix-db/src/membership.rs` integration tests: cross-tenant reads return `None` |
| Fail-closed policy | Tenant lifecycle + ACL checks fail closed; `RequireAuth` rejects suspended/missing tenants |
| Membership lifecycle | `MembershipRepo::create/get/list_for_tenant` with RLS tenant context |

## Evidence

- Migration: `crates/helix-db/migrations/0036_foundation_integrity_identity.sql` (+ down file).
- Repository: `crates/helix-db/src/membership.rs` with per-transaction `set_tenant_context`.
- Registration guard: `services/auth-adapter/src/main.rs` `ory_register` rejects caller-supplied `tenant_id`.
- Scope gating: `crates/auth-client/src/lib.rs` `dev_scopes_for_label`; `auth-adapter` passes `read`/`write`/`audit_read` to Kratos.
- Verification: `cargo build --workspace`, `cargo test --workspace --all-features`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` all pass; Postgres migrations apply cleanly.

## Dependencies

- `011.1` must close first so security-critical identity code can be reviewed
  under version control and a clean CI matrix.

## Rollback / compensation

- Migrations added by this packet must include forward and rollback scripts.
- Existing tenant/user rows are preserved; new columns default to safe values.
- A bad registration change can be reverted without data loss.
