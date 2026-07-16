---
product: HelixFlow
catalog_order: 03
status: target-state-spec
horizon: 60 months
current_maturity: prototype
primary_users: [individual operator, operations team, process owner, automation builder, approver, integration engineer]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
---

# 1. Category claim

HelixFlow is a sovereign work orchestration product that lets people automate long, cross-system processes while keeping every step visible, bounded, recoverable, and under human authority.

# 2. Five-year destination

- **Useful product:** A visual and code-friendly builder creates scheduled, event-driven, human, service, and agent workflows with durable runs, approvals, retries, compensation, and clear operations.
- **Category-defining advantage:** The product treats authority, evidence, reversibility, and honest uncertainty as part of the workflow itself, not as logs added later.
- **Frontier capability:** Workflows can coordinate trusted capabilities across several sovereign homes and improve from measured outcomes without giving agents open-ended control.
- **Human authority:** People approve high-impact steps, publish definitions, grant capabilities, accept exceptions, resolve meaning conflicts, and authorise irreversible external actions.

# 3. Users and hard jobs

| User | Hard jobs | Failure feared most |
|---|---|---|
| Individual operator | Automate repeat work and understand every run | A hidden automation acts twice or silently stops |
| Operations team | Coordinate long processes across people and systems | Partial completion leaves systems inconsistent |
| Process owner | Define rules, approvals, deadlines, evidence, and outcomes | The diagram looks right but execution means something else |
| Automation builder | Create, test, version, deploy, and migrate workflows | A connector or schema change breaks live runs |
| Approver | Understand the request, scope, evidence, urgency, and effect | A misleading approval screen authorises too much |
| Integration engineer | Build safe connectors and event mappings | Secrets leak or duplicate events cause duplicate actions |
| Auditor | Reconstruct why a run acted and what remains uncertain | Logs are incomplete, mutable, or detached from the definition |

# 4. Product laws

1. A published workflow is an immutable version. Editing creates a new draft.
2. Every run binds the exact definition, inputs, authority, connector versions, and policy.
3. External effects require idempotency, a declared retry rule, and a recovery or compensation plan.
4. An approval never defaults to approved because of timeout, outage, or missing person.
5. Agents cannot publish workflows, approve their own steps, expand scope, or retrieve secret values.
6. Slow work always shows queue, current branch and step, heartbeat, elapsed time, next event, and safe controls.
7. A crash resumes from a durable checkpoint or reports the exact uncertain boundary.
8. Delete enters a 30-day recoverable bin by default.
9. Simulation and test mode cannot touch production systems or production state.
10. Connector failure, skipped work, unknown outcome, and partial compensation are distinct visible states.
11. The same definition has tested behaviour on Windows, macOS, and Linux where its declared capabilities exist.
12. High-stakes advice or automation never replaces the licensed or accountable human decision.

# 5. Scope boundaries

**HelixFlow owns:** workflow definitions, triggers, schedules, conditions, loops, parallel branches, human tasks, approvals, timers, connector calls, agent steps, run state, checkpoints, retries, compensation, simulation, version migration, run evidence, workflow recovery, and operator UX.

**HelixCore owns:** identity, tenant and policy decisions, capabilities and secret-safe use, durable job primitives, objects, audit/proof, billing, notifications, operations, and stable project identity. Domain products own the meaning and validation of their records.

**HelixFlow will not attempt:** to be a general programming language; to become a raw-secret vault; to hide non-deterministic agent work inside a normal connector; to promise exactly-once delivery across systems that do not support idempotency; or to make legal, medical, financial, employment, or safety decisions without the responsible human.

# 6. Signature experiences

