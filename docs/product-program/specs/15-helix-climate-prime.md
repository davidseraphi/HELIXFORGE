# HelixClimatePrime — climate risk, adaptation, and transition system

```yaml
product: HelixClimatePrime
catalog_order: 15
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [climate risk teams, adaptation planners, sustainability leaders, public and financial decision makers]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixClimatePrime is an evidence system that connects real assets and communities to
climate hazards, emissions, scenarios, assumptions, decisions, actions, outcomes, and
proof without presenting a scenario as a prediction or a model score as a decision.

## 2. Five-year destination

The useful product is an inventory, asset and exposure map, scenario workspace,
adaptation register, transition-plan tracker, disclosure room, and evidence viewer.
The leading product keeps physical and transition risk models comparable, shows data
lineage and uncertainty, and tracks whether actions change measured outcomes. The
frontier product can explore compound hazards, supply networks, distributional
effects, and robust portfolios across many possible futures. It never allocates
capital, trades, publishes a disclosure, changes a public plan, or makes a safety
decision without the authorized human and governance process.

## 3. Users and hard jobs

- A risk team needs asset-level exposure and fears precise numbers built from weak
  spatial or model assumptions.
- An adaptation planner needs options, dependencies, equity, costs, and outcomes
  across long time horizons.
- A sustainability team needs reproducible emissions and transition records, not a
  spreadsheet that loses factors and boundaries.
- A public decision maker needs community impacts and uncertainty, not only asset
  value.
- A finance or audit reviewer needs a trace from disclosure to source and approval.
- A data owner needs license, residency, sensitivity, and downstream-use controls for
  commercial and community data.

## 4. Product laws

1. A scenario is a conditional possible future, never a forecast or promise.
2. Hazard, exposure, vulnerability, consequence, and decision are separate records.
3. Every metric names boundary, period, unit, method, factor/model version, source,
   coverage, and uncertainty.
4. Model ensembles and disagreement remain visible; no hidden averaging creates false
   certainty.
5. Physical risk, transition risk, emissions, adaptation, resilience, and disclosure
   are linked but not collapsed into one score.
6. Distributional impacts and affected communities appear beside financial results.
7. Agents may gather, calculate, compare, and draft; humans approve assumptions,
   action, capital, public claims, and disclosure.
8. A failed source, stale factor, or missing asset becomes `unknown`, not zero.
9. Critical journeys are accessible, portable, and independently verifiable across
   platforms; no model, data vendor, or proof provider is a hard dependency.

## 5. Scope boundaries

ClimatePrime owns authorized data lineage, organizational and asset boundaries,
emissions inventories, hazards, exposure and vulnerability models, scenario analysis,
adaptation and transition options, action tracking, disclosure drafts, and proof.
HelixCore owns shared identity, policy, audit, capabilities, jobs, objects, billing,
and operations; Aether is reached through a provider-neutral proof interface with a
local fallback. It
does not replace emergency management, engineering design, financial advice,
environmental assessment, statutory reporting, community consent, or scientific
peer review. It does not trade, allocate funds, operate infrastructure, publish
claims, or certify compliance. Those actions require qualified humans and the proper
external systems.

## 6. Signature experiences

1. **Build a governed climate baseline.** Entry: a team defines organization, assets,
   communities, time, purpose, and authority. Progress: source identity, spatial match,
   boundaries, licenses, gaps, and quality appear. Human decision: data owners accept
   the baseline. Proof: every transform and exclusion is recorded. Failure and
   recovery: bad joins or unknown licenses are quarantined. Export: a baseline manifest
   and approved geospatial bundle.
2. **Produce a reproducible emissions inventory.** Entry: a user selects reporting
   boundary and period. Progress: activity data, factors, scope/category, conversions,
   estimates, gaps, and review status remain visible. Human decision: inventory owner
   accepts methods and exclusions. Proof: every total drills to source and factor.
   Failure and recovery: missing data stays estimated or unknown, never zero. Export:
   inventory, calculation ledger, and assurance packet.
3. **Explore physical climate risk.** Entry: a team chooses assets, hazards, horizons,
   scenarios, and models. Progress: data staging, downscaling, exposure, vulnerability,
   consequence, ensemble spread, and sensitivity stream visibly. Human decision: a
   risk owner accepts the interpretation. Proof: versions, spatial operations, and
   uncertainty are kept. Failure and recovery: partial model failure remains visible
   and resumable. Export: a scenario result crate.
4. **Design an adaptation pathway.** Entry: a planner selects a risk and objectives.
   Progress: options, prerequisites, costs, co-benefits, equity, maladaptation risk,
   lead time, triggers, and monitoring form a pathway. Human decision: authorized
   leaders and affected stakeholders select actions. Proof: reasons and rejected
   options are recorded. Failure and recovery: changing assumptions reopens the
   decision. Export: pathway and safety-case bundle.
5. **Track a transition plan.** Entry: a team records a human-approved target and
   boundary. Progress: levers, dependencies, cost, emissions impact, policy/market
   assumptions, ownership, and evidence are shown. Human decision: management approves
   commitments and capital. Proof: target versions and actual outcomes remain linked.
   Failure and recovery: slippage and target changes are visible, not rebased away.
   Export: transition-plan evidence.
6. **Prepare a defensible disclosure.** Entry: an authorized preparer selects a
   framework and period. Progress: required facts, source coverage, controls,
   estimates, review, and unresolved gaps appear. Human decision: responsible officers
   approve exact text and numbers. Proof: claim-to-evidence links and sign-off are
   signed. Failure and recovery: a stale source or changed calculation blocks publish
   status. Export: disclosure draft, tagged data, and proof, never auto-publication.

## 7. Capability map

F0 is foundation, F1 is the useful product, F2 is the trusted-team product, F3 is
advanced category leadership, and F4 is the frontier network. Every row inherits this full contract: its invariants are the
product laws and scenario/forecast/observation truth boundaries; authority is the
exact named human and policy in Sections 10–11; evidence is input/output hashes,
versions, actor, assumption, decision, and ledger event; failure is a durable
`blocked`, `failed`, `unknown`, partial, or quarantine state with retry or recovery;
and test acceptance includes denial, model failure, source loss, recovery, and
cross-platform cases in addition to the row's named check. The row names its
domain-specific inputs, output, and strongest acceptance test.

| ID | First gate | Capability contract |
|---|---|---|
| HCL-F0-01 | G0 | **Stable organization, asset, and place identity.** Inputs are approved registers and geometries; outputs are versioned identities independent of folder path. Acceptance: duplicate, spatial mismatch, and identity migration fixtures cannot silently merge records. |
| HCL-F0-02 | G0 | **Climate data provenance ledger.** Inputs are source, transform, model, review, decision, and disclosure events; output is an append-only graph. Acceptance: forced crash and concurrent writers lose no acknowledged event. |
| HCL-F0-03 | G0 | **Metric contract engine.** Inputs are boundary, period, unit, method, factor/model version, source, coverage, and uncertainty; output is a typed metric or quarantine. Acceptance: unknown data cannot be coerced to zero and incompatible totals cannot add. |
| HCL-F1-01 | G1 | **Emissions inventory workbench.** Inputs are activity data, factors, scopes/categories, and controls; output is a versioned inventory with estimates and gaps. Acceptance: every total reconciles to source and calculation rows. |
| HCL-F1-02 | G1 | **Asset and exposure map.** Inputs are approved asset/place geometry and hazard layers; output is a time- and resolution-aware intersection with quality. Acceptance: coordinate, coverage, license, and spatial-resolution loss are visible. |
| HCL-F1-03 | G1 | **Scenario registry.** Inputs are official or reviewed scenario datasets; output is an immutable scenario version with narrative, variables, horizon, and limits. Acceptance: no result can omit the exact scenario and model chain. |
| HCL-F2-01 | G2 | **Physical-risk analysis.** Inputs are hazard, exposure, vulnerability, consequence, scenario, and uncertainty; output is distributions and sensitivities. Acceptance: model disagreement and missing vulnerability remain visible. |
| HCL-F2-02 | G2 | **Transition-risk analysis.** Inputs are policy, technology, market, legal, and business assumptions; output is conditional impacts and dependencies. Acceptance: an agent cannot choose an assumption set or present it as a forecast. |
| HCL-F2-03 | G2 | **Adaptation and transition action register.** Inputs are human-approved options, owners, costs, dependencies, triggers, safeguards, and indicators; output is a versioned pathway. Acceptance: an action cannot be `effective` without measured outcome evidence and review. |
| HCL-F3-01 | G3 | **Compound-risk twin.** Inputs are multiple hazards, cascades, networks, scenarios, and local vulnerability; output is a bounded scenario graph. Acceptance: correlation is not labeled causation and unmodeled dependencies remain named. |
| HCL-F3-02 | G3 | **Disclosure evidence room.** Inputs are framework requirements, approved metrics, governance, controls, and claims; output is a reviewable draft and proof map. Acceptance: agents cannot publish or sign, and stale evidence revokes ready status. |
| HCL-F3-03 | G3 | **Independent climate proof bundle.** Inputs are licensed source references, transforms, model containers, outputs, decisions, and redactions; output is a portable verification crate. Acceptance: another platform can validate integrity and rerun allowed calculations. |
| HCL-F3-04 | G3 | **Robust portfolio studio.** Inputs are approved goals, constraints, scenarios, options, and equity rules; output is a set of robust portfolios and trade-offs. Acceptance: no single optimum is selected and capital action is unavailable. |
| HCL-F3-05 | G3 | **Community and distributional impact layer.** Inputs are consented local data and reviewed indicators; output shows who may benefit or bear risk. Acceptance: small-group privacy and community-governance checks pass before release. |
| HCL-F3-06 | G3 | **Living transition and adaptation safety case.** Inputs are assumptions, actions, incidents, outcomes, model drift, and reviews; output is a structured argument with open risks. Acceptance: independent scientific and governance reviewers, not the system, accept it. |
| HCL-F4-01 | G4 | **Federated climate evidence exchange.** **Input:** organization-approved metric, source, transform, model, scenario, proof, license, purpose, and expiry manifests. **Output:** a revocable cross-organization graph of comparable evidence, disagreement, coverage, and uncertainty. **Invariant:** raw restricted data stays in local custody; observed, estimated, forecast, and scenario truth remain distinct; the exchange cannot publish a disclosure or trigger policy, legal, capital, procurement, or physical action. **Authority:** each organization's named data and scientific stewards approve contribution and withdrawal, and human reviewers accept shared mappings. **Evidence:** source and method hashes, boundary, units, review, mapping, access, use, dispute, expiry, and revocation. **Failure:** incompatible boundary, unit, license, model, or stale review quarantines the comparison instead of producing a total. **Acceptance:** ten independent nodes process 100,000 unit, boundary, source-loss, join, and revoke cases with zero invalid aggregation or restricted-data escape. |
| HCL-F4-02 | G4 | **Cross-organization transition dependency observatory.** **Input:** human-approved action pathways, infrastructure and supply dependencies, scenarios, thresholds, owners, safeguards, and disclosure policy. **Output:** a versioned map of shared dependencies, conflicts, sensitivities, and coordination options. **Invariant:** the observatory cannot select a pathway, allocate capital, set policy, sign a disclosure, procure, schedule, or control infrastructure. **Authority:** each organization approves only its own facts and commitments; a named multi-party human forum records shared conclusions. **Evidence:** pathway versions, assumptions, dependency sources, simulations, conflicts, owner decisions, dissent, and revocations. **Failure:** stale assumptions, missing owner, contradictory dependency, or withdrawal blocks the affected conclusion and leaves alternatives visible. **Acceptance:** 10,000 synthetic network-change and cascading-failure cases preserve every local veto, expose all seeded hidden dependencies, and create zero external commitment or actuation. |
| HCL-F4-03 | G4 | **Community-governed adaptation learning network.** **Input:** community-approved adaptation questions, consented indicators, local context, outcome evidence, distributional rules, safeguards, and withdrawal terms. **Output:** cross-community evidence about outcomes, limits, distribution, and transferable questions. **Invariant:** small-group privacy and local meaning remain protected; the network cannot select, fund, permit, contract, deploy, or operate an adaptation project. **Authority:** each community's named governance body controls participation, interpretation, disclosure, and withdrawal, with independent scientific review. **Evidence:** consent, indicator definitions, context, cohort checks, analyses, dissent, benefit/risk distribution, use, and revocation. **Failure:** privacy risk, weak denominator, incompatible indicator, harm signal, or withdrawal stops the affected contribution and marks pooled results partial. **Acceptance:** 100 approved synthetic or historical cross-community evaluations honor every stop and withdrawal, detect all seeded small-group risks, and create zero project-selection, funding, or physical action. |

## 8. Domain model

| Record | Ownership, lifecycle, and relationships |
|---|---|
| `OrganizationBoundary` / `ReportingPeriod` | Human-approved entities, operations, consolidation method, purpose, jurisdiction, and versions. |
| `Asset` / `Place` / `CommunityContext` | Stable identity, geometry version, owner/custodian, sensitivity, spatial quality, and relationships. |
| `DataSourceVersion` / `LicenseGrant` / `TransformRun` | Publisher, retrieval time, checksum, coverage, unit, method, license, code, parameters, and output. |
| `MetricDefinition` / `MetricValue` | Boundary, period, unit, method, factor/model, measured/estimated state, uncertainty, coverage, and review. |
| `EmissionActivity` / `EmissionFactorVersion` / `InventoryVersion` | Source flow, scope/category, conversion, factor geography/time, exclusions, totals, and assurance. |
| `ScenarioVersion` / `ClimateModelRun` | Narrative, variables, pathway, hazard, model chain, resolution, horizon, ensemble, and limits. |
| `Exposure` / `VulnerabilityModel` / `ConsequenceEstimate` | Asset/place, time, hazard, sensitivity, adaptation state, distribution, uncertainty, and reviewer. |
| `Option` / `PathwayVersion` / `DecisionTrigger` | Adaptation or transition action, dependency, cost, benefit, equity, safeguard, trigger, owner, and approval. |
| `DisclosureClaim` / `EvidenceLink` / `OfficerApproval` | Framework requirement, text/value, source, calculation, control, limits, and exact sign-off. |
| `RecoveryItem` / `RetentionRule` / `LegalHold` | Deleted draft, restore date, reporting/audit hold, purge authority, and proof. |

## 9. System architecture

- A Rust climate kernel enforces identity, units, metric contracts, temporal/spatial
  meaning, authority, event atomicity, and disclosure state.
- PostgreSQL with geospatial support stores durable domain records; encrypted object
  storage holds large datasets and model artifacts by content hash.
- A source gateway records publisher, fetch time, license, checksum, version, and
  coverage, then quarantines schema or identity drift.
- A transparent calculation engine produces row-level inventories and reproducible
  transforms. Sandboxed model workers operate on approved data and resource limits.
- A scenario registry separates narratives, models, variables, and local assumptions.
  It supports ensembles without erasing disagreement.
- A user-owned capability broker grants narrow source and compute leases. Aether is
  preferred for signed proof; local signed bundles and offline verification are
  mandatory fallbacks.
- HelixCore supplies shared identity, policy, audit, capabilities, jobs, objects,
  billing, and operations behind domain interfaces; ClimatePrime retains climate truth.
- The event flow is request or ingest, authority and license check, metric/domain
  validation, atomic record plus event, projection, notification, and proof.
  Background work uses durable HelixCore jobs, idempotent checkpoints, visible
  ensemble progress, and explicit cancellation.
- Offline inventory review, pathway work, recovery, and verification use a local
  encrypted store. Versioned data, geospatial, factor, scenario, model, framework,
  and proof adapters are contract-tested extension points.

## 10. Agent and automation contract

| Role | May do | Must not do |
|---|---|---|
| Inventory assistant | Map approved activity data, factors, and gaps | Approve boundaries, exclusions, totals, or disclosure |
| Climate analyst | Run bounded models and compare scenario outputs | Call a scenario a prediction or choose the official result |
| Adaptation planner | Draft options, dependencies, safeguards, and indicators | Commit funds, direct public action, or ignore affected people |
| Transition analyst | Draft pathways and track evidence | Set targets, trade, procure, or claim achievement |
| Disclosure clerk | Map approved facts to requirements and draft text | Sign, publish, or hide uncertainty and stale evidence |

Each lease names project, geography, period, sources, methods, tools, data classes,
time, and compute/spend. Agents cannot retrieve credentials or widen a scenario.
Progress shows source/model stage, elapsed time, last signal, completed ensemble
members, failed members, and uncertainty. Human denial, approval, pause, cancel,
grant, use, and revocation are signed metadata-only events. Every draft enters named
domain or officer review; unit, spatial, schema, policy, and evidence checks validate
the result. Reversal restores a prior version or revokes a grant without rewriting
history.

## 11. Trust, safety, and privacy

| Safety case | Prevention, human authority, proof, and recovery |
|---|---|
| Scenario presented as prediction | Conditional language, scenario identity, ensemble spread, sensitivity, and explicit limits. A qualified human accepts interpretation. |
| False precision or bad spatial join | Unit, coordinate, resolution, coverage, uncertainty, source age, and validation gates. Low-quality results cannot appear as exact values. |
| Greenwashing or false disclosure | Claim-to-evidence map, factor/method version, change invalidation, independent review, and exact officer approval. No auto-publish capability. |
| Maladaptation or inequitable action | Safeguards, affected-community review, distributional analysis, reversible pilots, monitoring, and human governance. |
| Licensed or sensitive data misuse | Purpose-bound leases, local custody, data minimization, license enforcement, restricted export, and metadata-safe audit. |
| Unsafe deletion | Draft analyses enter a 30-day bin where lawful. Reporting, assurance, litigation, public-record, grant, scientific, or contractual duties may block purge. Immediate access quarantine does not destroy evidence. Permanent deletion needs data-owner authority, impact preview, re-authentication, and signed proof. |

Scientific review, model validation, data ethics, community governance, disclosure
control, privacy, security, and disaster-recovery tests are release gates. No single
global model is treated as universally fit. Tenant and project separation are
enforced in the database and object layer. Data is encrypted in transit and at rest,
residency and licenses follow source policy, and incident recovery can quarantine
access, revoke leases, preserve evidence, and restore a reviewed state.

## 12. Proof and audit

Proof records boundary, asset/place identity, source and license, retrieval time,
transform code, factor and model versions, scenario, spatial operation, units,
coverage, uncertainty, failed ensemble members, assumptions, human decisions, claim
links, and redaction. Aether is the preferred signed proof provider; local signed
bundles and an offline verifier are required. Proof does not establish that a
scenario will occur, that a model is complete, that an action caused an outcome, or
that a disclosure meets every jurisdictional requirement.

## 13. UX system

The primary surfaces are Home, Boundaries, Inventory, Assets, Scenarios, Risks,
Pathways, Disclosures, Evidence, and Recovery. Each result leads with geography,
period, scenario, model set, data coverage, uncertainty, and current human decision.
The interface begins with plain consequences and reveals equations, factors, maps,
and provenance progressively. Maps never use color alone and expose resolution and
missing coverage. Long model runs show dataset staging, ensemble members, failures,
elapsed time, last signal, and no fake percent. Completion, drift, source expiry,
failed model, and approval needs create private notifications. Moving an asset or
changing a boundary previews affected inventories, risk results, and disclosures.
Reversible edits offer undo; empty states explain the first safe action; plain-language
errors state what happened, what remains safe, and how to recover. Keyboard and touch
paths have the same project and authority checks.

## 14. Interoperability and standards

All links below were verified from the official body on 2026-07-15.

- The [GHG Protocol standards](https://ghgprotocol.org/standards) guide corporate
  inventory structure, with the [Scope 3 Standard](https://ghgprotocol.org/corporate-value-chain-scope-3-standard)
  used for value-chain categories. Loss caveat: current standards are being updated;
  factor choice, jurisdiction, assurance, and product-level claims need separate rules.
- [IFRS S2 Climate-related Disclosures](https://www.ifrs.org/issued-standards/ifrs-sustainability-standards-navigator/ifrs-s2-climate-related-disclosures/)
  is a disclosure adapter when adopted or chosen. Loss caveat: jurisdictional
  adoption, materiality, current amendments, reporting boundary, and officer judgment
  remain outside automated conformance.
- The [NGFS Scenarios Portal](https://www.ngfs.net/ngfs-scenarios-portal/explore/)
  supplies versioned macro-financial scenario inputs. Loss caveat: the scenarios are
  possible futures for exploration and need local adaptation; they are not forecasts.
- [ISO 14091:2021](https://www.iso.org/standard/68508.html) guides vulnerability,
  impacts, and risk assessment for adaptation. Loss caveat: it does not provide local
  hazard data, community consent, engineering design, or one required scoring model.

Every adapter records edition, retrieval date, local adoption, unsupported fields,
and semantic loss. A framework name never creates a compliance badge.

## 15. Cross-platform contract

Windows, macOS, and Linux pass identical identity, unit, geospatial, inventory,
scenario, uncertainty, disclosure, recovery, and export fixtures. Two current browser
engines pass the six journeys and accessible map/table alternatives. Offline mode
supports approved data review, calculation replay, pathway work, and proof verification.
GPU or cluster compute is optional; a smaller correct CPU fixture is always available.
Managed compute cannot receive data without a brokered lease and declared residency.
Install, migration, backup, restore, and uninstall checks use synthetic assets and
disposable state. The CLI and container surfaces support administration, import/export,
approved calculation, and fresh checks only; they cannot trade, allocate, publish, or
certify. Optional platform features use capability detection and a safe fallback.

## 16. Reliability and performance budgets

- Acknowledged source, metric, decision, and disclosure events have RPO 0 under forced
  crash and concurrent-writer release tests.
- During each calendar month, 99.9% of authorized local metadata reads complete
  without server error; data-provider and model-provider outages are separate states.
- An inventory with 10 million activity rows recalculates an affected subtotal in p95
  under 5 seconds after a single factor change on the reference workstation.
- An asset portfolio of 1 million features opens to a useful aggregate map in p95
  under 4 seconds with prepared local indexes.
- Every model run emits a meaningful stage or ensemble-member signal within 10
  seconds; after 30 seconds it shows `no recent signal`.
- Local cancellation is acknowledged in 2 seconds and reaches a checkpoint in 60
  seconds. Remote compute remains pending until confirmed.
- Metadata RTO is 30 minutes; source/model object RTO is 4 hours in the supported
  self-hosted profile, tested quarterly.
- Create, import, calculation, and model requests use idempotency keys retained for at
  least 24 hours; a duplicate returns the original durable result.
- Offline mode cannot fetch current data or submit remote compute. Unsynced work stays
  visible. If an optional source or model fails, local inventory, pathway review,
  recovery, and export remain in a named degraded state.

## 17. Success measures

Measure metrics with complete method and source lineage, inventory reconciliation,
unknowns not coerced to zero, model disagreement shown, decisions tested across
multiple scenarios, adaptation safeguards and affected groups reviewed, disclosure
claims linked to current evidence, unauthorized publication/capital action blocked,
accessible journey completion, cross-platform export and independent bundle
validation, and recovery drill success. Do not measure risk-score
count, modeled assets, or disclosure length as success by themselves. Business
measures are renewal after a verified inventory or risk journey, support burden per
active organization, and cost per independently auditable decision packet.

## 18. Delivery plan

- **G0 — Truthful foundation (0–6 months):** freshly prove shared service startup; replace generic records with
  stable identity, provenance ledger, metric contracts, atomic writes, synthetic
  climate fixtures, disposable-state tests, and three-platform CI.
- **G1 — Useful single-player product (6–18 months):** ship inventory workbench, asset/exposure map, scenario
  registry, accessible uncertainty UX, live progress, notifications, and recovery.
- **G2 — Trusted team product (18–30 months):** add physical and transition analysis, action register,
  official adapters, transparent calculations, and non-production pilots.
- **G3 — Category leader (30–42 months):** add compound-risk twin, disclosure evidence room,
  portable proof, robust portfolio, distributional impact, living safety cases,
  and independent scientific, disclosure, and privacy review.
- **G4 — Frontier network (42–60 months):** ship HCL-F4-01 the federated climate
  evidence exchange, HCL-F4-02 the transition dependency observatory, and HCL-F4-03
  the community-governed adaptation learning network only after community-governance,
  prospective-evaluation, and founder gates. Fresh G4 proof requires ten independent
  organization nodes, 100,000 metric and revocation cases, 10,000 dependency and
  cascade cases, 100 community evaluations, every local veto and withdrawal honored,
  zero restricted-data escape, and zero autonomous disclosure, policy, legal, capital,
  procurement, project-selection, infrastructure, or physical action.

Each gate runs fresh Rust and web builds, unit/integration/contract tests, scientific
goldens, six end-to-end journeys, unit/spatial/source-loss, accessibility,
authorization, model-failure, disclosure invalidation, migration, recovery, security,
Windows/macOS/Linux packaging, and browser checks. Stored results do not satisfy it.

## 19. Current truth and gap

The live source is a generated scaffold with generic `scenarios` and `risk_scores`
create/list/get records using title, body, status, and metadata. It has no stable
asset/place identity, metric contract, emissions inventory, geospatial semantics,
scenario dataset, climate model, adaptation plan, disclosure control, UI, or climate
domain test. The assistant has only echo and product-catalog tools. The web folder
contains only `package.json`. The live backend now applies route state and calls the
shared graceful-shutdown server helper; the earlier startup defect is repaired in
source, but this spec-only pass did not run a fresh build. A table called
`risk_scores` is not a climate-risk engine. The first honest slice is HCL-F0-01
through HCL-F0-03 plus fresh build proof and reviewed synthetic fixtures.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Internal truth | Versioned spatiotemporal evidence and metric graph | Architecture decision |
| Scenario meaning | Conditional possible future, never forecast | Scientific review |
| Uncertainty | Visible distributions, coverage, and model disagreement | Scientific review |
| Human authority | Humans approve boundaries, assumptions, actions, capital, and disclosure | Governance decision |
| Agent action | Calculate, compare, draft, and flag only | Founder and safety decision |
| External data | Versioned source plus license and loss record | Data-governance decision |
| Compliance | No automated compliance badge | Legal/disclosure review |
| Delete | 30-day recovery where lawful; reporting and holds override | Retention decision |
| Proof | Aether preferred, local signed bundle required | Architecture decision |

## 21. Definition of category-defining done

- [ ] Every number drills to boundary, period, unit, method, source, version, coverage,
  and uncertainty.
- [ ] Scenarios remain conditional and model disagreement remains visible.
- [ ] Physical risk, transition risk, emissions, adaptation, equity, and disclosure
  are connected without becoming one false score.
- [ ] No agent can allocate capital, trade, publish, certify, or direct physical action.
- [ ] Affected communities and distributional effects are part of material decisions.
- [ ] Independent bundles validate and rerun allowed calculations across platforms.
- [ ] Independent climate science, domain, disclosure, community, privacy, and security
  reviewers accept the safety case.
- [ ] Windows, macOS, Linux, web, offline, accessibility, recovery, and packaging
  gates pass from fresh source.
- [ ] The product states clearly what every result does and does not prove.
