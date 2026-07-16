# HelixForge shared product contract

## Status

This is a target-state contract for every HelixForge product and for the
standalone HelixAnvil integration. It does not claim the present code meets the
contract. Product sheets add domain rules but cannot weaken these rules.

## 1. One foundation

HelixCore owns common platform truth. A product must not create a private copy
of identity, access policy, audit, secret storage, capability grants, jobs,
notifications, billing, evidence, recovery, or deployment health.

HelixCore itself is replaceable at its outer boundaries. Every network, model,
cloud, identity, storage, payment, custody, and proof provider is reached through
a versioned adapter with a local or self-hosted fallback.

## 2. Stable identity

Every tenant, person, project, product, workspace, resource, agent, capability,
job, artifact, and evidence bundle has a stable opaque identity.

- A path, URL, display name, email address, device name, or provider ID is a
  binding, not the identity.
- Moving a project between folders or machines keeps its identity.
- Merging or splitting identities creates an explicit, reversible decision record.
- Removing a project revokes its bindings and grants. It does not delete shared
  secrets or evidence used by other projects.
- New-project scaffolding registers identity and a safe capability contract.

## 3. Identity and access

Public registration creates a person or an unclaimed private space. It never
accepts an arbitrary existing tenant as authority. Joining an organization
requires an invitation, verified domain process, or accountable administrator.

Access decisions include tenant, resource, action, purpose, role, time, device
or workload identity where needed, residency, and policy version. The database
must provide a second isolation layer so one missing application filter cannot
cross tenants.

Roles are split. Normal administration does not automatically include secret
export, audit rewrite, billing override, break-glass, signing, or permanent delete.

## 4. Capability broker

Projects and workloads request named capabilities such as
`github.repository.read`, `models.inference.local`, or `payments.refund.request`.

- A project never knows a secret storage path.
- An agent can see safe metadata and request a capability. It can never retrieve
  the raw value.
- A user-owned broker gives one approved process the exact capability, scope,
  purpose, time, quota, and network target.
- Signing, payment, and similar sensitive operations happen inside the broker
  when possible, without exporting a private key.
- Grants, denials, use, rotation, expiry, and revocation create signed
  metadata-only events.
- Custody is pluggable. A legacy read-only backend may support migration, but it
  cannot become a permanent application API.

## 5. All-or-nothing state changes

An accepted command must not leave the product changed while its audit, job, or
outbox reports failure. Domain state, audit intent, event outbox, idempotency
record, and job transition commit in one database transaction where possible.

External effects use this order:

1. validate authority and idempotency;
2. write a durable intent and outbox entry;
3. commit;
4. perform the external effect;
5. record its observed result;
6. compensate or request human action when completion is uncertain.

Retries use the same idempotency key. `Unknown` is a real final user-visible
state until evidence resolves it. A timeout never means cancellation or failure
unless the worker or provider confirms that state.

## 6. Durable jobs and visible progress

Every task that may take more than one second is a job with:

- stable identity;
- requested outcome;
- owner and authority lease;
- stages and durable checkpoints;
- real process or remote-operation identity;
- start, last signal, elapsed time, and resource use;
- current state: queued, starting, running, waiting, blocked, cancelling,
  cancelled, failed, completed, or unknown;
- safe pause, resume, retry, cancel, and recovery rules;
- outputs, checks, warnings, and proof.

The interface never invents a smooth percentage for work that has no measurable
total. It shows completed stages, the current stage, last signal, and an honest
time range if one can be measured. Completion, failure, waiting-for-human, and
long silence create persistent in-app notifications. Important completions may
also use native desktop notifications.

## 7. Delete, undo, and recovery

Ordinary user deletion enters a recoverable bin for at least 30 days.

- The preview shows selected count, dependants, shared use, retention, and the
  final deletion date.
- Restore preserves stable identity, versions, links, and access policy.
- Permanent deletion is separate, explicit, audited, and requires stronger
  authority and recent authentication.
- Legal hold, safety records, financial books, clinical records, research
  withdrawal, short-lived cache expiry, and immutable evidence follow their
  named policies instead of one false universal timer.
- Irreversible real-world actions use compensation and incident recovery, not a
  misleading undo button.

## 8. Evidence and proof

Important work produces a proof bundle containing:

- what was requested;
- stable identities and policy versions;
- exact authority and approvals;
- source identities and content hashes;
- tool, model, environment, and adapter versions;
- commands, job transitions, and external acknowledgments;
- artifacts and fresh checks;
- human and independent reviews;
- known limits and what the bundle does not prove;
- signatures and a verification guide.

HelixCore owns canonical logical identity and the provider-neutral proof and
capability contracts. Aether may attest that identity and act as the preferred
external proof or capability provider. It does not create a second canonical
identity or policy model. HelixForge also implements a local signed provider so
offline or standalone use remains possible.

## 9. Agent contract

Every agent is a workload with a stable identity and an exact lease. Its manifest
states readable resources, writable resources, tools, network targets, compute,
time, token or money budget, output destinations, approval points, and forbidden actions.