| Journey | Entry point | Visible progress | Human decision | Completion proof | Failure and recovery | Export or portability |
|---|---|---|---|---|---|---|
| Build and publish a workflow | New workflow | Draft validation, missing inputs, authority map, simulation, review, publish state | Owner approves definition, capabilities, and failure policy | Signed immutable definition and fresh test report | Invalid draft remains editable; prior published version stays active | Export definition, schemas, policies, and test fixtures |
| Run a long multi-step process | Run now or trigger | Queue, worker, branch map, current step, completed/total, heartbeat, next wait, elapsed time | Approve gated steps or cancel | Run bundle with inputs, step receipts, outputs, and limitations | Resume checkpoint, retry safe step, compensate, or mark outcome unknown | Export run and proof bundle |
| Handle a human approval | Approval inbox | Purpose, requester, exact scope, affected records, evidence, deadline, alternatives | Approve, narrow, reject, delegate, or ask for more evidence | Signed decision bound to run and step | Timeout follows explicit reject/escalate policy, never approve | Export decision record without secret values |
| Recover from partial external failure | Incident or failed run | Succeeded effects, failed effect, uncertainty, compensation plan, live recovery | Process owner chooses retry, compensate, accept, or manual repair | Recovery receipt and fresh reconciliation | Failed compensation remains open and alerts accountable owner | Export incident and reconciliation evidence |
| Receive an event safely | Trigger center | Validation, deduplication, mapping, policy, workflow start | Owner approves new source and production binding | Trigger receipt linked to event identity and run | Duplicate is ignored; invalid event quarantined; replay is controlled | Export mapping and quarantined event metadata |
| Upgrade a live workflow | Versions | Compatibility scan, active-run count, migration simulation, canary, rollout | Owner chooses drain, migrate, or keep old runs | Version rollout report and rollback point | Failed canary halts; active runs keep their bound semantics | Export all definition versions and migrations |
| Delete and restore automation | Workflow menu or Recovery | Dependants, schedules, active runs, bin expiry, restore preview | Confirm bin move; elevated approval for purge | Tombstone and restored binding report | Active effects are stopped or explicitly detached before deletion | Export before purge |
| Delegate a bounded agent step | Step builder and run view | Context sources, capability request, plan, tool use, output, checks | Human approves scope and any external effect | Agent-step evidence and accepted/rejected output | Stop revokes lease; partial output stays draft; retry uses a new attempt | Export agent output and metadata-only tool evidence |

# 7. Capability map

## F0 — Foundation

### FLOW-F0-001 — Versioned workflow contract

- **Gate:** G0. **Inputs:** typed triggers, nodes, edges, data contracts, policies, failure rules. **Outputs:** validated immutable definition version.
- **Invariants:** published versions never mutate; graph and executable semantics match. **Authority:** builder drafts; owner publishes.
- **Evidence:** schema validation, semantic checks, tests, and approval. **Failure:** unknown node or unbound input blocks publish.
- **Acceptance:** importing, exporting, and reloading a version produces the same executable graph and hash.

### FLOW-F0-002 — Durable run state machine

- **Gate:** G0. **Inputs:** definition version, trigger, inputs, idempotency key, authority snapshot. **Outputs:** ordered run and step states, checkpoints, result.
- **Invariants:** one legal transition at a time; acknowledged transition and audit commit together. **Authority:** engine advances only allowed states.
- **Evidence:** signed transitions and checkpoint hashes. **Failure:** worker loss returns to recovery, not success.
- **Acceptance:** forced termination at every state boundary causes no missing or impossible transition.

### FLOW-F0-003 — Exact capability execution

- **Gate:** G0. **Inputs:** workload identity, step capability, resource scope, duration, policy. **Outputs:** use result or denial.
- **Invariants:** values are never returned to workflow or agent; each step gets only its declared capability. **Authority:** human/policy broker grants.
- **Evidence:** metadata-only grant, use, denial, expiry, and revocation. **Failure:** expired lease stops before effect.
- **Acceptance:** a read-only connector step cannot write, list another tenant, or print its credential.

### FLOW-F0-004 — Recoverable workflow lifecycle

- **Gate:** G0. **Inputs:** disable, delete, restore, purge, legal hold. **Outputs:** stopped triggers, tombstone, 30-day bin, restore plan.
- **Invariants:** active runs and dependants are resolved explicitly; purge is separate. **Authority:** owner purges.
- **Evidence:** impact and lifecycle events. **Failure:** unresolved external effect blocks purge.
- **Acceptance:** a workflow deleted on day 1 restores on day 29 with schedules, versions, tests, and policies.

## F1 — Useful product

### FLOW-F1-001 — Visual and text workflow builder

