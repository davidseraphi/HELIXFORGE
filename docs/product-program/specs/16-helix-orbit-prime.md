# HelixOrbitPrime — space mission planning and operations twin

```yaml
product: HelixOrbitPrime
catalog_order: 16
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [mission designers, flight dynamics teams, flight controllers, spacecraft operators]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixOrbitPrime is a mission planning and operations twin that connects mission
intent, spacecraft and ground truth, orbit knowledge, constraints, contacts,
procedures, telemetry, decisions, anomalies, and proof while keeping command
authority and flight safety with named humans and independent mission systems.

## 2. Five-year destination

The useful product is a mission model, orbit and contact planner, procedure room,
telemetry replay, anomaly workspace, and evidence viewer. The leading product keeps
planning and operations synchronized through verified state estimates, constraint
checking, rehearsals, handoffs, and post-pass proof. The frontier product can compare
multi-spacecraft plans, ground networks, conjunction responses, and resilient mission
strategies under uncertainty. It never transmits a command, approves a maneuver,
changes a safety limit, allocates a ground resource, or declares a spacecraft safe
without the exact human authority and separately certified command path.

## 3. Users and hard jobs

- A mission designer needs to test concepts against mass, power, data, thermal,
  communications, orbit, and operations constraints.
- A flight dynamics team needs traceable state estimates, frames, epochs, covariance,
  force models, and maneuver assumptions.
- A scheduler needs conflict-free contacts and activities across spacecraft and ground
  networks.
- A flight controller needs current state, limit context, procedure steps, approvals,
  and clear last signal during slow work.
- An anomaly team needs raw telemetry, hypotheses, actions, and outcomes without
  hindsight rewriting.
- A mission assurance reviewer needs independent proof that planning, rehearsal,
  authority, and change control were followed.

## 4. Product laws

1. Planned, simulated, estimated, observed, commanded, and confirmed spacecraft state
   are distinct records.
2. Every orbit or attitude value names epoch, time system, reference frame, source,
   method, covariance or uncertainty, and version.
3. No agent can uplink, authorize, sign, release, or execute a spacecraft or ground
   command.
4. A command intent binds exact parameters, target, valid time, prerequisites,
   hazards, approvers, and expiry; any material change invalidates approval.
5. Contact and activity schedules show conflicts, margins, assumptions, and failed
   alternatives.
6. A delayed or missing signal is a visible state, never silent success.
7. Telemetry and external navigation/conjunction messages are untrusted until identity,
   schema, time, sequence, and plausibility checks pass.
8. Safe-mode, abort, launch, collision avoidance, and life-safety decisions remain
   with qualified mission authority and independent procedures.
9. Critical journeys are accessible, portable, and independently verifiable across
   platforms; no model, ground vendor, or proof provider is a hard dependency.

## 5. Scope boundaries

OrbitPrime owns provider-neutral mission models, orbit/contact planning, activity
scheduling, digital-twin simulation, procedure authoring and rehearsal, read-only
telemetry ingestion, anomaly investigation, conjunction decision support, and proof.
HelixCore owns shared identity, policy, audit, capabilities, jobs, objects, billing,
and operations; Aether is reached through a provider-neutral proof interface with a
local fallback.
It is not the command and control system, flight software, safety-rated ground
segment, launch range system, conjunction authority, or legal licensing system.
Command generation may be drafted as a non-executable intent, but encoding,
cryptographic release, uplink, and execution live behind an independent human-owned
boundary and are out of scope until separately certified.

## 6. Signature experiences

1. **Define a mission truth model.** Entry: the team imports approved spacecraft,
   payload, orbit, ground, and constraint data. Progress: identity, units, frames,
   versions, ownership, and missing facts appear. Human decision: subsystem owners
   accept their model. Proof: sources, transforms, reviews, and baselines are signed.
   Failure and recovery: incompatible units or identity are quarantined. Export: a
   provider-neutral mission bundle.
2. **Plan contacts and activities.** Entry: a planner selects horizon, spacecraft,
   ground assets, priorities, and constraints. Progress: orbit propagation, visibility,
   resource use, conflicts, margins, and optimization stage stream visibly. Human
   decision: operations authority accepts a schedule. Proof: inputs, solver version,
   rejected alternatives, and approvals are kept. Failure and recovery: cancelled
   planning resumes from a durable checkpoint. Export: schedule plus rationale.
3. **Rehearse a procedure.** Entry: a controller selects an immutable procedure and
   simulated state. Progress: prerequisites, steps, expected telemetry, branches,
   hazards, holds, and timing are visible. Human decision: qualified reviewers accept
   readiness. Proof: simulator version, injected faults, operator actions, and results
   are recorded. Failure and recovery: any procedure edit invalidates rehearsal proof.
   Export: a rehearsal evidence bundle.
4. **Run a live pass with read-only truth.** Entry: a controller opens an approved
   contact. Progress: link state, telemetry freshness, limits, procedure stage, last
   signal, and handoffs update without fake percent. Human decision: controllers use
   the independent command system for any action. Proof: received data and operator
   decisions are timestamped. Failure and recovery: signal loss enters a named state
   and activates the approved contingency checklist. Export: a pass record.
5. **Investigate an anomaly.** Entry: a rule or human opens an anomaly from observed
   data. Progress: timeline, data quality, affected subsystems, hypotheses, tests,
   decisions, and recovery state remain visible. Human decision: anomaly board accepts
   cause and corrective action. Proof: raw telemetry, versions, rejected hypotheses,
   and outcomes are kept. Failure and recovery: inconclusive cases stay open and can
   be replayed. Export: an anomaly review crate.
6. **Assess a conjunction and maneuver option.** Entry: an authorized conjunction
   message is received. Progress: identity, covariance, data age, screening, orbit
   assumptions, option simulation, mission cost, and uncertainty appear. Human
   decision: flight dynamics and mission authority decide whether to act in the
   independent command chain. Proof: messages, models, options, and decisions are
   signed. Failure and recovery: stale or invalid data blocks recommendation status.
   Export: a conjunction decision packet.

## 7. Capability map

F0 is foundation, F1 is the useful product, F2 is the trusted-team product, F3 is
advanced category leadership, and F4 is the frontier network. Every row inherits this full contract: its invariants are the
product laws and planned/simulated/observed/confirmed truth boundaries; authority is
the exact named human and policy in Sections 10–11; evidence is input/output hashes,
versions, actor, decision, and ledger event; failure is a durable `blocked`, `failed`,
`unknown`, stale, or quarantine state with retry or recovery; and test acceptance
includes denial, command-firewall, data-loss, recovery, and cross-platform cases in
addition to the row's named check. The row names its domain-specific inputs, output,
and strongest acceptance test.

| ID | First gate | Capability contract |
|---|---|---|
| HOP-F0-01 | G0 | **Stable mission, vehicle, and asset identity.** Inputs are approved registries and interface data; outputs are versioned identities independent of path or vendor. Acceptance: duplicate spacecraft, frame, unit, and epoch conflicts cannot merge silently. |
| HOP-F0-02 | G0 | **Mission event ledger.** Inputs are model, estimate, plan, procedure, telemetry, decision, and anomaly events; output is an append-only ordered history. Acceptance: concurrent writes and forced crash lose no acknowledged event. |
| HOP-F0-03 | G0 | **Authority and command firewall.** Inputs are role, mission phase, target, capability, purpose, time, and policy; output is exact read/draft permission or denial. There is no uplink capability. Acceptance: agents and core UI cannot reach command transport or signing keys. |
| HOP-F1-01 | G1 | **Orbit and attitude data kernel.** Inputs are versioned states, frames, epochs, time systems, covariance, and maneuvers; output is validated provider-neutral records. Acceptance: official message fixtures round-trip with declared loss. |
| HOP-F1-02 | G1 | **Contact and activity planner.** Inputs are states, ground assets, resources, priorities, constraints, and horizon; output is a conflict-checked candidate schedule. Acceptance: every accepted activity has margin and human approval. |
| HOP-F1-03 | G1 | **Procedure and rehearsal room.** Inputs are immutable steps, prerequisites, expected state, branches, hazards, and simulation; output is rehearsal evidence. Acceptance: a material edit invalidates earlier review and rehearsal. |
| HOP-F2-01 | G2 | **Read-only telemetry timeline.** Inputs are authenticated packets or archived data; output is raw-preserving, time-ordered parameters with quality and source. Acceptance: gap, duplicate, reorder, reset, and stale-signal fixtures remain visible. |
| HOP-F2-02 | G2 | **Resource and constraint twin.** Inputs are subsystem models, estimates, activities, and limits; output is power, data, thermal, pointing, propellant, and operations margins. Acceptance: unknown state cannot be shown as safe margin. |
| HOP-F2-03 | G2 | **Anomaly investigation graph.** Inputs are observed events, data quality, procedures, hypotheses, tests, and decisions; output is a versioned case. Acceptance: agents cannot close cause or approve corrective action. |
| HOP-F3-01 | G3 | **Conjunction decision support.** Inputs are authorized conjunction data, orbit/covariance versions, constraints, and maneuver options; output is conditional risk and trade-offs. Acceptance: no option becomes a command and stale inputs revoke ready status. |
| HOP-F3-02 | G3 | **Multi-mission ground scheduler.** Inputs are owner-approved demands, capabilities, priorities, and constraints; output is a fair candidate schedule with conflicts and rejected requests. Acceptance: resource allocation requires human authority. |
| HOP-F3-03 | G3 | **Portable mission proof bundle.** Inputs are approved models, messages, plans, telemetry references, procedures, decisions, and redactions; output is independently verifiable evidence. Acceptance: a clean machine validates hashes and replays allowed calculations. |
| HOP-F3-04 | G3 | **Resilient mission strategy studio.** Inputs are approved failure, weather, ground, orbit, supply, and operations scenarios; output is a set of robust strategies. Acceptance: it cannot select or enact a strategy. |
| HOP-F3-05 | G3 | **Bounded operations copilot.** Inputs are exact phase, approved procedures, current validated read-only state, and capability limits; output is questions, checks, and drafts. Acceptance: no command, limit change, or approval path exists. |
| HOP-F3-06 | G3 | **Continuous mission safety case.** Inputs are hazards, constraints, rehearsals, incidents, overrides, tests, and outcomes; output is a living argument with open gaps. Acceptance: independent mission assurance, not the system, accepts it. |
| HOP-F4-01 | G4 | **Multi-operator situational evidence exchange.** **Input:** operator-approved read-only orbit, covariance, conjunction, telemetry-quality, weather, ground-status, frame, time, unit, license, and expiry manifests. **Output:** a revocable cross-operator graph of agreement, conflict, freshness, and uncertainty. **Invariant:** raw restricted mission data stays in local custody; no result selects a maneuver, changes mission state, reaches command transport, exports a signing key, or becomes confirmed truth without human review. **Authority:** each mission's named flight authority and data steward approve contribution, use, and withdrawal; there is no global flight authority. **Evidence:** source and transform hashes, frame/time/unit checks, grants, queries, conflicts, reviews, expiry, and revocation. **Failure:** stale state, identity mismatch, incompatible frame or epoch, source loss, or withdrawal quarantines the affected edge. **Acceptance:** five independent simulated mission nodes process 100,000 frame, time, unit, conflict, join, and revoke cases with zero restricted-data escape or command-path reachability. |
| HOP-F4-02 | G4 | **Federated ground-resource negotiation room.** **Input:** human-approved demands, ground capabilities, time windows, priorities, constraints, fairness rules, margins, and withdrawal terms. **Output:** candidate allocations, conflicts, rejected requests, alternatives, and approval requests. **Invariant:** no resource is allocated, schedule activated, station configured, command generated, or mission commitment made until every affected owner gives separate human approval in its own system. **Authority:** each operator controls its requests and assets, while a named human coordination body records shared decisions. **Evidence:** demand versions, solver inputs, alternatives, fairness result, owner approvals, dissent, expiry, and withdrawal. **Failure:** missing approval, stale capability, margin breach, conflict, or withdrawal invalidates the affected candidate and preserves prior allocations. **Acceptance:** 10,000 synthetic multi-mission scheduling and revoke cases preserve every local veto, meet declared fairness/margin rules, and create zero command or automatic allocation. |
| HOP-F4-03 | G4 | **Cross-mission anomaly learning network.** **Input:** operator-approved redacted anomaly cases, source quality, hypotheses, tests, procedures, outcomes, context, and prohibited-use policy. **Output:** comparable patterns, disagreements, transfer limits, and candidate read-only checks for human investigation. **Invariant:** correlation is not causality; the network cannot close an anomaly, approve corrective action, change a procedure, select a strategy, or issue any physical or command action. **Authority:** each mission assurance body approves disclosure and interpretation, and independent human reviewers accept any cross-mission conclusion. **Evidence:** case and redaction hashes, context, analysis version, cited events, dissent, reviewer decision, use, and revocation. **Failure:** re-identification risk, incompatible context, missing source, disputed cause, or withdrawal removes the affected claim and marks dependent findings partial. **Acceptance:** 1,000 synthetic or historical anomaly replays expose every seeded context mismatch and false-causality claim with zero automatic closure, procedure change, or actuation. |

## 8. Domain model

| Record | Ownership, lifecycle, and relationships |
|---|---|
| `Mission` / `MissionPhaseVersion` | Stable identity, purpose, operator, authority, jurisdiction, phases, criticality, and configuration baseline. |
| `SpaceVehicle` / `SubsystemVersion` / `Payload` | Interfaces, units, limits, modes, dependencies, software/config versions, and subsystem-owner approval. |
| `GroundAsset` / `LinkCapability` | Location, owner, antenna/radio capability, schedule authority, validity, security class, and outages. |
| `OrbitState` / `AttitudeState` / `Covariance` | Epoch, time system, frame, method, source, units, uncertainty, prior state, and quality. |
| `ContactOpportunity` / `ActivityIntent` / `ScheduleVersion` | Window, resources, constraints, priority, conflicts, margins, rationale, and approval. |
| `ProcedureVersion` / `ProcedureStep` / `RehearsalRun` | Preconditions, expected state, branches, hazards, holds, role, simulation, operator actions, and result. |
| `TelemetryFrame` / `ParameterSample` / `QualityState` | Source, receive and onboard time, sequence, raw hash, decode version, unit, validity, and gap/reset links. |
| `CommandIntent` / `HumanApproval` | Non-executable target, parameters, valid time, prerequisites, hazards, exact payload hash, authority, and expiry. No transport lives here. |
| `AnomalyCase` / `Hypothesis` / `TestRun` / `Decision` | Trigger, timeline, evidence, candidate cause, test, rejected path, board decision, and outcome. |
| `ConjunctionCase` / `ManeuverOption` | Message identity, states/covariances, data age, methods, probability/uncertainty, constraints, options, and human decision. |
| `RecoveryItem` / `RetentionRule` / `LegalHold` | Deleted draft, restore date, mission assurance/incident hold, purge authority, and proof. |

## 9. System architecture

- A Rust mission kernel enforces stable identity, units, frames, time systems,
  state machines, authority, constraints, and atomic events.
- PostgreSQL stores durable mission metadata; encrypted object storage keeps raw
  messages, telemetry, model artifacts, and proof by content hash.
- Flight-dynamics and planning workers are sandboxed, deterministic where possible,
  resource-bounded, and isolated from command systems.
- Telemetry gateways are read-only and treat every message as untrusted until
  authentication, schema, sequence, time, and plausibility checks pass.
- The command boundary is a one-way exported intent and proof file for human review.
  OrbitPrime holds no command private key and exposes no uplink transport.
- A user-owned capability broker grants exact data/provider leases. Aether is the
  preferred proof service, with local signatures and offline verification as fallback.
- HelixCore supplies shared identity, policy, audit, capabilities, jobs, objects,
  billing, and operations behind domain interfaces; OrbitPrime retains mission truth.
- The event flow is request or read-only ingest, mission-authority check, frame/time/
  domain validation, atomic record plus event, projection, notification, and proof.
  Background work uses durable HelixCore jobs, idempotent checkpoints, visible
  solver progress, and explicit cancellation.
- Offline planning, rehearsal, replay, recovery, and verification use a local encrypted
  store. Versioned navigation, telemetry, ground, solver, and proof adapters are
  contract-tested extension points; none may reach the command boundary.
- A replay environment can recreate planning and analysis from immutable inputs
  without touching production mission state.

## 10. Agent and automation contract

| Role | May do | Must not do |
|---|---|---|
| Mission model assistant | Check units, interfaces, constraints, and missing facts | Accept subsystem truth or alter a baseline |
| Planner | Generate and compare bounded candidate schedules | Allocate resources, approve, or publish a schedule |
| Procedure assistant | Draft steps, expected states, tests, and review questions | Release, execute, or bypass a hold point |
| Telemetry analyst | Organize validated read-only data and flag anomalies | Change spacecraft state, limits, or anomaly status |
| Flight-dynamics analyst | Compare approved orbit and maneuver cases | Approve a maneuver or create an executable command |

Each agent lease names mission, phase, assets, data classes, tools, time, compute, and
output. It cannot retrieve secrets, signing keys, or command transport. Progress
shows current stage, solver/model, elapsed time, last signal, data age, failed cases,
and waiting human authority. Cancellation and revocation are signed; external work
remains pending until stop is confirmed. Every draft enters named mission review;
frame, time, unit, schema, authority, and evidence checks validate the result.
Reversal restores a prior version or revokes a grant without rewriting history.

## 11. Trust, safety, and privacy

| Safety case | Prevention, human authority, proof, and recovery |
|---|---|
| Unauthorized spacecraft action | No command transport or signing keys; one-way non-executable intent export; exact human mission authority and independent command system required. |
| Wrong frame, epoch, time, or unit | Typed records, explicit conversions, official fixtures, invariant checks, independent calculation, and review before plan acceptance. |
| Stale or corrupt telemetry/nav data | Authentication, schema, sequence, time, covariance, plausibility, and source-age checks; invalid data is quarantined. |
| Unsafe schedule or procedure | Constraint margins, configuration binding, rehearsal, hold points, dual review for critical steps, and invalidation after change. |
| Conjunction false confidence | Multiple sources where available, covariance and data-age display, sensitivity, independent flight-dynamics review, and no auto-maneuver. |
| Unsafe deletion | Draft plans and analyses enter a 30-day bin where lawful. Mission assurance, incident investigation, regulator, license, contract, export-control, or legal hold may block purge. Access can be quarantined immediately. Permanent deletion requires mission data authority, impact preview, re-authentication, and signed proof. |

Mission hazard analysis, command-boundary penetration tests, independent flight-
dynamics validation, human-factors testing, contingency rehearsal, export-control
review, and disaster recovery are release gates. Human spaceflight and launch
operations require separate, stricter certification and are not assumed in scope.
Tenant and mission separation are enforced in the database and object layer. Data is
encrypted in transit and at rest, residency follows mission policy, and incident
recovery can quarantine access, revoke leases, preserve evidence, and restore a
reviewed state without reaching command systems.

## 12. Proof and audit

Proof records mission/configuration identity, source messages, frames, epochs, time
systems, units, model and solver versions, constraints, schedules, margins, procedure
versions, rehearsals, telemetry hashes, data gaps, hypotheses, human decisions,
command intents, and known limits. Sensitive mission data and secret values stay out
of metadata-only events. Aether is preferred for signed proof; a local signed bundle
and offline verifier are required. Proof does not show that a spacecraft executed a
command, that telemetry is physically true, or that a maneuver is safe without
independent mission evidence.

## 13. UX system

The primary surfaces are Home, Mission, Orbit, Contacts, Plan, Procedures, Live Pass,
Telemetry, Anomalies, Conjunctions, Evidence, and Recovery. The persistent context bar
shows mission, vehicle, configuration, phase, time system, data age, link state, last
signal, and authority boundary. Planned, simulated, observed, and confirmed state use
both text and shape, not color alone. Detail reveals from operator summary to frames,
covariance, raw messages, and solver trace. Long planning shows stages, candidate and
rejected counts, elapsed time, last signal, and truthful uncertainty. Completion,
signal loss, limit breach, stale data, conflict, and approval need create clear private
notifications. Moving an activity or editing a procedure previews affected resources,
margins, approvals, and rehearsals. Reversible edits offer undo; empty states explain
the first safe action; plain-language errors state what happened, what remains safe,
and how to recover. Keyboard and touch paths have the same mission and authority
checks.

## 14. Interoperability and standards

All links below were verified from the official body on 2026-07-15.

- [CCSDS 502.0-B-3 Orbit Data Messages](https://ccsds.org/searchpubs/) is the orbit
  state exchange adapter. Loss caveat: local force models, operations constraints,
  trust, and mission authority need companion records.
- [CCSDS 508.0-B-1 Conjunction Data Message](https://ccsds.org/publications/bluebooks/)
  is the conjunction exchange adapter. Loss caveat: a message is input to human
  assessment, not proof of collision probability or a maneuver decision.
- [CCSDS 133.0-B-2 Space Packet Protocol](https://ccsds.org/publications/allpubs/entry/3264/)
  is a read-only packet decoding adapter. Loss caveat: mission-specific packet
  definitions, authentication, time correlation, and quality remain required.
- [CCSDS 727.0-B-5 CFDP](https://ccsds.org/publications/bluebooks/) can support
  declared file-transfer interfaces. Loss caveat: OrbitPrime does not use its file
  management services to control spacecraft storage and has no uplink path.
- The [UN COPUOS Space Debris Mitigation Guidelines](https://www.unoosa.org/documents/pdf/spacelaw/sd/COPUOS-GuidelinesE.pdf)
  inform mission safety and end-of-life cases. Loss caveat: they do not replace
  national licensing, current technical standards, or mission-specific analysis.

Each adapter records issue, schema, source, validation, unsupported fields, and
semantic loss. Standards never bypass the command firewall.

## 15. Cross-platform contract

Windows, macOS, and Linux pass identical time, frame, unit, orbit-message, telemetry,
planning, procedure, crash-recovery, and proof fixtures. Two current web engines pass
the six journeys and accessible non-visual alternatives. Offline mode supports mission
review, deterministic small-case propagation, schedule inspection, procedure rehearsal,
telemetry replay, and proof verification. GPU or cluster compute is optional; a
correct CPU reference remains. Managed compute receives only brokered data and can
never reach command transport. Install, upgrade, backup, restore, and uninstall tests
use simulated missions and disposable state. The CLI and container surfaces support
administration, import/export, replay, and fresh checks only; neither can reach
command transport or signing keys. Optional platform features use capability
detection and a safe fallback.

## 16. Reliability and performance budgets

- Acknowledged model, plan, decision, and anomaly events have RPO 0 under forced crash
  and concurrent-writer tests in every release.
- During each calendar month, 99.95% of authorized local mission metadata reads
  complete without server error; link and provider outages are separate states.
- A mission with 100 spacecraft and 30 days of prepared contacts opens to a useful
  plan in p95 under 3 seconds on the reference workstation.
- Validated live telemetry displays within p99 1 second of gateway receipt under the
  supported ground profile; data age is always shown.
- Live mode emits a link, telemetry, procedure, or explicit waiting signal every 2
  seconds; after 5 seconds it shows `signal delayed` and after the mission threshold
  it follows the approved loss-of-signal procedure.
- Local planning cancellation is acknowledged in 2 seconds and reaches a checkpoint
  within 30 seconds. Remote compute remains pending until confirmed.
- Metadata RTO is 15 minutes and mission object RTO is 1 hour in the supported
  self-hosted profile, tested before each operational release.
- Create, import, planning, and telemetry-ingest requests use idempotency keys retained
  for at least 24 hours; a duplicate returns the original durable result.
- Offline mode cannot receive current telemetry, schedule a live resource, or export
  to a command system. If an optional source or model fails, local planning, rehearsal,
  replay, recovery, and proof remain in a named degraded state.

## 17. Success measures

Measure frame/time/unit defects caught before plan use, schedules with proven margins,
procedure changes that trigger rereview, rehearsal defects found, telemetry gaps and
stale states shown, anomalies with complete evidence, conjunction decisions with
visible uncertainty, unauthorized command paths blocked, accessible journey
completion, cross-platform export and independent bundle replay, and
recovery/contingency drill success. Do not measure passes scheduled, commands
drafted, or telemetry volume as mission success by themselves. Business measures are
renewal after a verified planning or anomaly journey, support burden per mission, and
cost per independently replayable mission packet.

## 18. Delivery plan

- **G0 — Truthful foundation (0–6 months):** freshly prove shared service startup; replace generic records with
  stable mission/asset identity, mission ledger, command firewall, atomic writes,
  simulated fixtures, disposable-state tests, and three-platform CI.
- **G1 — Useful single-player product (6–18 months):** ship orbit/attitude kernel, contact/activity planner,
  procedure rehearsal, accessible mission UX, live progress, notifications, and
  lawful recovery with no command path.
- **G2 — Trusted team product (18–30 months):** add read-only telemetry, resource/constraint twin, anomaly
  graph, official message adapters, and independent calculation comparisons.
- **G3 — Category leader (30–42 months):** add conjunction support, multi-mission scheduling,
  portable proof, resilient strategy, bounded operations assistance, continuous
  safety cases, and independent mission assurance and human-factors review.
- **G4 — Frontier network (42–60 months):** ship HOP-F4-01 the multi-operator
  situational evidence exchange, HOP-F4-02 the ground-resource negotiation room,
  and HOP-F4-03 the cross-mission anomaly learning network only after simulated
  operational pilots, formal safety cases, command-firewall verification, and founder
  approval. Fresh G4 proof requires five independently governed simulated mission
  nodes, 100,000 frame/time/unit and revocation cases, 10,000 candidate allocations,
  1,000 anomaly replays, every local veto and withdrawal honored, zero restricted-data
  escape, and zero autonomous maneuver, allocation, schedule, procedure, strategy,
  anomaly closure, command, signing, uplink, or physical actuation.

Each gate runs fresh Rust and web builds, unit/integration/contract tests, official
and synthetic goldens, six end-to-end journeys, command-firewall, frame/time/unit,
telemetry-gap, accessibility, authorization, migration, recovery, security,
Windows/macOS/Linux packaging, and browser checks. No recorded rehearsal or prior
green report substitutes for a fresh run.

## 19. Current truth and gap

The live source is a generated scaffold with generic `assets` and `passes`
create/list/get records using title, body, status, and metadata. It has no mission
identity, spacecraft model, frame/time/unit semantics, orbit propagator, contact
planner, telemetry decoder, procedure engine, command firewall, safety case, UI, or
Orbit domain test. The assistant has only echo and product-catalog tools. The web
folder contains only `package.json`. The live backend now applies route state and
calls the shared graceful-shutdown server helper; the earlier startup defect is
repaired in source, but this spec-only pass did not run a fresh build. A generic
`passes` table is not a mission operations system. The first honest slice is
HOP-F0-01 through HOP-F0-03 plus fresh build proof, simulated mission fixtures, and
independent command-boundary review.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Internal truth | Versioned mission graph with explicit frames, times, units, and state kinds | Architecture and mission-assurance decision |
| Command path | Absent from OrbitPrime | Founder plus independent safety/certification approval |
| Signing keys | Never exported; not held by agents or OrbitPrime | Security and mission-authority decision |
| Agent role | Read, check, simulate, compare, and draft only | Formal safety review |
| Live data | Read-only and untrusted until validated | Mission assurance decision |
| Conjunction | Decision support only; no auto-maneuver | Flight-dynamics and mission-authority decision |
| First test data | Simulated missions only | Pilot governance decision |
| Delete | 30-day recovery where lawful; mission holds override | Retention decision |
| Proof | Aether preferred, local signed bundle required | Architecture decision |

## 21. Definition of category-defining done

- [ ] Planned, simulated, estimated, observed, commanded, and confirmed state are
  never confused.
- [ ] Every orbit and attitude value carries frame, epoch, time system, source,
  method, and uncertainty.
- [ ] No agent, client, model worker, or OrbitPrime service can reach command transport
  or private command keys.
- [ ] Schedules expose conflicts and margins; procedures bind configuration and
  rehearsal; changes invalidate proof.
- [ ] Signal loss, stale data, failed models, and unresolved anomalies are visible in
  real time.
- [ ] Independent proof bundles validate and replay allowed work on every platform.
- [ ] Independent flight dynamics, mission assurance, human-factors, security, and
  legal reviewers accept the safety case.
- [ ] Windows, macOS, Linux, web, offline, accessibility, recovery, and packaging
  gates pass from fresh source.
- [ ] The product stays valuable even though command and final mission authority
  remain outside it.
