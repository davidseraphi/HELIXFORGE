# HelixSynthBio — safe design-build-test-learn biology workspace

```yaml
product: HelixSynthBio
catalog_order: 11
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [synthetic biologists, laboratory leads, biosafety officers, research reviewers]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixSynthBio is a human-led workspace that connects safe biological design to its
assumptions, risk review, simulation, experiment plan, observations, interpretation,
and portable proof without turning an agent into a wet-lab operator.

## 2. Five-year destination

The useful product is a local-first design registry, sequence and construct editor,
risk-review workspace, protocol planner, experiment tracker, and evidence viewer.
The leading product links every result to the exact design version, sample lineage,
instrument run, analyst action, and uncertainty. The frontier product can compare
many safe candidate designs in a bounded simulation space and suggest the next
informative experiment. It never orders material, controls laboratory equipment,
or releases a biological design on its own. A qualified human and the institution's
biosafety authority keep final control.

## 3. Users and hard jobs

- A synthetic biologist needs to compare designs without losing sequence identity,
  assumptions, or failed results.
- A laboratory lead needs a clear handoff from approved design to a controlled
  experiment and fears silent protocol drift.
- A biosafety officer needs early risk context, complete change history, and the
  power to block work.
- A data scientist needs measured data with sample, method, unit, and uncertainty,
  not a clean chart with weak provenance.
- A reviewer needs to reproduce the reasoning while protecting sensitive sequence
  and security information.
- An institution needs local custody, clear access limits, and a lawful retention
  policy for high-consequence work.

## 4. Product laws

1. Design, prediction, experiment plan, observation, and conclusion are different
   records and are never presented as the same kind of truth.
2. Risk review starts before a design can move toward physical work.
3. No agent may order DNA, release a sequence, operate equipment, or approve work.
4. Every biological object has a stable identity and immutable versions.
5. Negative, failed, and inconclusive results remain part of the evidence.
6. Sensitive sequence content follows least privilege and is excluded from general
   search, notifications, logs, and portfolio memory.
7. External models and databases are versioned sources, not unquestioned truth.
8. Core authoring, review, and evidence export work without a cloud dependency.
9. Critical journeys are accessible, portable, and independently verifiable across
   platforms; no model, vendor, or proof provider is a hard dependency.

## 5. Scope boundaries

SynthBio owns design records, safe computational analysis, risk-review routing,
experiment plans, sample lineage, result capture, interpretation, and evidence
bundles. HelixCore owns shared identity, policy, audit, capabilities, jobs, objects,
billing, and operations; Aether is reached only through a provider-neutral proof
interface with a local fallback. It does not replace an institutional biosafety committee, a laboratory
information management system, a regulated quality system, or trained laboratory
staff. It does not provide instructions for harmful biological activity. It does
not execute physical work. Equipment control, procurement, clinical use, release
to the environment, and human or animal work require separate systems, laws,
authorities, and explicit human approval.

## 6. Signature experiences

1. **Create a safe design record.** Entry: the scientist imports an approved public
   record or starts a blank design. Progress: identity, components, source, and
   validation appear step by step. Human decision: the scientist accepts the
   canonical version. Proof: source hashes and edits are recorded. Failure and
   recovery: invalid or unsupported data stays quarantined with a reason and can
   be corrected. Export: SBOL and a human-readable design bundle.
2. **Run an early risk screen.** Entry: a design is proposed for review. Progress:
   data sensitivity, host context, intended use, and policy checks show live status.
   Human decision: a biosafety officer allows computational work, asks for more
   facts, restricts access, or blocks the design. Proof: policy version, reviewer,
   reasons, and decision are signed. Failure and recovery: unavailable checks create
   `unknown`, never `safe`; review can resume from the last durable step. Export: a
   redacted risk-review packet.
3. **Compare safe candidate designs.** Entry: the scientist selects approved design
   versions and a bounded model. Progress: queued, running, completed, failed, and
   uncertain candidates stay visible. Human decision: the scientist chooses which
   result is worth further review. Proof: model, parameters, seed, limits, and all
   rejected candidates are kept. Failure and recovery: a cancelled run stops at a
   checkpoint and can restart without duplicate results. Export: comparison data
   plus an evidence report.
4. **Plan a controlled experiment.** Entry: an approved design version is linked to
   a study question. Progress: materials, controls, measurements, hazards, and
   approvals form a visible readiness path. Human decision: laboratory lead and
   biosafety authority approve or reject the plan. Proof: every approval binds to
   the exact plan version. Failure and recovery: any material change invalidates
   the old approval and returns the plan to review. Export: a non-executable plan
   and approval manifest.
5. **Follow samples and results.** Entry: staff register authorized samples and
   instrument outputs. Progress: custody, processing, quality checks, and missing
   links are visible. Human decision: an analyst accepts, rejects, or repeats a
   measurement. Proof: sample lineage, source files, units, calibration context,
   and analyst actions are immutable. Failure and recovery: orphan data is held for
   reconciliation; deleted drafts enter the recovery bin. Export: a provenance-rich
   result crate.
6. **Review what the evidence supports.** Entry: a team opens a completed study.
   Progress: claims are mapped to supporting and conflicting results. Human
   decision: the principal investigator approves the conclusion and its limits.
   Proof: attestations and evidence hashes travel with the claim. Failure and
   recovery: a challenged claim becomes `under review` without erasing its history.
   Export: an independent review bundle with a redaction manifest.

## 7. Capability map

The capability ID is stable. F0 is foundation, F1 is the useful product, F2 is
the trusted-team product, F3 is advanced category leadership, and F4 is the
frontier network. `First gate` says when it must
first pass fresh checks. Every row inherits this full contract: its invariants are
the product laws and typed truth boundaries; authority is the exact named human and
policy in Sections 10–11; evidence is input/output hashes, versions, actor, decision,
and ledger event; failure is a durable `blocked`, `failed`, `unknown`, or quarantined
state with retry or recovery; and test acceptance includes denial, failure, recovery,
and cross-platform cases in addition to the row's named check. The row names its
domain-specific inputs, output, and strongest acceptance test.

| ID | First gate | Capability contract |
|---|---|---|
| HSB-F0-01 | G0 | **Biological design identity.** Inputs are SBOL, FASTA, GenBank, or manual records; outputs are immutable design versions. Source, alphabet, topology, and access class are required. Ambiguous imports fail into quarantine. Acceptance: round-trip and identity fixtures pass on all desktop systems. |
| HSB-F0-02 | G0 | **Lineage ledger.** Inputs are version, sample, result, and decision events; output is an append-only graph. Acknowledged events have zero loss and idempotent replay. Acceptance: forced-crash and concurrent-writer tests recover one valid history. |
| HSB-F0-03 | G0 | **Policy and risk state.** Inputs are intended use, institutional policy, jurisdiction, and design facts; output is `allowed`, `restricted`, `blocked`, or `unknown`. Only a named human authority may change a block. Acceptance: missing evidence never returns `allowed`. |
| HSB-F1-01 | G1 | **Design workspace.** Inputs are approved components and constraints; output is a versioned design graph and visual map. Meaning-changing edits require preview and confirmation. Acceptance: sequence, feature, and graph views remain equivalent. |
| HSB-F1-02 | G1 | **Safe analysis runner.** Inputs are approved designs, bounded models, seeds, and resource limits; outputs are labeled predictions with uncertainty. It cannot call procurement or laboratory tools. Acceptance: sandbox and capability-denial tests pass. |
| HSB-F1-03 | G1 | **Experiment-plan builder.** Inputs are a question, design version, controls, measures, hazards, and local templates; output is a reviewable plan, never executable device commands. Acceptance: material edits invalidate approvals. |
| HSB-F2-01 | G2 | **Sample and material lineage.** Inputs are authorized identifiers and custody events; output is a versioned lineage graph. Sensitive identifiers are compartmented. Acceptance: no child result can exist without a valid parent or explicit reconciliation state. |
| HSB-F2-02 | G2 | **Measurement ingestion.** Inputs are instrument files and method context; outputs are immutable raw objects plus normalized observations. Unit or schema loss is explicit. Acceptance: malformed and duplicate data cannot become accepted results. |
| HSB-F2-03 | G2 | **Claim-to-evidence map.** Inputs are candidate claims, observations, analyses, and limits; output is a signed support/conflict graph. Agents may draft but not accept claims. Acceptance: every accepted claim has human attestation and at least one traceable source. |
| HSB-F3-01 | G3 | **Institutional review room.** Inputs are risk packets and policy changes; output is a time-bound decision with conditions. Reviewer conflicts are visible. Acceptance: expired or superseded approval blocks downstream release. |
| HSB-F3-02 | G3 | **Design-build-test-learn graph.** Inputs are approved cycle records; output is a cross-cycle learning graph that preserves failures. It cannot infer causality from correlation alone. Acceptance: every recommendation shows the evidence and uncertainty path. |
| HSB-F3-03 | G3 | **Reproducible study crate.** Inputs are allowed designs, plans, environments, data, analysis, and attestations; output is a portable, redacted bundle. Acceptance: a clean machine validates hashes and recreates allowed analysis. |
| HSB-F3-04 | G3 | **Bounded next-experiment advisor.** Inputs are approved search spaces and goals; output is ranked non-executable proposals. High-consequence or policy-unknown work is excluded. Acceptance: an agent cannot widen the search space or bypass human review. |
| HSB-F3-05 | G3 | **Privacy-preserving multi-site learning.** Inputs are institution-approved summaries; output is a model update with contribution proof. Raw sensitive records stay in custody. Acceptance: withdrawal removes future use and produces a signed revocation event. |
| HSB-F3-06 | G3 | **Independent safety case generator.** Inputs are hazards, controls, tests, incidents, and decisions; output is a living safety argument with open gaps. It never certifies itself. Acceptance: an independent human review is required before the case can be marked accepted. |
| HSB-F4-01 | G4 | **Sovereign biological evidence federation.** **Input:** institution-approved, redacted claim and evidence manifests with purpose, access, and expiry policy. **Output:** a revocable cross-site support, conflict, and reproducibility graph. **Invariant:** raw sequences, sensitive sample or material records, credentials, and custody remain local; there is no design, procurement, laboratory, or device actuation path. **Authority:** each institution's named human biosecurity and data stewards approve its binding, disclosures, and withdrawal; no global administrator may override them. **Evidence:** signed manifests, policy versions, queries, contribution hashes, denials, expiry, and revocation events. **Failure:** stale, conflicting, withdrawn, or policy-unknown contributions are quarantined and stop future use. **Acceptance:** five independent nodes join, query, withdraw, and rejoin across 10,000 policy cases with zero raw-record disclosure or unauthorized effect. |
| HSB-F4-02 | G4 | **Federated reproducibility challenge network.** **Input:** approved reproducible study crates, declared analysis questions, bounded compute, and site-run validation requests. **Output:** comparable verification results with method, environment, uncertainty, and disagreement. **Invariant:** only approved computational analysis runs inside each site's controlled sandbox; no wet-lab procedure, instrument command, sample action, or automatic scientific conclusion is created. **Authority:** the local custodian approves every run and disclosure, and independent human reviewers interpret the result. **Evidence:** crate and environment hashes, approvals, run logs, result digests, redactions, and reviewer attestations. **Failure:** an unavailable site, incompatible method, or failed check returns partial or unknown and cannot be counted as confirmation. **Acceptance:** 100 approved cross-site reruns on Windows, macOS, and Linux reproduce declared results or expose a signed mismatch with zero custody escape. |
| HSB-F4-03 | G4 | **Human-governed research portfolio forum.** **Input:** approved non-executable proposals, safety cases, evidence strength, resource ranges, institutional limits, and conflicts. **Output:** a transparent portfolio of options, dependencies, disagreements, and open risks. **Invariant:** the system cannot select, fund, procure, synthesize, schedule laboratory work, or treat ranking as scientific approval. **Authority:** every participating institution's named review body accepts only its own commitments, and a separate human forum records shared decisions. **Evidence:** proposal versions, scoring rules, dissent, conflicts, votes, conditions, and revocations. **Failure:** missing authority, unsafe scope, policy conflict, or stale safety evidence blocks the affected option without blocking unrelated review. **Acceptance:** 1,000 simulated portfolio changes preserve every local veto, expose all material conflicts, and create zero executable biological or procurement action. |

## 8. Domain model

| Record | Ownership, lifecycle, and relationships |
|---|---|
| `ResearchProgram` | Institution-owned container for purpose, authority, jurisdiction, access class, and retention policy. |
| `BiologicalDesign` / `DesignVersion` | Stable identity with immutable versions; links components, sequences, intended function, source, and risk state. |
| `RiskCase` / `RiskDecision` | Versioned hazards, policy checks, missing facts, reviewer identity, conditions, expiry, and decision. |
| `StudyQuestion` / `ExperimentPlanVersion` | Connects an approved design to controls, measures, analysis plan, and required approvals. |
| `MaterialLot` / `Sample` / `CustodyEvent` | Compartmented lineage; no raw sensitive name is copied into general audit text. |
| `InstrumentRun` / `RawObject` / `Observation` | Immutable source object and normalized value with method, unit, quality, and uncertainty. |
| `AnalysisRun` / `ModelSnapshot` | Reproducible environment, inputs, parameters, seed, outputs, failures, and limitations. |
| `Claim` / `EvidenceLink` / `Attestation` | Versioned statement linked to supporting, conflicting, and missing evidence plus human decisions. |
| `RecoveryItem` / `LegalHold` | Soft-deleted object, purge date, restore history, hold reason, and lawful authority. |

## 9. System architecture

- A Rust domain kernel validates design identity, lineage, state transitions, and
  authority before writes.
- PostgreSQL stores durable metadata and row-level access rules; an object store
  keeps large encrypted source files by content hash.
- Sandboxed workers run allowed parsers and models with no procurement, equipment,
  or unrestricted network capability.
- A policy service combines institutional rules, jurisdiction adapters, risk state,
  and human approvals. `Unknown` is a first-class result.
- A user-owned capability broker grants narrow, time-bound access to approved data
  and services; secret values never enter agent context.
- HelixCore supplies shared identity, policy, audit, capabilities, jobs, objects,
  billing, and operations behind domain interfaces; SynthBio retains domain truth.
- The event flow is request, authority check, domain validation, atomic record plus
  event, projection, notification, and proof. Background jobs use durable HelixCore
  jobs, idempotent checkpoints, visible progress, and explicit cancellation.
- Offline authoring, review, recovery, and verification use a local encrypted store.
  Versioned parser, model, instrument-file, and proof adapters are contract-tested
  extension points; none may add physical action.
- Aether is the preferred proof service. A local signature and verification module
  is the offline fallback.
- The web and desktop clients use one domain API and one state language. No hidden
  browser-only safety logic is allowed.

## 10. Agent and automation contract

| Role | May do | Must not do |
|---|---|---|
| Design assistant | Explain records, draft annotations, compare approved designs | Create hidden sequence changes, release designs, or claim safety |
| Analysis planner | Propose bounded models, controls, and checks | Expand the approved search space or hide failed runs |
| Risk clerk | Gather policy facts and draft a risk packet | Approve, downgrade, or close a risk case |
| Study coordinator | Draft plans and identify missing readiness items | Start physical work, order material, or operate equipment |
| Evidence reviewer | Map claims to evidence and flag conflicts | Accept conclusions or remove inconvenient results |

Every agent has an exact project, purpose, data-class, tool, time, and resource
lease. It streams stage, elapsed time, last signal, open uncertainty, and waiting
authority. A human may pause, revoke, or cancel it. Cancellation records what
stopped and what could not be recalled. Every draft enters named human review;
schema, domain, policy, and evidence checks validate the result. Reversal restores
the prior version or revokes the grant without rewriting history.

## 11. Trust, safety, and privacy

| Safety case | Prevention, human authority, proof, and recovery |
|---|---|
| Harmful or dual-use design support | Risk screen before analysis; sensitive content isolation; no sequence-to-order or equipment tools; biosafety authority can block. Proof records policy and decision, while restricted details stay redacted. |
| AI treats prediction as fact | Predictions use a distinct type, show model scope and uncertainty, and cannot become observations. A scientist accepts interpretations. |
| Unapproved physical work | Plans are non-executable. Material, equipment, and procurement connectors are outside the agent boundary and require separate approved systems. |
| Sensitive sequence disclosure | Least privilege, local custody, field-level encryption, redacted notifications, export review, and signed access events. |
| Sample or result mix-up | Stable identifiers, custody checks, duplicate detection, and reconciliation quarantine. A human resolves identity conflicts. |
| Unsafe deletion | Delete moves work to a 30-day recovery bin where lawful. Legal hold, biosafety incident, regulated retention, or active investigation blocks purge. Immediate access quarantine may hide dangerous material without destroying evidence. Permanent deletion requires re-authentication, named authority, impact preview, and signed proof. |

Threat models, safety cases, misuse tests, privacy reviews, and incident drills are
release gates. The product never labels itself compliant or safe from an automated
check alone. Tenant separation is enforced in the database and object layer. Data
is encrypted in transit and at rest, residency is a deployment policy, and an
incident can quarantine access, revoke leases, preserve evidence, and start reviewed
recovery without exposing restricted content.

## 12. Proof and audit

Each important event records actor, authority, design or plan version, source hashes,
policy version, tool version, time, result, uncertainty, and prior event. The proof
bundle states what was asked, what ran, what changed, what failed, what a human
decided, and what remains unknown. Secret values and restricted sequence content do
not enter metadata-only audit events. Aether is preferred for signed proof; an
offline verifier and local signed bundle are mandatory fallbacks. Proof does not
show that a biological system will behave as predicted or that an institution has
met every law.

## 13. UX system

The primary surfaces are Home, Designs, Risk Review, Studies, Samples, Results,
Claims, Evidence, and Recovery. The default view shows identity, current state,
next safe action, last signal, and waiting human decision. Expert detail unfolds
progressively. Long analysis shows queue, current stage, elapsed time, last useful
signal, resource use, and a truthful unknown estimate. Completion, failure, blocked
state, and approval requests create in-app notifications and optional desktop
notices. Moving or replacing a selected design item shows the affected relationships
before confirmation. Destructive actions show impact, recovery date, and legal-hold
status. Restricted content never appears in notification previews. Reversible edits
offer undo; empty states explain the first safe action; plain-language errors state
what happened, what remains safe, and how to recover. Keyboard and touch paths have
the same authority checks.

## 14. Interoperability and standards

All links below were verified from the official body on 2026-07-15.

- [SBOL 3.1.0](https://sbolstandard.org/datamodel-specification/) is the preferred
  exchange model for biological designs. Loss caveat: local risk, custody, approval,
  and retention records remain Helix extensions and must travel in the proof bundle.
- [NCBI FASTA](https://www.ncbi.nlm.nih.gov/genbank/fastaformat) and
  [GenBank](https://www.ncbi.nlm.nih.gov/genbank/) are import and export adapters for
  sequence records. Loss caveat: FASTA carries little feature or provenance detail,
  and imported database records are evidence sources, not verified design intent.
- The [WHO Laboratory Biosafety Manual, fourth edition](https://www.who.int/publications/i/item/9789240011311)
  guides the risk-based biosafety case. Loss caveat: local law, organism, facility,
  and activity determine the real controls; the software cannot assign containment.
- The [WHO Laboratory Biosecurity Guidance](https://www.who.int/publications/i/item/9789240095113)
  guides protection of high-consequence material, technology, and information.
  Loss caveat: institutional and national oversight cannot be replaced by a screen.

Standards are versioned adapters. Import reports unsupported fields and never drops
meaning silently.

## 15. Cross-platform contract

Windows, macOS, and Linux must pass the same design, risk-state, lineage, encryption,
crash-recovery, export, and accessibility fixtures. The web client must pass the same
six journeys in two current browser engines. Offline mode supports design review,
approved local analysis, and evidence verification. GPU work is optional; a correct
CPU path remains available. Managed deployment may add scale but cannot weaken local
custody, authority, or export. Install, upgrade, backup, restore, and uninstall are
tested on every desktop platform with no production data paths used by CI. The CLI
and container surfaces support administration, import/export, and fresh checks only;
they expose no hidden physical-action route. Every optional platform feature uses
capability detection and a safe fallback.

## 16. Reliability and performance budgets

- Acknowledged lineage and decision writes have RPO 0 under forced termination and
  concurrent writers, measured in every release build.
- During each calendar month, 99.95% of authorized metadata reads complete without
  server error; maintenance windows are reported separately.
- A workspace with 50,000 design nodes opens to a useful summary in p95 under 2
  seconds on the reference desktop; common edits confirm in p95 under 150 ms.
- A running analysis emits a meaningful signal at least every 10 seconds or changes
  to `no recent signal` after 30 seconds.
- Local cancellation is acknowledged in 2 seconds and reaches a safe worker
  checkpoint in 30 seconds; a remote job stays `cancel requested` until confirmed.
- Metadata recovery has RTO 30 minutes; encrypted object recovery has RTO 4 hours in
  the supported self-hosted profile, tested quarterly.
- An import of 100,000 records may fail partially only through an explicit quarantine
  manifest; accepted records and rejects must sum to the input count.
- Create, import, and analysis requests use idempotency keys retained for at least 24
  hours; a duplicate returns the original durable result.
- Offline mode does not fetch external databases or submit remote work. Unsynced work
  stays visibly local. If an optional model or source fails, authoring, review,
  recovery, and export remain available in a named degraded state.

## 17. Success measures

Measure time from design import to a reviewed risk state, percent of results with
complete lineage, unsafe actions blocked before execution, protocol changes caught
before approval, failed and negative results retained, independent bundle validation,
accessible journey completion, cross-platform export success, recovery drill success, and time for a reviewer to
find what a claim does not prove. Do not use design count or agent activity as a
success measure. Business measures are renewal after a verified study journey,
support burden per active institution, and cost per independently validated bundle.

## 18. Delivery plan

- **G0 — Truthful foundation (0–6 months):** freshly prove the shared service startup; replace generic
  records with design identity, risk state, and lineage ledger; add crash recovery,
  temporary-state tests, three-platform CI, and truthful capability reporting.
- **G1 — Useful single-player product (6–18 months):** ship design workspace, safe local analysis, plan builder,
  risk-review journey, accessibility, notifications, and 30-day recovery.
- **G2 — Trusted team product (18–30 months):** add sample lineage, measurement ingestion, claim maps,
  approved instrument-file adapters, and independently verifiable study crates.
- **G3 — Category leader (30–42 months):** add institutional review rooms, cross-cycle learning,
  bounded next-experiment advice, privacy-preserving multi-site learning, living
  safety cases, stronger compartmentation, and independent biosecurity and privacy review.
- **G4 — Frontier network (42–60 months):** ship HSB-F4-01 sovereign evidence federation,
  HSB-F4-02 reproducibility challenges, and HSB-F4-03 the human-governed portfolio
  forum only after formal safety, misuse, and founder gates. Fresh G4 proof requires
  five independently governed nodes to join and revoke cleanly, 100 approved
  cross-site reruns, 10,000 policy-boundary cases, 1,000 portfolio changes, zero raw
  custody escape, and zero biological, laboratory, device, or procurement actuation.

Every gate runs fresh Rust and web builds, unit and integration tests, domain
fixtures, the six end-to-end journeys, accessibility, authorization, recovery,
migration, redaction, security, and Windows/macOS/Linux packaging checks. Cached or
prewritten status cannot satisfy a gate.

## 19. Current truth and gap

The live source is a generated scaffold. Its backend exposes generic `designs` and
`sims` create/list/get records backed by generic title, body, status, and metadata
fields. It registers an assistant with only echo and product-catalog tools. The web
folder contains `package.json` but no product UI. There are no SynthBio domain tests,
risk engine, biological design model, lineage model, analysis engine, or safety
case. The live backend now applies route state and calls the shared graceful-shutdown
server helper; the earlier startup defect is repaired in source, but this spec-only
pass did not run a fresh build. Existing build artifacts do not prove current source.
The first honest slice is HSB-F0-01 through HSB-F0-03 plus fresh build and CI proof.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Internal truth | Versioned domain graph, not generic JSON blobs | Architecture decision and migration proof |
| First useful mode | Local design, review, and bounded analysis | Product decision |
| Physical action | Outside SynthBio and unavailable to agents | Founder, institutional, legal, and safety approval |
| Risk result | `unknown` is distinct from `allowed` | Safety review |
| Human authority | Scientist plus named institutional biosafety authority | Governance decision |
| Sensitive data | Local custody and compartmented access | Privacy and security review |
| Delete | 30-day recovery where lawful; holds override purge | Legal-retention decision |
| Proof | Aether preferred, offline signed bundle required | Architecture decision |
| Standards | Versioned adapters with explicit loss reports | Interoperability decision |

## 21. Definition of category-defining done

- [ ] A scientist can move from design identity to a reviewed study without hidden
  state or a cloud dependency.
- [ ] Risk, prediction, observation, and conclusion remain visibly different truths.
- [ ] No agent can order, release, execute, approve, or hide biological work.
- [ ] Every accepted result has complete design, sample, method, unit, and decision
  provenance.
- [ ] Negative and conflicting evidence travels with every claim.
- [ ] A redacted bundle validates on an independent machine and states its limits.
- [ ] Restricted content never leaks through search, logs, notification, proof, or
  portfolio memory.
- [ ] Windows, macOS, Linux, web, offline, recovery, accessibility, and safety gates
  pass from fresh source.
- [ ] Independent biosafety, privacy, and domain reviewers accept the safety case.