- **Gate:** G1. **Inputs:** nodes, forms, schemas, expressions, reusable subflows. **Outputs:** accessible graph and canonical text form.
- **Invariants:** visual and text views round-trip without semantic loss. **Authority:** builder edits drafts.
- **Evidence:** validation and diff between versions. **Failure:** unsupported imported construct remains visible and non-runnable.
- **Acceptance:** keyboard-only users build, connect, configure, validate, and publish the reference workflow.

### FLOW-F1-002 — Triggers, schedules, and event inbox

- **Gate:** G1. **Inputs:** manual call, schedule, webhook, event, file, domain change. **Outputs:** validated deduplicated trigger or quarantine.
- **Invariants:** stable source plus event ID deduplicates; time zone is explicit. **Authority:** owner binds production sources.
- **Evidence:** receipt, mapping version, policy, and run link. **Failure:** invalid or late event follows named policy.
- **Acceptance:** replaying one event 100 times creates one logical run.

### FLOW-F1-003 — Human tasks and approval

- **Gate:** G1. **Inputs:** purpose, scope, evidence, options, deadline, eligible approvers. **Outputs:** signed decision and next transition.
- **Invariants:** no self-approval where separation is required; timeout cannot approve. **Authority:** named accountable human.
- **Evidence:** presented facts, decision, actor, and exact effect. **Failure:** unavailable approver escalates or rejects per policy.
- **Acceptance:** changing scope after approval invalidates the decision and reopens review.

### FLOW-F1-004 — Visible execution and operator control

- **Gate:** G1. **Inputs:** durable run events, worker heartbeat, logs, waits, external receipts. **Outputs:** live graph, timeline, alerts, controls.
- **Invariants:** progress comes from real state; unknown duration uses phases. **Authority:** allowed operator pauses, cancels, retries, or recovers.
- **Evidence:** operator actions and resulting state. **Failure:** stale heartbeat becomes lost-worker state within budget.
- **Acceptance:** a one-hour waiting run never appears frozen or complete and sends a final notification.

## F2 — Category leader

### FLOW-F2-001 — Connector framework

- **Gate:** G2. **Inputs:** versioned connector manifest, schemas, capabilities, rate and retry rules. **Outputs:** validated calls and typed results.
- **Invariants:** private network access off by default; errors are typed; secrets brokered. **Authority:** connector steward approves production.
- **Evidence:** resolved connector version, request shape, redacted response facts. **Failure:** incompatible schema blocks rollout.
- **Acceptance:** a connector upgrade canary detects a breaking response before active workflow migration.

### FLOW-F2-002 — Compensation and reconciliation

- **Gate:** G2. **Inputs:** completed effects, declared compensators, current external facts, owner policy. **Outputs:** restored, compensated, accepted, or unresolved state.
- **Invariants:** compensation is a new effect, never history deletion. **Authority:** owner approves irreversible recovery.
- **Evidence:** original and compensating receipts plus reconciliation. **Failure:** failed compensation stays open with owner and deadline.
- **Acceptance:** every reference multi-system failure ends reconciled or visibly unresolved, never falsely succeeded.

### FLOW-F2-003 — Simulation and safe rollout

- **Gate:** G2. **Inputs:** definition, synthetic/recorded redacted events, connector doubles, policy, load model. **Outputs:** paths, effects, cost, timing, risk, coverage.
- **Invariants:** simulation cannot use production capabilities or state. **Authority:** human publishes after review.
- **Evidence:** fixture versions, branch coverage, assertions, differences. **Failure:** uncovered high-risk branch blocks publish.
- **Acceptance:** production endpoints reject all simulation identities and capability contracts.

### FLOW-F2-004 — Team governance and process quality

- **Gate:** G2. **Inputs:** ownership, review rules, run outcomes, waits, errors, exceptions. **Outputs:** bottleneck, risk, and improvement proposals.
- **Invariants:** recommendations show source data and uncertainty; speed never overrides safety policy. **Authority:** process owner changes definition.
- **Evidence:** measurement window, sample, exclusions, approved change. **Failure:** sparse data remains insufficient evidence.
- **Acceptance:** every suggested change links to affected runs and a safe experiment.

## F3 — Advanced category leadership

