# HelixVitaPrime — longitudinal precision-health research system

```yaml
product: HelixVitaPrime
catalog_order: 18
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [participants, research teams, data stewards, clinicians in research roles]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixVitaPrime is a participant-controlled research system that turns years of
health, behaviour, laboratory, device, and genomic data into reproducible
questions without turning uncertain research into medical advice.

## 2. Five-year destination

The useful product is a consent-led longitudinal study workspace with data
import, quality checks, cohorts, protocols, analysis, participant feedback, and
portable research bundles. The category advantage is a living personal and
population timeline where every result keeps its source, consent, transformation,
uncertainty, and limits. The frontier is privacy-preserving research across many
institutions without pooling raw participant data. Human researchers approve
protocols and claims. Licensed clinicians remain responsible for clinical care.
A cohort means a defined group of participants.

## 3. Users and hard jobs

- Participants need control, understandable consent, and useful feedback. They
  fear hidden reuse and results presented without context.
- Researchers need clean longitudinal data and fear bias, leakage, and results
  that cannot be repeated.
- Data stewards need enforceable purpose and retention rules and fear a grant
  that silently grows.
- Clinician-researchers need context and fear research output entering care as
  an unreviewed recommendation.
- Review boards need a full protocol and change history and fear unreported
  drift from the approved study.

## 4. Product laws

1. Consent is versioned, specific, revocable, and checked at every use.
2. Withdrawal stops future use; prior lawful results are handled by the recorded
   protocol and jurisdiction policy, not silently deleted or retained.
3. Research output is never labeled diagnosis or treatment advice.
4. Source data and transformed data remain linked but separate.
5. Missingness, cohort changes, exclusions, and failed analyses stay visible.
6. A model cannot train on a dataset without an exact approved purpose lease.
7. Participants can see access, use, sharing, derived data, and revocation state.
8. Export includes consent and provenance, not only measurements.

## 5. Scope boundaries

VitaPrime owns longitudinal research records, consent, study design, cohort
logic, research analysis, biomarker and genomic interpretation evidence, and
participant research feedback. CuraPrime owns active clinical care. Well owns
private daily wellbeing. NovaLabs owns general scientific programs. VitaPrime
does not prescribe, diagnose, promise life extension, or replace ethics review.

## 6. Signature experiences

| Journey | Entry point | Visible progress | Human decision | Completion proof | Failure and recovery | Export or portability |
|---|---|---|---|---|---|---|
| Participant enrolment | Accept a study invitation or open local enrolment. | Show consent section, understanding check, selected source import, and data-map status. | The participant chooses each optional use, source, sharing limit, and whether to sign. | Save the exact consent version, answers, signature, source receipts, and data map. | A failed import does not change consent; the participant can retry, remove a source, or restore a draft from the 30-day bin. | Export a plain-language consent receipt and machine-readable data map. |
| Longitudinal timeline | Open My Data or an approved research view. | Show indexing, source quality, missing periods, transformations, conflicts, and last refresh. | The viewer chooses sources and filters; only an authorized steward accepts a correction. | Save the query, source versions, filters, and visible quality or uncertainty report. | Missing or conflicting data stays visible; the last valid timeline remains available during repair. | Export the selected timeline with units, source links, consent, and provenance. |
| Cohort design | Open an approved study and a draft participant-group definition. | Show rule validation, estimated count, missing data, bias warnings, and approval state. | A researcher proposes rules; a data steward approves access and freezes the version. | Store the exact criteria, query hash, count, exclusions, approval, and frozen snapshot. | Invalid rules or a group below the allowed size block use; the draft can be fixed or restored. | Export the criteria, version, and summary without raw participant rows. |
| Reproducible analysis | Start from a frozen plan, approved data snapshot, and sandbox lease. | Show stage, records read, exclusions, resource use, spend, checks, and last durable checkpoint. | The researcher approves the run and later decides whether the result may enter review. | Record dataset, consent and protocol versions, code, environment, outputs, checks, and review state. | Cancel stops new reads and local compute; a crash resumes from a safe checkpoint and cannot release a partial result. | Export a signed research bundle that can rerun against an allowed equivalent dataset. |
| Participant result | Open a reviewed research result prepared for release. | Show plain-language drafting, clinical-boundary check, accessibility check, reviewer state, and delivery. | A named reviewer approves wording; the participant chooses whether to view, save, or share it. | Store the reviewed communication version, audience, delivery receipt, evidence, and limits. | Failed review keeps the result private; a corrected message is a new version and the prior approved version can be restored. | Export an accessible result with its evidence guide and research-only label. |
| Revoke or withdraw | Open Consent or Uses and select a grant or study. | Show affected future uses, data copies, derived records, remote sites, acknowledgments, and pending actions. | The participant confirms the exact withdrawal or revocation effects. | Create a signed event and a report of every binding removed, retained by policy, or still pending. | Remote silence remains pending and retries safely; local authority changes immediately and never waits for a remote site. | Export the withdrawal receipt, policy explanation, and site acknowledgment list. |
| Federated study | Open an approved multi-site protocol and site list. | Show site approval, dispatch, local checks, records included, denials, aggregate return, and privacy review. | Each site owner approves its local grant; the lead researcher approves the final combined analysis. | Store the signed protocol, site receipts, local run proofs, aggregate hashes, privacy checks, and review. | A denied or offline site stays explicit; the system never fills its result with guessed or raw data. | Export the aggregate package and site proofs without exporting row-level records. |

## 7. Capability map

| ID | Gate | Input | Output | Invariant | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| VP-F0-01 | G0 | Registration facts and source identity bindings. | Stable participant, study, sample, and observation IDs. | A path, name, device, or provider ID is never the logical identity. | A participant or approved registrar creates a binding; agents may only propose matches. | Creator, binding source, confidence, decision, old/new IDs, and time. | A collision or uncertain match is quarantined and never silently merged. | Move, rename, duplicate-name, and provider-change fixtures keep the same logical IDs. |
| VP-F0-02 | G0 | Consent terms, purpose, actor, resource, time, and requested action. | Allow or deny decision plus a narrow use lease. | Every read, compute, export, and model use requires one active matching permission. | The participant grants or revokes; a steward enforces but cannot widen the grant. | Consent version, understanding check, purpose, decision, lease, and revocation history. | Missing, expired, conflicting, or withdrawn consent fails closed with a plain reason. | Exhaustive policy tests deny every request missing one required consent dimension. |
| VP-F0-03 | G0 | Raw source values, units, source time, mappings, and transformation code. | Preserved raw record, normalized observation, quality flags, and full lineage. | Raw input never changes and no mapping loss is silent. | A curator proposes a map; an approved steward accepts it. | Source hash, mapping version, code hash, inputs, outputs, units, and findings. | Unknown code or unit is quarantined and remains visible for repair. | Round-trip fixtures preserve every source value and report every seeded unmapped field. |
| VP-F1-01 | G1 | Approved observations, transformations, source quality, and viewer policy. | Longitudinal timeline with missing, derived, and conflicting values marked. | Measured, reported, corrected, and derived values stay distinct. | Participants see their data; researchers see only the approved study view. | Query, source versions, filters, quality flags, uncertainty, and access event. | A failed source refresh leaves the last valid view and names stale data. | A ten-year, one-million-observation fixture loads with every seeded gap and conflict visible. |
| VP-F1-02 | G1 | Study question, objectives, outcomes, eligibility, analysis plan, and power assumptions. | Versioned protocol ready for review or frozen execution. | No analysis runs without one complete approved frozen protocol. | Researchers draft; ethics and accountable study roles approve. | Protocol diff, reviewers, decisions, rationale, signature, and freeze hash. | Missing fields, approval, or changed source data block execution. | Protocol fixtures reject every missing required field and every post-approval silent edit. |
| VP-F1-03 | G1 | Participant identity, active studies, consents, use events, messages, and results. | Portal views and signed participant actions. | The portal shows every known use and never turns pending remote work into completed work. | The participant controls consent, export, withdrawal, and optional sharing. | Action request, exact effect preview, signature, delivery, and recovery event. | An unavailable service shows pending or unavailable and preserves the draft. | All seven participant journeys pass with tenant isolation and no hidden data use. |
| VP-F2-01 | G2 | Approved protocol and computable participant-group criteria. | Count preview, bias and privacy warnings, frozen group snapshot, and exclusions. | Criteria, exclusions, missing data, and a minimum allowed group size stay visible. | A researcher drafts; a steward approves data use and snapshot creation. | Criteria version, query hash, count, exclusions, warnings, approval, and snapshot hash. | Invalid criteria or a group below policy size blocks row access and analysis. | Seeded group fixtures reproduce exact membership and catch every hidden exclusion mutation. |
| VP-F2-02 | G2 | Frozen plan, approved data snapshot, exact lease, code, and environment. | Sandboxed run, checkpoints, results, and signed proof. | The run reads and writes only approved data, tools, time, compute, network, and output paths. | A human approves the lease; the Analysis Agent executes but cannot widen it. | Dataset hash, consent and protocol versions, image, code, resources, outputs, and checks. | Timeout, lease breach, or cancellation makes the result not releasable. | A clean sandbox reruns every reference analysis and blocks each seeded lease escape. |
| VP-F2-03 | G2 | Collection, custody, processing, assay, quality, and disposal events. | Complete sample and assay lineage with current state. | A sample event appends; it never overwrites prior custody or measurement facts. | Trained staff record events; policy controls disposal and correction. | Actor, time, location class, sample ID, method, calibration, result, and signature. | A missing custody step or calibration quarantines the result from analysis. | Reference samples trace from collection to disposal and every broken-link fixture is blocked. |
| VP-F3-01 | G3 | Approved genomic artifact and versioned knowledge sources. | Evidence-graded research interpretation with conflicts and limits. | Output is research evidence, never diagnosis, treatment, or an unqualified fact. | A qualified human reviewer approves release; agents may only draft. | Artifact hash, source versions, evidence grades, conflicts, review, and expiry. | Stale, conflicting, or unsupported evidence is flagged and cannot become a released claim. | Seeded interpretation fixtures preserve every conflict and block all unsupported claims. |
| VP-F3-02 | G3 | Approved datasets, features, model code, population, and intended use. | Model version, evaluation, calibration, drift, and group-level performance report. | No model releases without population, limits, calibration, and subgroup evidence. | Model governance approves use; agents cannot promote a model. | Training snapshot, code, features, metrics, group results, reviews, and release decision. | Missing or failed evidence keeps the model in draft or retired state. | Mutation tests removing intended use, calibration, or subgroup results always fail release. |
| VP-F3-03 | G3 | Reviewed result, evidence, uncertainty, audience, and communication policy. | Plain-language participant message and clinical-boundary review. | Observation, research inference, uncertainty, and clinical follow-up advice stay separate. | A named reviewer approves wording; the participant chooses receipt and sharing. | Draft versions, source result, reviewers, accessibility check, delivery, and retraction state. | Missing evidence or review blocks delivery; correction creates a new version. | Communication fixtures detect every missing research label, limit, or review decision. |
| VP-F4-01 | G4 | Signed protocol, site grants, local data rules, task image, and return schema. | Privacy-checked aggregates and per-site execution proof. | Raw participant records stay at each site and small groups are not disclosed. | Every site owner approves locally; the lead researcher approves combined analysis. | Site grant, task hash, local checks, count, privacy report, return hash, and denial. | A denied or offline site remains explicit and cannot be replaced with guessed data. | A three-site synthetic study completes with one denial and exposes no row-level record. |
| VP-F4-02 | G4 | Pre-registered one-person hypothesis, schedule, observations, and safety limits. | One-person (N-of-1) research report with adherence, uncertainty, and deviations. | The plan includes any period without the tested exposure, called washout, and never issues a treatment order. | Participant and researcher approve; a clinician reviews when the protocol requires care judgment. | Pre-registration, schedule, adherence, observations, deviations, statistics, and reviews. | Safety concern, poor adherence, or protocol drift pauses the study and blocks a firm claim. | A synthetic crossover fixture reports every planned period, deviation, and uncertainty correctly. |
| VP-F4-03 | G4 | Approved longitudinal features, model versions, and research question. | Testable systems hypotheses with source links and uncertainty. | No hidden health score, diagnosis, or treatment instruction is produced. | Researchers approve tests; participant and ethics policy control data use. | Inputs, model, reasoning trace, alternatives, uncertainty, review, and proposed test. | Weak or conflicting evidence returns an open question, not a recommendation. | Seeded data with a known confounder produces a warning and no causal claim. |

## 8. Domain model

| Record group | Owner | Creation and change | Version rule | Retention and delete | Main relationships |
|---|---|---|---|---|---|
| `Participant`, `IdentityBinding` | The participant under stable HelixCore identity policy. | Registration creates the participant; approved matching adds or removes source bindings. | Logical participant ID never changes; each binding has its own revision and decision. | Identity follows participant and legal policy; deleting one binding never deletes shared evidence or another participant. | Bindings connect one participant to source accounts, studies, samples, consent, and observations. |
| `Consent`, `ConsentTerm`, `Purpose`, `DataUseGrant`, `Withdrawal` | The participant owns choices; the study is accountable for enforcement. | Consent is signed from exact terms; grants derive from it; withdrawal or revocation appends a new event. | Signed versions are immutable; later choices supersede but never rewrite earlier ones. | Keep for the legal and ethics period; ordinary drafts use the 30-day bin; final deletion follows policy and proof. | A grant links participant, consent version, purpose, study, resources, actions, time, and outputs; withdrawal lists every affected grant. |
| `Study`, `ProtocolVersion`, `Site`, `CohortDefinition`, `CohortSnapshot` | Accountable study organization and named principal researcher. | Study and site membership are approved; protocol and criteria changes create reviewable versions; snapshots freeze membership. | Approved protocol and cohort snapshots are immutable and content-addressed. | Retain through the study and required archive; unapproved drafts use the 30-day bin. | Protocol governs sites, purposes, criteria, analysis plans, communications, and retention. |
| `SourceRecord`, `Observation`, `Transformation`, `QualityFinding` | Participant or approved study custodian, as policy states. | Raw import creates source records; normalization creates linked observations; fixes and transforms append new versions. | Raw source never changes; measurements require unit and source time; derived values name code and inputs. | Source policy controls retention; removal or withdrawal creates a visible state and never silently breaks released proof. | Observations link participant, source, transformation, quality findings, consent, and later analysis. |
| `Specimen`, `Assay`, `GenomicArtifact` | Approved study or biobank custodian; the participant retains policy rights. | Collection, custody, processing, measurement, transfer, and disposal append events. | Every event and artifact is immutable; corrections are linked new records. | Follow consent, biospecimen policy, law, and legal hold; disposal does not erase required lineage. | Specimens connect participant and protocol to assays; assays create observations and genomic artifacts. |
| `AnalysisPlan`, `AnalysisRun`, `ModelVersion`, `Result`, `Claim`, `Communication` | Study owner; accountable humans own release decisions. | Plans freeze before a run; runs append; model, result, claim, and communication changes create versions. | Released records are immutable and point to exact inputs, code, environment, review, and limits. | Keep with the approved study and result policy; retraction changes state but preserves the old evidence. | Runs use cohort snapshots and plans; results support or limit claims; communications target approved audiences. |
| `ProofBundle` | Exporting participant or study, signed by named actors. | Created only after referenced records are durable; verification adds a report. | Manifest and artifacts are content-addressed; a correction creates a new bundle. | Retain for the declared research and legal period; deletion or legal hold is separately authorized and evidenced. | Bundle links identity, consent, protocol, data snapshot, code, run, result, review, communication, withdrawal effects, and limits. |

## 9. System architecture

- A consent-policy service decides access before data location is revealed.
- A versioned ingestion and mapping pipeline preserves raw and normalized forms.
- Columnar research stores may accelerate approved analysis; PostgreSQL remains
  the control record.
- Sandboxed compute receives short data-use leases and writes only to an
  approved result area.
- A privacy gateway handles export review, minimum group sizes, and federated
  responses according to study policy.
- HelixCore supplies identity, capabilities, audit, jobs, objects, and recovery.
- CuraPrime integration uses explicit clinical/research boundaries and review.

## 10. Agent and automation contract

| Role | May read | Tools | May draft | Approval required | Forbidden | Progress and checking | Stop or reversal |
|---|---|---|---|---|---|---|---|
| Data Curator | Approved source records, mapping dictionaries, units, and quality rules. | Mapping sandbox, unit checker, duplicate finder, and lineage viewer. | Mappings, corrections, and quality findings. | A data steward approves mappings that change normalized research data. | Erase raw values, invent a unit, widen purpose, or hide an unmapped field. | Show source, records checked, mapped, quarantined, warnings, and loss; run round-trip and unit tests. | Stop the import, preserve raw data, remove the draft mapping, and restore the last approved version. |
| Cohort Assistant | Approved protocol, permitted fields, group rules, and summary statistics. | Criteria builder, count preview, bias checks, and privacy-size check. | Group criteria, exclusions, and count estimates. | A steward approves row access and freezes the group snapshot. | Approve access, reveal small groups or rows, add hidden exclusions, or change consent. | Show rules evaluated, estimated count range, missing data, exclusions, bias, and approval; reproduce the query hash. | Cancel preview, delete its temporary result, and restore the last frozen definition. |
| Analysis Agent | Frozen plan, approved snapshot, exact lease, and named tools. | Sandboxed compute, checkpoint, approved package store, and result validator. | Analysis outputs and a report inside the approved result area. | A human approves the lease and later approves result release. | Read outside the snapshot, contact an undeclared network target, widen the lease, diagnose, prescribe, or export rows. | Show phase, records read, exclusions, resource use, spend, warnings, and checkpoint; rerun required checks. | Stop new reads within 2 seconds, terminate owned compute within 10 seconds, and mark partial output not releasable. |
| Evidence Reviewer | Protocol, consent decision, snapshot, code, outputs, methods, claims, and limits. | Provenance viewer, statistics checks, bundle verifier, and review comments. | Findings, alternative explanations, and bounded claim wording. | A separate accountable human resolves critical findings and approves release. | Contact participants, rewrite source evidence, hide failed analyses, or approve its own authored claim. | Show items checked, failed checks, open findings, and coverage; verify in a clean environment. | Withdraw review approval, reopen findings, and restore the last reviewed claim state. |
| Participant Guide | Public study information and the participant's approved consent screen and choices. | Plain-language glossary, understanding check, accessibility tools, and data-map viewer. | Explanations and questions for the participant. | The participant alone makes consent, sharing, export, and withdrawal choices. | Steer consent, predict health, diagnose, prescribe, rank the participant, or access hidden research records. | Show the current consent section and unanswered questions; test key understanding without coercion. | Stop guidance, clear unsaved suggestions, and return to the unchanged consent choice. |

No agent may re-identify protected records or export row-level data.

## 11. Trust, safety, and privacy

Health and genomic data use the highest sensitivity class. Access is per study,
purpose, role, participant policy, time, and output. Keys are brokered; agents
never retrieve them. Re-identification and small-group disclosure tests run
before export. The 30-day bin applies to ordinary drafts and local copies.
Participant withdrawal, source deletion, research retention, and legal hold use
separate policy flows because not every record may lawfully follow one timer.
Every exception is visible and signed.

## 12. Proof and audit

Evidence covers consent version, protocol approval, dataset snapshot, source
quality, transformations, software environment, analysis plan, actual code,
exclusions, outputs, statistical checks, reviews, communication wording, and
withdrawal effects. Proof does not make an observational association causal or
a research result clinically valid. Aether is preferred for external proof and
capability leases; a local signed evidence package remains available.

## 13. UX system

Participants see Home, My Data, Studies, Consent, Uses, Results, Messages,
Exports, and Recovery. Researchers see Protocols, Cohorts, Data Quality,
Analysis, Models, Evidence, Approvals, and Operations. Consent uses a short
summary with optional deeper legal and technical detail. Every chart exposes
population, missingness, units, source, uncertainty, and filters. A running
analysis never uses a fake percentage; it shows completed stages and last
signal. Sensitive exports require a preview of exactly what will leave.

## 14. Interoperability and standards

- [HL7 FHIR R5](https://hl7.org/fhir/) is the clinical exchange adapter,
  verified from HL7 on 2026-07-15.
- [OMOP CDM 5.4](https://ohdsi.github.io/CommonDataModel/) is an observational
  analysis adapter; the official OHDSI page currently recommends 5.4.
- [GA4GH products](https://www.ga4gh.org/our-products/) guide genomic exchange,
  identity, and responsible data use.
- [GA4GH Data Use Ontology](https://www.ga4gh.org/product/data-use-ontology-duo/)
  supplies computable research-use terms.
- [CDISC foundational standards](https://www.cdisc.org/standards/foundational-standards)
  are adapters for clinical-research exchange and submission workflows.

Versioned mapping reports state every loss, local extension, and unmapped code.

## 15. Cross-platform contract

Participant consent, timeline, export, withdrawal, protocol review, and evidence
viewing work in current browsers on Windows, macOS, and Linux. Local research
nodes and compute workers pass the same container and native fixtures on all
three systems. Mobile capture is an adapter, not a requirement for G1. Offline
participant actions queue safely, show pending status, and never pretend that a
revocation reached remote sites before confirmation.

## 16. Reliability and performance budgets

`p95` means at least 95 of 100 measured actions meet the target. Idempotency
means safely repeating one request creates no extra work.

| Area | Numeric objective | Measurement window | Failure or degraded behaviour |
|---|---|---|---|
| Consent data loss | Zero acknowledged consent, grant, revocation, or withdrawal events lost across 1,000 forced-crash points. | Every release candidate. | Recovery uses the last durable decision; uncertain remote effects remain pending. |
| Idempotency | Ten thousand repeats of one signed participant action create exactly one event and one effect. | Every G0+ release candidate. | A changed payload is rejected and linked to the original action. |
| Access trace | 100% of successful reads, compute uses, model uses, and exports link to an active decision and purpose event. | Continuous release gate over all access fixtures. | Missing policy evidence denies the action and creates a visible security event. |
| Timeline latency and scale | p95 below 2 seconds for first view of 10 years and 1 million indexed observations. | Ten-minute test on each published reference machine per release. | The view streams partial ranges with clear gaps and never hides stale sources. |
| Offline use | Consent review, timeline browsing, draft export, recovery, and revocation request work during an eight-hour network-disabled test. | Every G1+ release candidate. | Remote effects show pending; local grants stop immediately where local policy allows. |
| Analysis cancellation | Stop new reads within 2 seconds and terminate the owned local process tree within 10 seconds in at least 99 of 100 trials. | Every G2+ release candidate. | A survivor is named; partial output is quarantined and cannot become a result. |
| Concurrency | A 16-core, 32 GB reference node serves 100 timeline readers and 8 sandboxed analyses without tenant or result mixing. | Thirty-minute load and isolation test per G2+ release. | Admission control queues work before resource exhaustion and shows position. |
| Group scale | Preview a permitted group over 100,000 synthetic participants in p95 below 5 seconds without exposing row data. | Every G2+ release candidate. | The system returns a bounded count range or unavailable state, never raw fallback rows. |
| Recovery | Metadata recovery objective is 15 minutes; a 100 GB encrypted reference store restores and verifies within 4 hours. | Quarterly clean-machine drill on Windows, macOS, and Linux workers. | The prior copy stays read-only until the restored copy validates. |
| Site degradation | A silent federated site is marked unavailable within 30 seconds after its declared heartbeat window. | Fault injection for every G4 release candidate. | Other site results remain separate; the combined study stays incomplete until policy permits release. |
| Release integrity | Zero result releases while provenance, quality, privacy, required review, or communication checks are pending, failed, or unknown. | Every result fixture in every release candidate. | The result remains draft with exact blocking reasons and recovery actions. |

## 17. Success measures

| Outcome | Target | Window and method |
|---|---|---|
| Participant understanding | At least 90% of participants answer every key purpose, sharing, withdrawal, and risk question correctly before signing. | Quarterly study with at least 20 participants or representative test users. |
| Consent honoured | 100% of policy fixtures deny unapproved use; zero confirmed unapproved access in a release period. | Every release suite plus monthly incident review. |
| Withdrawal truth | 100% of local bindings change within 60 seconds; every remote binding shows acknowledged or pending truth. | One hundred synthetic withdrawals per release and quarterly site drill. |
| Mapping loss | 100% of seeded unknown codes, units, extensions, and dropped fields appear in the mapping report. | Every adapter release. |
| Reproducible analysis | At least 95% of supported reference analyses rerun within declared tolerance in a clean environment. | Monthly G2+ matrix across all three operating systems. |
| Quality detection | 100% of seeded duplicate, unit, time, missingness, and impossible-value faults are caught before analysis release. | Every release candidate. |
| Model-group evidence | 100% of releasable models report intended population, calibration, and every policy-required subgroup result. | Model release gate. |
| Time to approved group | Median under 30 minutes from approved protocol to frozen participant-group snapshot for trained researchers. | Quarterly usability study with at least eight researchers or data stewards. |
| Accessibility | All seven journeys complete by keyboard and screen reader with zero blocking WCAG 2.2 AA finding. | Quarterly automated and human review. |
| Portable export | 100% of reference consent, timeline, analysis, withdrawal, and proof exports validate on Windows, macOS, and Linux. | Every release candidate. |
| Real adoption | At least three independent study teams each complete one reviewed synthetic or approved low-risk study before G2 closes. | Quarterly product review with team interviews. |

Data volume, model count, and raw agent-call count are not success measures.

## 18. Delivery plan

| Gate | Build | Tests | Safety and authority | UX proof | Windows, macOS, Linux | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| G0 (0–6 months), truthful foundation | Stable identity, versioned consent and purpose, raw provenance, release gates, and native CI. | Identity move, policy denial, consent crash/idempotency, mapping round-trip, tenant isolation, and release-integrity tests. | Synthetic data only; every data action fails closed without exact consent and purpose. | A participant can review, understand, sign, revoke, recover, and export consent without hidden effects. | Native services and the same identity, consent, crash, and mapping fixtures pass on all three. | Generic studies/cohorts convert through a reversible reported migration; no generic row is called clinical truth. | From a clean machine, an operator registers a synthetic participant, crashes during consent, recovers, and proves the final decision. |
| G1 (6–18 months), useful product | Participant portal, timeline, protocol builder, quality pipeline, and one complete local study. | Seven journey, offline, accessibility, timeline scale, source conflict, protocol, deletion, and export tests. | Research-only labels and clinical boundaries are enforced; agents remain draft-only. | Participants and researchers see sources, missing data, uncertainty, progress, completion, recovery, and export. | Browser and local-node tests pass; signed packages or containers are exercised on all three. | Source and protocol upgrades preserve raw values, consent links, old readable versions, and rollback. | A participant and researcher complete a synthetic study journey and restore a deleted draft without developer help. |
| G2 (18–30 months), trusted team product | Participant-group workbench, sandbox analysis, sample lineage, team review, withdrawals, and two standards adapters. | Criteria mutation, small-group privacy, sandbox escape, cancellation, concurrent review, lineage, restore, and adapter round-trip tests. | Exact data-use leases, row-export denial, separation of duties, and signed withdrawal effects are mandatory. | Export and analysis previews show exact data, purpose, audience, limits, stage, safe stop, and proof. | Sandboxes, workers, adapters, bundle verification, and clean restore pass on all three. | Consent, group, analysis, and adapter schemas migrate with rollback; old approvals gain no new power. | An operator runs, cancels, and reproduces a synthetic analysis, then completes a withdrawal drill across two sites. |
| G3 (30–42 months), category leader | Genomic research interpretation, governed models, participant communication review, and multi-site pilots. | Stale-source, unsupported-claim, calibration, subgroup, drift, communication, privacy, and clean-room reproduction tests. | No diagnosis or treatment order; independent privacy, security, clinical-boundary, and scientific review resolves all P0/P1 findings. | A reviewer can trace every result and message to source, consent, population, code, uncertainty, and limits. | Model and evidence verification pass on all three; platform differences are published. | Knowledge and model versions remain readable, comparable, retractable, and reversible at the release boundary. | An external reviewer reproduces one pilot result and a participant verifies the exact message and data-use history. |
| G4 (42–60 months), frontier network | Site-owned federated analysis, one-person research studio, and long-horizon hypothesis tools. | Three-site denial, raw-data escape, small-group, protocol drift, safety pause, confounder, adversarial, and long-run recovery tests. | Every site and participant controls grants; ethics, privacy, bias, safety, and external reproducibility gates pass before real use. | Federation shows each site, approval, records included, denial, aggregate, uncertainty, pause, withdrawal, and exit. | Local site runners and verifier tools pass on all three; each adapter declares limits. | Sites can export or revoke bindings without deleting lawful shared aggregates or another site's records. | Independent observers run a synthetic three-site study with one denial and prove no row-level data left a site. |

## 19. Current truth and gap

The live code has generic `studies` and `cohorts` records. It has no consent
engine, protocol, measurement model, research analysis, genomics, participant
portal, or domain tests. The safest first slice is stable identities plus
VP-F0-02 and VP-F0-03, proven with one synthetic, non-clinical longitudinal
dataset. Real participant data is forbidden before that gate closes.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Product role | Research system, not care or diagnosis | Founder plus clinical governance |
| Consent | Versioned terms and purposes, fail closed | Ethics and architecture decision |
| Raw data | Preserved and immutable; corrections become new versions | Data-governance decision |
| First data | Synthetic fixtures only | Privacy readiness gate |
| Analytics form | Reproducible code and declared methods | Scientific review |
| Model release | Intended use, population, limits, calibration, and subgroup evidence | Model-governance gate |
| Data export | Denied by default; reviewed purpose lease | Data steward approval |
| Delete | Policy engine, with 30-day bin for ordinary user work | Legal/ethics exception |

## 21. Definition of category-defining done

- [ ] Participants can understand, control, export, and revoke permitted uses.
- [ ] Every result can be traced to source, consent, protocol, code, and review.
- [ ] Research and clinical action remain visibly and technically separate.
- [ ] Multi-site work can run without central raw-data custody.
- [ ] Bias, uncertainty, failed analyses, and missing data remain visible.
- [ ] Windows, macOS, Linux, accessibility, recovery, privacy, and independent
      scientific-review gates pass.
