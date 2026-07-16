---
product: HelixCore
catalog_order: 00
status: target-state-spec
horizon: 60 months
current_maturity: alpha
primary_users: [platform operator, product builder, security owner, team administrator, auditor]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
---

# 1. Category claim

HelixCore is a sovereign product operating substrate that lets a person or organisation run, govern, prove, recover, and move its software work without giving one vendor permanent control.

# 2. Five-year destination

- **Useful product:** One calm control plane manages identity, projects, policies, capabilities, jobs, objects, evidence, recovery, billing, and health for every HelixForge product.
- **Category-defining advantage:** Important actions produce proof that another machine can check, while the owner can move the full system between local, self-hosted, and managed operation.
- **Frontier capability:** Policy-aware agents and workloads receive short, exact capability leases and can use sensitive operations without receiving raw secret values.
- **Human authority:** A named accountable person approves high-impact grants, permanent deletion, release, payment changes, key rotation, and cross-organisation trust.

# 3. Users and hard jobs

| User | Hard jobs | Failure feared most |
|---|---|---|
| Platform operator | Install, upgrade, monitor, back up, restore, and move the platform | A silent partial failure corrupts tenant data |
| Product builder | Reuse identity, policy, jobs, objects, proof, and capabilities through stable interfaces | Core changes break every product |
| Security owner | Define least authority, review grants, rotate custody, and investigate incidents | An agent or tenant crosses its allowed boundary |
| Team administrator | Add people, projects, roles, retention, and spend limits | A simple admin action causes permanent loss |
| Auditor | Check what happened without trusting the running service | A green status is based on stale or self-reported checks |
| Individual owner | Run privately on one machine and leave with all useful data | Lock-in, hidden cloud use, or an unrecoverable machine loss |

# 4. Product laws

1. The owner can operate locally and export a complete, documented home.
2. A folder path is never the identity of a tenant, project, person, workload, or proof.
3. No agent, plugin, or project receives a raw secret merely because it can request a capability.
4. A durable action and its audit event succeed together or fail together.
5. A release gate reports only checks run fresh against the exact candidate.
6. Delete enters a recoverable 30-day bin by default. Permanent deletion is separate, explicit, and audited.
7. Slow work always shows phase, last heartbeat, elapsed time, safe next action, and a final notification.
8. Human approval is required before authority, money, irreversible loss, or external publication increases.
9. Tenant separation is enforced in both application policy and storage policy.
10. Failure is shown honestly. Degraded, skipped, unknown, and not configured are not called healthy.
11. Keyboard, touch, screen reader, reduced-motion, and high-contrast use are first-class.
12. Every critical provider has a local fallback or a documented safe stop.

# 5. Scope boundaries

**HelixCore owns:** stable identity; authentication adapters; policy decisions; tenant boundaries; capability contracts and leases; job control; object custody; audit and proof events; recovery; entitlement state; shared operations; release gates; common UI shell; provider-neutral contracts.

**Other products own:** their domain records, domain rules, domain agents, domain-specific user journeys, and any licensed professional decision. Aether may provide stronger external proof and capability brokering through an adapter.

**HelixCore will not attempt:** to be every product UI; to make legal, medical, financial, safety, or employment decisions; to expose secret values to agents; to claim that a signature proves a statement is true; or to make a managed cloud required for normal operation.

# 6. Signature experiences