### FLOW-F3-001 — Human-agent workflow cells

- **Gate:** G3. **Inputs:** bounded task, selected context, tools, capability lease, evaluation. **Outputs:** draft, classification, plan, or tool result.
- **Invariants:** non-deterministic step is explicit; agent cannot approve itself or hide source. **Authority:** human gates consequential effect.
- **Evidence:** model/provider, context references, plan, tool uses, output, checks, and limitations without secrets.
- **Failure:** timeout or low confidence routes to a person, not silent fallback.
- **Acceptance:** replacing the model provider changes no authority rule and preserves the same evidence contract.

### FLOW-F3-002 — Living process twin

- **Gate:** G3. **Inputs:** version graph, current run state, resource facts, simulated events, policy. **Outputs:** forecast, failure paths, safe alternatives.
- **Invariants:** forecast is advice, not an event; assumptions remain visible. **Authority:** owner chooses intervention.
- **Evidence:** model version, inputs, assumptions, error history. **Failure:** poor calibration disables automated suggestions.
- **Acceptance:** forecasts are scored against later outcomes and withdrawn when error budget is exceeded.

### FLOW-F3-003 — Sovereign cross-home orchestration

- **Gate:** G4. **Inputs:** signed inter-home contract, event, capability boundary, proof policy. **Outputs:** coordinated sub-runs and joint receipt.
- **Invariants:** each home controls its own data and effects; no central secret or super-admin. **Authority:** accountable owner in each home approves binding.
- **Evidence:** bilateral grants, handoff receipts, local results, revocations. **Failure:** partition pauses unsafe transitions and leaves local state readable.
- **Acceptance:** three homes complete a shared workflow and any home can leave without corrupting the others.

## F4 — Frontier network

### FLOW-F4-001 — Federated process contract

- **Gate:** G4. **Inputs:** local event and action schemas, meaning definitions, policy limits, trust bindings, version ranges, and sample fixtures. **Outputs:** signed inter-home contract, explicit field map, semantic-loss report, tests, and expiry.
- **Invariants:** no field, unit, authority rule, failure meaning, or irreversible effect changes silently between homes. **Authority:** an accountable process owner in every participating home approves the shared contract and later versions.
- **Evidence:** source schemas, mappings, loss decisions, fixture results, approvals, versions, compatibility range, use events, and revocations. **Failure:** an unknown meaning, missing required field, incompatible version, or absent approval blocks activation.
- **Acceptance:** three homes with different local schemas agree one contract, detect a seeded missing required field before a run, and execute the corrected fixtures with the same declared meaning.

### FLOW-F4-002 — Distributed compensation and exit

- **Gate:** G4. **Inputs:** inter-home contract, sub-run receipts, completed effects, compensators, current external facts, owner decisions, and exit policy. **Outputs:** reconciled, compensated, accepted, safely paused, or visibly unresolved joint state plus an exit package.
- **Invariants:** each home controls its own effects; compensation is a new effect, never history deletion; retry cannot duplicate a completed external action. **Authority:** the affected home owner approves irreversible compensation and exit.
- **Evidence:** original effects, handoffs, idempotency keys, compensating actions, acknowledgments, unresolved owners, deadlines, revocations, and final reconciliation. **Failure:** partition or failed compensation stops dependent transitions and leaves exact local and joint uncertainty visible.
- **Acceptance:** one of three homes fails after an external effect, the others avoid duplicate action, complete the approved compensation or mark it unresolved, and let the failed home exit without corrupting retained histories.

# 8. Domain model

