# HelixEdu — evidence-first adaptive learning

```yaml
product: HelixEdu
catalog_order: 6
status: target-state-spec
horizon: 60 months
current_maturity: prototype
primary_users: [learners, educators, mentors, learning organisations, credential reviewers]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

> **Target-state rule:** The future product is specified in Sections 1–18 and
> 20–21. Section 19 records the live code that exists now.

## 1. Category claim

HelixEdu is a learner-owned mastery system that connects teaching, practice,
assessment, feedback, and portable proof without reducing learning to time spent
or one opaque score.

## 2. Five-year destination

The useful product lets an educator build courses, learners work online or
offline, and both sides see progress through lessons, practice, assessment, and
feedback. The category-defining advantage is a transparent mastery graph: every
claim about learning links to the goal, work, assessment rule, feedback, human
judgement, and later use. The frontier capability is a private learning network
where people carry verified skills across schools, work, and self-directed
projects while adaptive tutors learn only within consented limits. Educators and
learners retain authority over goals, content, high-stakes grades, credentials,
sharing, and any action that changes opportunity.

## 3. Users and hard jobs

- **Learners** need to understand, practise, prove, and carry what they know.
  They fear a score that judges them without showing how to improve.
- **Educators** need to design learning and see where help is needed. They fear
  automation that hides mistakes or removes professional judgement.
- **Mentors and families** need a clear, consented view of support needs. They
  fear surveillance or access that continues after it is no longer wanted.
- **Learning organisations** need cohorts, rosters, quality, and portability.
  They fear vendor lock-in and unsafe handling of child data.
- **Credential reviewers** need to verify a learning claim. They fear a badge
  with no issuer, criteria, evidence, or revocation state.

## 4. Product laws

1. A learner can see why the system believes a skill is known or not yet known.
2. Time, clicks, and completion alone never prove mastery.
3. A high-stakes grade or credential needs an accountable human decision.
4. Tutor output is guidance, not hidden assessment or psychological profiling.
5. A learner owns a portable copy of their approved work and credentials.
6. Child and student data use the smallest purpose, access, and retention.
7. Accessibility and low-bandwidth offline learning are core, not later add-ons.
8. Course, rubric, assessment, and mastery meaning change only by version.
9. Long generation, grading, import, and sync work shows real progress.
10. Uncertain evidence is labelled uncertain and never turned into a firm score.

## 5. Scope boundaries

HelixEdu owns learning goals, mastery maps, courses, lessons, activities,
assessments, submissions, feedback, learning plans, cohorts, credentials, and
learner portfolios. HelixCore owns identity, policy, audit, capabilities, jobs,
objects, billing, and operations. HelixCollab owns general team communication.
HelixInsights owns broad analytics. Aether may provide proof and capability
leases through an adapter.

HelixEdu is not a replacement for a school information system, licensed teacher,
special-needs professional, psychologist, or regulator. It does not diagnose a
learner or make an admission, discipline, employment, or other high-stakes
decision. Such decisions remain with accountable people and organisations.

## 6. Signature experiences

1. **Build a course from clear goals.** **Entry:** an educator starts with a
   goal, standard, or existing course. **Visible progress:** structure, content,
   rights, accessibility, and assessment checks show separately. **Human
   decision:** the educator approves every published version. **Completion
   proof:** the course links goals, lessons, activities, rubrics, and checks.
   **Failure and recovery:** failed imports keep a recoverable draft and loss
   report. **Export:** the course and its assets use open or documented formats.
2. **Learn online or offline.** **Entry:** a learner opens Today's Plan.
   **Visible progress:** lesson stage, saved work, local sync, and mastery evidence
   are clear. **Human decision:** the learner may change pace, ask for help, or
   hide optional signals. **Completion proof:** completed work and checks are
   attached to the goal. **Failure and recovery:** offline work queues safely and
   conflicts never overwrite silently. **Export:** the learner can take approved
   work, feedback, and progress.
3. **Practise with a bounded tutor.** **Entry:** the learner asks for help on one
   goal. **Visible progress:** the tutor shows its plan, sources, checks, and
   remaining steps. **Human decision:** the learner chooses hints, explanation,
   or a mentor; an educator sets allowed tools. **Completion proof:** practice
   attempts and cited course sources are kept. **Failure and recovery:** unsafe,
   uncertain, or off-topic output stops and routes to a person. **Export:** the
   learner can save or delete the session under policy.
4. **Submit and receive fair feedback.** **Entry:** a learner submits work.
   **Visible progress:** upload, checks, plagiarism or similarity review, human
   queue, and release state are separate. **Human decision:** a named educator
   owns the final high-stakes grade. **Completion proof:** rubric version,
   feedback, evidence, and approval are linked. **Failure and recovery:** broken
   files or disputed feedback keep the original and support resubmission or
   appeal. **Export:** submission and feedback form a portable packet.
5. **See mastery, not just completion.** **Entry:** learner or educator opens the
   mastery map. **Visible progress:** proven, developing, stale, disputed, and
   unknown states use words and shapes. **Human decision:** an educator may
   accept other evidence or correct a mapping by new version. **Completion
   proof:** each state links to its evidence and rule. **Failure and recovery:**
   weak evidence stays unknown. **Export:** the map and evidence can be exported.
6. **Issue and verify a credential.** **Entry:** an educator selects a completed
   achievement. **Visible progress:** identity, criteria, evidence, approval,
   signing, and delivery checks show. **Human decision:** an authorised issuer
   approves issue or revocation. **Completion proof:** the signed credential has
   a verification and status path. **Failure and recovery:** signing or delivery
   failure never marks it issued. **Export:** the learner holds the credential in
   a standard format.
7. **Move a learner or cohort safely.** **Entry:** an administrator starts an
   import, export, or transfer. **Visible progress:** users, courses, enrolments,
   grades, resources, consent, and errors are counted. **Human decision:** owners
   approve sensitive transfer and identity matches. **Completion proof:** source
   and destination counts and samples reconcile. **Failure and recovery:** the
   transfer can resume or roll back without duplicate learners. **Export:** no
   school or platform ID is the sole identity.

## 7. Capability map

### F0 — foundation

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| EDU-F0-001 | G0 | Person, organisation, role, consent | Stable learning identity | Identity is not a path or vendor ID; child policy is explicit | Organisation grants role; learner controls optional sharing | Grant, consent, revocation | `access_denied` | WHEN a learner changes organisation, the system SHALL preserve identity while revoking old bindings. |
| EDU-F0-002 | G0 | Goal facts and relationships | Versioned mastery graph | Cycles and missing goals fail validation; meaning never changes in place | Educator or standards owner approves | Graph diff and approval | `invalid_graph` | WHEN a goal meaning changes, the system SHALL keep old evidence linked to the old version. |
| EDU-F0-003 | G0 | Submission or progress command | Domain record plus event | Write and evidence event commit together; retries are idempotent | Domain policy | Transaction and replay proof | `not_committed` | WHEN a crash occurs during progress save, recovery SHALL show either the prior state or one complete new state. |
| EDU-F0-004 | G0 | Offline operation log | Merged or disputed state | No silent last-write-wins on grades, consent, or mastery | User may sync; human resolves meaning conflicts | Operation log and resolution | `conflict` | WHEN two devices edit the same graded work, the system SHALL preserve both and request resolution. |

### F1 — useful product

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| EDU-F1-001 | G1 | Goals, content, rights, activities | Course version | Published content is immutable; rights and accessibility fields are required | Educator approves publish | Checks, preview, approval | `draft` or `blocked` | WHEN a course lacks required text alternatives, publish SHALL be blocked with exact locations. |
| EDU-F1-002 | G1 | Course and learner plan | Enrolment and learning path | Enrolment does not imply consent to optional analytics | Learner/organisation enrols by policy | Terms, consent, plan version | `waiting` or `withdrawn` | WHEN a learner withdraws, new optional processing SHALL stop while required records follow policy. |
| EDU-F1-003 | G1 | Lesson actions and work | Durable progress evidence | Completion and mastery are separate values | Learner acts; educator defines rules | Attempt, time range, result | `in_progress` or `sync_pending` | WHEN a lesson is completed without mastery evidence, the system SHALL show complete and mastery unknown. |
| EDU-F1-004 | G1 | Assessment, rubric, submission | Result and feedback draft | Original submission and rubric version are immutable | Educator owns final high-stakes result | Checks, annotations, approval | `needs_review` or `disputed` | WHEN an agent drafts a score, release SHALL remain blocked until a named educator approves it. |

### F2 — category leader

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| EDU-F2-001 | G2 | Goal, approved content, learner request | Tutor response and practice | Tutor cites allowed sources and cannot write grades | Learner asks; educator sets tools | Sources, plan, safety checks | `stopped` or `needs_human` | WHEN the tutor lacks grounded support, it SHALL say it is unsure and route to approved help. |
| EDU-F2-002 | G2 | Evidence and mastery rule | Mastery claim | Claim names strength, freshness, and contrary evidence | Educator approves overrides | Rule evaluation and evidence links | `unknown` or `disputed` | WHEN evidence expires under its rule, mastery SHALL move to stale, not silently remain proven. |
| EDU-F2-003 | G2 | Cohort, consented signals, goals | Support queue | Protected traits and private text are excluded by default | Educator chooses intervention | Signal reasons and action | `insufficient_evidence` | WHEN a learner is flagged, the educator SHALL see the exact signals and a non-automated review path. |
| EDU-F2-004 | G3 | Achievement, criteria, evidence, issuer | Signed credential | Issue and revocation require authorised human; no credential without criteria | Issuer approves | Signature, status, evidence refs | `signing_failed` or `revoked` | WHEN signing fails, no credential SHALL appear issued and retry SHALL keep one identity. |

### F3 — advanced category leadership

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| EDU-F3-001 | G3 | Mastery, goals, preferences | Adaptive plan proposal | Adaptation is explainable and cannot narrow opportunity silently | Learner and educator approve material changes | Reasons, alternatives, decision | `not_recommended` | WHEN a path removes a goal, the system SHALL explain why and require approval. |
| EDU-F3-002 | G3 | Learner-held credentials and consent | Selective proof | Only chosen claims are shared; verifier cannot demand hidden fields through the product | Learner approves every presentation | Consent and disclosed fields | `denied` | WHEN a learner selects one skill, the export SHALL not include unrelated grades or identity fields. |
| EDU-F3-003 | G3 | Institution adapters and transfer plan | Federated learning record | Each custodian keeps control and can revoke its binding | Learner and institutions approve | Transfer, mapping, revocation | `mapping_loss` | WHEN a target cannot express evidence, transfer SHALL stop or list the loss before approval. |
| EDU-F3-004 | G3 | Anonymised, consented learning evidence | Tested teaching insight | No individual decision is made from cohort research | Research authority and ethics approval | Protocol, privacy check, result | `privacy_floor_failed` | WHEN the privacy floor is not met, the system SHALL release no cohort result. |

### F4 — frontier network

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| EDU-F4-001 | G4 | Learner-held mastery claims, credentials, selected work, institution mappings, consent | Cross-institution learner-owned mastery graph and selective presentation | Canonical learner identity and raw work stay under learner control; unlike goals are not merged silently | Learner approves every field and recipient; issuers sign their claims; institutions cannot extend consent | Goal mappings, issuer status, disclosure manifest, consent, verification and revocation events | `mapping_loss`, `credential_revoked`, or `denied` | WHEN a target cannot preserve goal meaning or evidence, the network SHALL show the loss and require learner approval before transfer. |
| EDU-F4-002 | G4 | Learner-selected goals and gaps, provider-approved courses or mentors, safety and cost limits | Explainable cross-provider learning option set | Private grades and protected traits remain local; no option enrols, contacts, or narrows opportunity automatically | Learner chooses fields and option; educator/guardian authority applies where required; provider accepts enrolment | Query grant, used fields, option reasons, source quality, human choice | `insufficient_match` or `provider_unavailable` | WHEN evidence is weak or an option uses paid placement, the network SHALL label it and show a non-paid or no-result path. |
| EDU-F4-003 | G4 | Ethics-approved protocols, consented local learning evidence, minimum cohort and privacy rules | Privacy-protected cross-institution teaching findings with contrary outcomes | No individual record or small-group result leaves; findings cannot make learner decisions | Learners or lawful representatives opt in; each institution and ethics authority approve; educators decide use | Protocol, grants, local checks, privacy proof, aggregate, contrary evidence, withdrawals | `privacy_floor_failed` or `study_inconclusive` | WHEN any cohort falls below its approved floor or consent is revoked, the network SHALL release no affected result and SHALL recompute future findings. |

## 8. Domain model

`Learner`, `Educator`, `Organisation`, `GuardianRelationship`, `Consent`, and
`Cohort` relate to `LearningGoal`, `GoalVersion`, `GoalRelation`, `Course`,
`CourseVersion`, `Module`, `Lesson`, `Activity`, `Resource`, `Licence`,
`Enrolment`, `LearningPlan`, `Attempt`, `Submission`, `Assessment`, `Item`,
`Rubric`, `RubricVersion`, `ScoreDraft`, `GradeDecision`, `Feedback`, `Appeal`,
`MasteryRule`, `MasteryClaim`, `TutorSession`, `SupportSignal`, `Intervention`,
`Achievement`, `Credential`, `Presentation`, and `Revocation`. Goals, courses,
rubrics, assessments, submissions, grades, and credentials are versioned or
immutable. Ownership, subject, issuer, purpose, retention, and consent are real
fields. Private learner notes are not generic metadata and do not enter broad
analytics by default.

## 9. System architecture

- A Rust learning engine validates goals, courses, enrolment, submissions,
  mastery, grade, credential, and appeal lifecycles.
- Application services handle authoring, delivery, sync, assessment, tutoring,
  review, credentials, transfer, and export.
- PostgreSQL is the durable team source; object storage holds learning assets and
  submissions; an embedded local store supports private offline learning.
- A transactional outbox commits learning records and audit events together.
- Sandboxed workers handle import, media processing, accessibility checks, safe
  tutor work, assessment checks, credential issue, and export.
- Versioned adapters isolate learning platforms, roster systems, assessment
  formats, credential wallets, and content stores.
- HelixCore supplies identity, policy, audit, capabilities, jobs, objects,
  billing, operations, stable identity, notifications, and recovery.
- Offline sync uses explicit operations and human conflict resolution for
  grades, consent, and mastery meaning.

## 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Visible progress, check, stop, reverse |
|---|---|---|---|---|---|
| Course-design agent | Approved goals, resources, licences; authoring and accessibility tools | Course, lesson, activity, rubric drafts | Publish and rights claim | Copy unlicensed work or invent source facts | Shows source, rights, and checks; draft versions undo safely. |
| Tutor agent | One learner's approved goal, course sources, and allowed work | Hints, examples, practice, explanations | Tool use beyond policy or contact with a person | Grade, diagnose, shame, profile, or reveal answers against rules | Streams plan and citations; safety and lesson tests check; learner can stop/delete. |
| Assessment assistant | Submission, rubric, permitted reference set | Feedback and score suggestion | Every high-stakes grade and release | Change submission, rubric, or final grade | Shows criterion-by-criterion basis; educator accepts, edits, or rejects; original stays. |
| Learning coach | Learner-approved mastery and plan | Goal, schedule, and support suggestions | Material path change or third-party sharing | Restrict opportunity or message others alone | Shows reason and alternatives; learner can dismiss or reverse. |
| Credential agent | Approved achievement, issuer policy, evidence | Credential preview and verification report | Issue, revoke, or disclose | Create issuer authority or expose unrelated claims | Shows signing and status stages; verifier checks; failed issue leaves no live credential. |

Agents get narrow leases, never raw secrets. For children or protected education
records, policy can disable an agent role completely.

## 11. Trust, safety, and privacy

Tenant, organisation, relationship, role, purpose, learner age, resource, and
exact capability control access. Database and object-store rules enforce tenant
separation. Sensitive education records, private notes, disability information,
and child data receive separate labels and purpose limits. Encryption applies in
transit and at rest. Guardian authority is represented, time-limited, and never
silently treated as permanent ownership of an adult learner's record. Data
residency and local law are deployment policy, not hard-coded assumptions.

Delete moves drafts, learner-created work where policy allows, tutor sessions,
and optional profile data to a recoverable 30-day bin. Required education,
credential, safeguarding, and legal-hold records follow their stated retention.
Permanent deletion is a separate, re-authenticated, explicit, audited action.
Controls address bullying, grooming, cheating, prompt injection, unsafe tutor
advice, answer leakage, plagiarism false positives, surveillance, discriminatory
ranking, malicious content, and bulk export. Incident response can disable an
agent or adapter, preserve evidence, revoke leases, notify accountable adults,
and restore from signed state without hiding uncertainty.

## 12. Proof and audit

Proof includes course and rubric versions, source and rights checks, learning
goal, attempts, submission hash, feedback, score draft, human grade decision,
appeal, mastery rule, supporting and contrary evidence, credential criteria,
issuer approval, signature, status, consent, and disclosure. An independent
verifier can check integrity, issuer authority, status, mappings, and permitted
evidence. Proof does not show that teaching was good, that a learner fully owns a
skill in every setting, or that a grade was fair beyond its recorded process.

Aether is the preferred proof and capability provider through neutral
interfaces. Local signing, verification, capability leases, and export remain
available when Aether is offline or absent.

## 13. UX system

The learner surfaces are Today, Learn, Practise, Ask, Work, Progress, Portfolio,
and Recovery. Educators use Courses, Learners, Review, Support, Credentials,
Evidence, and Settings. Basic views use plain goals and next steps; deeper views
reveal rubrics, evidence, mastery rules, and interoperability mappings. The
product targets [WCAG 2.2 Level AA](https://www.w3.org/TR/WCAG22/) and supports
keyboard, touch, screen reader, captions, transcripts, reduced motion, readable
language, zoom, and alternatives to timed or drag-only work.

Imports, media work, tutor responses, assessment checks, credential issue, and
sync show real stages, saved work, elapsed time, last signal, and cancel state.
Completion leaves a durable item and optional device notice. Selection has a
clear check. Moving a lesson, goal, learner, or evidence item previews affected
links and requires confirmation when meaning or access changes. Safe undo is
immediate; delete uses the 30-day bin. Empty states teach the next useful step.
Errors preserve work, avoid blame, say what was saved, and name a person or safe
action for help.

## 14. Interoperability and standards

- [1EdTech LTI 1.3](https://www.imsglobal.org/spec/lti/v1p3/) integrates learning
  platforms and tools with modern authentication. Roles, local settings, and
  deep product state may not map and must be reported.
- [1EdTech OneRoster 1.2](https://standards.1edtech.org/oneroster/specifications/standards/v1p2)
  exchanges organisations, users, courses, enrolments, resources, and gradebook
  data. Local consent and fine-grained mastery evidence need extensions or a
  separate bundle.
- [1EdTech QTI 3.0](https://www.imsglobal.org/spec/qti/v3p0/oview) imports and
  exports assessment content, scoring, responses, and accessibility data. Custom
  item behaviour may not survive another player.
- [1EdTech Open Badges 3.0](https://www.imsglobal.org/spec/ob/v3p0/) carries
  portable achievement credentials and evidence references aligned to the
  [W3C Verifiable Credentials Data Model 2.0](https://www.w3.org/TR/vc-data-model-2.0/).
  A credential still depends on issuer trust and does not prove broad mastery.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) sets the accessibility target.

Every adapter pins its profile, uses conformance fixtures, and previews lost
roles, goals, rubric rules, accommodations, consent, evidence, signatures, and
extensions before commit.

## 15. Cross-platform contract

Course rules, assessment scoring, mastery evaluation, proof, migration, and
recovery use the same fixtures on Windows, macOS, and Linux. The browser supports
full learning and review. Desktop adds offline packs, local files, notifications,
and a user-owned broker. The CLI supports import, validate, package, sync,
export, and verify, not every teaching interaction. Containers support managed
or self-hosted servers and workers. Offline packs declare size, expiry, allowed
tools, and sync limits before download. Cameras, microphones, text-to-speech,
notifications, and secure storage use capability detection with typed text,
manual upload, in-app notice, or other safe fallback.

## 16. Reliability and performance budgets

- Acknowledged submission, grade, consent, credential, and progress writes have
  zero allowed data loss in forced-crash tests.
- Saved learner work is durable within 1 second locally; online save completes
  under 500 ms at p95 over a rolling 30-day window, excluding file upload.
- A long job shows its first durable stage within 2 seconds and a local heartbeat
  no older than 5 seconds while active.
- Local cancel is accepted within 2 seconds and stops work within 30 seconds;
  external state remains `cancel_requested` until confirmed.
- Submission, credential, roster, and sync commands are idempotent for at least
  30 days and across provider retries.
- One course supports 10,000 concurrent learners and 1,000 submissions per minute
  in the supported managed profile without losing order or tenant separation.
- Offline mode supports 30 days or 20 GB per learner; expiry and remaining space
  are visible seven days before the limit.
- Managed committed metadata has recovery point zero and 1-hour recovery time;
  self-hosted documented recovery target is 4 hours.
- If tutor models, Aether, notifications, or an LMS adapter fail, downloaded
  learning, manual teaching, submission, local proof, and export continue.

## 17. Success measures

- Learners can explain their goal, evidence, next step, and uncertainty.
- Improvement from first attempt to later independent work, not time or clicks.
- Median educator time from submission to useful, reviewed feedback.
- High-stakes grades released without named human approval; target zero.
- Tutor answers grounded in allowed sources and correctly routed when unsure.
- Portable courses and credentials that validate on another supported system.
- Accessibility task success and serious issue counts across learner and educator
  journeys for the declared WCAG scope.
- Offline completion and conflict-recovery success under real low-bandwidth tests.
- Consent withdrawal, 30-day restore, and incident drill success.
- Retained learners and organisations who report better outcomes and lower
  administration effort, not raw enrolment or agent-call counts.

## 18. Delivery plan

| Gate | Build | Test | Safety | UX | Cross-platform | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Stable identity, goal graph, atomic learning ledger, jobs, recovery | Domain, crash, sync, signature tests | Child-data policy, tenant rules, secret broker | Honest progress, review, Recovery | Rust and packaging CI on Windows, macOS, Linux | Dry-run importer for current courses/enrolments | Fresh install, offline save, crash, restore, verify |
| **G1 — Useful single-player product (6–18 months)** | Authoring, lessons, work, assessment, feedback, mastery | Complete learner/educator journeys and scoring fixtures | Consent, content, export checks | Accessible learning and authoring | Web, desktop, CLI, container, 30-day offline | Course and work package migrations | Fresh real course completion on each OS |
| **G2 — Trusted team product (18–30 months)** | Cohorts, review queues, tutor, support, roles | Race, permission, tutor safety, appeal tests | Safeguarding and privacy review | Human-in-loop and slow-work journeys | Low-bandwidth and device matrix | OneRoster/LTI mappings and rollback | Fresh cohort, appeal, incident, and recovery drill |
| **G3 — Category leader (30–42 months)** | Adaptive plans, credentials, portfolios, adapter kit | QTI, badges, mastery, scale conformance | Bias, accessibility, credential threat review | Evidence and uncertainty comprehension | Mixed deployment and wallet proof | Verified institution transfer | External educator, learner, accessibility, and security review |
| **G4 — Frontier network (42–60 months)** | Build EDU-F4-001 learner-owned mastery graphs, EDU-F4-002 cross-provider option matching, and EDU-F4-003 consented research cooperation | Mapping-loss, selective-disclosure, paid-placement, privacy-floor, bias, child-safety, partition, revoke, and malicious-node tests | Independent child-safety, education, ethics, privacy, fairness, and security review; no automated high-stakes decision | Learner-controlled disclose, compare, enrol, withdraw, research-consent, no-result, and exit journeys | Mixed Windows/macOS/Linux institution nodes prove offline consent, credential verification, local custody, and safe degradation | Add/remove an institution or provider, revoke selected learner bindings, and preserve stable identity plus other issuers' credentials | Independent transfer, match, verify, consent, withdraw, remove-node, disaster-recover, and export exercise covering all F4 evidence |

Fresh release-candidate evidence is required at every gate. Skipped tests,
synthetic-only tutor checks, or stale accessibility reports cannot close a gate.

## 19. Current truth and gap

The live Rust source has real courses, course publication, enrolments, progress,
and a completion event. Progress is constrained from 0 to 100. This is a useful
backend prototype, not yet a learning product. There is no lesson-content model,
assessment, rubric, submission, feedback, credential, adaptive plan, tutor,
educator or learner UI, offline workflow, or domain test suite. The service also
has the repository's shared application-state compile problem. Domain changes
and audit or billing events are not one atomic write.

The most important gap is that completion currently has no evidence of learning.
The safest first slice is EDU-F0-002 through EDU-F1-004: one versioned goal, one
lesson, one submission, one rubric, one human-approved result, and one mastery
state that remains unknown until evidence passes. Test all writes, crashes, and
sync using temporary state only.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Identity | Stable learner, goal, course, submission, and credential IDs; never folder paths or provider IDs | Architecture decision and transfer proof |
| Learning truth | Completion, score, and mastery are separate records | Education product review |
| High-stakes result | Named educator approval is mandatory | Founder, education, legal, and safety approval |
| Durable write | Domain record, event, and idempotency result commit together | Founder-approved integrity exception |
| Tutor | Grounded in approved course sources; bounded tools; no diagnosis or grade | Safety review |
| Standards | LTI, OneRoster, QTI, and Open Badges behind versioned adapters | Interoperability review |
| Proof provider | Aether preferred; signed local fallback required | Provider-neutrality review |
| Secrets | User-owned capability broker; agents never receive values | Security review |
| Delete | 30-day recovery bin; legal, safeguarding, and credential holds may block destruction | Legal-retention decision |
| Accessibility | WCAG 2.2 AA target plus learning-specific accommodations | Accessibility and educator review |
| First slice | Goal → lesson → submission → human feedback → evidence-backed mastery | Product decision with equal learning proof |
| Admissions, discipline, employment ranking | Never an autonomous product capability | Founder plus legal and ethics approval |

## 21. Definition of category-defining done

- [ ] Learners and educators complete all seven journeys with real courses.
- [ ] Every mastery claim links to goals, work, rules, feedback, and contrary data.
- [ ] No high-stakes grade or credential issues without accountable human approval.
- [ ] Tutor output is grounded, bounded, stoppable, and honest when unsure.
- [ ] Child, student, consent, purpose, and retention controls pass independent review.
- [ ] Atomic writes, offline sync, and crash recovery lose no acknowledged work.
- [ ] Independent credentials and learning bundles verify without the live server.
- [ ] Learners move their work and revoke old bindings without identity loss.
- [ ] WCAG 2.2 AA scope and learning accommodations pass human tests.
- [ ] Windows, macOS, Linux, web, offline, CLI, and container limits are proven.
- [ ] The 30-day bin, legal hold, permanent delete, appeal, and restore work.
- [ ] External security, privacy, educator, learner, and accessibility reviews close.
- [ ] The product states what a grade, mastery claim, and credential do not prove.
