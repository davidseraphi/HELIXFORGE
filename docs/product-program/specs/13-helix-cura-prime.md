# HelixCuraPrime — human-led clinical operations and care intelligence

```yaml
product: HelixCuraPrime
catalog_order: 13
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [clinicians, care coordinators, clinical operations teams, patients and authorized carers]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixCuraPrime is a human-led care operations system that turns fragmented clinical
information into an inspectable patient timeline, care plan, team workflow, and
evidence trail while keeping diagnosis, treatment, orders, consent, and clinical
accountability with qualified humans.

## 2. Five-year destination

The useful product is a consent-aware longitudinal record viewer, care-plan room,
task and handoff system, results inbox, and patient communication workspace. The
leading product reconciles information across authorized systems, detects missing or
conflicting facts, and proves who saw and acted on them. The frontier product can
simulate care pathways, forecast operational risk, and draft patient-specific options
with uncertainty and source evidence. It never diagnoses, prescribes, places an
order, changes a clinical record, contacts a patient, or acts on an emergency without
the exact human authority and regulated integration required for that setting.

## 3. Users and hard jobs

- A clinician needs the right facts at the right time and fears a confident summary
  built from stale or wrong-patient data.
- A care coordinator needs closed-loop referrals, results, tasks, and handoffs across
  organizations.
- A clinical operations lead needs reliable flow, capacity, safety signals, and proof
  without exposing unnecessary patient data.
- A patient or authorized carer needs understandable plans, choices, consent, access,
  and correction paths.
- A safety reviewer needs complete alert, override, delay, and outcome evidence.
- A health organization needs local law, clinical governance, device regulation,
  privacy, retention, and downtime controls.

## 4. Product laws

1. The patient, encounter, author, source, time, status, and clinical context of every
   record are explicit.
2. Source data, reconciled facts, model estimates, recommendations, and human decisions
   are different types of truth.
3. No agent may diagnose, prescribe, order, administer, consent, discharge, message,
   or close a clinical task.
4. A safety alert shows evidence, limits, owner, and response; no silent alert
   suppression is allowed.
5. Break-glass access is time-bound, reason-bound, visible, and reviewed.
6. Patient consent and legal authority travel with data and downstream actions.
7. Missing, delayed, conflicting, or out-of-range data is visible and cannot be
   summarized away.
8. The system remains useful in downtime and never claims that software alone makes
   care safe or compliant.
9. Critical journeys are accessible, portable, and independently verifiable across
   platforms; no model, record vendor, or proof provider is a hard dependency.

## 5. Scope boundaries

CuraPrime owns authorized clinical data reconciliation, timelines, care-plan drafts,
task routing, handoffs, results follow-up, patient communication drafts, operational
intelligence, and proof. HelixCore owns shared identity, policy, audit, capabilities,
jobs, objects, billing, and operations; Aether is reached through a provider-neutral
proof interface with a local fallback. It is not an autonomous clinician, electronic prescribing
system, medical device controller, emergency dispatch service, or legal medical
record unless separately validated, certified, and contracted for that role. Final
diagnosis, treatment, order entry, consent, patient contact, discharge, and emergency
action stay with qualified people and approved clinical systems.

## 6. Signature experiences

1. **Build a trustworthy patient timeline.** Entry: an authorized clinician opens a
   patient context from an approved system. Progress: identity match, source fetch,
   terminology mapping, deduplication, and conflict checks stream visibly. Human
   decision: the clinician accepts or corrects reconciled facts. Proof: source,
   version, author, status, and mapping are linked. Failure and recovery: identity
   doubt blocks merging and opens reconciliation. Export: a consent-filtered timeline
   bundle.
2. **Prepare for a care encounter.** Entry: a clinician selects the visit purpose.
   Progress: active problems, medicines, allergies, recent results, open referrals,
   and missing data appear with source age. Human decision: the clinician selects
   what matters and approves the note plan. Proof: viewed data and accepted summary
   are recorded. Failure and recovery: unavailable sources remain named gaps.
   Export: a draft brief, never a signed note.
3. **Close the loop on a result.** Entry: an approved interface receives a result.
   Progress: patient match, final/preliminary state, abnormal flag, responsible team,
   acknowledgement, action, and completion are visible. Human decision: a clinician
   interprets and chooses action. Proof: every handoff and delay is timestamped.
   Failure and recovery: unowned or unacknowledged results escalate; retries are
   idempotent. Export: a follow-up proof card.
4. **Coordinate a care plan.** Entry: a clinician creates or imports an approved goal.
   Progress: interventions, tasks, dependencies, consent, barriers, owners, and dates
   form one plan. Human decision: clinicians and patient approve relevant parts.
   Proof: plan versions and participant decisions are signed. Failure and recovery:
   material change returns affected parts to review. Export: a role-filtered plan.
5. **Draft a patient communication.** Entry: a clinician selects a verified result or
   plan event. Progress: plain-language draft, language, accessibility, recipient,
   privacy, and delivery channel are checked. Human decision: authorized staff edit
   and send through the approved system. Proof: approved content hash and delivery
   outcome are recorded. Failure and recovery: failed delivery becomes an owned task,
   not a silent success. Export: communication proof without hidden clinical detail.
6. **Review an operational safety signal.** Entry: a team opens a queue or pathway.
   Progress: data completeness, denominators, delays, confounders, and uncertainty are
   shown. Human decision: a clinical governance lead chooses an intervention. Proof:
   metric definitions, data versions, exclusions, and decisions are kept. Failure and
   recovery: low-quality data blocks causal claims. Export: a de-identified review
   bundle where lawful.

## 7. Capability map

F0 is foundation, F1 is the useful product, F2 is the trusted-team product, F3 is
advanced category leadership, and F4 is the frontier network. Every row inherits this full contract: its invariants are the
product laws and typed clinical-truth boundaries; authority is the exact named human
and policy in Sections 10–11; evidence is input/output hashes, versions, actor,
consent, decision, and ledger event; failure is a durable `blocked`, `failed`,
`unknown`, or reconciliation state with retry or recovery; and test acceptance
includes denial, failure, recovery, wrong-patient, and cross-platform cases in
addition to the row's named check. The row names its domain-specific inputs, output,
and strongest acceptance test.

| ID | First gate | Capability contract |
|---|---|---|
| HCP-F0-01 | G0 | **Patient and encounter identity.** Inputs are authorized identifiers and source context; output is a confidence-scored match or a reconciliation case. No uncertain match merges records. Acceptance: wrong-patient, duplicate, and cross-tenant fixtures are blocked. |
| HCP-F0-02 | G0 | **Clinical provenance ledger.** Inputs are ingest, view, reconcile, alert, handoff, decision, and export events; output is an append-only history. Acceptance: concurrent writes and forced crashes lose no acknowledged event. |
| HCP-F0-03 | G0 | **Consent and authority policy.** Inputs are patient choices, legal basis, role, purpose, break-glass reason, and local policy; output is an exact permit or denial. Acceptance: denied data never reaches UI, agent, notification, or proof payload. |
| HCP-F1-01 | G1 | **Longitudinal timeline.** Inputs are source records and mappings; output is a time-ordered, source-preserving view with conflicts and gaps. Acceptance: source status and later correction never vanish through deduplication. |
| HCP-F1-02 | G1 | **Clinical reconciliation room.** Inputs are conflicting problems, allergies, medicines, results, or demographics; output is a human decision with source links. Acceptance: agents may suggest but cannot accept a reconciliation. |
| HCP-F1-03 | G1 | **Care-plan and task graph.** Inputs are clinician-approved goals, actions, owners, dependencies, consent, and dates; output is a versioned plan. Acceptance: no task can close without the named evidence or human confirmation required by policy. |
| HCP-F2-01 | G2 | **Results closed loop.** Inputs are authorized result events; output tracks match, status, owner, acknowledgement, action, and completion. Acceptance: missing owner, failed delivery, or overdue action escalates visibly. |
| HCP-F2-02 | G2 | **Handoff contract.** Inputs are patient context, reason, sender, receiver, urgency, and minimum data; output is a durable acceptance or rejection chain. Acceptance: sending is not completion; the receiver must acknowledge. |
| HCP-F2-03 | G2 | **Patient communication workspace.** Inputs are human-selected facts and an approved channel; output is an accessible draft and delivery intent. Acceptance: human approval binds exact content, recipient, and channel before send. |
| HCP-F3-01 | G3 | **Source-grounded decision support.** Inputs are approved patient facts, clinical rules, evidence versions, and local policy; output is a labeled suggestion with limits. Acceptance: it cannot place an order and every suggestion is independently dismissible. |
| HCP-F3-02 | G3 | **Operational pathway twin.** Inputs are de-identified or authorized flow events and capacity; output is a scenario, not a patient instruction. Acceptance: denominators, missingness, uncertainty, and policy assumptions are visible. |
| HCP-F3-03 | G3 | **Clinical proof bundle.** Inputs are consent-filtered records, mappings, workflows, decisions, and attestations; output is a verifiable bundle. Acceptance: independent validation proves integrity and the redaction manifest. |
| HCP-F3-04 | G3 | **Counterfactual care-path studio.** Inputs are clinician-approved scenarios and validated models; output compares possible pathways without selecting treatment. Acceptance: no output crosses into orders, messages, or the legal record automatically. |
| HCP-F3-05 | G3 | **Privacy-preserving learning network.** Inputs are approved local summaries or model updates; output is aggregate learning with site and consent controls. Acceptance: raw patient data stays in custody and revocation stops future contribution. |
| HCP-F3-06 | G3 | **Continuous clinical safety case.** Inputs are incidents, overrides, alert performance, model drift, tests, and controls; output is a living argument with open hazards. Acceptance: only an independent clinical governance body may accept it. |
| HCP-F4-01 | G4 | **Patient-governed continuity and portability network.** **Input:** consent-filtered minimum records, stable patient and encounter identity, source status, recipient, purpose, and expiry from participating institutions. **Output:** a verifiable transfer candidate, reconciliation state, and sender/receiver acknowledgement chain. **Invariant:** an uncertain identity never merges records; only the approved recipient receives content; the network cannot diagnose, select treatment, place an order, or alter a legal clinical record automatically. **Authority:** the patient or lawful representative controls consent, the sending clinician approves content, and the receiving clinician accepts it. **Evidence:** identity checks, consent version, source hashes, redaction, delivery, receipt, reconciliation, use, and revocation events. **Failure:** identity conflict, consent loss, recipient mismatch, stale source, or failed delivery blocks use and opens reconciliation. **Acceptance:** five independent non-production nodes pass 10,000 wrong-patient, consent, recipient, expiry, and revocation cases with zero false merge or unauthorized disclosure. |
| HCP-F4-02 | G4 | **Cross-site care handoff mesh.** **Input:** a clinician-approved handoff, minimum patient context, reason, urgency, open tasks, source versions, recipient, and consent. **Output:** a durable accepted, rejected, expired, or clarification-needed handoff with explicit ownership. **Invariant:** sending is never completion; no task, message, medicine, order, diagnosis, or treatment action is created without the responsible human's separate clinical action. **Authority:** the sending clinician approves exact content and recipient, the receiving clinician accepts responsibility, and patient authority remains binding. **Evidence:** content hash, sender approval, transport result, receiver decision, questions, ownership, expiry, and closure evidence. **Failure:** unavailable receiver, stale facts, consent change, or missing acknowledgement keeps ownership with the sender and escalates visibly. **Acceptance:** 1,000 synthetic handoffs across five nodes preserve ownership, expose every seeded gap, and create zero autonomous clinical action. |
| HCP-F4-03 | G4 | **Federated clinical evaluation commons.** **Input:** site-approved aggregate measures or model updates, evaluation protocol, intended use, subgroup rules, consent policy, and local safety limits. **Output:** comparable performance, calibration, failure, subgroup, and drift evidence with site-local uncertainty. **Invariant:** raw patient data stays in local custody; results cannot become a diagnosis, care recommendation, order, deployment approval, or patient-level inference. **Authority:** each site's clinical governance and privacy bodies approve participation and disclosure, and an independent human panel accepts any shared conclusion. **Evidence:** protocol and model hashes, cohort definition, privacy check, site result, exclusions, review, withdrawal, and revocation. **Failure:** small cohort, privacy risk, protocol drift, missing denominator, or site withdrawal invalidates the affected aggregate and future use. **Acceptance:** 100 site-controlled evaluations reproduce the declared protocol and 100,000 privacy, subgroup, and revocation cases produce zero raw-data movement or patient-level action. |

## 8. Domain model

| Record | Ownership, lifecycle, and relationships |
|---|---|
| `PatientIdentity` / `IdentityLink` | Source-specific identifiers, match evidence, confidence, authority, and reconciliation history; never a guess hidden as one patient. |
| `Encounter` / `CareEpisode` | Setting, organization, participants, reason, status, time, and links to source events. |
| `ClinicalRecordVersion` | Source system, resource identity, author, status, effective time, received time, content hash, correction, and supersession. |
| `Problem` / `Allergy` / `MedicationStatement` / `Observation` | Reconciled clinical fact with code system, source versions, assertion state, uncertainty, and human acceptance. |
| `CarePlanVersion` / `Goal` / `CareTask` | Human-approved purpose, owner, dependencies, consent, due state, completion evidence, and change history. |
| `ResultCase` / `Handoff` / `Escalation` | Match, clinical status, sender, receiver, acknowledgement, action, deadline, and closure proof. |
| `ConsentDirective` / `AuthorityGrant` / `BreakGlassEvent` | Purpose, scope, actor, basis, start, expiry, revocation, reason, and review. |
| `Recommendation` / `ClinicalDecision` | Input snapshot, rule/model version, suggestion, limits, clinician response, and outcome; they are never the same record. |
| `CommunicationDraft` / `DeliveryIntent` / `DeliveryReceipt` | Exact content version, recipient, channel, approval, send result, and follow-up state. |
| `RecoveryItem` / `RetentionRule` / `LegalHold` | Restore window, clinical-record constraints, purge authority, and signed lifecycle. |

## 9. System architecture

- A Rust clinical domain kernel enforces identity, consent, provenance, workflow
  state, handoff rules, and authority before a durable write.
- PostgreSQL stores metadata and row-level organization/patient access; encrypted
  object storage holds large documents and images by content hash.
- An interoperability gateway validates profiles, terminology versions, source
  identity, and idempotency before quarantine or acceptance.
- Terminology services are versioned and local-cache capable. A mapping never changes
  source content and always reports meaning loss.
- Decision-support workers are isolated from order, prescribing, device, and message
  execution. Their output enters a review queue.
- A user-owned broker grants a narrowly approved process exact source capabilities.
  Aether is the preferred proof layer; offline signed verification remains available.
- HelixCore supplies shared identity, policy, audit, capabilities, jobs, objects,
  billing, and operations behind domain interfaces; CuraPrime retains clinical truth.
- The event flow is request or ingest, identity and authority check, clinical/domain
  validation, atomic record plus event, projection, alert/notification, and proof.
  Background work uses durable HelixCore jobs, idempotent checkpoints, visible
  progress, escalation, and explicit cancellation.
- Offline review, care-plan drafts, recovery, and verification use a local encrypted
  downtime store. Versioned FHIR, terminology, source, model, and proof adapters are
  contract-tested extension points; none may add orders, messages, or device action.
- Downtime mode uses a read-only last-known-safe snapshot plus local queued notes that
  require reconciliation before joining the record.

## 10. Agent and automation contract

| Role | May do | Must not do |
|---|---|---|
| Timeline assistant | Organize authorized source records and flag conflicts | Merge uncertain patients or claim a reconciled fact |
| Care-plan assistant | Draft goals, tasks, barriers, and follow-up questions | Diagnose, prescribe, order, discharge, or close care |
| Results clerk | Route, remind, and escalate under approved policy | Interpret a result or mark clinical action complete |
| Communication assistant | Draft accessible language from selected facts | Select recipients, reveal hidden data, or send |
| Operations analyst | Analyze approved aggregate flow and scenarios | Direct patient care or infer causality without evidence |

Every agent lease names patient or cohort, purpose, role, data classes, tools, time,
and output destination. Agents cannot see raw credentials. Progress shows sources,
stage, elapsed time, last signal, uncertainty, and waiting clinician. Pause, cancel,
revocation, and override are signed events. Agents never self-approve. Every draft
enters named clinician review; identity, terminology, schema, policy, and evidence
checks validate the result. Reversal restores a prior version or revokes access
without rewriting the clinical history.

## 11. Trust, safety, and privacy

| Safety case | Prevention, human authority, proof, and recovery |
|---|---|
| Wrong patient | Strong source context, confidence thresholds, demographic conflict checks, and reconciliation quarantine. Only authorized staff can resolve. |
| Unsafe clinical recommendation | Validated scope, source evidence, uncertainty, independent display, no execution tools, and clinician review. A recommendation is never an order. |
| Missed result or handoff | Durable ownership, acknowledgement, timers, escalation, downtime queues, and closure evidence. Sending does not equal received. |
| Privacy or consent breach | Least privilege, purpose binding, consent policy, field filtering, local custody, redacted notices, break-glass review, and metadata-safe proof. |
| Silent model or terminology drift | Version pinning, shadow evaluation, drift alarms, rollback, and clinical governance approval before promotion. |
| Unsafe deletion | User drafts enter a 30-day bin where lawful. The legal medical record, active care, safety incident, patient request, regulation, or hold may require longer retention and block purge. Access can be quarantined immediately without destroying evidence. Permanent deletion requires named data authority, impact preview, re-authentication, and signed proof. |

Clinical hazard analysis, privacy impact assessment, human factors tests, model
evaluation, cybersecurity review, downtime drills, and local regulatory classification
are release gates. Clinical use starts only inside a named intended-use claim. Tenant
and patient separation are enforced in the database and object layer. Data is
encrypted in transit and at rest, residency follows deployment policy, and incident
recovery can quarantine access, revoke leases, preserve evidence, notify accountable
humans, and restore a reviewed state.

## 12. Proof and audit

Proof records source identity, patient/encounter context, consent and authority,
terminology and profile versions, reconciliation, views where policy requires,
recommendations, human decisions, handoffs, alerts, overrides, communication intents,
and known gaps. Audit metadata avoids clinical text unless needed and authorized.
Every export has a consent and redaction manifest. Aether is the preferred signed
proof provider; a local verifier and signed bundle are mandatory fallbacks. Proof
does not show that care was clinically correct, that source data was accurate, or
that an outcome was caused by an intervention.

## 13. UX system

The primary surfaces are Home, Patients, Timeline, Care Plans, Results, Handoffs,
Messages, Operations, Evidence, and Recovery. The patient identity banner, encounter,
consent state, data age, and source state remain visible. The interface starts with
the immediate job and reveals source detail, terminology, provenance, and raw data
progressively. High-risk actions use clear verbs and an impact preview. Long work
shows sources checked, current stage, elapsed time, last signal, and what needs a
clinician. Results, failed delivery, overdue handoff, source outage, and required
approval create private notifications without clinical detail on a lock screen.
Keyboard, screen-reader, contrast, zoom, reduced-motion, and low-bandwidth paths are
release requirements. Reversible edits offer undo; empty states explain the first
safe action; plain-language errors state what happened, what remains safe, and how to
recover. Keyboard and touch paths have the same patient and authority checks.

## 14. Interoperability and standards

All links below were verified from the official body on 2026-07-15.

- [HL7 FHIR R5](https://hl7.org/fhir/) is a clinical data exchange adapter. Each
  integration declares profiles and capability statements. Loss caveat: base FHIR
  does not settle local workflow, consent, terminology, or clinical fitness.
- The [current DICOM edition](https://www.dicomstandard.org/current/) is the imaging
  and related-information adapter. Loss caveat: image interpretation, acquisition
  quality, and clinical display need separately validated systems.
- [SNOMED CT specifications](https://docs.snomed.org/snomed-ct-specifications) support
  clinical meanings with edition and version identity. Loss caveat: licensing,
  national extensions, local subsets, and correct clinical use remain deployment
  responsibilities.
- [LOINC](https://loinc.org/about/) identifies observations, measurements, and
  documents. Loss caveat: a code does not carry the result value, unit, method,
  reference range, or interpretation by itself.
- [WHO ICD-11](https://www.who.int/news-room/fact-sheets/detail/icd-11) supports
  reporting and classification adapters. Loss caveat: it is not used as a substitute
  for detailed clinical terminology or autonomous diagnosis.

Every adapter records version, profile, terminology release, mapping provenance,
validation result, and unsupported meaning. No silent coercion is allowed.

## 15. Cross-platform contract

Windows, macOS, and Linux pass identical identity, consent, FHIR fixture, terminology,
handoff, closed-loop result, encryption, recovery, and export tests. Two current web
engines pass the six journeys. The desktop client supports encrypted local cache and
downtime review. The web client cannot expose more data than the same policy allows
on desktop. Mobile-sized browser layouts support urgent review but do not add hidden
clinical actions. Install, upgrade, migration, backup, restore, and uninstall use
synthetic data and disposable state. Regional managed deployments prove residency
and data-flow policy before use. The CLI and container surfaces support administration,
validated import/export, and fresh checks only; they expose no hidden diagnosis,
order, message, or device route. Optional platform features use capability detection
and a safe fallback.

## 16. Reliability and performance budgets

- Acknowledged identity, consent, result, handoff, and decision events have RPO 0
  under crash and concurrent-writer tests for every release.
- In each calendar month, 99.95% of authorized local clinical metadata reads complete
  without server error; upstream-source outages are shown separately.
- A patient summary with 10 years and 100,000 source events becomes useful in p95
  under 2 seconds on the reference clinical workstation.
- A final result accepted from an interface enters an owned or reconciliation state
  within 10 seconds in p99 under the supported profile.
- A safety workflow emits a meaningful progress or waiting signal at least every 5
  seconds; after 15 seconds without signal it changes to `signal delayed`.
- Local cancellation is acknowledged within 2 seconds. A connector remains pending
  until the external system confirms stop or completion.
- Downtime read snapshot RTO is 15 minutes; metadata recovery RTO is 30 minutes;
  object recovery RTO is 4 hours, tested quarterly.
- Create, ingest, handoff, and delivery-intent requests use idempotency keys retained
  for at least 24 hours; a duplicate returns the original durable result.
- Offline mode cannot fetch current external records or send a message. Unsynced work
  stays visible. If an optional source or model fails, local review, care planning,
  recovery, and export remain in a named degraded state.

## 17. Success measures

Measure wrong-patient events prevented, records with complete source provenance,
result loops closed within policy, accepted handoffs, reconciliation defects caught,
clinician time saved without higher override or harm signals, patient understanding,
consent denials enforced, agent actions rejected outside scope, accessible journey
completion, cross-platform export and independent bundle validation, and
recovery/downtime drill success. Do not use recommendation count,
alerts fired, or screen time as a quality measure. Business measures are renewal
after a verified closed-loop journey, support burden per active care team, and cost
per safely completed and independently auditable workflow.

## 18. Delivery plan

- **G0 — Truthful foundation (0–6 months):** freshly prove service startup; replace generic records with
  patient/encounter identity, consent authority, and an atomic clinical ledger; add
  synthetic fixtures, disposable-state tests, and three-platform CI.
- **G1 — Useful single-player product (6–18 months):** ship timeline, reconciliation, care plan, clinician review,
  downtime, accessibility, private notifications, and lawful recovery.
- **G2 — Trusted team product (18–30 months):** add closed-loop results, handoffs, patient communication,
  validated FHIR/terminology adapters, and proof bundles in non-production pilots.
- **G3 — Category leader (30–42 months):** add bounded decision support, pathway twin,
  counterfactual care paths, privacy-preserving learning, continuous safety cases,
  and external clinical safety, privacy, and human-factors review under a named intended use.
- **G4 — Frontier network (42–60 months):** ship HCP-F4-01 patient-governed portability,
  HCP-F4-02 the cross-site handoff mesh, and HCP-F4-03 the federated evaluation
  commons only after prospective evaluation, regulatory classification, and founder
  approval. Fresh G4 proof is limited to approved non-production pilots and requires
  five independently governed nodes, 10,000 wrong-patient and consent cases, 1,000
  synthetic handoffs, 100 site-controlled evaluations, zero raw-patient-data escape,
  and zero autonomous diagnosis, treatment, order, message, deployment, or record change.

Each gate runs fresh Rust and web builds, unit/integration/contract tests, synthetic
clinical fixtures, the six journeys, wrong-patient, consent, terminology, downtime,
accessibility, recovery, migration, redaction, security, Windows/macOS/Linux
packaging, and browser checks. A model card or stored report cannot replace the run.

## 19. Current truth and gap

The live source is a generated scaffold with generic `care_cases` and `notes`
create/list/get endpoints and generic title, body, status, and metadata fields. It
has no patient identity, encounter, consent, clinical terminology, FHIR profile,
care plan, closed-loop result, decision support, safety case, or clinical-domain
test. The assistant has only echo and product-catalog tools. The web folder contains
only `package.json`. The live backend now applies route state and calls the shared
graceful-shutdown server helper; the earlier startup defect is repaired in source,
but this spec-only pass did not run a fresh build. Nothing in the scaffold is a
clinical system or medical device. The first honest slice is HCP-F0-01 through
HCP-F0-03 plus fresh build proof, synthetic fixtures, and independent clinical
architecture review.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Intended use | Care operations and decision support, not autonomous care | Founder, clinical, legal, and regulatory decision |
| Clinical authority | Named qualified human | Clinical governance decision |
| Agent action | Organize, draft, route, and flag only | Safety and regulatory approval |
| Orders and messages | No autonomous place, sign, send, or close | Clinical safety approval |
| Identity doubt | Reconciliation case, never automatic merge | Patient-safety review |
| Data custody | Purpose-bound, least privilege, local/self-hostable | Privacy and security review |
| Interop | Versioned FHIR/terminology profiles, not generic JSON | Architecture and clinical review |
| Delete | 30-day recovery where lawful; clinical retention overrides | Legal and records decision |
| Proof | Aether preferred, offline signed bundle required | Architecture decision |

## 21. Definition of category-defining done

- [ ] The system never merges uncertain patients or hides source conflict.
- [ ] Source facts, reconciled facts, estimates, suggestions, and human decisions stay
  visibly separate.
- [ ] No agent can diagnose, prescribe, order, message, consent, discharge, or close
  a clinical action.
- [ ] Every result and handoff has a named owner, acknowledgement, action, and closure
  proof or visible escalation.
- [ ] Consent, break-glass, redaction, retention, and patient correction work across
  every client and export.
- [ ] Clinicians can use the product safely during source outages and downtime.
- [ ] Independent clinical, privacy, security, human-factors, and regulatory reviewers
  accept the intended-use safety case.
- [ ] Windows, macOS, Linux, web, offline, accessibility, recovery, and packaging
  gates pass from fresh source with synthetic data.
- [ ] The product states clearly what its evidence does and does not prove.