| Record | Owner and relationships | Lifecycle and version rules | Retention |
|---|---|---|---|
| Workflow | Tenant project; contains immutable versions | Stable ID; draft, active, disabled, binned | 30-day bin |
| WorkflowVersion | Trigger, graph, schemas, policies, tests, migration | Immutable once published | All versions used by retained runs |
| NodeDefinition | Typed action, input/output contracts, retry and recovery | Versioned as part of workflow | Follows version |
| EdgeDefinition | Source outcome, condition, target, data mapping | Versioned as part of workflow | Follows version |
| TriggerBinding | Source, mapping, dedupe, policy, target version | Effective-time versions | Audit retention |
| Schedule | Time rule, time zone, missed-run policy | Versioned; disable is explicit | Workflow life |
| Run | Exact workflow version, trigger, authority, status, timing | Durable legal state transitions | Operational policy plus proof summary |
| StepAttempt | Node, inputs refs, lease, worker, output refs, state | Immutable attempt; retry creates another | Run retention |
| Checkpoint | Completed frontier, context refs, external receipts | Content-addressed and immutable | Run retention |
| HumanTask | Purpose, scope, options, eligible people, deadline | Open, decided, expired, or cancelled | Decision retention |
| Decision | Exact presented facts, actor, outcome, comment | Immutable; changed scope creates new task | Long-term where consequential |
| ConnectorDefinition | Operations, schemas, capabilities, limits, owner | Signed published versions | All used versions |
| EventReceipt | Stable source/id, mapping, validation, disposition | Immutable; duplicate links first receipt | Audit policy |
| ExternalEffect | Target, idempotency key, request hash, outcome, receipt | Attempt history immutable | Run and legal policy |
| Compensation | Original effect, compensator, result, reconciliation | New effect linked to original | Run and incident policy |
| Simulation | Fixtures, connector doubles, coverage, assertions, result | Immutable result for exact version | Release evidence |
| RecoveryItem | Workflow/trigger/schedule tombstone and expiry | Restore, expire, or purge | 30 days default |

# 9. System architecture

- **Domain engine:** workflow compiler, durable state machine, scheduler, trigger inbox, policy/authority resolver, connector runtime, human-task service, compensation engine, simulator, and migration engine.
- **Application services:** builder API, worker coordinator, event gateway, schedule service, connector registry, approval inbox, run operations, notification service, and evidence exporter.
- **Adapters:** Helix product APIs, OpenAPI-described services, event brokers, files, mail/calendar, databases, local commands, agent providers, Aether, and local proof/capability fallback.
- **Storage:** tenant-enforced relational definition/run state; encrypted object storage for large inputs/outputs/logs; append-only proof receipts; no raw secret values in definitions or runs.
- **Event flow:** trigger receipt and run enqueue commit atomically; workers claim step leases; step transition and outbox commit together; consumers deduplicate.
- **Background work:** all workflow execution, migration, simulation, export, reconciliation, and agent work uses durable workers with heartbeats.
- **Offline behaviour:** local drafts, simulation with local fixtures, run-history reading, and proof verification work offline. Production triggers, remote capabilities, and cross-home transitions wait for connection.
- **Extension points:** node types, connectors, triggers, expression engines, agent providers, event transports, simulators, proof providers, and migration transforms.
- **Dependencies:** HelixCore supplies stable identity, policy, capability brokering, jobs, objects, proof, billing, notification, and operations. Aether is preferred but optional.

# 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Progress, check, stop, reverse |
|---|---|---|---|---|---|
| Workflow design assistant | Selected process notes, schemas, connector metadata, simulation tools | Draft graph, tests, failure plan, questions | Publish or bind production trigger | Read secrets, invent connector facts, publish itself | Draft phases; schema/simulation checks; stop; discard draft |
| Run operator assistant | One run’s redacted state, runbook, safe diagnostics | Diagnosis and recovery options | Retry effect, compensate, skip, or accept uncertainty | Change history or mark unknown as success | Timeline and evidence; fresh reconciliation; stop; restore checkpoint |
| Connector steward | Connector spec, schemas, test endpoint, compatibility corpus | Connector version and migration | Production capability and rollout | Use production in simulation or expose values | Test progress; contract tests; stop canary; rollback version |
| Approval summariser | Exact request, scope, evidence, policy, alternatives | Plain-language brief | The decision itself | Approve, narrow, delegate, or hide missing facts | Source links; validation; close request; decision creates next state |
| Process improvement assistant | Approved aggregate run facts and outcome measures | Bottleneck hypothesis and experiment | Definition or policy change | Optimise away required control or inspect private payloads | Analysis stages; back-test; stop; revert experiment |

# 11. Trust, safety, and privacy