| Journey | Entry point | Visible progress | Human decision | Completion proof | Failure and recovery | Export or portability |
|---|---|---|---|---|---|---|
| Create a sovereign home | Welcome flow | Storage, identity, key custody, and health steps | Choose local, self-hosted, or managed custody | Signed home manifest plus fresh doctor report | Resume from the last safe checkpoint; no half-created home | Export home manifest, public keys, policy, and data map |
| Register a stable project | Projects, New project | Identity creation, capability scan, contract validation | Approve project owner and requested capabilities | Project identity and signed registration event | Invalid contracts stay as drafts; retry is idempotent | Export project descriptor independent of folder path |
| Grant a capability | Authority inbox | Request scope, process identity, duration, risk, and decision state | Approve, narrow, deny, or require two people | Signed grant or denial with no secret value | Expired or revoked leases stop safely; rollback restores prior policy | Export metadata-only grant history |
| Run and watch a long job | Work center | Queued, starting, running phase, heartbeat, current step, evidence count | Pause, cancel, or approve a gated step | Result bundle, checks, logs, resource use, and signed completion | Crash resumes from a durable checkpoint or reports exact loss boundary | Export result and proof bundle |
| Delete and restore | Any object action menu | Impact preview, dependency count, bin expiry date | Confirm move to bin; separate approval for permanent deletion | Tombstone event and restore token | Restore returns identity and links; conflict is shown before change | Export deleted item before permanent removal |
| Release a product | Release gate | Fresh build, tests, security, packaging, migration, and rollback checks | Named owner approves exact candidate | Signed gate report bound to source and artifact hashes | Any skipped or failed gate blocks release; rerun creates a new report | Export report and independently verifiable artifacts |
| Back up and move home | Operations, Portability | Snapshot, consistency check, encryption, transfer, restore rehearsal | Choose destination and custody | Restore drill and content inventory match | Source stays active until target proves healthy; rollback is one action | Open manifest plus encrypted portable archive |
| Investigate an incident | Trust center | Timeline assembly, affected bindings, containment, recovery tasks | Revoke, rotate, restore, or declare false alarm | Signed incident record and fresh post-recovery checks | Evidence is preserved; uncertain facts stay marked unknown | Export redacted incident bundle |

# 7. Capability map

## F0 — Foundation

### HC-F0-001 — Stable identity registry

- **Gate:** G0. **Inputs:** owner, tenant, project, workload, and key registrations. **Outputs:** stable identifiers and versioned bindings.
- **Invariants:** identifiers never depend on paths or host names. **Authority:** owners create homes; tenant admins create tenant resources.
- **Evidence:** registration, binding change, and revocation events. **Failure:** duplicate or ambiguous identity is blocked.
- **Acceptance:** moving a project to a new folder and machine keeps the same identity and proof chain.

### HC-F0-002 — Authentication and policy decision

- **Gate:** G0. **Inputs:** verified subject, tenant, action, resource, context. **Outputs:** allow or deny with reason and policy version.
- **Invariants:** deny by default; no caller-selected admin role. **Authority:** security owners publish policy; users authenticate.
- **Evidence:** metadata-only decision event for sensitive actions. **Failure:** unavailable policy service fails closed.
- **Acceptance:** an unauthenticated registration cannot join an existing tenant without a valid invitation.

### HC-F0-003 — Atomic proof ledger

- **Gate:** G0. **Inputs:** domain change and canonical event. **Outputs:** committed change, ordered event, signature, and receipt.
- **Invariants:** all-or-nothing commit; no accepted content mismatch; one ordering lock. **Authority:** services append; nobody rewrites.
- **Evidence:** linked, asymmetrically signed receipts. **Failure:** crash recovery completes or rolls back the whole write.
- **Acceptance:** forced termination at every write boundary produces either both records or neither.

### HC-F0-004 — Durable job control

- **Gate:** G0. **Inputs:** versioned job request, authority lease, idempotency key. **Outputs:** state machine, heartbeats, checkpoints, result.
- **Invariants:** one logical execution per idempotency key; cancellation is cooperative and observable. **Authority:** caller may stop its own job.
- **Evidence:** state transition events and checkpoint hashes. **Failure:** lost worker returns the job to recovery, never silent success.
- **Acceptance:** a killed worker resumes or ends failed within the recovery budget with the last durable step shown.

### HC-F0-005 — Tenant-enforced storage

- **Gate:** G0. **Inputs:** tenant-bound records and object references. **Outputs:** isolated rows and encrypted objects.
- **Invariants:** storage policy checks tenant identity; cross-tenant links are impossible. **Authority:** scoped services only.
- **Evidence:** isolation test report and denied-access events. **Failure:** missing tenant context is rejected.
- **Acceptance:** adversarial integration tests cannot read, link, update, or delete another tenant’s records.