Agents may propose broader access but cannot grant it. They cannot approve their
own high-impact work, read raw secret values, hide failed attempts, rewrite
evidence, widen a task, contact a person, publish, pay, sign, deploy, permanently
delete, or operate physical systems without the exact human-approved capability.

Builder and reviewer roles are separate for gated changes. Model output is
untrusted input until schema, policy, source, and domain checks pass.
Source comments, imported documents, issue text, web pages, tool output, and
dependency messages are also untrusted evidence. They never become authority or
instructions merely because an agent can read them.

## 10. Shared UX language

Every product uses the same semantic states:

- blue: active work;
- amber: waiting for a person;
- violet: waiting for an external signal;
- green: completed and checked;
- red: failed or unsafe;
- grey: unknown or not yet checked.

Color is never the only signal. Each state has text, icon, timestamp, and detail.

The default screen answers:

1. Where am I?
2. What is happening?
3. What needs me?
4. What changed?
5. What can I safely do next?

Advanced controls appear through progressive reveal. Selection and movement
show a visible count and destination. Destructive work shows effect and recovery.
Errors explain what happened, what was preserved, and the safest next action.

## 11. Accessibility

The release floor is [WCAG 2.2 AA](https://www.w3.org/TR/WCAG22/) for web
surfaces, verified from the W3C Recommendation on 2026-07-15, and the
equivalent native platform accessibility behaviour.

- Complete keyboard paths exist for every core journey.
- Screen-reader names, roles, state changes, errors, and progress are tested.
- Text supports 200% scaling without lost content or actions.
- Focus is visible and logical.
- Motion can be reduced.
- Meaning does not depend on color, sound, hover, or fine pointer control.
- Touch targets meet the product's declared mobile or tablet contract.

## 12. Cross-platform contract

Windows, macOS, and Linux are equal development and validation targets for
shared Rust services, CLI tools, browser clients, local storage, migrations,
packaging, backup, restore, and the common golden journeys.

- Rust toolchain files do not force one host target globally.
- CI uses native Windows, macOS, and Linux workers.
- Shell automation has a portable core and thin PowerShell and POSIX wrappers.
- Filesystem code tests separators, drive roots, links, case behaviour,
  permissions, long paths, Unicode, file locking, and atomic rename behaviour.
- Containers are a deployment choice, not a requirement for local use.
- Desktop products ship signed native packages and update/recovery tests for all
  three systems.

## 13. Data contract

Every record has stable identity, tenant, owner or authority, schema version,
created and changed time, provenance, retention class, and deletion state where
appropriate. Domain objects use typed fields and invariants. `metadata` may hold
extensions but cannot replace the domain model.

Imports preserve source values. Normalization creates linked values and records
mapping, units, version, and loss. Exports contain data, relationships, history,
policy, and proof in documented open formats.

## 14. Reliability contract

- Readiness returns a failing status code when required dependencies are not ready.
- Release gates execute fresh checks; they do not read cached success flags.
- An acknowledged durable write meets its declared data-loss budget.
- Backups are incomplete until a clean restore and verification test passes.
- Every external call has timeout, retry, idempotency, circuit-break, and unknown-outcome rules.
- Every worker has ownership, heartbeat, real cancellation, and orphan recovery.
- Resource exhaustion creates backpressure before corruption or broad failure.
- Failure drills include process kill, power loss, disk full, network partition,
  slow dependency, clock change, corrupt message, and partial restore as relevant.

## 15. Release gates

Every merge runs the practical subset; every milestone runs the full set:

1. Rust format, lint, unit, property, integration, and documentation checks.
2. TypeScript format, lint, type, unit, component, and accessibility checks.
3. Browser end-to-end journeys on supported engines.
4. Native Windows, macOS, and Linux build and package tests.
5. Database migration forward, rollback-or-compensation, tenant isolation, and
   concurrency tests.
6. Job, cancellation, idempotency, all-or-nothing write, crash, and recovery tests.
7. Secret scan, dependency review, access-control tests, and gated security review.
8. Backup/restore, export/import, 30-day bin, and stable-identity move tests.
9. A fresh product-capability report generated from real checks.

A gate records command, environment, start and end time, exit status, artifact
hashes, and skipped checks with reasons. `Passed` means all required checks ran.

## 16. Product maturity

- **Scaffold:** identity, target sheet, source layout, no product claim.
- **Prototype:** one domain journey works with synthetic data; no user-ready claim.
- **Alpha:** useful end-to-end local journey, recovery, tests, and known limits.
- **Beta:** team use, migrations, operations, access isolation, and all three OS gates.
- **Production:** external users, independent review, support, incident response,
  restore proof, honest service objectives, and no open P0/P1 findings.
- **Category-defining:** the product's G4 claim is proven by real user outcomes,
  portability, trust, and independent evidence, not by feature count.

## 17. Dependency rule

A product may depend only on a capability already proven at the needed maturity.
It must have a degraded mode when a non-core product is unavailable. Pulse,
frontier agents, federation, and physical control cannot become critical paths
before their own safety and reliability gates close.