- Access checks tenant, workflow, run, step, connector, action, purpose, and environment. Storage enforces tenant separation.
- Sensitive classes are personal data, business confidential data, regulated records, credentials, signing operations, production commands, payment effects, and incident evidence.
- Definitions reference named capabilities, never credentials. The broker gives one workload exact use for a short lease.
- Connectors have allowlisted destinations, private-network policy, request/response limits, redaction, rate limits, replay control, and clear data residency.
- Human tasks show purpose, data, exact effect, requester, alternatives, deadline, and policy. Consent and decision withdrawal effects are explicit.
- Test and simulation identities cannot reach production state or capabilities.
- Delete moves definitions, triggers, schedules, and safe artifacts to the 30-day bin. External effects are not “deleted”; they are compensated or reconciled.
- Legal hold blocks purge of required run and decision evidence.
- Misuse controls include trigger rate limits, connector quotas, anomaly detection, two-person break-glass, immediate lease revocation, and dead-letter review.
- Incident recovery freezes triggers, revokes capabilities, preserves receipts, identifies uncertain effects, compensates where safe, and reruns reconciliation.

# 12. Proof and audit

Important definition, publication, trigger, run, step, capability, approval, connector, external effect, compensation, migration, operator, delete, restore, and purge actions create canonical signed events.

A run bundle contains the exact workflow version, trigger receipt, input references, authority snapshot, connector versions, step attempts, idempotency keys, external receipts, approvals, outputs, checks, compensation, final reconciliation, and known unknowns. An independent verifier can check integrity, ordering, signatures, declared policy, and captured external receipts. It cannot prove that an external service told the truth, that an uncaptured side effect did not occur, or that an agent output is correct.

Aether is the preferred proof and capability provider through an adapter. HelixCore retains local proof, policy, and capability fallback.

# 13. UX system

