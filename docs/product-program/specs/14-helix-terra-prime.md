# HelixTerraPrime — whole-farm digital twin and decision system

```yaml
product: HelixTerraPrime
catalog_order: 14
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [farmers, agronomists, farm managers, cooperatives and land stewards]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixTerraPrime is a whole-farm decision system that keeps land, crops, animals,
water, work, costs, observations, forecasts, and outcomes in one inspectable twin
while keeping farming knowledge, land rights, and physical action under human control.

## 2. Five-year destination

The useful product is an offline-capable field map, season plan, observation book,
work tracker, input register, harvest record, and evidence viewer. The leading product
builds a live farm twin from trusted observations, sensors, weather, remote sensing,
and accounting data, with clear age and uncertainty. The frontier product can compare
crop, water, soil, biodiversity, labor, and financial scenarios over several seasons.
It never applies chemicals, controls machinery, opens water, buys inputs, or changes
land records on its own. A farmer, agronomist, veterinarian, or other qualified human
chooses and approves every physical action.

## 3. Users and hard jobs

- A farmer needs timely advice despite weak connectivity and fears a model that does
  not understand the field.
- An agronomist needs comparable observations, methods, units, and outcomes across
  seasons.
- A farm manager needs work, inputs, machinery, labor, cost, and safety in one plan.
- A cooperative needs useful shared learning without taking ownership of member data.
- A land steward needs soil, water, habitat, and biodiversity consequences beside
  yield and profit.
- A reviewer or buyer needs traceable claims without forcing a farmer to disclose all
  private operations.

## 4. Product laws

1. Observed, sensed, imported, estimated, forecast, and recommended values stay
   visibly different.
2. Every map layer names coordinate system, time, resolution, source, method, quality,
   and license.
3. The farm twin shows data age, gaps, disagreement, and uncertainty.
4. No agent may control machinery, irrigation, livestock systems, procurement, or
   chemical application.
5. A recommendation includes assumptions, expected benefit, risk, cost, and a safe
   way to reject it.
6. Farmer data stays in farmer-approved custody and is not silently pooled or sold.
7. Local and indigenous knowledge can be recorded with ownership and access chosen by
   its contributor; it is not treated as free training data.
8. Deletion, correction, export, and migration preserve land and season identity.
9. Critical journeys are accessible, portable, and independently verifiable across
   platforms; no model, equipment vendor, or proof provider is a hard dependency.

## 5. Scope boundaries

TerraPrime owns farm identity, parcels, seasons, plans, observations, inputs, work,
resource balances, scenarios, recommendations, outcome evidence, and approved data
exchange. HelixCore owns shared identity, policy, audit, capabilities, jobs, objects,
billing, and operations; Aether is reached through a provider-neutral proof interface
with a local fallback. It does not replace agronomic, veterinary, environmental, worker-safety,
or legal advice. It is not a safety-rated machinery controller, pesticide label,
water-right register, land title system, or autonomous trading system. Any connector
to equipment is read-only by default. Physical actuation requires a separate certified
controller, exact human approval, local law, and an independent safety boundary.

## 6. Signature experiences

1. **Create the farm twin.** Entry: the owner imports or draws approved parcels and
   names the farm's authority and data policy. Progress: geometry, coordinate system,
   ownership, soil, infrastructure, and missing layers appear. Human decision: the
   owner accepts each parcel and boundary. Proof: source files, transforms, edits,
   and decisions are recorded. Failure and recovery: overlap or identity conflict is
   quarantined. Export: GeoPackage plus a farm manifest.
2. **Plan a season.** Entry: a farmer selects a parcel, crop or enterprise, goals, and
   limits. Progress: rotation, soil, water, labor, inputs, cost, weather assumptions,
   and risks build a visible plan. Human decision: farmer and chosen adviser approve
   actions. Proof: every plan version and assumption is kept. Failure and recovery:
   material edits invalidate affected approvals. Export: a human-readable season plan.
3. **Record a field observation.** Entry: a worker uses an offline map or form.
   Progress: location, time, method, photos, units, and quality are checked. Human
   decision: an authorized reviewer accepts or corrects it. Proof: device time,
   author, geometry, and original attachment are preserved. Failure and recovery:
   sync conflict opens comparison; no last-write-wins loss. Export: observation bundle.
4. **Investigate a problem.** Entry: a farmer selects a symptom, area, or alert.
   Progress: observations, weather, operations, sensor quality, candidate causes, and
   missing checks appear. Human decision: a qualified person chooses an inspection or
   action. Proof: evidence and rejected hypotheses are retained. Failure and recovery:
   weak evidence stays `uncertain`, not a diagnosis. Export: an investigation packet.
5. **Compare farm scenarios.** Entry: a user chooses approved alternatives for crop,
   irrigation, soil, grazing, or habitat. Progress: assumptions, compute stage, yield,
   water, cost, emissions, labor, biodiversity, and uncertainty stream visibly. Human
   decision: the owner selects or rejects a plan. Proof: models, versions, inputs, and
   sensitivity are kept. Failure and recovery: incomplete scenarios can resume from
   checkpoints. Export: a scenario comparison crate.
6. **Prove an outcome without giving away the farm.** Entry: the owner selects a claim
   and disclosure audience. Progress: required evidence, gaps, redaction, and consent
   are shown. Human decision: the owner approves exact fields. Proof: provenance,
   transformations, and omissions are signed. Failure and recovery: unsupported claims
   cannot be published. Export: a minimal disclosure bundle and local full proof.

## 7. Capability map

F0 is foundation, F1 is the useful product, F2 is the trusted-team product, F3 is
advanced category leadership, and F4 is the frontier network. Every row inherits this full contract: its invariants are the
product laws and observed/estimated/forecast truth boundaries; authority is the
exact named human and policy in Sections 10–11; evidence is input/output hashes,
versions, actor, decision, and ledger event; failure is a durable `blocked`, `failed`,
`unknown`, conflict, or quarantine state with retry or recovery; and test acceptance
includes denial, failure, recovery, offline-conflict, and cross-platform cases in
addition to the row's named check. The row names its domain-specific inputs, output,
and strongest acceptance test.

| ID | First gate | Capability contract |
|---|---|---|
| HTP-F0-01 | G0 | **Stable farm and parcel identity.** Inputs are approved geometry, ownership/authority, coordinate reference, and source; outputs are versioned farm and parcel records. Acceptance: overlap, invalid geometry, path change, and duplicate import cannot change identity silently. |
| HTP-F0-02 | G0 | **Observation provenance ledger.** Inputs are observation, sensor, plan, action, review, and export events; output is an append-only graph. Acceptance: concurrent offline sync and forced crash lose no acknowledged event. |
| HTP-F0-03 | G0 | **Farm data authority.** Inputs are owner, contributor, purpose, resource, capability, time, and sharing policy; output is a narrow grant or denial. Acceptance: revoked data cannot enter future agents, exports, or pooled learning. |
| HTP-F1-01 | G1 | **Offline field map and observation book.** Inputs are approved base layers and observations; output is a local-first map and versioned record. Acceptance: offline creation, conflict review, restore, and later sync pass on every desktop platform. |
| HTP-F1-02 | G1 | **Season-plan graph.** Inputs are parcels, goals, rotations, work, inputs, water, labor, costs, and constraints; output is a versioned plan. Acceptance: dependent actions and approvals update visibly after a material change. |
| HTP-F1-03 | G1 | **Sensor and weather ingestion.** Inputs are allowlisted sources with time, location, unit, method, and quality; output is raw plus normalized observations. Acceptance: stale, duplicate, out-of-range, and unit-conflict data is flagged, not averaged away. |
| HTP-F2-01 | G2 | **Whole-farm resource ledger.** Inputs are approved water, nutrients, feed, energy, inputs, labor, harvest, and loss events; output is an uncertain balance. Acceptance: missing flows and estimation methods remain visible. |
| HTP-F2-02 | G2 | **Investigation workspace.** Inputs are symptoms, evidence, history, and bounded knowledge; output is ranked hypotheses and safe next checks. It is not a diagnosis. Acceptance: agents cannot prescribe or execute an intervention. |
| HTP-F2-03 | G2 | **Outcome and claim proof.** Inputs are plans, observations, actions, models, and outcomes; output is a claim-to-evidence graph with redaction. Acceptance: every published claim lists coverage, uncertainty, and omitted data. |
| HTP-F3-01 | G3 | **Farm scenario twin.** Inputs are approved alternatives, model versions, climate/weather assumptions, and resource limits; output compares yield, cost, water, soil, labor, emissions, and habitat. Acceptance: forecast is never shown as observed outcome. |
| HTP-F3-02 | G3 | **Read-only equipment adapter layer.** Inputs are declared vendor data; output is normalized telemetry and capability status. Acceptance: there is no actuation route and unsupported fields have a loss report. |
| HTP-F3-03 | G3 | **Cooperative learning room.** Inputs are owner-approved summaries; output is aggregate benchmarks with cohort and uncertainty. Acceptance: minimum cohort, withdrawal, anti-reidentification, and no-sale policy tests pass. |
| HTP-F3-04 | G3 | **Multi-season resilient planning.** Inputs are human-approved climate, market, soil, and water scenarios; output is a portfolio of robust plans, not one prediction. Acceptance: every plan exposes assumptions and worst tested outcomes. |
| HTP-F3-05 | G3 | **Bounded farm advisor.** Inputs are exact parcel, season, approved evidence, and action classes; output is reviewable options. Acceptance: it cannot buy, schedule, command, or apply anything, and a human can dismiss it fully. |
| HTP-F3-06 | G3 | **Land stewardship evidence network.** Inputs are consented, minimized indicators; output supports landscape learning while land-level data stays in custody. Acceptance: project removal revokes bindings without deleting shared public reference data. |
| HTP-F4-01 | G4 | **Farmer-owned regional coordination room.** **Input:** each owner's approved needs or offers, resource class, time range, coarse location, constraints, fairness policy, and withdrawal terms. **Output:** candidate matches, conflicts, dependencies, and transparent allocation options. **Invariant:** the room cannot buy, sell, contract, schedule labor, release water, move material, command equipment, or bind any farm. **Authority:** every farm owner approves only its own disclosure and commitment, while a named human cooperative body records any shared decision. **Evidence:** offers, matching rules, alternatives, owner decisions, dissent, expiry, and revocation events. **Failure:** missing authority, unfairness, stale availability, conflict, or withdrawal removes the affected match without changing another farm. **Acceptance:** 20 independently controlled farm nodes complete 10,000 join, conflict, match, and revoke cases with every local veto preserved and zero financial or physical action. |
| HTP-F4-02 | G4 | **Federated land observation exchange.** **Input:** consented and minimized soil, water, weather, crop, habitat, and management observations with source, unit, method, resolution, license, and purpose. **Output:** a revocable regional evidence layer with quality, coverage, uncertainty, and provenance. **Invariant:** farm-level data stays local unless the owner approves exact disclosure; observations cannot trigger enforcement, sale, credit, insurance, prescription, or equipment action. **Authority:** each owner and local data steward approve contribution, aggregation, use, and withdrawal. **Evidence:** source hashes, transformations, cohort rule, access, aggregate, uncertainty, use, expiry, and revocation. **Failure:** small cohort, unit or spatial conflict, source loss, privacy risk, or withdrawal invalidates the affected aggregate. **Acceptance:** five cooperatives exchange 100,000 mixed-unit and spatial fixtures with zero unapproved farm-level disclosure and clean removal of every revoked binding. |
| HTP-F4-03 | G4 | **Human-governed resilience trial network.** **Input:** farmer- and agronomist-approved trial questions, non-actuating protocols, baseline, outcome measures, safety conditions, local context, and stop rules. **Output:** comparable trial evidence, disagreement, transfer limits, and candidate questions for human review. **Invariant:** the system cannot prescribe, apply an input, operate equipment, schedule work, infer causality without the declared design, or select the next intervention. **Authority:** participating farmers approve local activity, qualified human reviewers approve interpretation, and each site may stop or withdraw. **Evidence:** protocol version, approvals, context, observations, deviations, stop events, analysis, dissent, and withdrawal. **Failure:** unsafe scope, protocol drift, missing baseline, adverse event, or withdrawal stops that site's contribution and marks pooled results partial. **Acceptance:** 100 cross-site synthetic or historical trial replays preserve local stop rules, expose all seeded bias and missingness, and create zero prescription or actuation. |

## 8. Domain model

| Record | Ownership, lifecycle, and relationships |
|---|---|
| `Farm` / `AuthorityBinding` | Stable project identity, owner/steward, jurisdiction, land relationship, data policy, and retention. |
| `Parcel` / `BoundaryVersion` / `ManagementZone` | Geometry, coordinate reference, source, quality, validity period, parent/neighbor links, and approval. |
| `Season` / `Enterprise` / `PlanVersion` | Crop, livestock, forestry, or habitat purpose; goals, constraints, assumptions, actions, dependencies, and approvals. |
| `Observation` / `Sample` / `SensorReading` | Location, time, method, unit, source, quality, raw attachment, review state, and uncertainty. |
| `OperationIntent` / `WorkRecord` | Proposed versus observed work, resource, operator, equipment, safety authority, time, result, and evidence. |
| `InputLot` / `HarvestLot` / `Transfer` | Identity, quantity, unit, source/destination, custody, quality, and claim relationships. |
| `ResourceFlow` / `BalanceVersion` | Water, nutrient, feed, energy, labor, cost, emission, or loss with measured/estimated status. |
| `ScenarioRun` / `ModelSnapshot` / `Recommendation` | Exact inputs, assumptions, versions, output ranges, limits, rejected options, and human decision. |
| `Claim` / `EvidenceLink` / `DisclosureBundle` | Statement, scope, evidence, gaps, owner consent, redaction, audience, and expiry. |
| `RecoveryItem` / `RetentionRule` | Deleted draft or record, restore date, hold, dependency impact, and purge authority. |

## 9. System architecture

- A Rust farm kernel owns identity, geometry validation, temporal records, units,
  access policy, state transitions, and atomic events.
- PostgreSQL with geospatial support stores durable metadata and geometry; encrypted
  object storage keeps imagery, photos, and instrument files by hash.
- A local synchronization engine supports offline append, conflict detection, and
  human reconciliation without last-write-wins data loss.
- Source adapters ingest sensors, weather, remote sensing, and machinery telemetry
  through declared schemas and quality checks. Equipment adapters are read-only.
- Sandboxed model workers have bounded data, compute, and network access. Predictions
  return to review and cannot create work orders.
- The capability broker grants one approved process exact farm and source access.
  Aether is preferred for proof; a local signed verifier is the offline fallback.
- HelixCore supplies shared identity, policy, audit, capabilities, jobs, objects,
  billing, and operations behind domain interfaces; TerraPrime retains farm truth.
- The event flow is local or remote request, farm-authority check, geometry/domain
  validation, atomic record plus event, projection, sync/notification, and proof.
  Background work uses durable HelixCore jobs, idempotent checkpoints, visible
  progress, and explicit cancellation.
- Offline mapping, observation, planning, recovery, and verification use a local
  encrypted store. Versioned geospatial, sensor, weather, model, and proof adapters
  are contract-tested extension points; equipment adapters remain read-only.

## 10. Agent and automation contract

| Role | May do | Must not do |
|---|---|---|
| Farm mapper | Suggest geometry fixes and classify approved layers | Change ownership, title, or accepted boundaries |
| Season planner | Draft plans, dependencies, costs, and questions | Schedule or approve physical work |
| Field scout | Organize observations and propose safe inspections | Diagnose, prescribe, or apply treatment |
| Scenario analyst | Run bounded comparisons and explain sensitivity | Present one forecast as certain or select a plan |
| Evidence clerk | Build disclosure drafts and redaction previews | Publish, sell, or pool farmer data |

Every agent lease names farm, parcel, season, data classes, purpose, tools, time, and
resource budget. Progress shows source age, current stage, elapsed time, last signal,
uncertainty, and required human decision. Agents cannot retrieve secrets, widen scope,
or self-approve. Pause, cancellation, denial, grant, use, and revocation are signed.
Every draft enters named owner or domain review; geometry, unit, schema, policy, and
evidence checks validate the result. Reversal restores a prior version or revokes a
grant without rewriting field history.

## 11. Trust, safety, and privacy

| Safety case | Prevention, human authority, proof, and recovery |
|---|---|
| Harmful field action | No actuation or procurement capabilities; label, law, local conditions, and qualified review remain external requirements. A human approves any physical work in the proper system. |
| Bad sensor or map causes wrong advice | Source identity, time, location, unit, calibration context, quality checks, conflict display, and safe fallback. Low-quality data lowers confidence visibly. |
| False agronomic certainty | Observations, estimates, forecasts, and recommendations use separate types with ranges and assumptions. A qualified human chooses action. |
| Farmer data exploitation | Owner-controlled custody, purpose limits, minimized sharing, no-sale default, aggregate thresholds, withdrawal, and signed access metadata. |
| Land-rights harm | Boundaries describe management context, not title. Ownership and tenure claims require official systems and human review. |
| Unsafe deletion | User-created drafts and ordinary operational records enter a 30-day bin where lawful. Food safety, financial, environmental, labor, insurance, certification, or legal-hold duties may block purge. Immediate access quarantine remains possible. Permanent deletion requires owner authority, re-authentication, impact preview, and signed proof. |

Agronomic safety review, equipment threat model, environmental impact review, privacy
assessment, field usability tests, and offline recovery drills are release gates.
Tenant and farm separation are enforced in the database and object layer. Data is
encrypted in transit and at rest, residency follows owner policy, and incident
recovery can quarantine access, revoke leases, preserve evidence, and restore a
reviewed state without pooling private farm data.

## 12. Proof and audit

Proof records farm and parcel identity, geometry source and transform, data source,
method, unit, quality, model version, assumption, plan, observed work, reviewer,
decision, conflict, redaction, and known gap. Metadata events avoid copying private
farm content. Disclosure bundles contain only owner-approved fields and include an
omission manifest. Aether is the preferred signed proof service; a local signed
bundle and verifier are required offline. Proof does not show land title, biological
causality, future yield, legal compliance, or that an operation happened safely
unless independent evidence establishes it.

## 13. UX system

The main surfaces are Home, Map, Seasons, Observations, Work, Resources, Scenarios,
Evidence, Sharing, and Recovery. A field card shows parcel, season, crop or purpose,
data age, weather/source state, open risk, and next human action. The interface works
with keyboard, touch, sunlight contrast, large text, low bandwidth, and offline mode.
Detail reveals progressively from plain farm language to raw source and model math.
Long imports and scenarios show stage, records accepted/quarantined, elapsed time,
last signal, and uncertainty. Completion, conflict, source outage, approval, and
failed sync create clear private notifications. Dragging a boundary or moving work
previews affected plans, observations, and claims. Delete shows restore date and
retention blocks. Reversible edits offer undo; empty states explain the first safe
action; plain-language errors state what happened, what remains safe, and how to
recover. Keyboard and touch paths have the same farm and authority checks.

## 14. Interoperability and standards

All links below were verified from the official body on 2026-07-15.

- [OGC SensorThings API Part 1: Sensing](https://docs.ogc.org/is/15-078r6/15-078r6.html)
  is an adapter for sensor observations and metadata. Loss caveat: source quality,
  calibration, ownership, farm semantics, and safe tasking need local controls; Part
  2 tasking is not enabled for physical actuation.
- [OGC GeoPackage 1.4.0](https://www.ogc.org/standards/geopackage/) is the preferred
  portable offline geospatial container. Loss caveat: farm authority, temporal plan,
  consent, and proof records need declared extension tables or a companion manifest.
- [IETF RFC 7946 GeoJSON](https://www.rfc-editor.org/info/rfc7946/) is a simple web
  geometry exchange adapter. Loss caveat: it fixes WGS 84 coordinate semantics and
  does not carry uncertainty, topology, styling, or full provenance.
- [ISO 19115-1:2014](https://www.iso.org/standard/53798.html) guides geographic
  metadata. Loss caveat: the standard is under revision and does not define farm
  domain meaning, access authority, or observation quality by itself.

Every import records format version, coordinate reference, transformation, license,
unsupported fields, and precision loss. No geometry conversion is silent.

## 15. Cross-platform contract

Windows, macOS, and Linux pass identical geometry, coordinate, unit, offline sync,
conflict, recovery, encryption, scenario, and export fixtures. Two current browser
engines pass the six journeys at desktop and touch widths. The local-first desktop
path remains fully useful without network access. GPU acceleration is optional; a
correct CPU path remains. Managed mode may add shared processing but cannot weaken
farmer authority or export. Install, update, backup, restore, and uninstall tests use
synthetic farms and disposable state. Field-device exports can be reviewed before
they enter the authoritative twin. The CLI and container surfaces support
administration, import/export, and fresh checks only; they expose no actuation path.
Optional platform features use capability detection and a safe fallback.

## 16. Reliability and performance budgets

- Acknowledged parcel, observation, plan, and decision events have RPO 0 in forced
  crash and concurrent offline-sync tests for every release.
- In each calendar month, 99.9% of authorized local farm metadata reads complete
  without server error; external sensor/weather outages are reported separately.
- A farm with 10,000 parcels and 5 million observations opens to a useful map in p95
  under 3 seconds on the reference workstation with cached tiles.
- An offline entry becomes durable locally within p99 500 ms and syncs idempotently;
  conflicts never overwrite either version.
- Long imports and scenarios emit a useful signal every 10 seconds or show `no recent
  signal` after 30 seconds.
- Local cancellation is acknowledged in 2 seconds and reaches a checkpoint in 30
  seconds. Remote source cancellation stays pending until confirmed.
- Metadata RTO is 30 minutes and object/map recovery RTO is 4 hours in the supported
  self-hosted profile, tested quarterly.
- Create, import, ingest, and scenario requests use idempotency keys retained for at
  least 24 hours; a duplicate returns the original durable result.
- Offline mode cannot fetch current weather or remote sources and cannot control
  equipment. Unsynced work stays visible. If an optional source or model fails,
  mapping, planning, review, recovery, and export remain in a named degraded state.

## 17. Success measures

Measure observations with complete source, time, place, unit, and method; offline
work recovered without conflict loss; bad sensor/map inputs caught; plans with
explicit assumptions and human approval; water, cost, soil, biodiversity, and labor
trade-offs reviewed together; farmer-controlled disclosures; unauthorized actuation
blocked; accessible journey completion; cross-platform export and independent bundle
validation; and recovery drill success. Do not measure
hectares indexed, recommendations generated, or data pooled as success by themselves.
Business measures are renewal after a verified season journey, support burden per
active farm, and cost per trusted plan or disclosure bundle.

## 18. Delivery plan

- **G0 — Truthful foundation (0–6 months):** freshly prove shared service startup; replace generic records with farm
  and parcel identity, observation provenance, data authority, atomic writes,
  disposable-state tests, and three-platform CI.
- **G1 — Useful single-player product (6–18 months):** ship offline map/observation book, season plans, safe source
  ingestion, accessible field UX, visible sync, notifications, and recovery.
- **G2 — Trusted team product (18–30 months):** add resource ledger, investigation room, outcome proof,
  approved geospatial/sensor adapters, and cooperative consent controls.
- **G3 — Category leader (30–42 months):** add farm scenario twin, read-only equipment adapters,
  cooperative learning, multi-season planning, bounded advice, stewardship evidence,
  external agronomic and privacy review, and field pilots.
- **G4 — Frontier network (42–60 months):** ship HTP-F4-01 farmer-owned regional
  coordination, HTP-F4-02 the federated observation exchange, and HTP-F4-03 the
  human-governed resilience trial network only after safety, equity, field-evidence,
  and founder gates. Fresh G4 proof requires 20 independently controlled farm nodes,
  five cooperative exchanges, 10,000 coordination cases, 100,000 provenance and
  privacy fixtures, 100 trial replays, every withdrawal honored, zero farm-level
  disclosure outside consent, and zero purchase, contract, schedule, prescription,
  equipment command, material movement, or other physical actuation.

Every gate runs fresh Rust and web builds, unit/integration/contract tests, geospatial
goldens, the six end-to-end journeys, unit/coordinate loss, offline conflict,
accessibility, authorization, migration, recovery, security, Windows/macOS/Linux
packaging, and browser checks. A cached demo cannot satisfy the gate.

## 19. Current truth and gap

The live source is a generated scaffold with generic `fields` and `observations`
create/list/get records using title, body, status, and metadata. There is no geometry,
coordinate system, farm identity, season plan, observation method, unit model, offline
sync, sensor adapter, scenario engine, UI, or Terra domain test. The assistant has
only echo and product-catalog tools. The web folder contains only `package.json`.
The live backend now applies route state and calls the shared graceful-shutdown server
helper; the earlier startup defect is repaired in source, but this spec-only pass did
not run a fresh build. The first honest slice is HTP-F0-01 through HTP-F0-03 plus
fresh build proof and synthetic geospatial fixtures.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Internal truth | Versioned spatiotemporal farm graph | Architecture and migration decision |
| Farm identity | Stable ID independent of folder or geometry path | Architecture decision |
| Physical action | No agent or core-product actuation | Founder, legal, domain, and safety approval |
| Equipment | Read-only adapters first | Independent equipment safety review |
| Uncertainty | Observed, estimated, forecast, and advised stay separate | Domain review |
| Data rights | Farmer-approved custody and no-sale default | Founder and governance decision |
| Offline | First-class, conflict-safe local operation | Product decision |
| Delete | 30-day recovery where lawful; duties may hold data | Retention decision |
| Proof | Aether preferred, local signed bundle required | Architecture decision |

## 21. Definition of category-defining done

- [ ] A farmer can plan, observe, review, and prove work during weak or absent network.
- [ ] Every map, observation, estimate, forecast, and recommendation shows source,
  age, quality, and uncertainty.
- [ ] No agent can buy, command, schedule, or apply a physical farm action.
- [ ] Farm data sharing is minimal, owner-approved, revocable, and independently
  auditable.
- [ ] Yield, money, water, soil, labor, emissions, and habitat trade-offs appear in
  one decision space without false certainty.
- [ ] Cross-platform exports preserve identity, geometry meaning, provenance, and
  declared loss.
- [ ] Independent agronomy, environmental, privacy, security, and field-usability
  reviewers accept the safety case.
- [ ] Windows, macOS, Linux, web, offline, recovery, accessibility, and packaging
  gates pass from fresh source.