## F1 — Useful product

### HC-F1-001 — Canonical control plane

- **Gate:** G1. **Inputs:** core health, work, authority, recovery, and proof state. **Outputs:** one shipping operator UI.
- **Invariants:** no competing admin surface; states use plain language. **Authority:** role-filtered views and actions.
- **Evidence:** UI action receipts and accessibility checks. **Failure:** stale data is labelled with age.
- **Acceptance:** an operator can diagnose, stop, restore, and verify a job without opening a shell.

### HC-F1-002 — Capability broker

- **Gate:** G1. **Inputs:** stable workload identity, exact capability, policy, lease duration. **Outputs:** narrow process-bound lease or denial.
- **Invariants:** agents see metadata, never values; sensitive signing stays inside custody. **Authority:** user-owned broker grants one process.
- **Evidence:** signed grant, denial, use, rotation, revocation events. **Failure:** expired binding cannot be reused.
- **Acceptance:** a test agent can use github.repository.read but cannot retrieve the credential or use repository.write.

### HC-F1-003 — Recoverable 30-day bin

- **Gate:** G1. **Inputs:** delete request and dependency graph. **Outputs:** tombstone, expiry, restore plan.
- **Invariants:** default retention is 30 days; shared secrets are not deleted when one project is removed. **Authority:** owner restores; elevated approval purges.
- **Evidence:** delete, restore, expiry, and purge events. **Failure:** conflicting restore pauses for review.
- **Acceptance:** an item deleted on day 1 restores with identity and links on day 29.

### HC-F1-004 — Truthful release gate

- **Gate:** G1. **Inputs:** exact source revision, dependency lock, artifacts, migration, target platforms. **Outputs:** fresh pass or blocking report.
- **Invariants:** no cached claim counts as fresh; skipped is not passed. **Authority:** automation checks; named human releases.
- **Evidence:** commands, environment, timestamps, hashes, outputs, and signatures. **Failure:** one required unknown blocks release.
- **Acceptance:** changing an artifact after tests invalidates the gate report.

## F2 — Category leader

### HC-F2-001 — Portable home and verified restore

- **Gate:** G2. **Inputs:** tenant snapshot, objects, policies, proofs, public keys. **Outputs:** encrypted archive and restore rehearsal.
- **Invariants:** open manifest; provider-specific data stays behind adapters. **Authority:** owner exports; destination proves custody.
- **Evidence:** inventory and source-to-target hash map. **Failure:** source remains authoritative until verified cutover.
- **Acceptance:** a home moves between two supported providers without identity or audit-chain loss.

### HC-F2-002 — Multi-tenant operations

- **Gate:** G2. **Inputs:** fleet health, upgrades, quotas, incidents. **Outputs:** staged changes and tenant-safe operations.
- **Invariants:** canary first; one tenant failure cannot spread silently. **Authority:** operators act within delegated estates.
- **Evidence:** rollout decisions and per-tenant results. **Failure:** automatic halt at a defined error budget.
- **Acceptance:** a failed upgrade rolls back affected tenants while healthy tenants remain available.

### HC-F2-003 — Honest entitlements and billing

- **Gate:** G2. **Inputs:** signed usage, plan policy, verified provider event. **Outputs:** entitlement change and reconciled invoice state.
- **Invariants:** no simulation outside local mode; money and entitlement changes are atomic and idempotent. **Authority:** provider confirms payment; human resolves disputes.
- **Evidence:** provider receipt, reconciliation, and adjustment event. **Failure:** uncertain payment never grants a paid plan.
- **Acceptance:** replaying a payment event cannot charge twice or grant twice.

### HC-F2-004 — Service-level operations

- **Gate:** G2. **Inputs:** metrics, traces, logs, synthetic checks, user-impact events. **Outputs:** budgets, alerts, incident links.
- **Invariants:** health derives from fresh checks; sensitive values are redacted. **Authority:** operators set budgets.
- **Evidence:** sampled check data and incident linkage. **Failure:** missing telemetry becomes unknown.
- **Acceptance:** readiness returns a failing status when any required dependency is unavailable.