- **Main surfaces:** Home, Workflows, Builder, Runs, Approvals, Events, Connectors, Evidence, Incidents, and Recovery.
- **Navigation:** stable left rail; workflow/run switcher; visual graph and text view; right inspector; timeline below; command search.
- **Progressive reveal:** start with outcome, current step, risk, and next action; reveal data mapping, policies, raw event, logs, retries, and proof on demand.
- **Keyboard and touch:** every node/edge action has a keyboard and form alternative; touch supports monitoring, approvals, pause/cancel, and recovery.
- **Accessibility:** target [WCAG 2.2](https://www.w3.org/TR/WCAG22/) AA; graph has a complete ordered-list representation and screen-reader labels.
- **Slow work:** queue, worker, branch progress, current step, real counts, heartbeat, elapsed time, waits, retry time, cancel safety, and completion notification are durable.
- **Approval:** one screen shows exact scope and effect; approve and reject are visually balanced; changed data invalidates the decision.
- **Selection and move:** selected nodes show checks and count; move/copy previews edge, variable, permission, and version impact with undo.
- **Delete and recovery:** explain trigger stop, active runs, dependants, 30-day expiry, restore, and purge.
- **Errors:** distinguish failed, timed out, cancelled, compensated, partially compensated, unknown, skipped, and blocked. Never collapse them to red/green only.

# 14. Interoperability and standards

- [JSON Schema 2020-12](https://json-schema.org/draft/2020-12) defines workflow input, output, connector, event, and portable bundle contracts. Runtime policy and semantic meaning remain Helix extensions.
- [CloudEvents specification](https://github.com/cloudevents/spec) is used behind event-source adapters. Delivery guarantees, identity, tenant policy, and deduplication remain explicit Helix contracts.
- [OpenAPI Specification](https://spec.openapis.org/oas/) imports HTTP operation descriptions into connector drafts. It does not provide authority, idempotency, privacy, or business meaning, so those must be added before production use.
- [Business Process Model and Notation](https://www.omg.org/spec/BPMN/) is an optional import/export adapter for process diagrams. Agent steps, capability leases, proof, and some recovery rules may require extensions and a loss report.
- [OpenTelemetry specification](https://opentelemetry.io/docs/specs/otel/) carries traces, metrics, and logs across runtime adapters. Workflow proof and decisions are separate durable records.

# 15. Cross-platform contract

- Definition validation, simulation, local worker, CLI, proof verification, import/export, and recovery pass CI on Windows, macOS, and Linux.
- Browser builder and operations views pass supported Chromium, Firefox, and WebKit tests.
- Desktop or local-agent features use capability detection for commands, file watches, notifications, key custody, and background execution.
- OS-specific commands must declare supported platforms and a safe alternate path. A workflow cannot claim portable when one required node is not.
- Containers are a Linux worker option. Native Windows and macOS workers remain required for platform-specific effects.
- Offline mode supports drafts, fixtures, simulation, queued local-only work, run-history reading, and proof verification. Remote effects wait.

# 16. Reliability and performance budgets

| Measure | Budget |
|---|---|
| Acknowledged transition loss | 0 run or step transitions lost per rolling 30 days |
| External duplicate effect | 0 duplicate effects for connectors that meet the idempotency contract per rolling 30 days |
| Trigger acceptance | p95 under 500 ms over each rolling 24 hours at reference load |
| Trigger to queued run | p95 under 1 second over each rolling 24 hours |
| Worker visibility | Heartbeat at least every 10 seconds; lost after 30 seconds without heartbeat |
| Cancellation | Acknowledged within 2 seconds; current safe boundary reached within 30 seconds or blocker shown |
| Approval delivery | 99.9% of eligible approval notifications delivered or visibly failed per rolling 30 days |
| Recovery point | 0 seconds for acknowledged state; object replica lag at most 5 minutes |
| Recovery time | Resume recoverable run within 5 minutes; reconcile reference incident within 1 hour |
| Definition availability | At least 99.95% eligible API success per rolling 30 days |
| Scale | 1 million active workflow definitions, 100,000 concurrent runs, and 100 million step transitions per day after measured proof |
| Graceful degradation | Policy/capability/ledger loss blocks effects; notification loss leaves inbox; telemetry loss marks unknown; optional Aether loss uses local fallback |

# 17. Success measures

- 90% of builders publish the reference workflow with tests, failure policy, and proof without support.
- 100% of consequential external steps declare authority, idempotency, and recovery behaviour.
- Zero confirmed cross-tenant effects or raw-secret exposures per rolling 12 months.
- At least 99.9% of completed runs have a terminal state and completion notification per rolling 30 days.
- At least 95% of recoverable workflow deletions requested within 30 days are restored.
- 100% of high-impact approvals bind exact unchanged scope and a named human.
- All critical builder, approval, operations, incident, and recovery journeys pass keyboard and screen-reader checks.
- Portable exports reproduce the same executable graph and validation result on a clean machine.
- Teams reduce manual handling time while keeping incident and duplicate-effect rates at or below their prior process.

# 18. Delivery plan

| Gate | Build | Test and safety | UX | Cross-platform | Migration and operator proof |
|---|---|---|---|---|---|
| G0 — Truthful foundation (0–6 months) | Full compile; typed immutable definitions; durable workers; atomic transitions; exact capabilities; 30-day bin | Crash, replay, tenant, secret, idempotency, cancellation tests | Honest run timeline and recovery states | Engine/CLI matrix on Windows, macOS, Linux | Import current four-step workflows and restore drill |
| G1 — Useful single-player product (6–18 months) | Builder, triggers, schedules, human tasks, visible execution, local connectors | Visual/text equivalence, timeout, event dedupe, accessibility | Complete builder, run, approval, evidence, recovery | Browser and local worker proof | Clean-machine export/import |
| G2 — Trusted team product (18–30 months) | Connector framework, compensation, simulation, governance | Production isolation, connector compatibility, partial-failure corpus | Connector, incident, process-quality views | Native/container capability matrix | Team cutover, rollback, and reconciliation drill |
| G3 — Category leader (30–42 months) | Agent cells and living process twin | Agent authority, calibration, provider replacement, independent verifier | Agent and forecast explanation | Offline verifier everywhere | Aether and local fallback exercise |
| G4 — Frontier network (42–60 months) | Sovereign cross-home orchestration, federated process contracts, distributed compensation, and exit | Schema-meaning, missing-field, partition, malicious peer, residency, idempotency, compensation, revoke, and exit tests | Cross-home trust, mapping, handoff, reconciliation, uncertainty, and exit controls | Three-home heterogeneous contract and run matrix | Contract negotiation, failed-home compensation, federation exit, and disaster exercise |

Every gate closes only from fresh build, test, safety, UX, platform, migration, and operator checks against the exact candidate.

# 19. Current truth and gap

**Present in live source:** one 530-line Rust service with authenticated routes to create and list workflows, get workflow details, enqueue runs, list runs and step events, request cancellation, append audit events, meter usage, and publish a completion event. Postgres repository calls make workflow and run records durable when a database exists. The current executor records running, succeeded, failed, and cancelled states; checks cancellation between steps; and supports echo, set, noop, fail, plus an intentionally skipped HTTP type.

**Scaffold or unproven today:** execution is synchronous and in-process inside the request. There is no separate worker queue, lease, checkpoint recovery, connector framework, scheduler, event trigger, human approval, compensation, simulation, visual builder, or web application source beyond a package manifest. Step definitions are free-form JSON and only four simple local step types actually execute. Step-event append failures are ignored in places. Cancellation cannot interrupt a currently running step. The shared registered agent has only echo and product-catalog tools. The full workspace does not compile and root integration/e2e suites are empty.

**Most important gap:** the prototype stores workflow runs, but it is not yet a durable automation engine that can survive process loss or control real external effects.

**Safest first vertical slice:** one typed three-step workflow with a durable worker, one safe idempotent connector double, one human approval, one forced crash and resume, one compensation, visible live progress, completion notification, 30-day delete/restore, portable export, and fresh Windows/macOS/Linux gates.

# 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Definition | Immutable published versions with canonical graph and text form | Architecture decision |
| Execution | Durable worker lease and legal state machine; never in request process | Architecture decision |
| State truth | Step transition, audit event, and outbox commit together | Founder approval for proof-equivalent alternative |
| External effects | Idempotency key, retry class, timeout, and recovery/compensation required | No exception for production |
| Approval timeout | Reject, escalate, or remain waiting; never auto-approve | Founder approval |
| Agents | Explicit non-deterministic node; cannot publish or self-approve | Founder approval |
| Secrets | Brokered exact use; never stored in definition/run or returned to agent | Founder approval |
| Simulation | Separate identity, capabilities, and temp state; production access impossible | Security review |
| Delete | 30-day recoverable bin; external effects reconciled, not erased | Founder approval |
| Slow work | Durable phase, heartbeat, real progress, cancellation, notification | Product decision with equivalent UX proof |
| Builder | Visual and canonical text views must round-trip | Architecture decision |
| Connectors | Versioned adapter with typed errors and compatibility tests | Architecture decision |
| Aether | Preferred proof/capability adapter with local fallback | Founder approval |
| Cross-platform | Windows, macOS, Linux, browser gates block release | Founder approval |
| Unknown outcome | Separate terminal/incident state, never mapped to success | No exception |
| Founder-only choices | Managed connector marketplace and execution pricing | Founder decision; does not block G0 |

# 21. Definition of category-defining done

- [ ] Real users complete every signature journey, including failure and recovery.
- [ ] Durable runs survive worker, process, machine, network, and dependency failure within budgets.
- [ ] External effects are idempotent or visibly declared unsafe and human-gated.
- [ ] Approvals bind exact scope, never self-approve, and never default to approved.
- [ ] Independent proof reconstructs definition, trigger, authority, steps, effects, recovery, and limits.
- [ ] Every long run remains visibly alive, stoppable where safe, and sends a completion notification.
- [ ] Workflow deletion follows the 30-day recovery contract; external effects are reconciled.
- [ ] Visual and text definitions round-trip and portable exports run on a clean system.
- [ ] Windows, macOS, Linux, browser, native-worker, container, and offline limits are tested.
- [ ] Critical builder and operator journeys meet WCAG 2.2 AA and pass assistive-technology review.
- [ ] Agents and connectors use exact capabilities without receiving secret values.
- [ ] Backup, restore, migration, rollback, connector failure, compensation, and incident drills meet budgets.
- [ ] Security review closes all critical and high findings or records a named, time-bound exception.
- [ ] Aether, any model provider, and any connector vendor can be removed without losing owned definitions or run history.
- [ ] The product states what simulation, signatures, external receipts, forecasts, and successful runs do not prove.
