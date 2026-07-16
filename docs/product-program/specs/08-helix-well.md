# HelixWell — private, user-owned wellbeing guidance

```yaml
product: HelixWell
catalog_order: 8
status: target-state-spec
horizon: 60 months
current_maturity: prototype
primary_users: [individuals, families with consent, wellbeing coaches, care professionals, researchers with consent]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

> **Target-state rule:** Sections 1–18 and 20–21 describe planned capability.
> Section 19 describes only what the live implementation proves now.

## 1. Category claim

HelixWell is a private wellbeing workspace where a person can understand daily
patterns, choose small actions, and share exact evidence with trusted people
without giving an agent or platform authority over their health.

## 2. Five-year destination

The useful product is a calm, local-first home for goals, habits, check-ins,
journals, routines, device observations, reflections, and user-chosen coaching.
The category-defining advantage is user-owned context: each suggestion explains
which observations, goals, limits, and uncertainty shaped it, and the user
controls what is remembered or shared. The frontier capability is a consented
care circle where a person can combine personal, device, and clinical data,
test gentle routines, and share selective proof across providers without one
vendor controlling custody. A person and their accountable care professionals
retain authority over goals, diagnoses, treatments, medication, emergencies,
sharing, and any action that affects health or rights.

## 3. Users and hard jobs

- **Individuals** need to notice patterns and act without shame. They fear
  surveillance, wrong advice, or private data used against them.
- **Families and trusted supporters** need a consented way to help. They fear
  access that becomes control or continues after consent ends.
- **Wellbeing coaches** need clear goals and progress within their scope. They
  fear becoming responsible for medical conclusions the product invented.
- **Care professionals** need a useful, limited summary rather than a data dump.
  They fear missing provenance, units, context, or a change made by an agent.
- **Researchers** need consented, privacy-protected patterns. They fear
  re-identification and data collected for one purpose being reused for another.

## 4. Product laws

1. The individual is the primary owner and viewer of personal wellbeing data.
2. The product supports reflection and coaching; it does not diagnose or treat.
3. A model suggestion is labelled, explained, uncertain, and never a clinical fact.
4. Sharing is purpose-bound, field-level, time-limited, visible, and revocable.
5. Agents cannot contact people, change care, or trigger an emergency alone.
6. Missing days and changed routines are not failure or moral judgement.
7. Device and clinical imports preserve source, unit, time, and known limits.
8. Core check-in, reflection, export, and deletion work without a cloud service.
9. Long import, analysis, and export work shows real progress and can be stopped.
10. Safety guidance clearly states limits and routes urgent needs to human help.

## 5. Scope boundaries

HelixWell owns personal goals, routines, habits, check-ins, observations,
journals, reflections, consented coaching, user-visible patterns, care circles,
and selective exports. HelixCore owns identity, policy, audit, capabilities,
jobs, objects, billing, and operations. HelixInsights owns general-purpose data
analysis. External clinical systems and devices connect through adapters.

HelixWell is not an electronic health record, medical device by default,
diagnostic system, emergency service, therapist, doctor, insurer, or employer
assessment tool. It must not give clinical advice, change medication, decide
care, or replace a licensed professional. Any future regulated function needs a
separate product, legal, clinical-safety, and founder gate.

## 6. Signature experiences

1. **Start privately.** **Entry:** a person chooses local or managed setup and a
   first goal. **Visible progress:** storage, encryption, recovery, sharing, and
   optional model choices are explained. **Human decision:** the person chooses
   what to track and what remains only on the device. **Completion proof:** a
   privacy receipt records settings without exposing values. **Failure and
   recovery:** setup can pause and resume without uploading data. **Export:** a
   portable encrypted home can be created at any time.
2. **Check in without pressure.** **Entry:** the person opens Today or a reminder.
   **Visible progress:** saved fields and local/sync state are clear. **Human
   decision:** every field can be answered, skipped, or removed. **Completion
   proof:** the entry shows time, source, and edits. **Failure and recovery:** an
   interrupted check-in preserves a draft and never invents zero values.
   **Export:** selected check-ins can leave as a readable table or data bundle.
3. **Build a gentle routine.** **Entry:** the person chooses a goal and one small
   action. **Visible progress:** plan, reminders, attempts, pauses, and changes
   are shown without streak pressure. **Human decision:** the person accepts,
   edits, pauses, or ends the routine. **Completion proof:** reflections, not only
   completion, show whether it helped. **Failure and recovery:** missed days do
   not punish or reset identity. **Export:** plan and reflections are portable.
4. **Understand a pattern.** **Entry:** the person asks about sleep, mood,
   activity, energy, or another tracked area. **Visible progress:** data range,
   missing values, calculations, checks, and uncertainty appear. **Human
   decision:** the person chooses whether to save or share an insight.
   **Completion proof:** the result links to exact observations and method.
   **Failure and recovery:** weak or conflicting data returns inconclusive.
   **Export:** data, analysis, visual, and limits form one bundle.
5. **Use a bounded wellbeing coach.** **Entry:** the person asks for reflection or
   planning help. **Visible progress:** approved sources, plan, tool use, and
   safety checks show. **Human decision:** the person chooses any action and may
   stop or delete the session. **Completion proof:** suggestions link to the
   person's stated goal and evidence. **Failure and recovery:** clinical,
   dangerous, or uncertain requests route to human support. **Export:** the
   person controls whether the session enters memory or an export.
6. **Share with a trusted person.** **Entry:** the owner selects a care-circle
   member and purpose. **Visible progress:** exact fields, history range, expiry,
   and future updates are previewed. **Human decision:** the owner approves and
   can revoke at any time. **Completion proof:** grant, views, exports, and
   revocation are signed metadata events. **Failure and recovery:** revoked or
   expired access stops future use without deleting shared source data.
   **Export:** the recipient gets only the approved packet.
7. **Bring in device or clinical data.** **Entry:** the person connects an
   adapter or imports a file. **Visible progress:** records, units, duplicates,
   timezones, mappings, and losses show. **Human decision:** the person approves
   each data class and destination. **Completion proof:** source and mapping are
   linked to observations. **Failure and recovery:** invalid values are
   quarantined and import can resume. **Export:** the original and normalized
   records remain available.
8. **Get urgent human help.** **Entry:** a user chooses Help Now or the system
   notices a configured safety phrase. **Visible progress:** it clearly says the
   product is not an emergency service and shows user-configured local options.
   **Human decision:** the person chooses whether and whom to contact. **Completion
   proof:** only the person's chosen action is recorded under privacy settings.
   **Failure and recovery:** if a service is unavailable, other human and local
   options remain visible. **Export:** no crisis label is shared without the
   person's action except where a lawful, clearly configured duty applies.

## 7. Capability map

### F0 — foundation

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| WELL-F0-001 | G0 | Person, local home, recovery choice | Stable private identity | Identity is path and provider independent; default audience is self | Person owns | Setup and recovery receipt | `locked` | WHEN the folder or device changes, the restored home SHALL keep identity and revoke the old device binding on request. |
| WELL-F0-002 | G0 | Data class, purpose, recipient, time | Consent grant | Deny by default; exact fields and expiry are required | Person grants and revokes | Grant, use, expiry, revocation | `denied` or `expired` | WHEN a grant expires, every later read SHALL fail and create a metadata-only denial event. |
| WELL-F0-003 | G0 | Observation or journal command | Record plus event | Value, unit, source, time, and event commit together or neither commits | Person or approved source | Transaction and replay proof | `not_committed` | WHEN a crash occurs during save, recovery SHALL show zero or one complete entry, never a value without source. |
| WELL-F0-004 | G0 | Long or offline job | Durable job and sync timeline | Heartbeats and completion reflect real work; conflicts never overwrite silently | Person may stop | Stages, heartbeat, conflict record | `delayed`, `conflict`, `cancelled` | WHILE an import runs, the UI SHALL show last activity and a working cancel action. |

### F1 — useful product

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| WELL-F1-001 | G1 | Person-chosen prompts and values | Versioned check-in | Every field is optional; missing is not zero | Person records | Time, source, edits | `draft` | WHEN a field is skipped, analysis SHALL treat it as missing and not as the lowest value. |
| WELL-F1-002 | G1 | Goal, action, schedule, reminders | Routine plan | No punitive streak or hidden behaviour scoring | Person accepts and changes | Plan versions and reflections | `paused` or `ended` | WHEN a day is missed, the system SHALL preserve history without punishment language or automatic reset. |
| WELL-F1-003 | G1 | Notes, attachments, sensitivity | Private journal entry | Search and models follow the same sensitivity policy | Person owns | Version, access, export log | `locked` | WHEN an entry is marked private-local, no managed worker SHALL receive its value. |
| WELL-F1-004 | G1 | Approved observations and method | Pattern report | Correlation is not causation; missingness and uncertainty show | Person runs and decides to save/share | Inputs, method, checks | `inconclusive` | WHEN evidence is weak, the system SHALL say inconclusive and SHALL NOT recommend a health action. |

### F2 — category leader

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| WELL-F2-001 | G2 | Goal, approved personal context, safe sources | Coaching draft | No diagnosis, treatment, medication, or hidden sharing | Person chooses every action | Sources, plan, safety checks | `needs_human` or `stopped` | WHEN a request is clinical or dangerous, the agent SHALL stop advice and show human help options. |
| WELL-F2-002 | G2 | Recipient, purpose, fields, duration | Care-circle view | Recipient sees only approved data and cannot extend access | Person grants; recipient accepts duty | Access and revocation events | `revoked` | WHEN one member is removed, shared secrets and other members' grants SHALL remain intact. |
| WELL-F2-003 | G2 | Device or clinical records | Normalized observations plus originals | Units, source, timezone, duplicates, and loss are explicit | Person approves classes | Mapping, validation, hashes | `quarantined` | WHEN a unit cannot map safely, import SHALL quarantine the value rather than guess. |
| WELL-F2-004 | G3 | User-selected record range | Professional summary | Summary labels user reports, device facts, and clinical facts separately | Person approves export | Selection manifest and signature | `incomplete` | WHEN source provenance is missing, the summary SHALL name the gap before export. |

### F3 — advanced category leadership

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| WELL-F3-001 | G3 | Goal, routine, baseline, consent | Personal experiment plan | One change, safety limits, and stop rules are explicit; not clinical treatment | Person approves; clinician if required | Protocol, observations, result | `stopped` or `inconclusive` | WHEN a stop rule fires, the plan SHALL stop and show the selected human support path. |
| WELL-F3-002 | G3 | Consent and data held by separate custodians | Federated personal view | Custody stays local; raw values move only under grant | Person approves each source | Query, grant, source proof | `source_unavailable` | WHEN one source is removed, the view SHALL mark the gap and revoke its binding without harming others. |
| WELL-F3-003 | G3 | Consented cohort protocol and minimum group | Privacy-protected aggregate | No individual result or small-group release | Person opts in; ethics authority approves | Protocol, privacy test, release | `privacy_floor_failed` | WHEN group size falls below the floor, the system SHALL release no result. |
| WELL-F3-004 | G3 | User-held records and claims | Selective wellbeing proof | User selects each claim; raw journal and unrelated data stay private | Person approves presentation | Disclosure manifest and signature | `denied` | WHEN one routine claim is shared, unrelated mood, diagnosis, or journal fields SHALL not leave. |

### F4 — frontier network

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| WELL-F4-001 | G4 | Person-approved source bindings, exact data classes, purposes, time windows, and local custodian capabilities | Person-owned federated wellbeing view with source-local queries | Raw journals and protected values remain local unless explicitly selected; one custodian cannot expand another grant | The person approves every source, field, purpose, and recipient; each custodian enforces its local policy | Grants, local query proofs, source versions, view manifest, use, expiry, revocation | `source_unavailable`, `grant_expired`, or `meaning_mismatch` | WHEN a source is removed or a grant expires, new queries SHALL stop, the view SHALL show the gap, and other sources SHALL remain intact. |
| WELL-F4-002 | G4 | Ethics-approved protocol, opted-in local records, minimum cohort and privacy rules, contrary outcomes | Privacy-protected cross-community wellbeing findings | No individual or small-group result leaves; a finding cannot diagnose, treat, rank, or target a person | Each person opts in and may withdraw; custodians and ethics authority approve; humans decide any use | Protocol, consent, local checks, privacy proof, aggregate, limits, withdrawal | `privacy_floor_failed`, `consent_withdrawn`, or `inconclusive` | WHEN any group falls below its floor or consent is withdrawn, the network SHALL release no affected result and SHALL recompute future findings. |
| WELL-F4-003 | G4 | Person-selected summary, goals, care-circle roles, professional recommendations, and exact communication grants | Shared care-coordination plan with signed proposals, decisions, and status | It is not a diagnosis or treatment engine; no agent changes care, medication, contact, or emergency action; the person sees every disclosure | Person controls sharing and routine choices; accountable professionals control clinical decisions; agents only draft | Selection manifest, grants, professional identity, proposal, human decision, plan version, revoke events | `approval_waiting`, `role_revoked`, or `clinical_boundary` | WHEN a proposed change affects clinical care or exceeds a role, the network SHALL block it until the named accountable professional and person approve. |

## 8. Domain model

`Person`, `WellbeingHome`, `DeviceBinding`, `DataClass`, `ConsentGrant`,
`Purpose`, `Recipient`, `Goal`, `Routine`, `RoutineVersion`, `Habit`, `Reminder`,
`CheckInTemplate`, `CheckIn`, `Observation`, `Unit`, `Source`, `JournalEntry`,
`Attachment`, `Reflection`, `PatternAnalysis`, `Insight`, `CoachSession`,
`Suggestion`, `SafetyBoundary`, `HelpPlan`, `CareCircle`, `ProfessionalSummary`,
`ImportBatch`, `Mapping`, `PersonalExperiment`, and `SelectiveProof` are explicit.
Every sensitive record has owner, subject, data class, source, purpose, retention,
and audience. Observations and imports are append-only versions; corrections link
to the original. Consent and routine changes are versioned. Free-form metadata
cannot replace clinical source, unit, time, consent, or safety fields.

## 9. System architecture

- A Rust wellbeing engine validates ownership, consent, goals, routines,
  observations, sharing, safety boundaries, and lifecycle transitions.
- Application services handle check-ins, reflection, analysis, coaching, import,
  care circles, selective export, recovery, and user-held backup.
- An encrypted embedded store is the default single-person home. PostgreSQL and
  object storage support consented team or managed deployments.
- Domain records, idempotency results, and outbox audit events commit atomically.
- Sandboxed workers handle import, normalization, analysis, safe coaching, and
  export with minimum required fields.
- Versioned adapters isolate devices, clinical systems, model providers,
  notifications, and human-help directories.
- HelixCore supplies identity, policy, audit, capabilities, jobs, objects,
  billing, operations, stable project identity, and recovery.
- Offline is a first-class mode. Sync uses explicit operations and surfaces every
  consent or meaning conflict to the person.

## 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Visible progress, check, stop, reverse |
|---|---|---|---|---|---|
| Reflection agent | Person-approved entries and safe reflection prompts | Questions and summaries | Saving to durable memory or sharing | Diagnose, shame, infer protected facts, or contact anyone | Shows sources and plan; person edits/discards; stop is immediate. |
| Routine coach | Goal, approved routines, user preferences | Small routine suggestions | Every schedule change and notification | Create punitive streaks, treatment, or hidden score | Shows reasons and uncertainty; person accepts; plan versions can roll back. |
| Pattern agent | Exact approved observation range; statistics tools | Correlation and trend report | Save, share, or use for action | Call correlation causation or fill missing data | Streams data/check stages; reproducible method checks; delete removes draft. |
| Import agent | One approved adapter and data classes | Mapping and duplicate proposals | New data class, broad history, or external transfer | Retrieve credential value or guess unit | Shows field mapping and losses; quarantine protects source; import can roll back. |
| Safety router | Current user message and configured help plan only | Plain limits and human options | Contact or disclose to any person | Diagnose risk, promise monitoring, or act secretly | Shows exact next action; user chooses; event records only permitted metadata. |

Agents use exact, short leases. They cannot retrieve secrets, expand their own
scope, or read a whole tenant because one `mine` flag was omitted.

## 11. Trust, safety, and privacy

The default audience for all wellbeing content is the person only. Access checks
combine subject, tenant, resource, data class, purpose, recipient, exact fields,
time, and capability. Database and object policies enforce this, not a user
interface filter. Sensitive wellbeing, mental-health, clinical, biometric,
location, journal, child, and relationship data use separate classes. Encryption
is required in transit and at rest; local-only encryption keys remain user owned.
Residency is explicit before managed processing.

Delete moves allowed records to a recoverable 30-day bin. The person can restore
with identity and links. Clinical imports, professional records, legal duties,
research protocols, or safety evidence may have a clear retention or legal hold.
Permanent deletion is separate, re-authenticated, explicit, and audited. Controls
cover stalking, coercive sharing, account takeover, inference, harmful advice,
prompt injection, malicious imports, notification leakage, re-identification,
and use by employers or insurers. Incident recovery can lock a home, revoke every
grant and lease, rotate bindings, isolate adapters, restore signed state, and
explain what may have been seen.

## 12. Proof and audit

Metadata-only proof covers identity, consent grant, purpose, fields, use,
revocation, source, mapping, observation hash, method, model and prompt version
where used, safety check, human decision, export selection, and deletion. Secret
or health values are not placed in audit events. An independent verifier can
check signatures, grant timing, source integrity, method identity, and selected
bundle contents. It cannot prove that self-reported data is true, a device is
accurate, a pattern causes an outcome, or a suggestion is medically safe.

Aether is preferred for provider-neutral proof and capability brokering. The
local fallback signs metadata only, verifies exports, enforces leases, and keeps
private values out of Aether and portfolio memory.

## 13. UX system

The main surfaces are Today, Goals, Routines, Check-ins, Journal, Patterns,
Coach, Care Circle, Data, Help, Evidence, and Recovery. The default experience is
quiet, private, and non-judgemental. It shows one user-chosen next step. Deeper
views reveal sources, units, missingness, methods, grants, and audit. The product
targets [WCAG 2.2 Level AA](https://www.w3.org/TR/WCAG22/) and supports keyboard,
touch, screen reader, zoom, reduced motion, plain language, and low-cognitive-load
alternatives.

Imports, analysis, exports, and model work show named stages, real record counts,
elapsed time, last signal, saved work, and cancel state. Completion leaves a
durable activity record plus an optional private device notice whose text the
person controls. Selected records show clear checks. Moving data into sharing,
memory, or a different data class shows a field-level preview and always asks
when privacy changes. Undo is immediate for drafts; delete uses Recovery. Empty
states avoid pressure. Errors never blame the person, preserve work, say what
remained local, and offer a human path when safety matters.

## 14. Interoperability and standards

- [HL7 FHIR R5](https://hl7.org/fhir/R5/) is a versioned adapter for clinical
  resources and REST interactions. Many real systems use other FHIR releases and
  national profiles, so every connection declares its version and mapping loss.
- [HL7 International Patient Access 1.1](https://hl7.org/fhir/uv/ipa/STU1.1/)
  offers a read-only, FHIR R4-based patient-access profile. It does not grant
  write authority and national rules may add requirements.
- [SMART App Launch 2.2](https://hl7.org/fhir/smart-app-launch/) supports scoped
  authorisation to FHIR systems. Broad wildcard scopes are not the default; the
  broker requests the smallest supported scope.
- [Open mHealth schemas](https://www.openmhealth.org/schemas/) are an adapter for
  patient-generated and device observations. Device-specific detail may not map
  and must remain with the original record.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) sets the accessibility target.

Clinical and device standards never become the internal ownership or consent
model. Import previews identity, subject, source, units, timezone, reference
range, status, coding, precision, and unsupported extensions before commit.

## 15. Cross-platform contract

Ownership, consent, observation, analysis, export, proof, migration, and recovery
fixtures run on Windows, macOS, and Linux. Browser mode supports normal use but
states clearly when local-only data is unavailable. Desktop adds encrypted local
storage, offline work, user-owned broker access, file import, and private
notifications. The CLI supports backup, import, validate, export, revoke, and
verify, not coaching conversation. Containers support self-hosted services but
do not weaken local custody. Offline mode supports all core private work. Health
device, secure storage, notification, camera, and sensor access use capability
detection with manual entry, file import, or in-app fallback.

## 16. Reliability and performance budgets

- Acknowledged observation, journal, consent, and revocation writes have zero
  allowed data loss in forced-crash tests.
- A local check-in save finishes within 200 ms at p95 over a rolling 30-day
  window for a home with 10 million observations.
- A long job creates a durable stage within 2 seconds and has a local heartbeat
  no older than 5 seconds while active.
- Local cancel is accepted within 2 seconds and stops work within 30 seconds;
  an external adapter remains `cancel_requested` until confirmed.
- Imports, grants, revocations, and exports are idempotent for at least 30 days.
- A revoked grant blocks new reads within 2 seconds locally and within 60 seconds
  for a reachable managed recipient; offline recipients are visibly pending.
- Offline supports 90 days and 100 GB in the declared desktop profile; storage,
  key, and sync limits warn before failure.
- Managed committed metadata has recovery point zero and 1-hour recovery time;
  self-hosted recovery target is 4 hours; local encrypted backup restore is
  tested on every release.
- If models, Aether, a device, clinical system, or notification service fails,
  check-ins, journal, routines, local proof, revoke, and export continue.

## 17. Success measures

- People can correctly explain who can see each data class and revoke access.
- Median time to complete a useful check-in or reflection without pressure.
- Saved pattern reports that correctly state missingness, uncertainty, and limits.
- Clinical or dangerous requests that route to human help without giving advice.
- Unauthorised cross-person or whole-tenant reads; target zero per quarter.
- Revocation, 30-day restore, encrypted backup, and incident drill success.
- Portable summaries and bundles that validate on another supported OS.
- Accessibility task success and serious issue counts for cognitive, keyboard,
  screen-reader, zoom, and touch journeys.
- User-reported helpful changes and trust, not streaks, time, or agent-call counts.
- Sustainable paid retention without selling or advertising from personal data.

## 18. Delivery plan

| Gate | Build | Test | Safety | UX | Cross-platform | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Stable private home, consent engine, atomic records/events, recovery | Cross-person denial, crash, revoke, signature tests | Clinical boundary, threat model, secret broker | Private default, honest jobs, Recovery | Rust and packaging CI on Windows, macOS, Linux | Dry-run current habits/logs/check-ins importer | Fresh local setup, denied read, crash, restore, revoke |
| **G1 — Useful single-player product (6–18 months)** | Goals, routines, check-ins, journal, patterns, backup | Complete private journeys and missing-data fixtures | Harmful-copy and privacy review | Calm accessible daily use | Web, desktop, CLI, container, 90-day offline | Encrypted home and observation migrations | Real 30-day local use simulation on each OS |
| **G2 — Trusted team product (18–30 months)** | Bounded coach, care circles, imports, grants | Agent safety, grant, device, conflict tests | External clinical-safety and privacy review | Consent preview, slow-work, help journeys | Device and low-connectivity matrix | FHIR/Open mHealth mappings and rollback | Fresh share, revoke, incident, and recovery drill |
| **G3 — Category leader (30–42 months)** | Professional summaries, experiments, adapter kit | Provenance, unit, experiment, scale tests | Model, coercion, re-identification review | Evidence and uncertainty comprehension | Mixed deployment and clinician export proof | Verified move from another wellbeing store | External user, clinician, accessibility, security review |
| **G4 — Frontier network (42–60 months)** | Build WELL-F4-001 person-owned federated views, WELL-F4-002 privacy-protected learning, and WELL-F4-003 care coordination | Field-consent, source-loss, meaning, privacy-floor, re-identification, coercion, clinical-boundary, partition, revoke, and malicious-node tests | Independent ethics, clinical-safety, privacy, security, legal, and accessibility review; no diagnosis, treatment, or hidden contact | Person-controlled connect, query, share, coordinate, withdraw, no-result, urgent-human-help, and exit journeys | Mixed Windows/macOS/Linux personal, custodian, and professional nodes prove local custody, offline revoke, safe degradation, and selective proof | Add/remove a source, project, provider, or care member; revoke selected bindings without deleting shared secrets or other users' records | Independent consent, query, care proposal, human decision, withdraw, remove-node, disaster-recover, export, and verify exercise covering all F4 evidence |

Every gate closes only from fresh release-candidate evidence. A safety test that
uses only prepared friendly prompts or a privacy check that bypasses the database
policy cannot pass.

## 19. Current truth and gap

The live Rust source has meaningful habits, habit logs, and mood/energy check-ins.
The database constrains mood and energy values to 1–10. This is an early backend
prototype. Privacy is not safe enough: list operations can expose a whole tenant
unless callers add `mine=true`. There is no consent engine, private-home model,
goals, routines, journal, pattern analysis, coach, device or clinical adapter,
care circle, product UI, or domain test suite. The service also has the shared
application-state compile failure, and domain writes are not atomic with audit
or billing events.

The P0 gap is access control at the data boundary, not more wellbeing features.
The safest first slice is WELL-F0-001 through WELL-F1-001: every query is
subject-scoped by enforced policy, a person records one optional check-in, skips
one field without creating zero, exports it, revokes a test grant, and restores
after a forced crash. Use temporary test state only.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Identity | Stable person/home/record IDs independent of folders, devices, providers | Architecture decision and restore proof |
| Privacy | Default audience self; access enforced at database/object boundary | Founder and independent privacy review |
| Data meaning | Missing is not zero; source, unit, time, and class are required | Domain review |
| Product boundary | Wellbeing reflection and coaching only; no diagnosis, treatment, or medication | Founder, clinical, legal, regulatory approval |
| Agent authority | Draft, explain, and suggest; never contact, diagnose, share, or act alone | Safety review |
| Secrets | User-owned capability broker; agents never retrieve values | Security review |
| Proof | Metadata-only; Aether preferred, local fallback; health values excluded | Privacy and proof review |
| Delete | Recoverable 30-day bin; explicit retention and legal hold may block destruction | Legal and clinical retention decision |
| Accessibility | WCAG 2.2 AA target plus cognitive-load and non-judgemental language review | Accessibility review |
| Standards | FHIR, IPA, SMART, Open mHealth behind declared adapters | Clinical interoperability review |
| First slice | Enforced self-only query → optional check-in → revoke → export → crash recovery | Product decision with equal privacy closure |
| Research, insurer/employer access, regulated function | Off by default and separate founder/ethics/legal gates | Founder approval |

## 21. Definition of category-defining done

- [ ] People complete all eight journeys while understanding ownership and limits.
- [ ] The database prevents cross-person and expired-purpose access by default.
- [ ] No model diagnoses, treats, changes medication, contacts, or shares alone.
- [ ] Every suggestion shows evidence, uncertainty, safety limits, and human choice.
- [ ] Consent is exact, visible, time-bound, revocable, and independently auditable.
- [ ] Atomic writes, offline work, and encrypted recovery lose no acknowledged data.
- [ ] Selective exports and proof validate without the live server or Aether.
- [ ] FHIR and device adapters name every unit, source, version, and mapping loss.
- [ ] WCAG 2.2 AA scope and cognitive-accessibility journeys pass human review.
- [ ] Windows, macOS, Linux, web, offline, CLI, and container limits are proven.
- [ ] The 30-day bin, retention, legal hold, permanent delete, and restore work.
- [ ] Independent security, privacy, clinical-safety, ethics, and accessibility reviews close.
- [ ] The product states clearly what personal data and proof cannot establish.
