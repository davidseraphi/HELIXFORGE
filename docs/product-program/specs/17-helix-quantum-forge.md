# HelixQuantumForge — hybrid quantum engineering workspace

```yaml
product: HelixQuantumForge
catalog_order: 17
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [quantum researchers, algorithm engineers, educators, hardware teams]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixQuantumForge is a provider-neutral workspace where a team can
design, test, compare, explain, and prove hybrid quantum-classical experiments
without tying its knowledge or workflow to one hardware company.

## 2. Five-year destination

The useful product is a visual and code-based circuit laboratory with a local
simulator, repeatable experiments, notebooks, tests, cost estimates, and clear
results. The category advantage is one portable experiment graph that compiles
to several providers while preserving inputs, compiler decisions, device
conditions, raw observations, uncertainty, and proof. The frontier is a
hardware-aware co-design system that can search circuit, error-mitigation, and
classical-control choices under a user-set budget. A human always approves paid
hardware runs, publication, and any claim of scientific advantage.

## 3. Users and hard jobs

- Researchers need reproducible experiments and fear results that cannot be
  repeated on another device or date.
- Algorithm engineers need portable compilation and fear silent semantic
  changes between providers.
- Hardware teams need workload evidence and fear benchmark gaming.
- Educators need inspectable simulation and fear tools that hide the physics.
- Reviewers need full provenance and fear claims based only on selected runs.

## 4. Product laws

1. Simulation and hardware observations are always labeled separately.
2. A result includes uncertainty, shot count, device snapshot, and compiler log.
3. No agent may purchase or submit a hardware run without an exact approved cap.
4. Provider adapters cannot change the canonical experiment record.
5. Failed and discarded trials remain visible in the experiment history.
6. Claims compare against named classical and quantum baselines.
7. All core authoring, simulation, and review work is usable offline.
8. Export includes source, intermediate form, environment, results, and proof.

## 5. Scope boundaries

QuantumForge owns experiment design, circuit and hybrid-program models,
compilation, simulation, provider submission, result analysis, and teaching.
HelixCapital owns payment records. HelixNovaLabs owns cross-discipline research
programs. HelixCode owns general source hosting and CI. QuantumForge is not a
claim that quantum computing is better for a task, and it never invents a
scientific conclusion from a score alone.

## 6. Signature experiences

| Journey | Entry point | Visible progress | Human decision | Completion proof | Failure and recovery | Export or portability |
|---|---|---|---|---|---|---|
| Learn by seeing | Open a lesson or blank circuit. | Show validation, simulation step, state, measurement, and elapsed time. | The learner chooses hints, measurements, and when to reset. | Save the circuit version, state trace, equations, and test result. | An invalid edit keeps the last valid graph; undo, retry, and the 30-day bin remain available. | Export the lesson, circuit, trace, and explanation as a local bundle. |
| Portable experiment | Import OpenQASM or open a native experiment. | Show parse, normalize, compile, and adapter comparison stages. | The engineer accepts or rejects every reported semantic difference. | Produce two compile plans with source hashes and a signed difference report. | A failed adapter leaves the canonical graph unchanged and can be retried from its checkpoint. | Export OpenQASM 3.1, QIR when supported, and the native experiment bundle. |
| Budgeted hardware run | Open a reviewed experiment and choose a remote device. | Show approval state, queue, device, shots, spend, last signal, and cancel state. | A human approves the exact device, shot, money, time, and residency lease. | Record provider acknowledgment, raw-result hash, device snapshot, spend, and ledger transitions. | Missing acknowledgment stays `unknown`; cancel stays requested until confirmed; retry creates a new attempt. | Export the run intent, approval, raw results, analysis, and proof bundle. |
| Noise investigation | Select a program version and named noise profiles. | Show each ideal, noisy, and observed run plus confidence and remaining work. | The team chooses models, seeds, tolerances, and which comparisons are valid. | Store distributions, assumptions, seeds, confidence intervals, and a reviewed comparison. | One failed run does not erase others; pause, retry, or replace a model from the last checkpoint. | Export data, models, seeds, plots, and the comparison report. |
| Fair benchmark | Start from a frozen benchmark protocol. | Show baseline stages, trial counts, tuning effort, exclusions, and failures. | A reviewer freezes baselines and approves the final claim wording. | Sign a report containing all trials, costs, uncertainty, and rejected claims. | Failed and discarded trials stay visible; an incomplete baseline blocks release rather than becoming a partial success. | Export machine-readable results, tables, protocol, and a research crate. |
| Reproduce a paper | Import a signed research crate. | Show validation, environment rebuild, allowed reruns, and difference checks. | The reviewer approves network, hardware, cost, and any substituted dependency. | Produce an independent difference report linked to every rerun and unchanged source item. | Missing inputs or incompatible tools are named; the review can pause and resume without inventing a result. | Re-export the crate with the new environment, results, differences, and verifier guide. |

## 7. Capability map

| ID | Gate | Input | Output | Invariant | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| QF-F0-01 | G0 | Typed OpenQASM or native source. | Versioned canonical graph and diagnostics. | Provider data never defines the canonical meaning. | A user edits; an agent may only draft inside its lease. | Source hash, parser version, graph hash, and diagnostics. | Invalid or ambiguous input creates no executable graph. | The same fixture produces the same graph hash on Windows, macOS, and Linux. |
| QF-F0-02 | G0 | Valid graph, seed, shots, and simulator profile. | State trace, raw counts, limits, and timing. | A named seed and build reproduce the reference result. | A local execution lease may run it; no remote authority is implied. | Graph hash, seed, build, raw counts, and golden-vector result. | A crash leaves the attempt incomplete and preserves its last durable state. | One hundred golden circuits pass on all three desktop systems. |
| QF-F0-03 | G0 | Run intent and allowed state transition. | Append-only run event and current derived state. | An accepted transition is durable exactly once and old events never change. | Only the owning worker or approved operator appends its allowed event. | Idempotency key, event hash, source, environment, actor, and timestamp. | An invalid transition is rejected; recovery returns `known` or `unknown`, never false success. | One thousand injected crash points cause no lost or duplicate accepted transition. |
| QF-F1-01 | G1 | Program version, edits, notebook cells, and view choice. | One synchronized visual, text, notebook, test, and explanation model. | Every view changes the same canonical graph through a transaction. | Users edit; agents propose isolated edits for review. | Edit transaction, before/after hashes, origin, and undo record. | A conflict keeps both versions and blocks silent overwrite. | One edit made in each view appears with the same graph hash in every other view. |
| QF-F1-02 | G1 | Program, test cases, expected distributions, and tolerances. | Passed, failed, or unknown test results with reasons. | A missing or timed-out required test can never count as passed. | A human defines or approves tests; the runner only executes them. | Test version, inputs, seed, environment, observations, and verdict. | Timeout, missing data, or invalid tolerance returns `unknown` or `failed`. | The reference suite detects every seeded wrong gate, distribution, and tolerance fixture. |
| QF-F1-03 | G1 | Program and named ideal or noise profiles. | Comparable distributions, uncertainty, and model assumptions. | Simulated, modeled, and hardware data are never merged or mislabeled. | A researcher selects models; an agent may suggest but not silently replace them. | Model version, parameters, seeds, runs, confidence method, and comparison. | Unsupported models fail before execution; partial runs remain visibly partial. | Repeated seeded runs reproduce the reference bands within the declared tolerance. |
| QF-F2-01 | G2 | Canonical compile plan and backend profile. | Provider plan, cost inputs, queue model, and semantic-loss report. | Unsupported meaning is reported before submission and is never dropped. | A user enables an adapter; the adapter cannot widen its capability manifest. | Adapter version, supported operations, passes, losses, and contract-test result. | Unsupported input blocks submission and leaves the graph unchanged. | Two independent mock adapters pass shared contract and loss-report fixtures. |
| QF-F2-02 | G2 | Reviewed run intent and exact device, shot, cost, time, and residency limits. | Remote submission, provider status, spend record, and result link. | No request may exceed or outlive the approved lease. | A human approves; the user-owned broker submits; agents never approve. | Lease, approval, request hash, provider acknowledgment, spend, and cancellation trail. | Cap breach is blocked; missing acknowledgment remains `unknown`; no automatic resubmit occurs. | Boundary tests block every one-unit overspend, expired lease, wrong device, and residency mismatch. |
| QF-F2-03 | G2 | Versioned graphs, passes, runs, claims, and comments. | Anchored review threads, decisions, and unresolved findings. | A comment stays attached to the exact reviewed version. | Reviewers decide findings; authors cannot silently close independent review. | Anchor, reviewer identity, decision, timestamp, and old/new version link. | A stale anchor is marked stale and must be re-reviewed. | Concurrent edit fixtures preserve every comment and flag all stale anchors. |
| QF-F3-01 | G3 | Canonical graph and one or more target profiles. | Inspectable compiler passes, target plans, and equivalence results. | No compiler rewrite occurs without a recorded pass and checked meaning. | Release policy selects approved passes; agents may search only in a sandbox. | Pass inputs/outputs, tool versions, equivalence checks, and losses. | Failed or unavailable equivalence proof blocks release for that target. | Golden compile fixtures show every pass and catch every seeded silent rewrite. |
| QF-F3-02 | G3 | Frozen protocol, all quantum trials, classical baselines, costs, and exclusions. | Uncertainty-aware benchmark report and bounded claim. | Every trial and tuning effort remains present, including failures. | A reviewer freezes baselines and approves claim wording. | Protocol hash, trial ledger, cost, statistics, exclusions, and review. | Missing baselines or hidden trials make the report not releasable. | Mutation tests that remove a trial, cost, or baseline always fail the release gate. |
| QF-F3-03 | G3 | Source, intermediate forms, dependencies, seeds, device facts, results, and analysis. | Signed reproduction bundle and verifier guide. | Every referenced artifact is content-addressed and can be checked independently. | The owner exports; a separate reviewer verifies. | Manifest, hashes, signatures, environment, checks, and known limits. | A missing or changed artifact invalidates the bundle without deleting it. | Every reference bundle validates and reruns on a clean supported operating system. |
| QF-F4-01 | G4 | Approved search space, objectives, device model, and budget limits. | Ranked candidates, rejected candidates, costs, and uncertainty. | Search never crosses an approved cost, device, time, or safety limit. | A human sets limits and approves any real run; an agent may search only inside them. | Search policy, every candidate, rejection reason, spend, and final selection. | A limit breach stops the search and records the last safe checkpoint. | Property tests generate limit edges and prove that no candidate outside them executes. |
| QF-F4-02 | G4 | Signed job, site policy, data or hardware grant, and return schema. | Site-local execution result, aggregate, and proof. | Raw protected data and custody stay at the owning site. | Each site owner grants access; no central agent can self-enrol a site. | Grant, route, site acknowledgment, local run proof, denial, and returned hash. | A denial or disconnected site remains explicit and cannot be replaced with guessed data. | A three-site synthetic test completes with one denial and exposes no raw site record. |
| QF-F4-03 | G4 | Canonical graph, lesson model, equations, and misconception tests. | Source-linked explanation, interactive trace, and learning check. | An explanation must match the graph and name uncertainty or limits. | Educators approve lessons; learners control hints and assessment sharing. | Graph version, explanation source, trace, answers, and test result. | Unsupported meaning is labeled unknown rather than invented. | The misconception suite catches every seeded wrong explanation and preserves the correct trace. |

## 8. Domain model

| Record group | Owner | Creation and change | Version rule | Retention and delete | Main relationships |
|---|---|---|---|---|---|
| `Workspace`, `Experiment` | Workspace owner under HelixCore policy. | Created by a user; names and membership change through audited commands. | Stable IDs remain; mutable labels have revision numbers. | Ordinary deletion enters the 30-day bin; legal hold blocks final deletion. | A workspace owns experiments; an experiment groups programs, tests, runs, analyses, and proof. |
| `ProgramVersion`, `CircuitGraph`, `ClassicalGraph`, `ParameterSet`, `TestCase` | Experiment owner. | An accepted edit creates a new transaction and program version. | Published versions are immutable and found by their content hash. | Versions referenced by a run or proof follow that evidence retention; unreferenced drafts use the 30-day bin. | Tests and parameters point to one program version; graphs never point to provider-owned JSON as truth. |
| `CompilePlan`, `CompilerPass`, `BackendProfile`, `DeviceSnapshot` | QuantumForge records them; the adapter supplies signed provider facts. | Each compile or provider refresh creates a new record. | Profiles and snapshots are time-bound immutable versions; passes link input and output hashes. | Keep while any run, analysis, or proof refers to them; adapter cache copies may expire after 30 days. | A compile plan links a program version to passes, target profile, device snapshot, and loss report. |
| `RunIntent`, `Approval`, `RunAttempt` | Intent owner; approval belongs to the accountable human; attempt belongs to its worker. | Intent is drafted, approved, then attempted through allowed ledger transitions. | Intent and approval freeze before submission; a retry is a new attempt. | Keep with financial and research policy; user deletion hides ordinary drafts but never rewrites the ledger. | An intent links program, parameters, target, caps, approval, attempts, and observations. |
| `Observation`, `NoiseModel`, `Analysis`, `Baseline`, `Claim` | Experiment owner; raw observations are custody-controlled records. | Observations append; models and analyses create new versions; claims require review. | Raw observations never change; corrections and reanalysis are new linked versions. | Content and policy control retention; a claim keeps every supporting, limiting, failed, and superseded analysis link. | Analyses consume observations and models; claims point to exact analyses and baselines. |
| `ProofBundle` | Exporting owner, signed by named actors and providers. | Created only after referenced records are durable; verification adds reports, not rewrites. | Manifest and every artifact are content-addressed; a correction is a new bundle. | Retain for the declared research and legal period; deletion requires explicit authority and leaves a signed tombstone where required. | Bundles link source, versions, runs, approvals, results, analyses, reviews, limits, and verifier guidance. |

## 9. System architecture

- A Rust experiment kernel parses, validates, simulates, and records runs.
- A provider-neutral intermediate layer supports source adapters and hardware
  adapters.
- Sandboxed workers run compilers, simulators, and analysis jobs.
- HelixPulse may provide low-latency queues and caches after Pulse reaches G2;
  PostgreSQL remains the durable source of truth.
- MinIO stores large raw results and environment bundles.
- HelixCore supplies identity, policy, jobs, audit, capabilities, and recovery.
- All adapters use declared capability manifests and contract tests.

## 10. Agent and automation contract

| Role | May read | Tools | May draft | Approval required | Forbidden | Progress and checking | Stop or reversal |
|---|---|---|---|---|---|---|---|
| Tutor | Public lessons and learner-approved workspace versions. | Local simulator, lesson tests, and explanation viewer. | Hints, examples, and practice circuits. | Learner approval before saving into the main experiment. | Submit jobs, read private work, score hidden traits, or claim a learning result not tested. | Show lesson step and simulator state; check explanations against the graph and misconception tests. | Stop immediately; discard the draft or restore the prior lesson checkpoint. |
| Experiment Planner | Approved programs, prior runs, budgets, and public research sources. | Protocol editor, local estimates, test builder, and sandboxed search. | Protocols, baselines, tests, and comparison plans. | Human approval before freezing a protocol or using remote resources. | Change an accepted claim, hide a failed trial, or widen budget and data limits. | Show current plan step, candidates, warnings, spend estimate, and unresolved choices; validate every required field. | Cancel search, preserve candidates, and restore the last approved protocol version. |
| Compiler Scout | Canonical graph, target manifests, and approved compiler tools. | Sandboxed compilers, equivalence checks, and cost estimator. | Candidate passes and target plans. | Human or release-policy approval before a candidate becomes an executable plan. | Change canonical meaning, use an undeclared tool, or submit to hardware. | Stream pass, input/output hash, equivalence status, losses, and remaining candidates. | Kill the sandbox; keep its trace; reject or roll back the candidate plan. |
| Run Operator | Approved intent, lease, provider status, and its own run events. | User-owned broker, provider adapter, ledger, cancel, and result validator. | Lease request and submission preview. | Accountable human approval for every paid or remote run. | Approve itself, expose credentials, exceed caps, or resubmit an unknown request. | Show approval, queue, device, spend, shots, last signal, and validation; check provider acknowledgment and schema. | Stop the owned local worker; request remote cancellation; record survivors and compensation without false success. |
| Reviewer | Source, all trials, analyses, claims, proof, and known limits. | Provenance viewer, statistics checks, bundle verifier, and comment tools. | Findings, alternative explanations, and bounded claim wording. | A named human resolves high-impact findings and approves publication. | Hide trials, rewrite evidence, contact a provider as the owner, or approve its own authored claim. | Show records checked, failed checks, open findings, and coverage; verify on a separate environment. | Withdraw approval, reopen the finding, and restore the last reviewed claim version. |

## 11. Trust, safety, and privacy

Workspaces use per-resource access and database-enforced tenant separation.
Credentials remain in the capability broker; agents receive leases, never raw
values. Export-controlled or confidential workloads use explicit policy labels.
Hardware results are untrusted input until schema, signature, and expected-run
checks pass. Delete enters a 30-day bin unless a legal hold applies. Permanent
deletion requires a second confirmation, re-authentication, and signed proof.

## 12. Proof and audit

Proof covers source identity, compiler versions and passes, environment, seed,
shots, device facts supplied by the adapter, approvals, raw-result hashes,
analysis code, and known limits. It does not prove that a provider's physical
device behaved honestly unless independent device evidence exists. Aether is
the preferred proof provider; a local signed bundle is the fallback.

## 13. UX system

The main surfaces are Home, Experiments, Circuit, Notebook, Runs, Compare,
Devices, Learn, Evidence, and Recovery. Basic mode shows a calm circuit and run
flow. Expert mode reveals pulses, compiler passes, noise assumptions, and raw
counts. A long run has a live timeline, queue position when available, spend,
shots completed, last signal, and cancel state. Completion creates an in-app
notification and optional desktop notice. Moving or replacing selected graph
nodes shows a preview and asks only when meaning changes.

## 14. Interoperability and standards

- [OpenQASM 3.1](https://openqasm.com/versions/3.1/) is the primary text import
  and export adapter, verified from its official specification on 2026-07-15.
  OpenQASM 3.0 remains an explicit compatibility import profile; its differences
  are reported before conversion.
- [QIR](https://www.qir-alliance.org/qir-book/concepts/what-is-qir.html) is an
  intermediate adapter for hybrid programs, verified from the QIR Alliance on
  2026-07-15.
- [W3C PROV-O](https://www.w3.org/TR/prov-o/) records portable provenance, and
  [RO-Crate 1.3](https://www.researchobject.org/ro-crate/specification/1.3/)
  packages research objects; both were verified from their official sources on
  2026-07-15.

Standards are adapters, not the internal source of truth. Import reports every
unsupported construct and never drops it silently.

## 15. Cross-platform contract

The editor, local simulator, test runner, bundle viewer, and offline lessons
must pass the same fixtures on Windows, macOS, and Linux. GPU and hardware
features use capability detection. A CPU path remains correct, even when slower.
The web client can review and submit, but secret-bearing provider access stays
in a user-owned broker process.

## 16. Reliability and performance budgets

`p95` means at least 95 of 100 measured actions meet the target; `p99` means 99
of 100. Idempotency means safely repeating one request creates no extra work.

| Area | Numeric objective | Measurement window | Failure or degraded behaviour |
|---|---|---|---|
| Data loss | Zero acknowledged run intents or ledger transitions lost in 1,000 forced-crash points per release. | Every release candidate. | Recovery returns the last durable state and marks any unconfirmed provider outcome `unknown`. |
| Idempotency | Ten thousand repeats of one idempotency key create exactly one intent and one allowed transition. | Every G0+ release candidate. | A payload mismatch is rejected and linked to the original key. |
| Editor latency | p95 below 100 ms for edit, validate, and redraw on a 10,000-operation graph. | Ten-minute test on each published reference machine per release. | Large-graph mode reduces visual detail but preserves edits and truth. |
| Durable transition | p99 below 500 ms across 10,000 local transitions. | Each supported operating system per release. | The UI shows `saving` or `degraded`; it never reports completion early. |
| Offline use | Complete author, simulate, test, review, recover, and export journeys during an eight-hour network-disabled test. | Every G1+ release candidate. | Remote providers are unavailable; queued work stays local and clearly pending. |
| Cancellation | Owned local process tree stops within 2 seconds in at least 99 of 100 tests. | Every release candidate. | A survivor is named; remote state remains `cancel requested` until acknowledged. |
| Concurrency | An 8-core, 16 GB reference machine queues 10,000 intents and runs 8 local simulations without lost or mixed state. | Thirty-minute load test per G1+ release. | Backpressure starts before memory exhaustion and shows queue position. |
| Scale | A proof bundle up to 10 GB validates incrementally using no more than 1 GB extra memory. | One reference bundle per G2+ release. | Validation pauses safely and names the failed artifact. |
| Recovery | Metadata recovery objective is 15 minutes; a 10 GB object bundle restores and verifies within 1 hour. | Quarterly clean-machine drill on every supported system. | The old copy remains readable until the restored copy validates. |
| Provider degradation | Adapter outage is visible within 10 seconds and does not block local work. | Fault injection against every enabled adapter per release. | Circuit, simulator, history, and export remain available; remote work is disabled. |

## 17. Success measures

| Outcome | Target | Window and method |
|---|---|---|
| Independent reproduction | At least 90% of supported reference bundles reproduce within declared tolerance on a clean second system. | Monthly G2+ matrix across Windows, macOS, and Linux. |
| Semantic-loss detection | 100% of seeded unsupported or changed constructs are reported before submission. | Adapter mutation suite for every release. |
| Cost forecast | Median error at or below 10% and p95 at or below 25% where a provider supplies final cost facts. | Rolling last 100 completed paid or simulated billing cases. |
| True cancellation | At least 99% of owned local jobs stop within 2 seconds; 100% of remote jobs show confirmed or still-requested truth. | One hundred cancellation trials per release. |
| Portable proof | 100% of release reference bundles validate on all three desktop systems. | Every release candidate. |
| Accessibility | All six journeys complete by keyboard and screen reader with zero blocking WCAG 2.2 AA finding. | Quarterly test with automated checks plus human review. |
| Time to useful result | Median under 30 minutes from opening a template to a reviewed local experiment for trained users. | Quarterly usability study with at least eight users. |
| Honest result | 100% of releasable results include uncertainty, shots, source, environment, and known limits. | Release gate over every result fixture. |
| Real adoption | At least three independent teams complete two reviewed experiments each in one quarter before G2 closes. | Product telemetry with team consent plus interviews. |

Circuit count and raw agent-call count are not success measures.

## 18. Delivery plan

| Gate | Build | Tests | Safety and authority | UX proof | Windows, macOS, Linux | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| G0 (0–6 months), truthful foundation | Canonical graph, deterministic simulator, run ledger, crash recovery, and truthful status. | Parser, 100 golden circuits, 1,000 crash points, idempotency, and ledger invariants. | Remote and paid runs disabled; leases and secrets fail closed. | A user can import, simulate, inspect raw counts, cancel, recover, and export without hidden work. | Native build and the same golden/crash fixtures pass on all three. | Generic jobs/circuits fixtures convert through a reversible, reported migration. | From a clean machine, an operator runs one two-qubit experiment, kills it, recovers, and verifies proof. |
| G1 (6–18 months), useful product | Synchronized editor, notebook, tests, noise lab, lessons, and complete local journeys. | End-to-end journey, offline, accessibility, edit conflict, noise, performance, and recovery tests. | Agents remain draft-only; all work is local unless an adapter is explicitly enabled. | All six journeys show progress, completion, failure, recovery, and export in plain language. | Native UI and headless fixtures pass; signed packages are exercised on all three. | Program and notebook schema upgrades preserve hashes, undo, and old readable exports. | A new user completes a reviewed local experiment and restores a deleted draft without developer help. |
| G2 (18–30 months), trusted team product | Two hardware adapters, human leases, budget control, team review, and portable bundles. | Adapter contracts, overspend edges, cancel truth, concurrent review, tenant separation, and bundle verification. | Every remote run requires exact human approval; no agent sees secrets or approves itself. | Submission preview shows device, meaning changes, cost, residency, queue, and safe stop. | Provider mocks and clean bundle verification pass on all three; platform limits are stated. | Adapter and lease versions migrate with rollback; no old approval silently gains power. | An operator completes one sandbox remote run, cancels another, and verifies both histories independently. |
| G3 (30–42 months), category leader | Inspectable compiler, equivalence checks, fair benchmark studio, and reproduction workflow. | Pass mutation, equivalence, hidden-trial, statistics, clean-room reproduction, and scale tests. | Failed proof or missing baseline blocks claim release; independent reviewer authority is separate. | Reviewers can trace claim to every pass, trial, cost, failure, and limit without raw database access. | The same reproduction bundle reruns on all three and reports allowed numeric differences. | Old compiler plans remain readable and can be rechecked under the new tool version. | An external science and security review reproduces a benchmark and resolves every P0/P1 finding. |
| G4 (42–60 months), frontier network | Bounded co-design, site-owned federation, and verified learning mode. | Property, adversarial-limit, three-site custody, privacy, misconception, and long-run recovery tests. | Site owners and accountable humans approve every grant and real execution; federation cannot become global custody. | Search and federation show limits, sites, denials, spend, uncertainty, pause, and exit. | Local clients and verifier tools pass on all three; site adapters declare platform support. | Every site can export and revoke its bindings without deleting shared public artifacts. | Independent reviewers observe a synthetic three-site run with one denial, no raw-data escape, and a complete signed audit. |

## 19. Current truth and gap

The live product currently has generic `jobs` and `circuits` records. It has no
quantum circuit model, compiler, simulator, provider adapter, UI, or domain
tests. The safest first slice is QF-F0-01 through QF-F0-03 with a two-qubit
local simulator and golden-vector proof. Nothing in this sheet should be marked
present until source and fresh tests prove it.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Internal model | Typed canonical experiment graph, not provider JSON | Architecture decision and migrations |
| First execution | Deterministic CPU simulator | Passing replacement fixtures |
| First language adapter | OpenQASM 3.1, with explicit OpenQASM 3.0 import compatibility | Product decision |
| Hardware access | Brokered lease plus human approval | Founder and safety review |
| Result truth | Raw observations immutable; analyses versioned | Architecture decision |
| Paid runs | Disabled until atomic budget reservation exists | Finance and trust gate |
| AI action | Draft and search only inside explicit limits | Safety review |
| Delete | 30-day recovery bin | Legal-retention exception |

## 21. Definition of category-defining done

- [ ] A novice can learn and an expert can inspect the same experiment.
- [ ] One program can be compared across local simulation and several providers.
- [ ] No run, cost, compiler rewrite, failed trial, or uncertainty is hidden.
- [ ] Independent bundles validate and reproduce on all three desktop systems.
- [ ] Agents stay inside exact capability, budget, device, and time leases.
- [ ] Accessibility, recovery, security, and scientific-review gates pass.
- [ ] The product clearly states what each result does and does not prove.