## F3 — Advanced category leadership

### HC-F3-001 — Independent proof exchange

- **Gate:** G3. **Inputs:** signed bundles, verifier policy, public trust roots. **Outputs:** local or Aether-backed verification result.
- **Invariants:** provider-neutral interface and local fallback. **Authority:** recipient chooses trust policy.
- **Evidence:** verifier version, inputs, checks, and limits. **Failure:** unavailable Aether never changes failed proof into passed.
- **Acceptance:** an offline verifier can check bundle integrity and clearly state what it cannot establish.

### HC-F3-002 — Policy simulation and safe rollout

- **Gate:** G3. **Inputs:** proposed policy, historical metadata, synthetic cases. **Outputs:** predicted allow/deny changes and rollout plan.
- **Invariants:** simulation never grants authority. **Authority:** human publishes after review.
- **Evidence:** case set, differences, approval, and rollback point. **Failure:** uncovered high-risk action blocks rollout.
- **Acceptance:** every changed decision links to a test case before policy publication.

### HC-F3-003 — Sovereign compute scheduler

- **Gate:** G4. **Inputs:** workload contract, data class, capability needs, platform facts, cost limit. **Outputs:** local or remote placement and proof.
- **Invariants:** placement obeys residency and custody; unavailable targets degrade safely. **Authority:** owner sets placement policy.
- **Evidence:** placement reason, attested runtime facts, resource record. **Failure:** no compliant target means no execution.
- **Acceptance:** the same signed job contract runs on Windows, macOS, and Linux where capabilities match.

## F4 — Frontier network

### HC-F4-001 — Cross-home trust fabric

- **Gate:** G4. **Inputs:** stable home identities, public keys, trust proposal, exact capabilities, expiry, and local policy. **Outputs:** bilateral or group trust bindings with narrow grants and revocation paths.
- **Invariants:** there is no global super-admin, shared raw secret, or implied parent access. **Authority:** an accountable owner in every participating home approves its own binding.
- **Evidence:** signed proposals, approvals, grants, uses, denials, rotations, revocations, and trust-path checks. **Failure:** unknown identity, expired proof, policy conflict, or partition blocks new cross-home effects while local work stays available.
- **Acceptance:** three homes exchange one approved capability; one home revokes its binding, loses future access within the declared expiry, and the other two continue without wider authority.

### HC-F4-002 — Federated continuity and exit

- **Gate:** G4. **Inputs:** encrypted recovery shares, replica contracts, identity map, retention policy, restore target, and exit request. **Outputs:** verified continuity copies, recovery quorum result, restored home, or complete exit package.
- **Invariants:** no host can read protected content or restore alone unless owner policy allows it; stable identity and proof survive recovery. **Authority:** the owner chooses custodians, recovery threshold, restore target, and exit.
- **Evidence:** custody receipts, share rotations, replica hashes, recovery approvals, restore checks, revocations, and deletion acknowledgments. **Failure:** too few valid shares or a mismatched replica stops recovery and leaves the source authoritative.
- **Acceptance:** after one of three custodians becomes unavailable, the owner restores a clean home from the remaining approved shares, verifies every retained object, and revokes the old bindings without deleting shared public proof.

# 8. Domain model

