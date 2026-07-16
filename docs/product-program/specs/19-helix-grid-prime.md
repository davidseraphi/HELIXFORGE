# HelixGridPrime — safe energy digital twin and optimization system

```yaml
product: HelixGridPrime
catalog_order: 19
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [site operators, energy managers, utilities, microgrid engineers, asset owners]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixGridPrime is a local-first energy twin that helps a home, campus, factory,
microgrid, or utility understand and optimize energy while keeping every control
action inside verified physical and human safety limits.

## 2. Five-year destination

The useful product connects meters, solar, storage, generators, loads, tariffs,
weather, and carbon signals into one explainable site model. The category
advantage is one plan-to-control chain: forecast, simulate, approve, execute,
observe, compare, and prove. The frontier is a federation of independently owned
sites that can offer flexibility without giving one operator raw private data or
unbounded control. Human operators own safety envelopes, emergency modes, market
participation, and every new form of remote control.

## 3. Users and hard jobs

- Site operators need reliable state and fear an optimization that harms people
  or equipment.
- Energy managers need cost and carbon planning and fear savings based on bad
  baselines.
- Engineers need a trustworthy digital twin and fear model drift.
- Asset owners need lifetime and warranty protection and fear hidden wear.
- Utilities and aggregators need dependable flexibility and fear unavailable or
  falsely reported capacity.

## 4. Product laws

1. Observe, recommend, approve, and control are separate authority levels.
2. A lost network never removes local protective control.
3. Safety envelopes live near the asset and cannot be widened by an agent.
4. Every command has target, value, validity window, reason, authorizer, and result.
5. Forecast uncertainty and missing sensors are visible in every plan.
6. Optimization includes equipment wear, comfort, resilience, and constraints,
   not only price.
7. A simulation cannot be presented as measured performance.
8. Manual override is immediate, local, visible, and audited.

## 5. Scope boundaries

GridPrime owns energy assets, telemetry, forecasts, digital twins, schedules,
flexibility, command policy, and energy evidence. Capital owns accounting and
settlement books. ClimatePrime owns climate scenarios and disclosures. Flow may
orchestrate non-real-time business processes. GridPrime does not replace
certified protection systems, equipment firmware, or licensed grid operators.

## 6. Signature experiences

| Journey | Entry | Visible progress | Human decision | Completion proof | Failure and recovery | Portability |
|---|---|---|---|---|---|---|
| **Connect a site** | An operator starts discovery in read-only mode. | Device identity, units, time, topology, sample count, and quality checks update live. | A qualified person confirms every asset and separately marks any possible control point. | A signed site version records mappings, tests, unknowns, and the read-only boundary. | Unknown units, duplicate identities, or stale data are quarantined; the import can resume without duplicate assets. | Export includes site, topology, asset identities, points, units, mappings, and loss report. |
| **Explain today** | The operator opens Live Flow for a site and time range. | Source, load, storage, cost, carbon, freshness, gaps, and estimated values are visible. | The operator accepts a correction only as a linked new version with a reason. | The energy balance links every total to measurements, estimates, quality, and calculation version. | Missing data changes the answer to partial or unknown; a safe reprocess uses the preserved raw stream. | Export includes raw references, corrected versions, balance calculation, chart data, and proof. |
| **Plan tomorrow** | An energy manager selects a horizon, objectives, and approved constraints. | Forecast stages, calibration, candidate count, infeasible constraints, time, and cost are live. | A person compares alternatives and approves one schedule or none. | The chosen plan preserves all candidates, assumptions, uncertainty, constraints, and approval. | Solver failure keeps the current safe schedule; the user may retry, narrow scope, or close with a reason. | Export includes forecasts, model versions, objective weights, alternatives, schedule, and limits. |
| **Approve control** | A proposed schedule is opened in Control Preview. | Exact assets, values, validity, rate limits, interlocks, simulation, acknowledgements, and feedback show separately. | A qualified operator narrows and signs the lease and exact command set. | Each command links the active envelope, approval, dispatch, acknowledgement, observation, and final state. | A timeout remains unknown and blocks unsafe retry; local stop or a tested compensating plan is offered. | Export includes command intents and metadata-only proof, never device credentials. |
| **Handle an outage** | A link, grid, or asset-loss signal opens the Resilience room. | Detection time, local mode, state of charge, critical loads, remaining time, overrides, and recovery steps stay live. | The local operator chooses priorities and any manual override within the site safety case. | The incident record links signals, mode changes, commands, overrides, observations, and recovery checks. | Cloud loss does not stop local protection; a failed recovery stays in incident state and can roll back to the last tested mode. | Export includes the incident timeline, local logs, model versions, decisions, and proof. |
| **Prove flexibility** | An approved event or contract is selected after delivery. | Baseline build, availability, dispatch, measured response, exceptions, quality, and settlement checks show. | Site and market reviewers accept, dispute, or correct by a linked version. | One signed package ties contract, baseline, event, observations, exceptions, and reviewer verdict. | Missing or disputed data blocks a final claim; recomputation preserves every prior version. | Export uses declared market adapters plus a native evidence bundle and semantic-loss report. |
| **Recover a mistake** | A user opens Recovery or a command exception. | Deleted objects, remaining bin time, affected links, safe restore checks, and any physical recovery plan are visible. | A person approves restore, permanent deletion, or a compensating physical action. | Restore preserves identity and links; permanent deletion or compensation creates a signed event. | Models and schedules restore from the 30-day bin. A physical command is never described as undone; only a tested compensating plan may run. | The recovery packet contains object versions, decisions, checks, and outcome without secrets. |

## 7. Capability map

| ID | Gate | Input | Output | Invariant | Authority | Evidence | Failure state | Acceptance |
|---|---|---|---|---|---|---|---|---|
| GP-F0-01 | G0 | Site facts, device discovery, unit catalog | Typed assets, connections, topology, points | Stable identity is not a path; unknown units and duplicate devices fail closed | Qualified mapper approves identity and any control flag | Discovery log, source facts, mapping diff, approval | `quarantined` | WHEN a duplicate device identity or unknown unit appears, the system SHALL reject trusted status and name the conflict. |
| GP-F0-02 | G0 | Point samples with source time, receive time, sequence, quality | Append-only measurements and gap records | Raw samples never change; correction is linked; every gap is explicit | Approved adapter writes; operator may add a correction reason | Sample hash, timestamps, sequence, quality, gap | `late`, `gap`, `rejected` | WHEN a sequence is missing, the system SHALL create a gap record within 5 seconds of the next good sample. |
| GP-F0-03 | G0 | Actor, role, site, asset, operation, bounds, time | Authority decision and exact lease | Observe, recommend, simulate, approve, dispatch, and emergency roles stay separate | User-owned broker grants; agent cannot self-grant | Grant, denial, use, expiry, revocation | `denied`, `expired`, `revoked` | WHEN a process asks outside its lease, the edge SHALL deny it and create a signed metadata-only event. |
| GP-F1-01 | G1 | Typed topology, measurements, corrections | Live and historic energy balance | Measured, estimated, stale, and missing values never share one state | Operator may select view; corrections require a person | Input versions, calculation, quality, result hash | `partial` or `unknown` | WHEN any material input is stale, the balance SHALL label the result partial and SHALL NOT use healthy green. |
| GP-F1-02 | G1 | Tariffs, carbon factors, comfort bands, wear rules | Versioned cost, carbon, comfort, health models | Every result names assumptions, units, region, and effective time | Energy manager approves model publication | Source, version, tests, approval | `invalid_assumption` | WHEN a tariff or factor changes meaning, the system SHALL create a new version and preserve old plans. |
| GP-F1-03 | G1 | Topology, asset models, historic operating periods | Replayable site twin and validation report | Simulation is labelled; model cannot control before validation and approval | Engineer validates; qualified operator controls eligibility | Environment, inputs, residuals, tests, approval | `not_validated` or `drifted` | WHEN replay error exceeds the approved limit, the twin SHALL lose control eligibility. |
| GP-F2-01 | G2 | Historic data, weather, price, availability, horizon | Probabilistic forecasts and calibration | Point estimates are never shown without ranges and freshness | Forecast agent runs; human approves use in control | Training window, features, model, intervals, back-tests | `stale` or `uncalibrated` | WHEN a forecast is published, it SHALL show horizon, interval, back-test error, and data cutoff. |
| GP-F2-02 | G2 | Forecasts, objectives, constraints, current state | Feasible alternatives or infeasibility report | Hard constraints cannot be traded for cost; one unexplained answer is forbidden | Planner proposes; human selects | Solver version, candidates, rejected constraints, approval | `infeasible` or `cancelled` | WHEN no candidate satisfies all hard limits, the system SHALL return no executable schedule. |
| GP-F2-03 | G2 | Approved schedule, safety envelope, exact lease | Command intents, dispatches, feedback, stop state | Edge validates every command; timeout is not success; lease cannot widen interlock | Qualified operator approves; edge has final dispatch guard | Dry-run, envelope, approval, command, ack, observation | `rejected`, `expired`, `unknown` | WHEN a command lacks observed completion by its deadline, state SHALL remain unknown and unsafe retry SHALL be blocked. |
| GP-F3-01 | G3 | Site state, outage scenarios, critical loads, fuel, storage | Resilience plans and drill reports | Local protection works without cloud; critical priorities are versioned | Site safety owner approves modes and overrides | Scenario, simulation, drill, incident, recovery | `unsafe_plan` or `drill_failed` | WHEN external network is cut in a drill, local safe mode SHALL enter within the site limit and preserve critical-load policy. |
| GP-F3-02 | G3 | Contract, baseline method, event, measurements | Flexibility result and settlement evidence | Baseline, availability, response, exception, and quality are explicit | Site and market reviewers accept or dispute | Contract, baseline versions, event, response, verdict | `disputed` or `insufficient_data` | WHEN required evidence is missing, the system SHALL block a final delivered-capacity claim. |
| GP-F3-03 | G3 | Site-approved health, forecast, plan, incident summaries | Fleet comparison and exception queue | Site ownership and policy stay local; fleet cannot widen control | Each site grants exact fields; fleet operator reviews | Grants, aggregates, source versions, access events | `partial_fleet` | WHEN a site revokes a field, new fleet reads SHALL stop without deleting other sites' data. |
| GP-F4-01 | G4 | Approved capacity, terms, availability proof | Federated offer, dispatch contract, signed outcome | Raw site data and permanent control do not leave; each site can leave | Site owner, market operator, and local safety gate approve | Grants, offer, acceptance, dispatch, result, revoke | `counterparty_unavailable` | WHEN a site revokes participation, new dispatches SHALL stop within 2 seconds locally and shared secrets SHALL remain for other sites. |
| GP-F4-02 | G4 | Twin residuals, operating regimes, candidate calibration | Drift alert and candidate model version | A changed model cannot control until replay, safety tests, and human approval pass | Calibration agent drafts; engineer and operator approve | Drift data, candidate, comparison, approval | `drifted` or `approval_waiting` | WHEN drift crosses the site threshold, the active model SHALL lose new-control eligibility until review. |
| GP-F4-03 | G4 | Community topology, policies, simulations, hardware lab | Bounded coordination plan and pilot evidence | Simulation and hardware-in-loop pass before live; local safety always wins | Utilities, site owners, and qualified operators approve pilot | Safety case, simulations, HIL results, training, approvals | `pilot_blocked` | WHEN any safety case or rollback drill fails, the live pilot SHALL remain disabled. |

## 8. Domain model

| Domain group | Owner | Lifecycle | Version rule | Retention and delete | Main relationships |
|---|---|---|---|---|---|
| Site and topology: `Site`, `Zone`, `Asset`, `AssetIdentity`, `Connection`, `TopologyVersion` | Site owner; qualified mapper approves control identity | discovered → mapped → tested → trusted → retired | Topology and identity changes create versions; stable IDs survive moves | Retired records enter the 30-day bin only when no operational or legal hold applies; control history remains | A site contains zones and assets; connections form a validated topology; plans pin one topology version |
| Telemetry: `TelemetryPoint`, `Measurement`, `QualityFlag`, gap and correction records | Site owns values; adapter is source agent | received → validated → accepted or quarantined → corrected by link | Raw measurement is immutable; correction is a new record with reason | Raw and corrected data follow site safety, market, privacy, and contract policy; ordinary derived views use the 30-day bin | Points belong to assets; measurements feed balance, forecast, twin, incident, and proof |
| External signals: `Tariff`, `CarbonSignal`, weather and market inputs | Energy manager owns selected source and use | imported → validated → active → superseded → expired | Source, region, unit, and effective period are versioned | Source licence and reporting policy set retention; deleted drafts use the bin | Signals feed forecasts, objectives, schedules, flexibility, and reports |
| Models: `Forecast`, `TwinModel`, `TwinCalibration`, `Constraint` | Engineer owns model; operator approves control use | draft → tested → approved → active → drifted → retired | Every training window, assumption, calibration, and constraint set is immutable by version | Drafts use the bin; models used in control or evidence follow operational retention | Models consume telemetry/topology and produce forecasts, alternatives, and validation evidence |
| Safety and authority: `SafetyEnvelope`, lease, role, policy | Qualified site safety owner; local gateway enforces | draft → simulated → approved → active → expired or revoked | Widening always creates a new version and full approval; edge keeps protected copy | Never hard-deleted while referenced by a command, incident, or safety case | Envelopes constrain schedules, leases, commands, overrides, and emergency modes |
| Planning: `Plan`, `Schedule`, candidate and infeasibility records | Energy manager chooses; planner only drafts | proposed → compared → approved or rejected → active → completed or expired | Approved plan and all alternatives remain fixed | Drafts and rejected plans use bin; executed plans follow evidence retention | Plans pin forecasts, models, constraints, topology, approvals, and commands |
| Control: `CommandIntent`, `Approval`, `Dispatch`, `Feedback`, `Override` | Qualified operator approves; edge gateway dispatches | proposed → approved → sent → acknowledged → observed, rejected, expired, or unknown | Commands are append-only; compensation is a new linked intent | Control and override records follow safety and legal policy, never ordinary delete | Every command links asset, lease, envelope, reason, approval, dispatch, observation, incident |
| Resilience and market: `Incident`, `FlexibilityOffer`, `Baseline`, `SettlementEvidence`, `ProofBundle` | Site owns incident; contract parties own signed outcome | open → contained → recovered → reviewed; offer → accepted → delivered/disputed → closed | Corrections and verdicts create linked versions | Contract, incident, legal-hold, and safety policies apply; drafts use bin | Incident links telemetry/control; flexibility links contract, baseline, dispatch, response, exceptions, verdict |

## 9. System architecture

- Edge gateways own device adapters, local buffering, time sync, quality checks,
  command interlocks, and offline safe mode.
- A telemetry plane stores durable raw readings and query-optimized series.
- A model service versions topology, forecasts, twins, and constraints.
- An optimization service runs in isolated workers and emits alternatives with
  reasons and infeasibility reports.
- A command gateway validates leases and envelopes at the edge, then records
  requested and observed state separately.
- HelixCore provides identity, policy, jobs, audit, capabilities, objects, and
  recovery; Pulse may later serve high-rate ephemeral state.

## 10. Agent and automation contract

| Agent | May read | Tools | May draft | Approval required | Forbidden | Checking | Stop and reversal |
|---|---|---|---|---|---|---|---|
| Mapper | Discovery packets, approved manuals, existing site/topology | Read-only discovery, unit validator, topology simulator | Asset identity, point map, topology, quality rules | Qualified person confirms every control point and trusted topology | Enable control, guess unit, merge duplicate identity, retrieve credential | Schema, unit, duplicate, connectivity, and read-only sample tests | Cancel stops discovery; import batch rolls back; trusted prior topology remains |
| Forecast Agent | Approved telemetry, weather, tariffs, model policy | Sandboxed feature, train, back-test, calibration tools | Forecast versions and driver explanation | Human approves use in a plan or control | Fill missing data silently, change raw records, present estimate as fact | Data cutoff, leakage, calibration, regime, freshness tests | Cancel stops worker and keeps checkpoint; active forecast remains unchanged |
| Planner | Approved forecasts, objectives, constraints, asset state | Isolated solver and replay twin | Alternative plans, infeasibility report, schedule | Human selects plan; control needs a later separate approval | Trade hard safety for price, dispatch, hide rejected options | Constraint, wear, comfort, resilience, replay, cost tests | Cancel solver within budget; no candidate becomes active; chosen schedule can be superseded by new approval |
| Operator Agent | Exact approved schedule, lease, envelope, current asset state | Edge dry-run and command request interface | Command intents inside exact bounds | Qualified operator approves command set unless a pre-approved narrow policy applies | Widen lease/interlock, add control class, export secret, declare timeout success | Edge revalidates identity, bound, rate, time, state, interlock, feedback | Lease revoke blocks new calls; local stop is immediate; physical reversal uses a tested compensating plan |
| Incident Guide | Current alarms, topology, approved runbooks, local state | Read-only diagnosis, simulation, checklist | Plain-language options and recovery checklist | Local operator chooses every override or recovery action | Hide alarm, silence evidence, assume emergency authority, contact external party alone | Runbook version, prerequisites, missing signal, safety stop checks | User stops guidance at once; prior safe mode stays; decisions reverse only through explicit new action |
| Flexibility Agent | Approved contract, baseline, site-granted data | Baseline calculator, event validator, evidence packer | Offer, response report, settlement evidence | Site owner approves offer; parties approve final result | Expose raw private data, commit permanent control, self-settle | Contract, baseline, quality, availability, response, exception checks | Revoke removes future access; disputed result stays open; corrections are linked versions |

Every agent shows data freshness, current stage, candidate count, failed
constraints, approvals, dispatch acknowledgements, last device feedback, elapsed
time, and the next expected signal. Leases name the exact site, asset, operation,
bounds, rate, validity, process identity, and emergency behaviour.

## 11. Trust, safety, and privacy

Read and control networks are separated. Device identity, mutual authentication,
signed adapter packages, network allowlists, and least privilege are required.
Sites set data-sharing policy. Occupancy and household patterns are sensitive.
Safety envelopes have version approval and protected local copies. Control
errors are fail-safe according to the asset profile, never one global default.
Drafts and models use a 30-day bin. Telemetry and control history follow safety,
market, legal-hold, and contract retention policies.

## 12. Proof and audit

Each controlled outcome links input data quality, model version, forecast,
constraints, alternatives, approval, exact commands, acknowledgments, observed
response, overrides, and exceptions. Evidence proves what GridPrime requested
and observed; it does not prove an uninstrumented physical state. Aether is the
preferred proof and capability provider; an offline local signed ledger is the
required fallback.

## 13. UX system

The main surfaces are Site, Live Flow, Assets, Forecast, Plan, Control,
Resilience, Flexibility, Incidents, Evidence, and Recovery. Basic view answers
“what is happening, why, and what needs me?” Advanced view reveals topology,
units, raw points, constraints, and solver detail. Every command has a preview
and live state: proposed, approved, sent, acknowledged, observed, rejected,
expired, or unknown. Slow optimization shows feasible candidates and blocking
constraints. The UI never uses green for a device with stale data.

## 14. Interoperability and standards

- [OpenADR 3](https://www.openadr.org/specification-download) is the demand and
  flexibility event adapter; the OpenADR Alliance identified it as the newest
  family addition when verified on 2026-07-15.
- [SunSpec information models](https://sunspec.org/specifications/) guide open
  distributed-energy device adapters, verified on 2026-07-15.
- [IEC 61968-9:2024](https://webstore.iec.ch/en/publication/75041) is an optional
  meter-data/CIM adapter, verified from IEC on 2026-07-15.
- [OASIS Energy Interoperation](https://www.oasis-open.org/standard/energyinterop-v1-0/)
  is an optional market-signal adapter.

No device becomes controllable merely because it can be read. Each adapter has
conformance fixtures and a declared loss map.

## 15. Cross-platform contract

Operator views, planning, simulation, evidence, and read-only adapters run on
Windows, macOS, and Linux. Production edge control targets a hardened Linux
profile first, but Windows and macOS can run safe simulators and read-only labs.
Every platform reports its capability level. A browser session never holds a
long-lived device credential. Offline edge operation is tested by cutting all
external network access.

## 16. Reliability and performance budgets

| Area | Numeric target | Measurement and failure behaviour |
|---|---|---|
| Data loss | Zero acknowledged topology, authority, command, incident, and market-event writes lost; zero silent telemetry loss | Forced-crash and storage-fault tests run for every release; telemetry gaps become records within 5 seconds of the next sample |
| Live view | A new accepted point appears within 2 seconds at p95 over each rolling 30-day window | Measured at source receipt to browser update for the supported site profile; stale state replaces green after the point limit |
| Offline edge | Local protection and the last approved safe mode run for at least 72 hours without cloud; buffer at least 7 days at 10,000 points/second | Quarterly full-network-cut drill; overflow policy is visible and never discards without a gap record |
| Idempotency | Command, approval, dispatch, incident, and market keys remain valid at least 90 days | Ten repeated requests return one intent or prior result; external unknown state blocks a new side effect |
| Concurrency | At most one active control lease per conflicting asset/command class; 1,000 concurrent plan readers do not delay edge checks beyond 100 ms p99 | Race tests prove conflict rejection; local interlock is independent of central locks |
| Scale | One supported managed region handles 100 sites, 1,000,000 points/second, 100 concurrent optimizations, and 10,000 assets per site | One-hour load test records ingest lag, query p95, solver queue, drops, and CPU; any skipped metric blocks the claim |
| Job start and heartbeat | Durable stage within 2 seconds; active local optimization heartbeat no older than 5 seconds | UI changes to delayed after 10 seconds without a real signal |
| Cancellation | Solver cancellation accepted within 2 seconds and local worker stops within 10 seconds | Partial candidates stay labelled incomplete; remote work remains `cancel_requested` until confirmed |
| Command truth | Intent is durable before dispatch; timeout never becomes success; observed state arrives inside the command validity window or remains unknown | Edge and hardware-in-loop tests cover lost, duplicate, late, and out-of-order acknowledgements |
| Recovery | Metadata RPO zero after acknowledgement and RTO 15 minutes; edge safe-state recovery follows the tighter site safety case | Quarterly restore plus local failover drill from signed ledger and protected envelope copy |
| Degradation | Loss of cloud, Aether, forecast model, market, or notifications never removes local protection, manual read-only state, local audit, or stop | Capability is marked unavailable within 10 seconds; current safe plan expires by its fixed validity and is not extended silently |

## 17. Success measures

| Measure | Threshold and window |
|---|---|
| Safety authority | Zero command outside an active envelope or lease in every release and each rolling 90-day production window |
| Command truth | At least 99.9% of acknowledged commands reach a verified observed or explicit rejected/expired state inside their validity window over rolling 30 days; unknown is never success |
| Telemetry honesty | 100% of detected sequence gaps and stale-point events create visible quality records; zero silent drops in monthly audit samples |
| Forecast calibration | For each published horizon and operating regime, the declared 90% interval contains 85–95% of outcomes over a rolling 90-day sample or the model loses approved status |
| Planning safety | Zero hard-constraint violations in simulation, hardware-in-loop, and field execution; every infeasible run returns named constraints |
| Resilience | 100% of quarterly network-loss drills enter the site safe mode within its approved time; critical-load duration error stays inside the site-declared tolerance |
| Outcome value | Cost, carbon, comfort, wear, and resilience change are reported against a versioned baseline for every monthly outcome claim; no single metric can hide a worse hard limit |
| Portability | 100% of quarterly sampled site exports import and validate on a clean supported system with all known loss listed |
| Recovery | 100% of quarterly metadata restore drills meet RPO zero and 15-minute RTO; failed drills block the release or field expansion |
| Accessibility | Zero critical or serious accessibility findings in release journeys; keyboard, screen-reader, zoom, and non-colour tasks pass on every release candidate |
| Operator trust | At least 85% of quarterly sampled operators can explain data freshness, active authority, plan trade-offs, and how to stop control without help |
| Business | At least 70% of paying sites renew annually because of verified savings, resilience, or reduced operating risk; command volume is not a success measure |

## 18. Delivery plan

| Gate | Build | Tests | Safety | UX | Windows/macOS/Linux | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Typed sites, assets, points, units, quality, atomic ledger, authority, simulator, recovery; no live control | Domain, unit, duplicate, gap, crash, idempotency, signature tests | Read/control separation, threat model, secret broker, production-state isolation | Read-only setup, freshness, unknown states, jobs, Recovery | Core, simulator, CLI, and package CI pass on all three systems | Dry-run importer from current sites/readings with counts, loss, rollback | Fresh install, import, network cut, crash, restore, verify on each OS |
| **G1 — Useful single-player product (6–18 months)** | Site connection, live balance, history, factors, replay twin for one solar-battery-load site | Full read-only journey, correction, replay, scale, accessibility tests | No control path; model eligibility and data privacy review | Accessible Site, Live Flow, Assets, Evidence | Desktop/web/CLI/container and read-only adapters proven on all three; Linux edge simulator | Versioned topology, point, unit, and model migrations with rollback | Operator connects the simulator, explains a day, exports, restores, verifies |
| **G2 — Trusted controlled product (18–30 months)** | Forecasts, alternatives, edge gateway, exact leases, one narrow command class | Forecast calibration, solver, race, HIL, lost-ack, timeout, stop tests | Independent control safety and security review; approved site safety case | Command preview, alternatives, live states, immediate stop | Planning and HIL pass on Windows/macOS/Linux; first hardened Linux edge package | Envelope, lease, command, adapter upgrade and rollback migrations | Qualified operator completes approve, dispatch, unknown, stop, compensation drill |
| **G3 — Category leader (30–42 months)** | Resilience, fleet, flexibility contracts/evidence, limited field pilots | Outage, islanding, fleet privacy, baseline, settlement, load tests | External safety, utility, privacy, and penetration assessment | Resilience room, fleet exceptions, dispute, slow-work feedback | Mixed-OS operator fleet with Linux edge and offline proof | Live-site topology/model/control migration with canary and rollback | Quarterly outage, restore, dispute, and field rollback proof at pilot sites |
| **G4 — Frontier network (42–60 months)** | Federated capacity, continual calibration, community coordination | Malicious node, partition, revoke, drift, HIL, adversarial coordination tests | Formal safety cases, regulator/utility approval, training and independent review | Site-controlled grants, federation status, exit and recovery | Mixed-node Windows/macOS/Linux planning plus hardened Linux edges | Remove a site/project and revoke bindings without deleting shared secrets or proof | Independent federate, dispatch, revoke, leave, disaster, and rollback exercise |

A gate closes only from fresh release-candidate builds, tests, safety checks,
journeys, migrations, and operator evidence. A simulator skip, stale safety case,
or green wrapper around an unknown device state cannot close a gate.

## 19. Current truth and gap

The live code has generic `sites` and `readings` records. It has no typed energy
assets, units, topology, telemetry quality, forecast, twin, optimizer, command
gateway, safety envelope, UI, or tests. The first safe slice is a read-only
simulated solar, battery, meter, and load site implementing GP-F0-01 through
GP-F1-01. Live control is forbidden before G2.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| First mode | Read-only simulation and replay | G1 proof |
| Control architecture | Edge safety gateway; central planner cannot bypass it | Safety architecture decision |
| Unknown device state | Unknown, never assumed safe or successful | Safety review |
| Optimization | Return alternatives and infeasibility reasons | Product decision |
| Credentials | User-owned capability broker | Security review |
| First live pilot | One bounded site and one command class | Founder plus qualified operator |
| Network loss | Local safe mode continues | Site safety case |
| Delete | 30-day bin for drafts; policy retention for operations | Safety/legal exception |

## 21. Definition of category-defining done

- [ ] One model joins physical state, forecast, plan, control, result, and proof.
- [ ] No agent or central outage can widen local safety authority.
- [ ] Every estimate, stale value, command, override, and unknown state is visible.
- [ ] Sites can leave with their models, history, policies, and evidence.
- [ ] Windows, macOS, Linux, edge-offline, accessibility, recovery, safety, and
      independent security gates pass.
- [ ] Federated value is possible without central raw-data or permanent-control custody.
