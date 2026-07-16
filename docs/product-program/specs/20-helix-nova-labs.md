# HelixNovaLabs — reproducible scientific discovery system

```yaml
product: HelixNovaLabs
catalog_order: 20
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [researchers, laboratory teams, research software engineers, reviewers, funders]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixNovaLabs is a discovery operating system where the complete path from
question to claim remains executable, reviewable, portable, and honest about
failure.

## 2. Five-year destination

The useful product combines an electronic lab notebook, protocols, instruments,
data, code, computation, samples, findings, and publication packages. The
category advantage is a live research object: a claim can be opened to reveal
the exact evidence graph and, where permission allows, rerun from a clean
environment. The frontier is a federated discovery network where teams can test
hypotheses across private data and specialist facilities without centralizing
custody. Humans choose research questions, approve risky work, interpret results,
and decide what may be published.

## 3. Users and hard jobs

- Researchers need freedom and repeatability and fear losing context between
  notebook, data, code, and publication.
- Lab teams need sample and instrument truth and fear label or calibration errors.
- Research software engineers need durable environments and fear hidden manual steps.
- Reviewers need inspectable evidence and fear selected or changed analyses.
- Funders and institutions need useful progress signals and fear vanity reports.

## 4. Product laws

1. A finding is not a claim until its evidence, analysis, uncertainty, and review exist.
2. Negative, null, failed, and abandoned work remains searchable and attributable.
3. Raw observations are immutable; correction creates a linked version.
4. A protocol and analysis can be frozen before execution.
5. Compute environments, inputs, seeds, and manual steps are part of the result.
6. Agents never fabricate observations, citations, approvals, or replication.
7. Export includes relationships and provenance, not a folder of disconnected files.
8. Open science is encouraged, but consent, safety, intellectual property, and
   participant rights can require controlled access.

## 5. Scope boundaries

NovaLabs owns general research programs, protocols, samples, instruments,
research data, computation, evidence graphs, claims, peer review, and research
packages. SynthBio owns biology design and wet-lab safety. QuantumForge owns
quantum experiments. VitaPrime owns human precision-health studies. Other domain
products provide specialist engines through versioned adapters. NovaLabs does
not decide that a finding is true merely because a workflow completed.

## 6. Signature experiences

| Journey | Entry | Visible progress | Human decision | Completion proof | Failure and recovery | Portability |
|---|---|---|---|---|---|---|
| **Question to registered plan** | A researcher opens a question and chooses a study type. | Hypotheses, measures, exclusions, analysis, power assumptions, risks, reviewers, and unresolved fields are visible. | Named researchers approve the frozen protocol and analysis plan; an ethics or safety body approves when required. | A signed registration pins question, hypotheses, methods, analysis, risks, people, and time. | Missing authority or unresolved fields block registration; the draft remains recoverable and can be versioned. | Export includes the typed plan, narrative, relationships, approvals, identifiers, and proof. |
| **Execute a protocol** | An authorised person starts an approved protocol version. | Current step, timer, material, instrument, calibration, safety checks, deviation, next action, and last save show live. | A person confirms manual steps, deviations, safety holds, and any restart. | The run links actual steps, people, materials, instruments, observations, deviations, and final state. | A safety stop pauses work; recovery resumes only from an allowed checkpoint or closes the run with a reason. | Export includes protocol version, run log, observation references, deviations, and evidence. |
| **Data to analysis** | A researcher selects a frozen analysis plan and approved dataset versions. | Environment build, input checks, stage, resource use, tests, checkpoint, failures, and outputs stream to the run record. | A human approves sensitive data use, extra resources, plan changes, and interpretation. | The completed run pins environment, code, inputs, seeds, commands, outputs, checks, and limits. | Cancel stops local compute and records the last safe checkpoint; a changed rerun is a new linked run. | Export includes executable workflow, environment, inputs or references, results, and proof. |
| **Claim review** | A reviewer opens one claim and its evidence graph. | Supporting, contradicting, limiting, missing, and inaccessible evidence plus rerun checks stay visible. | The reviewer accepts, rejects, limits, or requests more work and signs the verdict. | Claim, exact wording, scope, uncertainty, evidence, analysis, comments, and verdict are linked. | Missing access or failed rerun blocks a clean verdict; the reviewer can issue a limited or unresolved result. | Export includes the claim graph, permitted evidence, review, signatures, and loss report. |
| **Independent reproduction** | Another team imports a research package on a clean system. | Package validation, mappings, local tools, instruments, data differences, reruns, and comparison update live. | The reproducing team approves substitutions and signs its independent conclusion. | A reproduction report pins source package, local changes, runs, comparisons, failures, and verdict. | Unsupported or missing parts remain explicit; the team may map, request access, or close as not reproducible. | The reproduction is itself a portable research object linked to the original. |
| **Publish or share** | An owner selects a claim or research object and an audience. | Identifier, licence, consent, embargo, access policy, citation, package, signature, and repository checks show. | Accountable owners approve exact public, embargoed, or controlled fields. | The release record links the published package digest, policy, approvals, destination, and known limits. | A failed deposit or identifier remains pending and retry does not create a duplicate release. | Export targets RO-Crate 1.3 plus a native manifest; controlled data may remain as access references. |
| **Recover work** | A user opens Recovery or a failed run checkpoint. | Bin age, affected relationships, checkpoint safety, retention holds, and restore checks are shown. | A person chooses restore, clone, close with reason, or separately authorised permanent deletion. | Restore keeps stable identity and provenance; deletion or withdrawal creates a signed event. | Drafts restore for 30 days. Immutable observations and regulated records use correction, withdrawal, or policy retention, not silent deletion. | Recovery and withdrawal evidence can be exported without exposing restricted values. |

## 7. Capability map

| ID | Gate | Input | Output | Invariant | Authority | Evidence | Failure state | Acceptance |
|---|---|---|---|---|---|---|---|---|
| NL-F0-01 | G0 | Research entities and typed relationships | Stable research-object graph | Identity is path independent; dangling or invalid relationship cannot publish | Object owner edits; governed relations need reviewer | Graph diff, schema check, actor, time | `invalid_graph` | WHEN an entity is removed, the system SHALL show every affected protocol, run, claim, review, and package before commit. |
| NL-F0-02 | G0 | Observation bytes/value, source time, units, quality, custody | Immutable observation plus content identity | Raw input never changes; correction is a new linked version | Approved person/instrument records; steward corrects with reason | Hash, source, unit, quality, custody, correction | `quarantined` | WHEN source, unit, or custody is missing, the observation SHALL not enter trusted analysis. |
| NL-F0-03 | G0 | Frozen work request, environment, resources, idempotency key | Durable run ledger and outputs | Transitions are atomic and idempotent; cancel is truthful; production state is not used in tests | Researcher starts within policy; broker grants tools | Plan, environment, stages, heartbeat, outputs, final state | `delayed`, `failed`, `cancelled` | WHEN a crash occurs at any transition, recovery SHALL yield one valid state and no orphan completed output. |
| NL-F1-01 | G1 | Structured fields, narrative, attachments, offline operations | Versioned signed notebook entry | Signature pins exact content; offline conflict never overwrites silently | Author writes; signer confirms | Versions, signature, attachments, sync log | `draft`, `conflict`, `withdrawn` | WHEN two devices edit a signed entry, the system SHALL preserve both and require a new signed resolution. |
| NL-F1-02 | G1 | Protocol steps, branches, risks, materials, instruments, approvals | Protocol version and guided run | Frozen protocol never changes in a run; deviation is explicit; safety stop wins | Research lead approves; safety/ethics authority when required | Version, risk checks, approvals, actual steps, deviations | `blocked`, `paused`, `deviated` | WHEN a required safety approval is absent, the protocol SHALL not start. |
| NL-F1-03 | G1 | Sample/material receipt, label, movement, instrument state | Lineage, custody, calibration, maintenance, disposal records | Stable identity and custody are explicit; expired calibration blocks trusted use | Custodian moves; qualified owner approves calibration/disposal | Scan, custody, calibration, maintenance, reason | `identity_conflict` or `calibration_expired` | WHEN calibration is expired, new observations SHALL be quarantined until a qualified decision. |
| NL-F2-01 | G2 | Frozen plan, exact dataset/code/environment, resource budget | Sandboxed analysis run and reproducible outputs | Same declared inputs reproduce inside tolerance; no hidden manual step | Analysis agent runs; human approves plan/resource changes | Code hash, environment, commands, seeds, tests, outputs | `not_reproducible` | WHEN a clean rerun differs outside tolerance, the result SHALL lose reproducible status and show the difference. |
| NL-F2-02 | G2 | Finding, proposed claim, evidence edges, uncertainty, scope | Versioned claim graph and readiness state | Support, contradiction, limitation, unknown, and reviewer state remain distinct | Researcher drafts; authorised human signs claim | Claim wording, edges, analyses, uncertainty, review | `evidence_incomplete` | WHEN required contrary evidence is unresolved, the claim SHALL not become releasable. |
| NL-F2-03 | G2 | Approved research object and access rules | RO-Crate/native package, import result, loss report | Unknown extensions are retained; loss is never silent; secret values never enter | Owner approves audience and export | Manifest, digests, mappings, validation, policy | `package_invalid` or `mapping_loss` | WHEN an import cannot express a relationship, it SHALL retain the source extension and name the loss. |
| NL-F3-01 | G3 | Published package, local tools/data/instruments, mapping choices | Reproduction plan, runs, comparison, verdict | Local differences are explicit and never rewritten as equivalence | Independent team approves substitutions and verdict | Source digest, mapping, local runs, comparison, signature | `not_reproducible` or `inconclusive` | WHEN a required component is unavailable, the report SHALL say not tested rather than reproduced. |
| NL-F3-02 | G3 | Claim graph, permitted data/code, reviewer comments | Living review and signed decision | Comment anchors remain stable; author cannot alter reviewer verdict | Reviewer owns verdict; author answers separately | Anchors, reruns, comments, decisions, resolution | `review_blocked` or `disputed` | WHEN reviewed content changes, prior verdict SHALL remain on the old version and the new version SHALL need review. |
| NL-F3-03 | G3 | Program graph, evidence maturity, run/review/reproduction states | Portfolio bottlenecks and decision-value view | Publication count is not maturity; restricted facts stay restricted | Program owner chooses shared measures | Metric definition, source graph, freshness, limits | `partial_portfolio` | WHEN a source is stale or hidden, the view SHALL show partial and SHALL NOT rank it as complete. |
| NL-F4-01 | G4 | Approved cross-site protocol, exact local grants, aggregation rules | Federated runs and signed aggregate | Raw protected data stays local; each site can deny, stop, or revoke | Each site, ethics/safety owner, and study lead approve | Grants, protocol, local runs, aggregate, revocations | `site_denied` or `aggregate_blocked` | WHEN one site revokes, new work there SHALL stop without deleting other sites' shared records. |
| NL-F4-02 | G4 | Approved literature, graph, negative results, open questions | Contradictions, missing tests, hypothesis suggestions | Agent cannot create observation, sign claim, or hide counterevidence | Researcher chooses any next test | Sources, search plan, graph paths, uncertainty | `insufficient_support` | WHEN evidence is mixed, the navigator SHALL show competing explanations and SHALL NOT promote one to a claim. |
| NL-F4-03 | G4 | Approved protocol step, instrument capability, safety envelope, lease | Bounded device request and observed result | Exact device, command, range, material, time, and stop are enforced locally | Qualified human releases each new control class | Envelope, approval, request, device ack, observation, stop | `rejected`, `unknown`, `safety_stop` | WHEN device feedback is missing or outside bounds, the connector SHALL stop and SHALL NOT report completion. |

## 8. Domain model

| Domain group | Owner | Lifecycle | Version rule | Retention and delete | Main relationships |
|---|---|---|---|---|---|
| Program and question: `ResearchProgram`, `Question`, `Hypothesis` | Program lead; named researchers own questions | proposed → reviewed → active → answered, rejected, or open | Meaning changes create a new version; rejected hypotheses remain searchable | Drafts use the 30-day bin; registered and cited records follow research policy | Program contains questions; hypotheses answer a question and link plans, findings, claims, and negative results |
| Protocol and governance: `Protocol`, `ProtocolVersion`, `Step`, `Risk`, `Approval`, `Deviation` | Research lead; ethics, safety, and domain authorities own approvals | draft → checked → approved → frozen → running → completed/paused/withdrawn | A run pins one frozen version; any change creates a new version; deviations never edit the plan | Drafts use bin; approved and executed versions follow ethics, safety, sponsor, and legal hold | Protocol tests hypotheses, uses samples/instruments, creates runs/observations, and links every approval/deviation |
| Notebook: `NotebookEntry`, attachment, signature, offline operation | Author owns entry; signer owns signature | draft → signed → amended by linked version → withdrawn | Signed content is immutable; conflicts create parallel versions and a signed resolution | Ordinary drafts use bin; signed entries follow project retention; withdrawal preserves provenance | Entries link question, protocol step, sample, instrument, observation, run, finding, and decision |
| Materials and instruments: `Material`, `Sample`, custody, `Instrument`, `Calibration`, maintenance, disposal | Lab and named custodian; qualified owner controls instrument state | received/created → labelled → in use → stored/transferred → consumed/disposed; instrument active → maintenance/calibration → retired | Identity and custody events append; calibration and label corrections link new records | Retention follows safety, consent, regulation, and material policy; no delete while referenced | Samples derive from materials; custody and instrument state qualify observations and protocol steps |
| Primary data: `Observation`, `Dataset`, `Transformation`, quality and correction | Research object owner; source person/instrument is recorded | captured → validated → accepted/quarantined → corrected by link → assembled into dataset | Raw observation is immutable; dataset and transformation are content-versioned | Primary and participant data follow consent, ethics, sponsor, and legal policy; derived drafts may use bin | Observation links sample, instrument, protocol step, unit, custody; transformations create datasets used by runs |
| Compute: `SoftwareEnvironment`, `AnalysisPlan`, `Run`, checkpoint, output | Researcher owns plan; run worker is accountable process | plan draft → frozen → queued → running → completed/failed/cancelled; rerun is new | Environment, code, inputs, seeds, commands, and outputs are pinned; no completed state without durable output | Environments and outputs follow project policy; failed and cancelled runs remain searchable | Run executes plan over datasets, creates outputs/findings, and links resources, tests, and deviations |
| Evidence and claim: `Finding`, `Claim`, `EvidenceEdge`, uncertainty, scope | Researcher owns draft; authorised human owns signed claim | finding observed → reviewed; claim draft → evidence-ready → reviewed → signed/rejected/withdrawn | Wording, scope, evidence, and verdict use immutable versions | Negative, null, contradictory, and withdrawn work remains under policy; drafts use bin | Evidence edges type support, contradiction, limitation, unknown, or unrelated links from source to claim |
| Review, reproduction, release: `Review`, `Reproduction`, `PublicationPackage`, `AccessPolicy`, `ProofBundle` | Independent reviewer/reproducer owns verdict; object owner controls audience | review open → decided/appealed; reproduction planned → run → verdict; package draft → released/withdrawn | Decisions and released package digests are immutable; correction or withdrawal is linked | Reviews, releases, identifiers, consent, embargo, and legal holds set retention | Review pins claim/object version; reproduction links source/local mappings; package contains permitted graph, artefacts, policy, and proof |

## 9. System architecture

- A graph service owns stable research identities and typed relationships.
- An append-only observation and run layer preserves primary records.
- Sandboxed compute executes versioned workflows and stores content-addressed
  outputs.
- Protocol adapters connect instruments through read-only simulation before any
  controlled action is allowed.
- Search indexes narrative and structured evidence while respecting access at
  query time.
- HelixCore supplies identity, policy, capabilities, jobs, audit, objects,
  notifications, and recovery.
- Specialist products expose domain engines through signed capability contracts.

## 10. Agent and automation contract

| Agent | May read | Tools | May draft | Approval required | Forbidden | Checking | Stop and reversal |
|---|---|---|---|---|---|---|---|
| Literature Scout | Approved databases, local library, research graph, access policy | Search, identifier resolver, citation validator, deduplicator | Source map, summary, open questions | Human approves sources used in protocol or claim | Fabricate citation, create observation, bypass access, present summary as evidence | Identifier, source, access, retraction/status, quotation and duplication checks | Cancel stops search; imported source batch rolls back; accepted source record is versioned, not erased |
| Protocol Assistant | Question, hypotheses, approved references, lab capability, safety policy | Protocol editor, unit checker, simulator, risk checklist | Steps, branches, materials, measures, safety and analysis fields | Research lead plus ethics/safety authority where required | Approve risk, start work, hide missing control, invent equipment ability | Completeness, unit, feasibility, power assumption, risk, authority tests | User stops draft; prior approved protocol stays active; changed plan is a new version |
| Data Steward | Approved observations, schemas, custody, consent, quality rules | Schema mapper, unit validator, duplicate and quality tools | Mapping, correction link, quarantine and quality report | Steward approves trusted dataset; participant policy may require owner | Erase raw input, guess unit, widen consent, merge identities silently | Hash, source, unit, custody, consent, missingness, duplicate tests | Cancel import; batch rolls back; quarantine preserves original; correction is reversible by another link |
| Analysis Agent | Frozen plan, exact approved inputs/code/environment, resource lease | Sandboxed compute, notebook/pipeline runner, tests, package builder | Run outputs, diagnostics, comparison | Human approves plan change, extra resource, sensitive export, interpretation | Change frozen plan, use undeclared data/network, write production state, call failed run complete | Environment, input hash, seed, test, resource, output and clean-rerun checks | Cancel accepted within 2 seconds; local job stops within 10 seconds; clone from checkpoint creates a new run |
| Skeptic Agent | Claim graph, permitted evidence, methods, negative results, reviews | Evidence traversal, leakage/control checklist, alternative-model sandbox | Contradictions, missing tests, alternative explanations | Human decides whether and how to respond | Suppress supporting or contrary evidence, sign verdict, invent flaw | Source coverage, control, leakage, scope, uncertainty, counterexample checks | Stop leaves a partial labelled review; user can dismiss suggestions but disposition stays recorded |
| Claim Assistant | Approved findings, analyses, evidence edges, scope and uncertainty | Claim editor, graph validator, plain-language and citation checks | Claim wording, limitation, abstract, response | Authorised human signs; separate owner approves release | Create observation, self-sign, publish, remove contrary evidence | Evidence readiness, wording-to-source, uncertainty, scope, review and policy checks | Draft can be discarded; signed claim changes only by a new linked version or withdrawal |
| Instrument Agent | Exact protocol step, device capability, material, local state, safety envelope | Brokered device simulator or bounded command connector | Device request and expected observation contract | Qualified human releases each new control class and live request | Retrieve secret, widen range/time, bypass interlock, assume missing feedback means success | Identity, calibration, material, range, duration, local interlock, acknowledgement, observed state | Lease revoke stops new calls; local emergency stop wins; physical reversal uses approved recovery protocol |

Every agent shows current stage, item, checkpoint, elapsed time, resource use,
warnings, evidence created, approval needed, and next expected signal. Agents can
see capability metadata and request access, but cannot retrieve secret values or
approve their own request.

## 11. Trust, safety, and privacy

Access is per research object, purpose, role, site, time, and output. Sensitive
human, environmental, commercial, and dual-use work uses stronger policy and
export review. Instrument control is isolated from narrative and model output.
Untrusted files and environments run in sandboxes. Secret values never enter
research objects. Ordinary drafts use a 30-day bin. Signed observations,
regulated records, and legal holds use versioned withdrawal or retention rather
than silent deletion.

## 12. Proof and audit

Proof covers identity, protocol, approvals, material and instrument lineage,
raw-data hashes, environment, actual code, run transitions, deviations, quality
checks, analyses, claims, reviews, reproductions, publication package, and known
limits. It proves process and artifact identity, not scientific truth. Aether is
the preferred external proof service; a local signed bundle supports offline work.

## 13. UX system

The main surfaces are Programs, Questions, Notebook, Protocols, Samples,
Instruments, Data, Analysis, Findings, Claims, Review, Reproduce, Publish,
Evidence, and Recovery. The default view follows today’s work. Deeper graph,
lineage, environment, and raw data are progressively revealed. Long runs show
actual steps and last durable output. Moving a sample or dataset previews all
links affected. Completion notifies the owner and reviewers. A failed run offers
resume from safe checkpoint, clone with change, or close with reason.

## 14. Interoperability and standards

- [W3C PROV-O](https://www.w3.org/TR/prov-o/) is the provenance exchange
  adapter, verified from the W3C Recommendation on 2026-07-15.
- [RO-Crate 1.3](https://www.researchobject.org/ro-crate/specification/1.3/index.html)
  is the current export and package adapter. The official specification marks it
  as the newest Recommendation, published 2026-06-22; this was verified on
  2026-07-15. NovaLabs also retains
  [RO-Crate 1.2 import compatibility](https://www.researchobject.org/ro-crate/specification/1.2/).
  A 1.2 import is preserved as received, mapped to the native graph, and given a
  version and semantic-loss report; it is never silently rewritten as native 1.3.
- Domain products add their official standards through adapters; NovaLabs keeps
  the common evidence graph independent of one discipline.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) sets the web accessibility target.

Standards remain versioned adapters. Imports retain unknown extensions and
produce a human-readable loss report for identities, relationships, profiles,
workflows, access policy, signatures, and local terms.

## 15. Cross-platform contract

Notebook, protocol, data review, local analysis, evidence validation, and package
import/export pass the same journeys on Windows, macOS, and Linux. Instrument
adapters declare supported systems and have simulators everywhere. Web review is
portable; local device and secret access remains in a user-owned process. Offline
entries sync with explicit conflict and pending states.

## 16. Reliability and performance budgets

| Area | Numeric target | Measurement and failure behaviour |
|---|---|---|
| Data loss | Zero acknowledged identities, signatures, approvals, observations, run transitions, claims, and reviews lost | Every release runs forced-crash tests at each write stage; a completed run requires durable ledger and outputs |
| Evidence view | First useful graph view under 2 seconds p95 over rolling 30 days for 100,000 linked objects in the reference local profile | If the full graph is slower, the UI shows a real partial stage and never an empty frozen screen |
| Offline | Notebook, protocol review, sample capture, and queued sync work for at least 30 days or 100 GB per device | Limits warn 7 days or 10 GB before exhaustion; offline conflicts never use silent last-write-wins |
| Idempotency | Run, import, observation, package release, identifier, and federation keys remain valid at least 90 days | Ten repeated requests return one identity or prior result; uncertain external deposit blocks a duplicate |
| Concurrency | 200 concurrent reviewers may comment on one 100,000-edge graph; a frozen protocol, signed claim, or released package cannot be changed in place | Race tests preserve every comment and create explicit version conflicts for meaning changes |
| Scale | One supported managed program stores 10 million linked research objects, 100 TB of content-addressed artefacts, and runs 1,000 concurrent sandbox jobs | Four-hour load test records graph/query p95, queue delay, output integrity, tenant isolation, and resource caps |
| Job start and heartbeat | Durable first stage within 2 seconds; active local job heartbeat no older than 5 seconds | UI changes to delayed after 10 seconds without a real signal and names the last durable checkpoint |
| Cancellation | Cancel accepted within 2 seconds; local compute stops within 10 seconds and records final checkpoint | External work remains `cancel_requested` until confirmed; partial output stays incomplete |
| Recovery | Metadata RPO zero after acknowledgement and RTO 15 minutes; managed object RPO 15 minutes and RTO 4 hours | Quarterly restore verifies graph, object digests, access policy, signatures, and one clean rerun |
| Claim gate | 100% of required evidence, review, and run states must be complete and known before release | Any pending, failed, unknown, withdrawn, or inaccessible required item blocks releasable state |
| Degradation | Loss of model provider, Aether, search index, repository, or one compute backend does not block local notebook, manual protocol, raw observation capture, local proof, or export | Missing capability is visible within 10 seconds; no fallback fabricates a source, result, or review |

## 17. Success measures

| Measure | Threshold and window |
|---|---|
| Research integrity | Zero fabricated observations, citations, approvals, signatures, or completed runs in every release and rolling 90-day audit |
| Clean rerun | At least 95% of packages labelled reproducible rerun on a clean supported system inside their declared tolerance in monthly samples |
| Independent reproduction | At least 80% of annually sampled open benchmark packages reach a clear reproduced, not reproduced, or inconclusive verdict; none stay falsely complete |
| Evidence completeness | 100% of released claims link scope, uncertainty, supporting and contrary evidence, analysis, human review, and known limits |
| Negative work | 100% of failed, null, negative, and abandoned registered runs remain searchable under policy for the full project retention period |
| Deviations | 100% of declared protocol deviations are linked before a run can close; quarterly audit finds zero hidden step changes |
| Protocol time | Median time from complete question draft to reviewed protocol is under 5 working days each quarter, excluding external ethics review |
| Portability | 100% of quarterly sampled RO-Crate 1.3 exports validate; all sampled 1.2 imports produce a mapping and semantic-loss report |
| Recovery | 100% of quarterly metadata restore drills meet RPO zero and 15-minute RTO; object restore meets the project target |
| Accessibility | Zero critical or serious accessibility findings in release journeys; keyboard, screen-reader, zoom, non-colour, and long-run tasks pass every release candidate |
| Researcher trust | At least 85% of quarterly sampled users can explain the current protocol version, data provenance, run state, evidence gap, and proof limit without help |
| Business and value | At least 70% annual renewal for paying teams tied to reproducibility, review, or saved research time; publication count and agent calls are not goals |

## 18. Delivery plan

| Gate | Build | Tests | Safety | UX | Windows/macOS/Linux | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Stable research graph, immutable observations, atomic run ledger, truthful jobs, local proof, recovery | Graph, unit, custody, crash, idempotency, signature, state-isolation tests | Threat model, tenant/access policy, secret broker, no live instrument control | Honest graph, run timeline, failure, cancel, Recovery | Rust/core, CLI, package, local store CI pass on all three | Dry-run importer from current experiments/findings with counts, mappings, loss, rollback | Fresh install, import, crash, restore, verify synthetic object on each OS |
| **G1 — Useful single-player product (6–18 months)** | Notebook, protocol builder/runner, samples, instruments, offline capture, one synthetic study | Full plan-to-observation journey, sync conflict, protocol safety, accessibility tests | Synthetic/read-only instruments; ethics and retention policy checks | Accessible Today's Work, Notebook, Protocol, Samples, Recovery | Desktop/web/CLI/container and simulators pass on Windows/macOS/Linux | Versioned notebook, protocol, sample, instrument migrations with rollback | Researcher runs, pauses, resumes, exports, restores, verifies study on each OS |
| **G2 — Trusted team product (18–30 months)** | Sandboxed analysis, claims, evidence graph, review, RO-Crate 1.3/native packages | Clean rerun, claim gate, multi-reviewer, package 1.3 export and 1.2 import tests | Sandbox, sensitive-data, prompt-injection, supply-chain, access review | Evidence, uncertainty, slow-run, review and completion notifications | Compute and package fixtures pass on all three; browser review matrix | Data/environment/claim/package migration and clean rollback | Independent reviewer reruns and verifies a controlled package on a clean system |
| **G3 — Category leader (30–42 months)** | Reproduction workspace, living review, portfolio intelligence, three-discipline pilots | Mapping, reproduction, scale, policy, discipline-adapter conformance tests | External research-integrity, privacy, dual-use, and security review | Reproduction differences, anchored review, partial portfolio states | Mixed-OS teams reproduce the same packages and offline records | Verified live research-program migration with canary, loss report, rollback | Three external teams reproduce, dispute, restore, and sign verdicts |
| **G4 — Frontier network (42–60 months)** | Federated execution, hypothesis navigator, bounded lab connector | Malicious site, partition, revoke, privacy floor, HIL, device-stop, adversarial agent tests | Formal ethics, biosafety/domain safety, dual-use, regulator, security, science review | Site grants, federation progress, instrument preview, stop, exit | Mixed-node Windows/macOS/Linux planning plus approved edge/device profiles | Remove a site/project and revoke bindings without deleting shared secrets or evidence | Independent federate, execute, revoke, stop, leave, disaster, and rollback exercise |

A gate closes only from fresh release-candidate builds, tests, safety checks,
journeys, migrations, and operator evidence. A skipped clean rerun, stale ethics
approval, or structurally valid package with missing relationships cannot pass.

## 19. Current truth and gap

The live code contains generic `experiments` and `findings` records. It has no
protocol engine, notebook, data lineage, compute runner, evidence graph,
reproduction, product UI, or domain tests. The first slice is NL-F0-01 through
NL-F1-01 using a small, synthetic, fully reproducible measurement study.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Source of truth | Typed research graph plus immutable primary artifacts | Architecture decision |
| First study | Synthetic and safe; no live instrument control | G1 safety proof |
| Observation correction | New linked version, never overwrite | Research-integrity review |
| Agent claims | Draft only; human signs | Founder and science governance |
| Compute | Reproducible sandbox with exact inputs and limits | Security review |
| Package | RO-Crate adapter plus native manifest | Portability decision |
| Negative results | Retained and searchable under policy | Research-governance decision |
| Delete | 30-day bin for drafts; policy flow for primary records | Legal/ethics exception |

## 21. Definition of category-defining done

- [ ] A claim can be followed back to every source, step, run, and decision.
- [ ] An independent team can validate and reproduce an allowed package.
- [ ] Negative work, deviations, uncertainty, and conflicting evidence remain visible.
- [ ] Agents accelerate work without manufacturing truth or self-approval.
- [ ] Teams can leave with usable objects, relationships, history, policies, and proof.
- [ ] Windows, macOS, Linux, accessibility, recovery, security, safety, and
      independent scientific-review gates pass.