| Record | Owner and relationships | Lifecycle and version rules | Retention |
|---|---|---|---|
| SovereignHome | One owner; contains tenants and trust roots | Stable ID; configuration is versioned | Until explicit portable purge |
| Tenant | Home or managed estate; contains memberships and projects | Stable ID; status transitions are audited | 30-day bin after closure, then policy |
| Membership | Tenant plus subject plus role bindings | Every role change is a new version | Active plus audit retention |
| ProjectIdentity | Tenant-owned; binds many locations and capability contracts | Path-independent; binding changes do not change ID | 30-day bin |
| WorkloadIdentity | Project-owned process or agent identity | Rotatable credentials; stable logical ID | Revocation record retained |
| PolicyBundle | Tenant-owned rules and tests | Immutable published versions; draft before publish | All published versions |
| CapabilityContract | Project asks for named capabilities | Versioned, least-authority schema | Project life plus audit retention |
| CapabilityLease | Contract, workload, scope, expiry, decision | Immutable grant; revoke creates state transition | Metadata retained, values never stored here |
| Job | Project request with state, checkpoints, and result refs | Append-only state transitions; retry links attempts | Configurable; proof summary retained |
| Object | Tenant object with class, hash, custody, and refs | Content-addressed revisions; tombstone on delete | 30-day bin then policy |
| AuditEvent | Ordered domain fact linked to prior receipt | Immutable; schema version carried per event | Policy with legal-hold override |
| ProofBundle | Evidence items, checks, signatures, limitations | Immutable manifest; superseded, never rewritten | At least as long as claimed artifact |
| ReleaseCandidate | Exact source, locks, artifacts, target matrix | New candidate when any input changes | Release history retained |
| RecoveryItem | Tombstone, dependencies, expiry, restore plan | Restored, expired, or purged state only | 30 days by default |
| Entitlement | Tenant plan and limits tied to verified source | Effective-time versions; no in-place history loss | Financial policy |

# 9. System architecture

- **Domain engine:** identity registry, policy decision point, capability broker, durable job state machine, recovery engine, proof ledger, and release-gate engine.
- **Application services:** gateway, identity adapter, agent/job hub, vault/object service, billing/entitlement service, observability service, and one control plane.
- **Adapters:** Postgres, object stores, event buses, identity providers, custody providers, payment providers, Aether proof/capability provider, local proof fallback, and platform runtimes.
- **Storage:** Postgres is authoritative for transactional records; object storage holds encrypted blobs; an append-only archive holds proof receipts; local mode uses the same contracts.
- **Event flow:** domain change, audit event, and outbox entry commit in one transaction; workers publish from the outbox; consumers are idempotent.
- **Background work:** durable workers claim leases, heartbeat, checkpoint, and release claims after timeout.
- **Offline behaviour:** read cached home state, queue explicitly supported local actions, show conflict before sync, and never pretend remote authority was granted offline.
- **Extension points:** versioned provider interfaces, capability handlers, proof verifiers, job executors, policy functions, import/export transforms, and UI modules.
- **Dependencies:** operating-system key custody where available; Postgres and object storage for shared deployment; Helix products through versioned contracts; Aether is optional and never a hard runtime dependency.

# 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Progress, check, stop, reverse |
|---|---|---|---|---|---|
| Operations assistant | Health metadata, runbooks, read-only diagnostics | Incident summary and recovery plan | Restart, failover, upgrade, restore | Read secrets or hide failed checks | Timeline and heartbeat; fresh doctor checks; cancel task; rollback checkpoint |
| Capability steward | Capability catalog and metadata-only requests | Narrow grant or denial | Every new sensitive grant | Retrieve or reveal values; self-approve | Lease state; policy simulation; revoke immediately |
| Release verifier | Candidate inputs and test runners | Gate report | Final release | Mark skipped as passed or reuse stale proof | Live check list; independent verify; cancel run; create new candidate |
| Migration assistant | Export inventory and adapter reports | Move and rollback plan | Cutover and source retirement | Delete source before target proof | Item counts and hashes; restore drill; stop transfer; return to source |
| Support assistant | Redacted tenant state and consented diagnostics | Plain-language explanation | Any data-changing action | Cross tenants or infer hidden data | Visible diagnostic steps; user confirms result; stop at any time |

# 11. Trust, safety, and privacy

- Access uses stable subjects, tenant membership, resource policy, purpose, and time-bound capability leases.
- Storage adds database-enforced tenant policy. Application checks are an extra layer, not the only layer.
- Data classes are public, internal, confidential, restricted, secret material, and proof metadata. Secret values never enter logs, prompts, audit events, proof bundles, exports, or support views.
- Data is encrypted in transit and at rest. Sensitive signing and decryption happen inside pluggable custody boundaries.
- Consent records state who agreed, purpose, data, duration, and withdrawal effect.
- Residency policy follows data and derived artifacts. A move is blocked when the destination is not allowed.
- Delete moves an item into a 30-day bin. Restore is available during that period. Permanent deletion needs a separate impact preview, elevated approval, and signed event.
- Legal hold prevents purge but does not silently restore access.
- Abuse controls include rate limits, tenant quotas, grant review, anomaly alerts, break-glass with two-person approval, and rapid revocation.
- Incident recovery preserves evidence, revokes affected bindings, rotates custody, restores from a verified point, and runs fresh checks.

