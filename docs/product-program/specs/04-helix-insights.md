# HelixInsights — decision memory for trustworthy analytics

```yaml
product: HelixInsights
catalog_order: 4
status: target-state-spec
horizon: 60 months
current_maturity: prototype
primary_users: [analysts, operators, product leaders, data stewards, reviewers]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

> **Target-state rule:** Sections 1–18 and 20–21 describe the product to build.
> Section 19 records what the live source proves today.

## 1. Category claim

HelixInsights is a decision-memory workspace that lets a team turn governed data
into clear decisions, then prove which data, rules, people, and outcomes shaped
each decision.

## 2. Five-year destination

The useful product is a local-first analytics workspace for datasets, governed
metrics, queries, charts, reports, alerts, and decision records. The
category-defining advantage is one trace from source data to metric, analysis,
human decision, action, and measured outcome. The frontier capability is a safe
decision laboratory that compares forecasts and counterfactual scenarios while
showing uncertainty and failed assumptions. Humans keep authority over metric
meaning, sensitive access, published claims, alerts that trigger action, and
every real-world decision.

## 3. Users and hard jobs

- **Analysts** need to answer a question without losing source meaning. They
  fear a polished chart built from stale, wrong, or changed data.
- **Operators** need live signals with clear causes and owners. They fear a
  silent pipeline or alert that looks healthy while it is late.
- **Product and business leaders** need to choose and later learn from the
  result. They fear confident numbers with no uncertainty or decision record.
- **Data stewards** need to control definitions, quality, purpose, and access.
  They fear copies that escape governance.
- **Reviewers and auditors** need to reproduce a result. They fear an export
  that contains a conclusion but not its inputs and transformations.

## 4. Product laws

1. Every number names its dataset version, metric definition, and calculation.
2. A metric never changes meaning in place; a meaning change creates a version.
3. Missing, late, estimated, filtered, and corrected data stay visible.
4. Forecasts and scenarios are labelled as estimates, never facts.
5. Agents may draft analysis, but humans approve metric meaning and decisions.
6. Access follows purpose, field sensitivity, tenant, and exact capability.
7. Core analysis, review, and export work without one cloud vendor.
8. Every important result has portable data, query, chart, and proof formats.
9. Long work reports real stages, checks, last activity, and cancellation state.
10. Honest failure is better than a partial result shown as complete.

## 5. Scope boundaries

HelixInsights owns dataset registration, data contracts, metric definitions,
analysis, visualisation, reports, alerts, decision records, and outcome review.
HelixCore owns identity, policy, jobs, audit, capabilities, objects, billing,
and operations. HelixCode owns general source control and CI. HelixCapital owns
the accounting book. Aether may provide proof and capability brokering through
an adapter. Insights is not a warehouse, a general spreadsheet, or a tool that
makes business decisions by itself. Statistical output does not replace an
accountable expert when the choice affects people, money, rights, or safety.

## 6. Signature experiences

1. **Bring in a governed dataset.** **Entry:** a steward chooses a file,
   database view, or adapter. **Visible progress:** profiling shows rows,
   columns, rejected values, and time remaining. **Human decision:** the steward
   confirms meaning, sensitivity, owner, and refresh rule. **Completion proof:**
   a signed dataset version and quality report are created. **Failure and
   recovery:** rejected rows are quarantined and the import can resume.
   **Export:** data, schema, mappings, and provenance leave in one bundle.
2. **Define one trusted metric.** **Entry:** an analyst opens the metric studio.
   **Visible progress:** sample results and dependency checks update as rules are
   written. **Human decision:** a metric owner approves name, formula, grain,
   filters, and effective date. **Completion proof:** tests and approval are
   linked to an immutable version. **Failure and recovery:** breaking changes
   create a draft version, not a silent edit. **Export:** definition, tests, and
   lineage use documented formats.
3. **Answer a hard question.** **Entry:** a user asks in plain language or opens
   a notebook. **Visible progress:** the system shows the plan, sources, query
   stages, and checks. **Human decision:** the user approves sensitive joins and
   final interpretation. **Completion proof:** the answer links every chart and
   claim to executable analysis. **Failure and recovery:** a failed stage keeps
   prior valid work and offers a safe retry. **Export:** notebook, query, data
   sample, chart, and evidence bundle are portable.
4. **Watch a live operation.** **Entry:** an operator opens a board or alert.
   **Visible progress:** freshness, lag, quality, and last successful signal are
   always shown. **Human decision:** the operator confirms an alert or changes
   its routing. **Completion proof:** acknowledgement and response are recorded.
   **Failure and recovery:** late data changes the state to delayed, not zero.
   **Export:** the time series and incident record can be downloaded.
5. **Make and revisit a decision.** **Entry:** a leader starts a decision record
   from an analysis. **Visible progress:** open evidence, objections, and missing
   approvals are visible. **Human decision:** named people choose, defer, or
   reject. **Completion proof:** the signed record includes assumptions and
   expected outcomes. **Failure and recovery:** the decision can be reversed
   without deleting history. **Export:** a complete decision packet is portable.
6. **Test a future scenario.** **Entry:** a user copies a trusted baseline into
   the scenario lab. **Visible progress:** assumptions, runs, uncertainty, and
   cost are live. **Human decision:** a person chooses which scenarios may inform
   a real action. **Completion proof:** outputs include sensitivity and failed
   assumptions. **Failure and recovery:** cancelled runs retain their state and
   can restart from a safe checkpoint. **Export:** model, inputs, seeds, outputs,
   and limits are included.
7. **Verify someone else's result.** **Entry:** a reviewer opens an evidence
   bundle. **Visible progress:** signature, schema, dependency, and rerun checks
   stream separately. **Human decision:** the reviewer accepts, disputes, or
   asks for more evidence. **Completion proof:** an independent difference
   report is signed. **Failure and recovery:** unavailable sources are named and
   never replaced silently. **Export:** the review and its limits are portable.

## 7. Capability map

### F0 — foundation

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| INS-F0-001 | G0 | Source and schema | Versioned dataset | Stable ID is not a folder path; source bytes never change in place | Steward approves sensitivity | Hash, schema, actor, time | `rejected` or `quarantined` | WHEN the same source is imported twice, the system SHALL keep one content identity and two import attempts. |
| INS-F0-002 | G0 | Dataset version and rules | Quality result | Missing and failed checks stay distinct | Steward sets rules | Check inputs and results | `failed_quality` | WHEN a rule fails, the system SHALL block trusted status and show the failed rows. |
| INS-F0-003 | G0 | Write command and idempotency key | Domain record plus event | Record and event commit together or neither commits | Service policy | Transaction and replay proof | `not_committed` | WHEN a crash occurs at any write step, recovery SHALL produce zero or one complete write, never a record without its event. |
| INS-F0-004 | G0 | Long job | Durable job timeline | Heartbeats are real; completion follows durable output | User may cancel own job | Stage, heartbeat, output hash | `delayed`, `failed`, `cancelled` | WHILE work runs, the UI SHALL show last activity and a working cancel action. |

### F1 — useful product

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| INS-F1-001 | G1 | Formula, grain, filters, units | Metric version | Meaning never changes in place | Metric owner approves publish | Definition, tests, approval | `draft` or `invalid` | WHEN formula meaning changes, the system SHALL create a new version and preserve old results. |
| INS-F1-002 | G1 | Question, datasets, parameters | Reproducible analysis | Source and query versions are pinned | Analyst runs; sensitive joins need approval | Query plan, logs, result hash | `partial` or `failed` | WHEN an analysis completes, a second machine SHALL reproduce the declared result within its stated tolerance. |
| INS-F1-003 | G1 | Analysis result | Accessible chart and table | Chart and table express the same values | Author publishes | Render checks and data link | `unpublished` | WHEN colour is removed, the view SHALL still communicate every state by text or shape. |
| INS-F1-004 | G1 | Metrics and schedule | Versioned report | Late data is labelled; old snapshots do not change | Owner approves recipients | Snapshot, freshness, delivery log | `late` or `delivery_failed` | WHEN data misses its freshness limit, the report SHALL say delayed and SHALL NOT show it as current. |

### F2 — category leader

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| INS-F2-001 | G2 | Source, transforms, outputs | Lineage graph | Every edge names a transformation | System records; steward may correct by new version | PROV mapping and hashes | `lineage_incomplete` | WHEN any result is selected, the user SHALL reach its raw source and transformation code. |
| INS-F2-002 | G2 | Threshold, evidence rule, route | Alert event | Missing data cannot become a healthy value | Owner approves activation | Evaluation and delivery log | `delayed`, `suppressed`, `failed` | WHEN input is late, the alert SHALL enter delayed state and notify its owner. |
| INS-F2-003 | G2 | Analysis, options, approvers | Decision record | Evidence and dissent are append-only | Named human decides | Signatures, evidence links, outcome date | `waiting`, `deferred`, `reversed` | WHEN a decision is reversed, the system SHALL preserve both decisions and their reasons. |
| INS-F2-004 | G3 | Decision and outcome measures | Learning review | Outcome cannot rewrite the original forecast | Review owner closes | Expected versus observed report | `outcome_unknown` | WHEN the review date arrives without data, the system SHALL show unknown, not success or failure. |

### F3 — advanced category leadership

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| INS-F3-001 | G3 | Model, baseline, assumptions | Forecast with intervals | Point estimate is never shown alone | Expert approves publication | Training window, tests, uncertainty | `not_validated` | WHEN a forecast is published, the system SHALL show its interval, horizon, and back-test error. |
| INS-F3-002 | G3 | Baseline and changed assumptions | Scenario comparison | Scenario data never becomes observed fact | Human selects action | Seeds, inputs, sensitivity | `inconclusive` | WHEN one assumption drives the result, the system SHALL mark that sensitivity before approval. |
| INS-F3-003 | G3 | Approved query and distributed sources | Federated aggregate | Raw protected rows stay with their custodian | Each custodian grants exact purpose | Grant, query, aggregate proof | `denied` or `insufficient_group` | WHEN a privacy floor is not met, the system SHALL return no aggregate and record the denial. |
| INS-F3-004 | G3 | Decision history and outcomes | Suggested reusable playbook | Suggestion never executes and shows contrary cases | Human approves any use | Similarity inputs and outcome record | `low_confidence` | WHEN evidence is weak or mixed, the system SHALL say so and SHALL NOT rank one action as best. |

### F4 — frontier network

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| INS-F4-001 | G4 | Custodian-approved metrics, definitions, queries, and disclosure grants | Cross-owner decision comparison with signed aggregates | Raw protected rows remain with each custodian; unlike metric meanings are never merged | Every custodian approves purpose, fields, privacy floor, and expiry | Grants, semantic mappings, local query proofs, aggregate hashes | `semantic_mismatch` or `privacy_floor_failed` | WHEN two metrics have different grain or meaning, the network SHALL keep them separate until accountable owners approve a mapping. |
| INS-F4-002 | G4 | Opted-in decision records, outcome measures, counterevidence, and study protocol | Privacy-protected outcome-learning library | No individual or organisation is ranked; a suggested pattern never executes and must show contrary outcomes | Contributors opt in; research governance approves protocol; human decides any use | Protocol, cohort test, privacy proof, supporting and contrary outcomes | `insufficient_diversity` or `inconclusive` | WHEN the approved cohort or diversity floor is not met, the system SHALL publish no reusable pattern. |
| INS-F4-003 | G4 | Approved models, assumptions, constraints, and node capability manifests | Multi-party scenario room with versioned local results | Each party controls its model and data; one node cannot alter another node's assumptions or authority | Every party approves its contribution and the shared comparison; no agent approves action | Node grants, input versions, local run proofs, comparison, withdrawal events | `node_unavailable` or `comparison_incomplete` | WHEN one node withdraws, new runs SHALL exclude it, revoke its bindings, and preserve the other parties' valid evidence. |

## 8. Domain model

`Workspace` owns `DataSource`, `Dataset`, `DatasetVersion`, `FieldDefinition`,
`DataContract`, `SensitivityLabel`, `QualityRule`, `QualityRun`, `RejectedRow`,
`Metric`, `MetricVersion`, `Query`, `AnalysisRun`, `Notebook`, `Chart`,
`Dashboard`, `Report`, `AlertRule`, `AlertEvent`, `Decision`, `DecisionOption`,
`Approval`, `Dissent`, `OutcomeMeasure`, `OutcomeReview`, `Scenario`,
`Forecast`, and `EvidenceBundle`. Dataset, metric, query, report, decision, and
model records use immutable versions. Tenant and owner are explicit fields.
Raw data retention follows its source policy; proof and decision records keep
longer policy-controlled retention. Relationships are typed, not hidden in a
generic metadata field.

## 9. System architecture

- A Rust domain engine validates datasets, metrics, decisions, and lifecycle
  transitions.
- Application services handle imports, queries, reports, alerts, decisions,
  scenarios, and exports.
- PostgreSQL is the durable source of truth; object storage holds large
  versions and bundles; an embedded local profile supports one user offline.
- A transactional outbox publishes durable domain events after the same commit.
- Sandboxed workers run queries, profiling, forecasts, renders, and exports.
- Adapters isolate warehouses, files, spreadsheets, stream sources, and model
  runtimes. Each adapter reports capability and data loss.
- HelixCore supplies identity, policy, audit, capabilities, jobs, objects,
  billing, operations, and the stable project identity.
- Offline edits use an operation log. Conflicts are shown to a person; the
  system never uses last-write-wins for metric or decision meaning.

## 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Visible progress, check, stop, reverse |
|---|---|---|---|---|---|
| Analyst agent | Approved datasets; query, chart, and statistics tools | Queries, charts, explanations | Sensitive join, publish, or decision link | Change source data or hide failed checks | Streams plan and checks; sandbox rerun verifies; cancel stops worker; drafts can be discarded. |
| Steward agent | Schemas, quality history, catalog | Contracts and quality rules | Rule activation and sensitivity change | Grant itself access or lower a label | Shows sampled impact; conformance tests check; version rollback restores prior rule. |
| Decision-review agent | Approved evidence and past outcomes | Options, objections, follow-up measures | Every decision and action | Choose for the user or erase dissent | Shows sources and uncertainty; human signs; reversal creates a linked event. |
| Scenario agent | Approved models and bounded inputs | Scenario plans and sensitivity tests | Expensive run or use in a real action | Present a scenario as observed fact | Shows run stages and budget; tests compare baselines; cancellation checkpoints safely. |

Agents receive exact, time-limited capability leases. They never receive raw
secrets. Their output is untrusted until domain validation and named checks pass.

## 11. Trust, safety, and privacy

Access control combines tenant, resource, role, purpose, field sensitivity,
and exact capability. Database policies enforce tenant separation. Sensitive
fields are encrypted in transit and at rest, with separate keys where the
deployment supports it. Consent records say who may use which data, for what
purpose, and until when. Exports apply the same policy as screens. Data
residency is selected per workspace and checked before a job moves data.

Delete sends user-created datasets, analyses, dashboards, reports, and decisions
to a recoverable 30-day bin by default. Restore keeps identity and links.
Permanent deletion is a separate, re-authenticated, explicit, audited act and is
blocked by a legal hold. Abuse controls cover bulk export, inference attacks,
prompt injection in imported text, malicious files, alert storms, and model
poisoning. Incident recovery can revoke leases, isolate an adapter, rotate keys,
rebuild from signed events, and tell affected users what is known and unknown.

## 12. Proof and audit

Important evidence includes source identity, content hash, schema, quality
checks, transformation code, metric version, query plan, environment, actor,
approval, report recipients, alert evaluation, decision, dissent, and outcome.
Events are signed and chained, with all-or-nothing writes and crash-recovery
tests. An independent verifier can check signatures, hashes, schemas, stated
tests, and a permitted rerun. The evidence does not prove that a source was true,
that a statistical method was wise, or that a human decision was good.

Aether is the preferred proof and capability provider through provider-neutral
interfaces. A local signer, local verifier, and local least-authority lease
service remain the fallback, so Aether is never a hard runtime dependency.

## 13. UX system

The main surfaces are Home, Catalog, Metrics, Analyze, Boards, Reports, Alerts,
Decisions, Scenarios, Evidence, and Recovery. Basic views lead with the question,
answer, freshness, uncertainty, and next human choice. Details reveal lineage,
queries, tests, and raw data only when requested. Keyboard and touch actions have
equivalent paths; all visual states also use text and shape. The target is
[WCAG 2.2 Level AA](https://www.w3.org/TR/WCAG22/).

Every slow action shows named stages, completed work, elapsed time, last real
heartbeat, likely next step, and a cancel control. Completion creates an in-app
record and an optional desktop notification; it never relies only on a toast.
Selecting, moving, or joining items shows a check mark and impact preview. A
meaning-changing move asks for confirmation. Undo is immediate where safe;
destructive work enters Recovery. Empty states explain what to add and why.
Errors use plain language, preserve work, name what failed, and offer the next
safe action.

## 14. Interoperability and standards

- [W3C PROV-O](https://www.w3.org/TR/prov-o/) maps entities, activities, and
  agents for portable provenance. A simple export may lose Helix-specific policy
  decisions and quality details.
- [W3C DCAT 3](https://www.w3.org/TR/vocab-dcat-3/) exchanges catalog, dataset,
  service, distribution, and version descriptions. It does not carry every
  internal metric or access rule.
- [OpenLineage](https://openlineage.io/docs/spec/) is an adapter for run, job,
  dataset, and facet events in data systems. Custom facets are declared and may
  be ignored by another tool.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) sets the web accessibility target.

CSV, JSON, Arrow, Parquet, SQL, and spreadsheet support are data-format
adapters, not the internal truth model. Every import previews type, null,
timezone, unit, rounding, and identifier losses before commit. Standards and
profiles are version-pinned per adapter and upgraded through contract tests.

## 15. Cross-platform contract

Windows, macOS, and Linux run the same domain, migration, proof, recovery, and
export fixtures. The web client supports review and managed work; desktop adds
offline files, local workers, notifications, and user-owned credentials. The CLI
supports scripted import, check, export, and verify, not every visual authoring
task. Containers support self-hosted server and workers. Offline mode supports
catalog browsing, local datasets, metrics, analysis, decisions, and queued sync
within configured storage limits. OS key stores, notifications, GPU, and file
watching use capability detection and have a safe manual or CPU fallback.

## 16. Reliability and performance budgets

- Acknowledged domain writes have a data-loss budget of zero; forced-crash tests
  run on every supported database path.
- Common catalog, metric, and decision reads complete under 300 ms at p95 over a
  rolling 30-day window for a 100-user workspace with 100,000 catalog records.
- A new long job shows its first durable stage within 2 seconds; a heartbeat is
  no older than 5 seconds while local work is active.
- Local cancellation is accepted within 2 seconds and stops the worker within 30
  seconds; remote work stays `cancel_requested` until the adapter confirms.
- Import and event commands are idempotent for at least 30 days by key.
- Concurrent metric edits never overwrite silently; a conflict is raised before
  publication.
- A single-user offline workspace supports 30 days and 100 GB of queued work;
  limits are visible before they are reached.
- Managed metadata has a recovery point of zero committed events and a recovery
  time target of 1 hour; self-hosted documented recovery target is 4 hours.
- If models, Aether, a warehouse, or notifications are down, local analysis,
  audit, and export continue with the missing capability clearly marked.

## 17. Success measures

- Median time from a question to a reviewed, reproducible answer.
- Share of published metrics with an owner, tests, lineage, and no overdue data.
- Share of decisions reviewed against their stated outcome on time.
- Number of silent-stale incidents; the target is zero per rolling quarter.
- Independent bundles that validate on another supported OS.
- Task completion and serious issue counts in keyboard, screen-reader, zoom, and
  touch tests against the declared WCAG 2.2 AA scope.
- Restore success from the 30-day bin and recovery drills.
- User trust: people can correctly explain source, freshness, uncertainty, and
  what the proof does not show.
- Paid team retention and value from saved review time, not chart or agent-call
  counts.

## 18. Delivery plan

| Gate | Build | Test | Safety | UX | Cross-platform | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Stable dataset identity, atomic ledger, jobs, recovery | Domain, concurrency, crash, signature tests | Tenant and capability policy; secret scan | Honest states, job timeline, Recovery | Rust and packaging CI on Windows, macOS, Linux | Import current datasets/metrics with dry-run report | Fresh install, crash, restore, verify, and uninstall record |
| **G1 — Useful single-player product (6–18 months)** | Catalog, metric studio, analysis, charts, reports | Golden calculations and end-to-end journeys | Field labels and export policy | Keyboard-first complete analyst flow | Desktop, web, CLI, container, offline fixtures | Versioned data and metric importer | Fresh source-to-report proof on each OS |
| **G2 — Trusted team product (18–30 months)** | Alerts, lineage, approvals, decision records | Multi-user race, permission, delivery tests | Purpose grants, tenant penetration review | Review queues, notifications, selection checks | Team sync and degraded network matrix | Team, role, and schedule migrations | Fresh team decision and recovery drill |
| **G3 — Category leader (30–42 months)** | Outcome learning, forecasts, adapter kit | Back-tests, adapter conformance, scale tests | Model, inference, and export threat review | Uncertainty and lineage comprehension tests | Hardware and browser support matrix | Adapter version and model migrations | External replay and accessibility review |
| **G4 — Frontier network (42–60 months)** | Build INS-F4-001 cross-owner comparisons, INS-F4-002 outcome-learning library, and INS-F4-003 multi-party scenario rooms | Semantic mismatch, privacy floor, node withdrawal, poisoning, replay, and contrary-evidence tests | Independent privacy, research-governance, security, and misuse review; no ranking or autonomous action | Custodian grant, human comparison, uncertainty, withdrawal, and honest no-result journeys | Mixed Windows/macOS/Linux nodes prove local custody, signed aggregates, offline denial, and safe degradation | Add/remove a custodian or project, revoke selected capabilities, and re-run comparisons without identity or shared-secret loss | Independent three-node compare, learn, withdraw, disaster-recover, export, and verify exercise with all F4 evidence |

A gate closes only when these checks run fresh against the release candidate.
Old reports and a successful command with skipped work do not close a gate.

## 19. Current truth and gap

The live Rust source has real `datasets`, `metrics`, and numeric metric-point
records. It checks that numeric values are finite. This is a meaningful backend
prototype, not only a name. It does not yet provide governed definitions,
quality contracts, lineage, queries, aggregation, dashboards, reports, alerts,
decisions, models, a product UI, or domain tests. Its server wiring also shares
the repository's current application-state compile problem. Product writes and
audit or billing writes are not one atomic operation.

The largest gap is a trustworthy path from dataset version to metric version to
visible result. The safest first vertical slice is INS-F0-001 through
INS-F1-003: import one CSV into temporary test state, approve one metric, render
one accessible chart and table, export its proof, then recover it after a forced
crash. No production state should be used in development tests.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Identity | Stable UUID/URN product, workspace, dataset, and metric IDs; never folder paths | Architecture decision plus migration proof |
| Durable store | PostgreSQL with an embedded local profile; object storage for large versions | Architecture review and three-OS recovery proof |
| Write integrity | Domain write, outbox event, and idempotency result commit in one transaction | Founder-approved integrity exception |
| Metric change | Immutable version with effective date and tests | Product and data-governance review |
| Agent authority | Draft and bounded analysis only; no self-grant or final decision | Safety review and founder approval |
| Proof provider | Aether preferred through an adapter; signed local fallback always works | Provider-neutrality review |
| Secrets | User-owned capability broker; agents never receive values | Security review |
| Delete | Recoverable 30-day bin; permanent delete is separate and audited | Legal-retention exception |
| Accessibility | WCAG 2.2 AA target plus keyboard, screen-reader, zoom, and touch tests | Documented accessibility review |
| Platform | Windows, macOS, Linux, web; no OS-only critical path | Founder-approved scope decision |
| First slice | Dataset → metric → accessible chart → evidence → crash recovery | Product decision with equal or stronger proof |
| Federation and paid automation | Off until G4/G2 safety and billing gates close | Founder approval |

## 21. Definition of category-defining done

- [ ] A user completes all seven signature journeys with real data.
- [ ] Every published number is traceable to versioned data, rules, and checks.
- [ ] Decisions preserve evidence, dissent, approval, reversal, and outcome.
- [ ] Forecasts and scenarios state uncertainty and never pose as observed fact.
- [ ] Atomic writes and crash recovery prove zero loss for acknowledged records.
- [ ] Independent proof validates without trusting the HelixInsights server.
- [ ] Agents stay inside exact leases and cannot retrieve secrets or self-approve.
- [ ] WCAG 2.2 AA scope passes automated and human accessibility review.
- [ ] Windows, macOS, Linux, web, CLI, container, and offline limits are proven.
- [ ] Portable exports preserve meaning or name every known loss.
- [ ] The 30-day bin, permanent deletion, legal hold, and restore are tested.
- [ ] External security, privacy, model, and data-governance reviews are closed.
- [ ] The product states clearly what its evidence and analysis do not prove.