# 12. Proof and audit

Important identity, policy, capability, job, object, release, recovery, billing, and operator actions create canonical events. A domain change, audit event, and delivery outbox commit together. Receipts are linked and signed with an asymmetric key. Key custody is separate from application memory where the platform permits it.

An exported bundle contains the manifest, hashes, public verification material, check inputs and outputs, actor and authority metadata, timestamps, and stated limitations. Another machine can check integrity, ordering, signatures, and the named checks. It cannot prove that an input was complete, that a human statement was honest, or that an external system behaved outside captured evidence.

Aether is the preferred stronger proof and capability provider through provider-neutral interfaces. HelixCore keeps a local signer, verifier, policy engine, and metadata ledger so Aether outage or removal causes reduced assurance, not platform failure.

# 13. UX system

- **Main surfaces:** Home, Work, Projects, Authority, Evidence, Recovery, Operations, and Settings.
- **Navigation:** one stable left rail, command search, recent items, and context breadcrumbs; advanced controls appear only after the user asks for detail.
- **Progressive reveal:** first show outcome, state, next action, and risk; then steps, evidence, logs, policy, and raw diagnostic data.
- **Input:** every action works by keyboard; touch targets are large enough for touch; focus and selection remain visible.
- **Accessibility:** target [WCAG 2.2](https://www.w3.org/TR/WCAG22/) AA; automated checks plus keyboard, screen-reader, zoom, contrast, and reduced-motion human tests.
- **Slow work:** show queued time, current phase, completed/total when knowable, last heartbeat, elapsed time, cancel safety, and an activity feed. Unknown duration uses phases, not fake percentages.
- **Completion:** in-app notification, optional operating-system notification, durable inbox item, and direct link to proof or recovery.
- **Move and selection:** selected objects show a check mark and count; moves show destination and affected links before commit; undo is offered when safe.
- **Delete:** explain the 30-day bin and expiry. Permanent deletion is never the default button.
- **Empty and error states:** say what is missing, why it matters, what is safe now, and the exact next action. Never show a blank spinner forever.

# 14. Interoperability and standards

- [OpenID Connect Core](https://openid.net/specs/openid-connect-core-1_0.html) is used behind the identity adapter for portable sign-in claims. Local subject and tenant identity remain HelixCore records; provider-specific assurance may be lost during import.
- [Web Authentication](https://www.w3.org/TR/webauthn/) is used for passkeys and strong browser authentication. Authenticator attestation and recovery features vary by platform.
- [Open Container Initiative specifications](https://specs.opencontainers.org/) are used behind workload packaging and runtime adapters. Host devices and policy capabilities may not survive an image move.
- [OpenTelemetry specification](https://opentelemetry.io/docs/specs/otel/) is used for portable traces, metrics, and logs. Vendor-specific dashboards and derived alerts may not export.
- [JSON Schema 2020-12](https://json-schema.org/draft/2020-12) is used for versioned capability, event, and export contracts. Custom policy meaning may need a Helix extension.
- [RFC 8032 EdDSA](https://www.rfc-editor.org/info/rfc8032/) defines the default Ed25519 proof signature profile. Custody adapters may use another reviewed profile while preserving verifier metadata.

# 15. Cross-platform contract

- Core contracts, migrations, CLI behaviour, backup/restore, and verification run in CI on current supported Windows, macOS, and Linux runners.
- Browser behaviour is tested in the supported Chromium, Firefox, and WebKit engines. Desktop packaging is optional per product, not required for the control plane.
- Containers are a Linux deployment option, not the only development path.
- Local mode uses platform capability detection for key custody, notifications, process isolation, file permissions, and background work.
- When a platform lacks a secure feature, the UI states the reduced assurance and offers a safe software fallback or blocks the action.
- Offline mode supports cached reads, local drafts, local proof verification, and explicit queued work. Cross-tenant grants and remote publication require reconnection.

# 16. Reliability and performance budgets

| Measure | Budget |
|---|---|
| Committed ledger data loss | 0 accepted domain changes without their matching event in every rolling 30-day window |
| Transactional record recovery point | 0 seconds for acknowledged writes; asynchronous object replicas may lag by at most 5 minutes |
| Control API availability | At least 99.95% successful eligible requests per rolling 30 days, excluding announced owner maintenance |
| Read latency | p95 under 300 ms for tenant-scoped control reads over each rolling 24 hours at reference load |
| Write latency | p95 under 750 ms for normal control writes over each rolling 24 hours, excluding user approval time |
| Job visibility | First state within 1 second; heartbeat age shown; worker considered lost after 30 seconds without heartbeat |
| Cancellation | Request acknowledged within 2 seconds; safe step stops within 30 seconds or reports why it cannot |
| Core recovery time | Restore control service within 30 minutes; complete tenant restore rehearsal within 4 hours at reference size |
| Idempotency | Duplicate accepted command produces one domain effect across a 24-hour retry window |
| Degradation | Identity, policy, or ledger loss blocks writes; proof-provider loss falls back locally; telemetry loss marks health unknown |
| Scale target | 10,000 active tenants, 100,000 concurrent jobs, and 1 billion retained events per regional control plane after measured load proof |

# 17. Success measures

- 90% of new self-hosted operators complete installation and a restore drill without support.
- 95% of capability requests are decided with a clear reason and no raw-secret exposure.
- Zero confirmed cross-tenant data disclosures per rolling 12 months.
- 100% of user-facing releases have a fresh, independently checkable gate report.
- At least 95% of destructive mistakes reported within 30 days are restored from the bin.
- All critical journeys pass keyboard and screen-reader tests on every release candidate.
- A standard tenant moves between two supported deployment providers within one working day with no identity loss.
- Fewer than 2% of successful jobs end without a user-visible completion notification.
- Revenue and entitlements reconcile with provider records with zero unexplained monetary difference at monthly close.

# 18. Delivery plan

| Gate | Build | Test and safety | UX | Cross-platform | Migration and operator proof |
|---|---|---|---|---|---|
| G0 — Truthful foundation (0–6 months) | Fix full build; stable IDs; tenant storage policy; atomic ledger; durable jobs; truthful readiness | Crash matrix, tenant adversarial tests, real signing round trip, fresh release gate | Canonical shell and honest job states | Rust and CLI matrix on Windows, macOS, Linux | Import current IDs without path dependence; restore drill and operator runbook |
| G1 — Useful single-player product (6–18 months) | Local home, capability broker, recovery bin, proof bundles | Secret non-disclosure, lease expiry, delete/restore, candidate-binding tests | Home, Work, Authority, Evidence, Recovery complete | Local install and upgrade on all three systems | Legacy key backend read-only; export/import proof |
| G2 — Trusted team product (18–30 months) | Team policy, multi-tenant operations, billing, portable home | RLS, two-person approval, payment idempotency, failover | Team inbox, incident and migration flows | Self-hosted and managed parity | Live provider-to-provider move and rollback |
| G3 — Category leader (30–42 months) | Independent proof exchange and policy simulation | External verifier and policy-change corpus | Proof comparison and safe rollout views | Offline verifier everywhere | Aether adapter plus local fallback exercised |
| G4 — Frontier network (42–60 months) | Sovereign compute placement, cross-home trust fabric, and federated continuity and exit | Residency, custody, compromise, partition, revocation, recovery-threshold, and exit tests | Explainable placement, trust, custody, recovery, and exit controls | Heterogeneous Windows, macOS, Linux execution and clean restore | Three-home capability-revocation and continuity exercises plus category audit |

A gate closes only when fresh commands run against the exact candidate and produce retained evidence. A document, old screenshot, or prior green run cannot close a gate.

# 19. Current truth and gap

**Present in live source:** ten shared Rust crates; six core services; typed IDs and scopes; Postgres repositories and migrations; NATS and MinIO adapters; Ory identity adapters; AES-GCM vault code; an audit hash chain with transactional append locking; service middleware; Docker Compose; a Helm chart; and a small Next.js console. Forty-one targeted core library tests passed. Console type checking, Compose validation, and Helm lint passed during the source audit.

**Scaffold or unsafe today:** the root workspace does not compile; formatting fails; the root is not a Git repository; CI is Ubuntu-only and contains soft-failing security steps; the toolchain is pinned to a Windows host; the later package directories and root integration/e2e suites are empty. Readiness always returns HTTP 200. The public registration path can create a caller-selected tenant identity with admin scopes. Database tenant isolation is not enforced with row policies. Billing can simulate a paid result. Audit content verification is permissive when HMAC is off, archive verification is unfinished, and vault read/admin scopes can expose values or plaintext data keys.

**Most important gap:** the foundation cannot yet make a truthful promise that identity, tenant data, money, secrets, and evidence remain safe across failure.

**Safest first vertical slice:** make one project registration and one durable job run use stable identities, database-enforced tenant policy, atomic domain-plus-ledger commit, truthful readiness, visible progress, cancellation, a 30-day bin, and a fresh cross-platform release gate. Do not widen the product catalog until this slice passes.

# 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Canonical identity | Stable generated IDs plus versioned bindings; never folder paths | Architecture decision and migration proof |
| Source of transactional truth | Postgres for shared deployments; same repository contracts in local mode | Architecture decision |
| Event delivery | Transactional outbox; consumers idempotent | Architecture and failure-test proof |
| Ledger writes | Domain change, event, and outbox are all-or-nothing | Founder approval plus proof-equivalent design |
| Ledger signatures | Asymmetric signature with public offline verification; Ed25519 default profile | Security review |
| Tenant isolation | Database policy and tenant-bound keys plus application checks | Security owner approval |
| Agent secrets | Metadata-only discovery; process-bound capability use; no value retrieval | Founder approval |
| Delete | 30-day recoverable bin; permanent purge is separate | Founder approval |
| Long work | Durable job state, heartbeat, phases, notification, cancellation | Product decision with equivalent UX proof |
| Shipping UI | One canonical control plane | Founder approval |
| Aether | Preferred adapter for proof and capabilities; local fallback always present | Founder approval |
| Legacy keys | Read-only legacy backend during migration and rollback | Security owner and migration proof |
| Platform support | Windows, macOS, and Linux gates block release | Founder approval |
| Billing outside local | Disabled until real provider, idempotency, reconciliation, and atomic entitlement tests pass | Finance and security approval |
| Unknown health | Display unknown or degraded; never coerce to healthy | No exception |
| Founder-only choices | Managed-service commercial model and final custody providers | Founder decision; does not block G0 |

# 21. Definition of category-defining done

- [ ] Real people complete all signature journeys in local, self-hosted, and managed modes.
- [ ] Identity survives folder, machine, and provider moves.
- [ ] Tenant isolation has application, database, and adversarial proof.
- [ ] Domain changes and audit receipts are atomic under concurrency and crash injection.
- [ ] Independent tools verify proof bundles and state clear limits.
- [ ] Agents use exact capabilities without receiving secret values.
- [ ] Every long task shows live state, can be stopped safely, and ends with a notification.
- [ ] Delete is recoverable for 30 days and permanent purge is explicit and proven.
- [ ] Windows, macOS, Linux, browser, CLI, container, and offline limits are tested and published.
- [ ] Critical journeys meet WCAG 2.2 AA and pass human assistive-technology review.
- [ ] Backup, restore, migration, rollback, key rotation, incident, and regional failure drills meet the budgets.
- [ ] Security review closes all critical and high findings or records a named, time-bound exception.
- [ ] Billing never grants value from simulated, missing, replayed, or uncertain payment.
- [ ] Aether can be removed without stopping core operation.
- [ ] The product states what its evidence, health checks, agents, and automation do not prove.
