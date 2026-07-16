# HelixForge Studio — a sovereign visual software factory

```yaml
product: HelixForge Studio
catalog_order: 10
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [founders, domain experts, designers, developers, platform teams]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

> **Target-state rule:** Sections 1–18 and 20–21 specify the planned product.
> Section 19 records the much smaller live scaffold that exists today.

## 1. Category claim

HelixForge Studio is a visual software factory where people and bounded agents
can design, build, test, ship, understand, and fully export serious applications
without giving a vendor ownership of source, data, identity, or runtime.

## 2. Five-year destination

The useful product is a visual and code workspace for screens, components, data,
logic, workflows, permissions, agents, tests, packaging, deployment, and
operations. The category-defining advantage is one typed application graph that
connects intent, design, source, runtime, tests, evidence, and releases, so a
change is visible from button to database and back. The frontier capability is a
self-improving software factory that can explore bounded design and
implementation options, prove them in isolated environments, and hand people a
reviewable choice rather than a hidden code dump. Humans keep authority over
product meaning, architecture, data models, permissions, dependencies, secrets,
migrations, costs, releases, destructive actions, and every agent capability.

## 3. Users and hard jobs

- **Founders and domain experts** need to turn a real workflow into software
  without losing control. They fear a demo that cannot become a durable product.
- **Designers** need interaction, content, accessibility, and state to survive
  into production. They fear a visual canvas disconnected from real behaviour.
- **Developers** need readable source, tests, diffs, debugging, and escape paths.
  They fear generated code that cannot be maintained outside the tool.
- **Platform teams** need policy, environments, builds, releases, and recovery.
  They fear agents with broad access or silent infrastructure changes.
- **Reviewers and customers** need proof of what was built and shipped. They fear
  screenshots or completion claims that do not match the release.

## 4. Product laws

1. The user owns complete source, data model, assets, tests, and release records.
2. Visual and code views share a typed model or declare a one-way boundary.
3. Generated output is readable, diffable, testable, and buildable outside Studio.
4. No agent may approve its own plan, expand access, retrieve secrets, or release.
5. Every dependency, permission, network call, and data migration is visible.
6. Preview, test, and build run in isolated, reproducible environments by default.
7. Accessibility, responsive behaviour, loading, failure, empty, and recovery
   states are part of the component contract.
8. Long work shows real stages, checks, artefacts, heartbeat, cost, and cancel state.
9. Delete enters a recoverable bin; published history is never silently rewritten.
10. A failed build, test, proof, or deployment cannot be presented as complete.

## 5. Scope boundaries

HelixForge Studio owns application models, visual authoring, code integration,
component systems, data and workflow design, preview, tests, build plans,
release review, extension packaging, and full export. HelixCode owns general
source hosting, review, and CI. HelixCore owns identity, policy, audit,
capabilities, jobs, objects, billing, operations, and stable project identity.
HelixPulse may later provide high-speed jobs and event transport. Aether may
provide proof and capability brokering through an adapter.

Studio is not a hidden hosting monopoly, general cloud provider, replacement for
professional engineering judgement, or authority to deploy regulated systems.
It does not promise that generated software is safe, lawful, secure, accessible,
or correct until the named checks and accountable people prove the declared scope.

## 6. Signature experiences

1. **Start from a real workflow.** **Entry:** a founder describes users, objects,
   events, decisions, and proof. **Visible progress:** the discovery canvas shows
   open questions, mapped objects, risks, and validation. **Human decision:** the
   founder approves the application boundary and first journey. **Completion
   proof:** a versioned product contract links to the graph and tests. **Failure
   and recovery:** unresolved meaning blocks generation but preserves the draft.
   **Export:** contract and graph use readable files in the repository.
2. **Build a complete screen.** **Entry:** a designer opens a screen and chooses a
   component or pattern. **Visible progress:** responsive, state, content,
   accessibility, data, and action checks update live. **Human decision:** the
   designer approves meaning-changing moves and interaction. **Completion proof:**
   preview, component contract, source diff, and tests remain linked. **Failure
   and recovery:** invalid changes show the exact broken contract and support undo.
   **Export:** source, tokens, assets, and story fixtures work outside Studio.
3. **Model data and permissions.** **Entry:** a developer or domain expert creates
   a domain object and lifecycle. **Visible progress:** fields, relations,
   invariants, policy, migration, seed, and API impacts show. **Human decision:**
   accountable owners approve schema and destructive migration. **Completion
   proof:** generated migration, policy tests, contract, and rollback plan pass.
   **Failure and recovery:** unsafe changes remain a draft. **Export:** schema,
   migrations, and API contract are standard project files.
4. **Connect logic and a long workflow.** **Entry:** a builder joins triggers,
   decisions, human approvals, jobs, and outcomes on a typed flow. **Visible
   progress:** each branch, retry, timeout, compensation, and live run is visible.
   **Human decision:** people approve side effects and authority boundaries.
   **Completion proof:** simulation, generated code, tests, and run evidence link
   to the graph. **Failure and recovery:** a failed step pauses with retry,
   compensate, or human-fix choices. **Export:** workflow code and state format
   run without Studio.
5. **Let an agent propose a feature.** **Entry:** a user gives one scoped goal and
   allowed paths/capabilities. **Visible progress:** plan, files, tools, tests,
   risk, time, and cost stream in a durable timeline. **Human decision:** the user
   approves plan, sensitive tools, and final patch. **Completion proof:** diff,
   fresh checks, screenshots, and evidence bundle are reviewable. **Failure and
   recovery:** cancel revokes the lease and keeps a safe worktree. **Export:** the
   patch and proof are normal repository artefacts.
6. **Import an existing application.** **Entry:** a team opens a supported repo.
   **Visible progress:** code, routes, components, schemas, dependencies, tests,
   and unsupported constructs are indexed. **Human decision:** the team chooses
   read-only, partial round-trip, or managed boundaries. **Completion proof:** a
   mapping and semantic-loss report is signed. **Failure and recovery:** Studio
   never rewrites unsupported code to make the import look complete. **Export:**
   leaving removes Studio files without breaking the original build.
7. **Ship through one truthful release room.** **Entry:** an owner creates a
   release candidate. **Visible progress:** build, unit, integration, browser,
   accessibility, security, package, migration, proof, and platform gates run
   fresh. **Human decision:** named owners approve release and rollback.
   **Completion proof:** artefact digests, checks, signatures, deployment result,
   and known limits form one release record. **Failure and recovery:** any failed
   gate blocks shipped status and rollback stays ready. **Export:** release bundle
   deploys through documented external tools.
8. **Move away without loss.** **Entry:** an owner starts Exit Studio. **Visible
   progress:** source, assets, data, migrations, secrets bindings, environments,
   extensions, and build reproduction are checked. **Human decision:** the owner
   accepts declared losses and revokes Studio bindings. **Completion proof:** a
   clean external machine builds, tests, runs, and verifies the export. **Failure
   and recovery:** the original workspace remains recoverable for 30 days.
   **Export:** the result has no mandatory Studio runtime or vendor account.

## 7. Capability map

### F0 — foundation

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| STU-F0-001 | G0 | Project facts and root binding | Stable project identity | Identity is independent of path, machine, and vendor | Owner registers and moves | Identity and binding events | `unbound` | WHEN a repo moves folders or OS, Studio SHALL retain project identity and safely update only the binding. |
| STU-F0-002 | G0 | Typed nodes, ports, relations | Versioned application graph | Invalid type or orphan reference cannot publish | Human owns graph meaning | Graph diff and validation | `invalid_graph` | WHEN a node is removed, the system SHALL show every affected screen, flow, test, and migration before commit. |
| STU-F0-003 | G0 | Change command and idempotency key | Domain state plus event | State, outbox, and replay result commit together | Domain policy | Transaction and crash proof | `not_committed` | WHEN a crash occurs during save, recovery SHALL show the old graph or one complete new version. |
| STU-F0-004 | G0 | Tool or agent request | Exact capability lease | Project, process, operation, resource, time, and budget are bounded; no value retrieval | User-owned broker grants | Grant, denial, use, revoke events | `denied` or `revoked` | WHEN a lease is revoked, the process SHALL fail its next protected call without exposing the secret. |

### F1 — useful product

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| STU-F1-001 | G1 | Screen, component, states, data bindings | Accessible responsive UI source | Loading, empty, error, slow, success, and recovery states are required | Designer approves | Preview, source diff, tests | `contract_incomplete` | WHEN a data component is published, it SHALL have tested loading, empty, error, and success states. |
| STU-F1-002 | G1 | Domain object, lifecycle, policy | Schema, migration, API contracts | Real fields replace generic metadata; destructive change needs plan | Domain owner approves | Schema diff, policy and migration tests | `migration_blocked` | WHEN a required field is added, the system SHALL generate or demand a backfill and rollback plan. |
| STU-F1-003 | G1 | Trigger, typed steps, decisions, approvals | Executable workflow | Every side effect has authority, retry, timeout, and recovery | Owner approves side effects | Simulation and branch tests | `waiting`, `failed`, `compensating` | WHEN a step retries, idempotency tests SHALL prove it does not duplicate the side effect. |
| STU-F1-004 | G1 | App graph and source tree | Deterministic generated source | Same inputs and toolchain create the same semantic output | User approves patch | Generator version, diff, build | `generation_failed` | WHEN generation runs twice from identical inputs, the second run SHALL produce no source diff. |

### F2 — category leader

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| STU-F2-001 | G2 | Existing repo and adapter rules | Mapped graph and boundary report | Unsupported code stays untouched and visible | Team selects round-trip scope | Mapping, loss, baseline hash | `partial_import` | WHEN a construct is unsupported, import SHALL label it read-only and SHALL NOT rewrite it. |
| STU-F2-002 | G2 | Scope, paths, capabilities, goal | Agent worktree and proposed patch | Agent cannot widen scope, approve, or merge; production state is isolated | User approves plan/tools/patch | Timeline, tool calls, diff, checks | `stopped` or `needs_approval` | WHEN the user cancels, the lease SHALL revoke and no protected tool call SHALL succeed afterward. |
| STU-F2-003 | G2 | Component contract and variants | Reusable design-system package | Semantics, accessibility, tokens, motion, and states are tested | Design-system owner publishes | Visual, interaction, a11y tests | `breaking_change` | WHEN a component contract breaks, dependants SHALL be listed and release SHALL require a version change. |
| STU-F2-004 | G3 | Release candidate and declared matrix | Canonical release record | Real fresh checks run; failed or skipped gate cannot be green | Named humans release | Logs, artefact digests, signatures | `release_blocked` | WHEN any required gate is skipped, the candidate SHALL not enter shipped state. |

### F3 — advanced category leadership

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| STU-F3-001 | G3 | Graph, trace, failing behaviour | Cross-layer causal view | Link follows typed contracts and real runtime evidence, not name similarity alone | User reads; agent suggests | Trace, graph links, confidence | `cause_unknown` | WHEN evidence is insufficient, the view SHALL say unknown and list the next check. |
| STU-F3-002 | G3 | Bounded goal, variants, budgets, checks | Ranked implementation options | Search stays sandboxed; ranking shows trade-offs and cannot self-merge | Human chooses one option | Candidates, failures, checks, costs | `inconclusive` | WHEN no candidate passes all hard checks, the system SHALL return none as acceptable. |
| STU-F3-003 | G3 | Extension manifest and package | Sandboxed extension | Capabilities are explicit; extension cannot access host or secret by default | Owner installs and grants | Package signature, grants, calls | `quarantined` | WHEN an extension requests a new capability, execution SHALL pause until explicit approval. |
| STU-F3-004 | G3 | Full project and exit plan | Self-contained external project | External build, test, run, migrate, and verify do not call Studio | Owner approves exit | Clean-machine proof and revoke events | `exit_incomplete` | WHEN Studio is unavailable, a clean supported machine SHALL still build and run the exported product. |

### F4 — frontier network

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| STU-F4-001 | G4 | Project-approved source slice or sealed build input, exact toolchain, checks, budget, remote worker capability | Federated build or review result with signed artefact and reproducible log | Source moves only when explicitly granted; worker receives no raw secrets; result cannot self-merge or self-release | Project owner approves input and worker; separate humans approve findings, patch, and release | Lease, input digest, environment, commands, checks, artefact digest, worker signature, revocation | `worker_untrusted`, `result_incomplete`, or `reproduction_failed` | WHEN a clean local verifier cannot reproduce the declared result, Studio SHALL reject it from the release gate. |
| STU-F4-002 | G4 | Signed extension or capability package, provider-neutral manifest, requested operations, policy, compatibility tests | Verified exchange entry and exact install lease | Listing does not grant access; every host capability is denied until user approval; provider removal cannot break project truth | Publisher signs package; exchange governance reviews metadata; project owner installs and grants each capability | Package/source digest, SBOM, tests, signatures, grants, calls, reports, revocations | `quarantined`, `incompatible`, or `capability_denied` | WHEN an extension requests an undeclared capability or fails isolation, the exchange SHALL quarantine that version and notify affected owners. |
| STU-F4-003 | G4 | Project-opted structural patterns, anonymised check outcomes, domain labels, contribution policy | Cross-project pattern library with evidence, counterexamples, and local suggestion | Source, secrets, customer data, proprietary text, and project identity do not enter the shared pattern; suggestion never edits or ranks a project | Each project owner opts in and may revoke; governance approves schema/privacy; human chooses any local use | Contribution manifest, privacy checks, pattern evidence, counterexamples, local acceptance/rejection, withdrawal | `privacy_check_failed`, `insufficient_evidence`, or `contribution_revoked` | WHEN a project revokes a contribution, future pattern builds SHALL exclude it while local projects and other contributors remain intact. |

## 8. Domain model

`StudioProject`, `ProjectBinding`, `ProductContract`, `UserJourney`,
`ApplicationGraph`, `GraphVersion`, `Module`, `Screen`, `Route`, `Component`,
`ComponentContract`, `Variant`, `VisualToken`, `ContentKey`, `DomainObject`,
`Field`, `Relation`, `Invariant`, `Lifecycle`, `PolicyRule`, `DataSource`,
`Action`, `Workflow`, `Step`, `HumanDecision`, `AgentRole`, `CapabilityContract`,
`Environment`, `SecretBinding`, `SourceBoundary`, `GeneratedFile`, `TestCase`,
`Preview`, `Build`, `Artifact`, `Migration`, `ReleaseCandidate`, `Release`,
`Deployment`, `Rollback`, `Extension`, and `ExitManifest` are explicit records.
Graph, contract, component, schema, workflow, policy, test, migration, and release
meaning is versioned. Generated files record owner and regeneration boundary.
Generic metadata cannot stand in for a field, action, permission, lifecycle,
component state, test, or migration.

## 9. System architecture

- A Rust graph kernel validates typed nodes, references, lifecycle, policy,
  source ownership, and change impact.
- A versioned intermediate representation connects visual authoring to source
  adapters and keeps a readable repository representation.
- Language and framework adapters parse, generate, format, build, and test only
  declared boundaries. Unsupported code remains user-owned and read-only.
- Sandboxed local workers and containers run previews, agents, migrations,
  builds, browsers, security tools, packaging, and export. Production state is
  never the development default.
- PostgreSQL stores managed collaboration state; an embedded store supports local
  work; Git remains the source history; object storage holds artefacts and proof.
- A transactional outbox commits Studio records and audit events together.
- HelixCore supplies identity, policy, audit, capabilities, jobs, objects,
  billing, operations, stable project identity, notifications, and recovery.
- A provider-neutral capability broker gives one approved process exact tools,
  paths, operations, time, and budgets. Secrets and signing keys never enter the
  graph, source, chat, logs, evidence, or agent output.

## 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Visible progress, check, stop, reverse |
|---|---|---|---|---|---|
| Product agent | Approved contract, research, graph | Journeys, requirements, acceptance tests | Product meaning and scope | Mark target state complete or invent user evidence | Shows assumptions and gaps; contract lint checks; versions can be rejected. |
| Design agent | Approved journeys, content, tokens, components | Screens, states, copy, interaction variants | Publish and meaning-changing move | Hide states, bypass accessibility, or replace source truth | Live preview and a11y checks; designer accepts; graph undo/restore works. |
| Engineering agent | Allowed source, graph, tests, docs; sandbox tools | Patch, migration, tests, docs | Plan, new dependency, sensitive tool, final patch | Read secrets, write production state, merge, release, self-approve | Durable timeline shows files/tools/checks; cancel revokes lease; worktree is recoverable. |
| Review agent | Diff, contracts, tests, artefacts, evidence | Findings and suggested fixes | Finding disposition and merge | Change code silently or approve own fix | Each finding links evidence; fresh rerun checks closure; rejected finding is recorded. |
| Release agent | Approved candidate, gates, package/deploy adapters | Release plan and rollback steps | Migration, cost, signing, deployment, release | Turn skipped/failed gate green or export key | Streams every gate and artefact; release owner signs; rollback is a named action. |

All roles use narrow leases. Agents can see capability names and request them,
but only a separate user-owned broker grants a specific approved process. A
signing action is brokered without exporting the private key.

## 11. Trust, safety, and privacy

Access combines tenant, project, environment, source path, object, operation,
role, purpose, process identity, time, and capability. Sandboxes deny network,
host filesystem, credentials, devices, and production state unless separately
granted. Tenant separation applies in database, object storage, indexes, caches,
jobs, logs, and previews. Source, customer data, prompts, designs, build logs,
secrets, and regulated data use separate sensitivity classes. Encryption is
required in transit and at rest. Data residency is checked before remote work.

Delete moves projects, branches managed by Studio, screens, components,
workflows, environments, and artefacts to a recoverable 30-day bin. Git history,
released artefacts, audit, legal holds, and external systems follow their own
visible retention. Permanent deletion is a separate, re-authenticated, explicit,
audited act. Controls cover prompt injection in repos, malicious dependencies,
build scripts, sandbox escape, path traversal, supply-chain attacks, secret
leakage, agent scope expansion, forged test results, unsafe migrations, extension
abuse, and deployment takeover. Incident recovery stops jobs, revokes leases and
extensions, rotates bindings, quarantines artefacts, restores signed state, and
proves what was or was not released.

## 12. Proof and audit

Proof records product contract version, graph and source hashes, human approvals,
agent plan, exact lease, tool calls as metadata, source diff, dependency changes,
test inputs and results, preview captures, accessibility checks, migration plan,
build environment, artefact digest, package, signer, deployment response,
rollback, and known limits. An independent verifier can check signatures,
content hashes, graph/schema contracts, fresh commands, artefact identity, and
release gates. Proof does not show that the product meets user needs, that tests
are complete, that a dependency is harmless, or that deployment infrastructure
is honest beyond captured evidence.

Aether is the preferred provider-neutral proof and capability service. A local
ledger, signer, verifier, lease engine, and offline evidence bundle remain fully
functional. Secret values never enter Aether or proof.

## 13. UX system

The canonical shipping boundary is one Studio shell with Home, Product, Design,
Data, Logic, Agents, Code, Test, Release, Operate, Evidence, and Recovery. The
same selected object stays in context across visual, code, data, policy, test,
and runtime views. Basic mode leads with the journey and safe next action. Expert
mode reveals typed graph, AST or source boundary, environment, capability,
performance, proof, and raw logs. The Studio itself and generated component
defaults target [WCAG 2.2 Level AA](https://www.w3.org/TR/WCAG22/).

Every long agent, preview, test, build, migration, import, and deployment has a
durable live timeline: plan, current stage, completed checks, changed files,
last heartbeat, time, cost, pause/cancel, and expected next signal. Completion
creates a durable inbox item and optional desktop notice; no success vanishes in
a toast. Selected and multi-selected objects show checks and count. Drag or move
previews destination, responsive effect, broken references, and source diff;
meaning changes require confirmation. Undo covers safe graph edits. Delete uses
the 30-day bin. Empty states provide a sample or the first real step. Errors say
what changed, what did not, where proof lives, and how to recover.

## 14. Interoperability and standards

- The [OpenAPI Specification](https://spec.openapis.org/oas/latest.html) imports
  and exports language-neutral HTTP API contracts. Runtime policy, event meaning,
  and every framework extension may not map.
- The [WHATWG HTML custom-elements standard](https://html.spec.whatwg.org/dev/custom-elements.html)
  supports portable web component adapters. Framework lifecycle and styling may
  still need a declared wrapper.
- [WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria/) supports accessible roles,
  states, and properties when native HTML is not enough. ARIA never replaces
  correct keyboard behaviour or native semantics.
- The [OCI Image Specification](https://specs.opencontainers.org/image-spec/)
  packages interoperable container images. An image format does not prove runtime
  security, platform support, or a safe build.
- [SPDX 3.0.1](https://spdx.github.io/spdx-spec/) exports software bill of
  materials and provenance facts. Unknown or incomplete dependency data stays
  marked unknown.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) sets the accessibility target.

Standards are version-pinned adapters with fixtures. Import and export show lost
types, validation, policy, workflow semantics, component states, source comments,
extensions, signatures, and environment assumptions before commit.

## 15. Cross-platform contract

Graph validation, deterministic generation, source mapping, migration planning,
proof, and recovery use the same fixtures on Windows, macOS, and Linux. Desktop
is the full local factory with files, sandbox, broker, local builds, browsers,
and notifications. Web mode supports managed authoring and review but clearly
labels missing local capabilities. The CLI supports validate, generate, diff,
test, build, package, export, verify, and recovery. Containers are optional
workers and deployment artefacts, not the only runtime. Offline mode supports
contracts, design, graph, code, local preview, tests, and export within installed
toolchains. Filesystem watch, secure storage, container runtime, GPU, browser,
signing hardware, and notifications use capability detection with polling,
local process, CPU, file, or in-app fallback.

## 16. Reliability and performance budgets

- Acknowledged graph, policy, release, approval, and audit writes have zero
  allowed data loss in forced-crash tests.
- Direct manipulation responds within 100 ms at p95 over a rolling 30-day window
  for screens with 10,000 nodes on supported desktop hardware.
- Incremental local preview begins within 2 seconds at p95 for a 100,000-line
  supported project; full-build progress starts within 2 seconds even if build is longer.
- Long work has a durable stage and heartbeat no older than 5 seconds while its
  local worker is active.
- Cancellation is accepted within 2 seconds, revokes new protected calls
  immediately, and stops a local worker within 30 seconds; external work remains
  `cancel_requested` until confirmed.
- Generate, build, migration, deploy, and agent commands are idempotent where
  they have side effects; retries never duplicate a release or migration.
- Concurrent graph edits never silently overwrite; merge is semantic for known
  nodes and an explicit conflict for meaning changes.
- A local project supports 1 million graph nodes, 1 million source files indexed
  incrementally, and 500 GB artefacts with declared storage limits.
- Managed committed metadata has recovery point zero and 1-hour recovery time;
  self-hosted documented recovery target is 4 hours.
- If models, Aether, managed cloud, container engine, or notifications fail,
  manual visual/code work, local process builds, tests, proof, and export continue.

## 17. Success measures

- Users complete all eight journeys and can explain the graph-to-source boundary.
- Generated projects build and test on a clean machine without Studio.
- Meaning-changing edits caught before source, schema, or migration damage.
- Long work with a real heartbeat, cancel, completion record, and no false green.
- Agent patches accepted after review, rejected safely, and never outside lease.
- Release candidates whose declared fresh gates all ran; skipped gates are zero.
- Independent proof ties shipped artefact to source, checks, approval, and signer.
- Accessibility task success and serious issue counts in Studio and generated defaults.
- Restore and complete exit success on all supported operating systems.
- Teams paying because delivery is safer and faster, not because code is locked in
  or because the product reports more agent calls.

## 18. Delivery plan

| Gate | Build | Test | Safety | UX | Cross-platform | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Stable project identity, typed graph, atomic ledger, capability broker, jobs, recovery | Graph, crash, lease, signature, deterministic tests | Sandbox and secret threat model; production-state isolation | Canonical shell, honest timeline, Recovery | Rust/core and packaging CI on Windows, macOS, Linux | Dry-run importer for current apps/pages | Fresh move-folder, edit, crash, revoke, restore, verify |
| **G1 — Useful single-player product (6–18 months)** | Screen/component editor, real domain model, workflows, source generation, preview | Complete app journey and generated-project tests | Dependency, policy, migration checks | Accessible visual/code/data flow | Desktop, web, CLI, container, offline local process | Graph/source schema migrations | Fresh small app built outside Studio on each OS |
| **G2 — Trusted team product (18–30 months)** | Repo import, agent worktrees, components, collaboration, review | Round-trip, multi-user, scope, cancellation tests | External sandbox and supply-chain review | Impact preview, review, slow-work notices | Toolchain and browser matrix | Existing-repo mapping and rollback | Fresh import, agent patch, reject/accept, incident drill |
| **G3 — Category leader (30–42 months)** | Canonical release room, causal graph, adapter kit, operations | Real fresh full-stack release gates and scale tests | Migration, deployment, extension security review | Evidence and release comprehension | Full Windows/macOS/Linux packaging CI | Verified live project and environment migration | Independent source-to-package and rollback proof |
| **G4 — Frontier network (42–60 months)** | Build STU-F4-001 federated build/review, STU-F4-002 verified capability exchange, and STU-F4-003 consented cross-project pattern library | Reproduction, source-scope, secret, sandbox-escape, supply-chain, undeclared-capability, poisoning, privacy, revoke, provider-loss, and malicious-worker tests | Independent architecture, supply-chain, privacy, security, marketplace, and AI-governance review; no self-merge or release | Exact grant, worker progress, result compare, extension install, quarantine, contribution, withdrawal, human choice, and exit journeys | Mixed Windows/macOS/Linux workers and owners prove clean reproduction, offline verification, provider loss, and local fallback | Remove Studio, a worker, provider, extension, or project contribution; revoke bindings without source, identity, or shared-secret loss | Independent dispatch, reproduce, review, install/quarantine, contribute/revoke, clean-machine build/run/verify, disaster, and exit exercise covering all F4 evidence |

Every gate closes only with fresh release-candidate evidence. A generated page,
mock preview, old test report, or green wrapper around skipped checks is not proof.

## 19. Current truth and gap

The live Rust source is a generic scaffold with `apps` and `pages` create, list,
and get operations. Its schema mainly stores generic metadata. It does not have a
canvas, typed application graph, component contract, source parser or generator,
data model, workflow engine, sandbox, agent authority, preview, test system,
release room, product UI, or domain test suite. The service also has the shared
application-state compile failure. Domain changes and audit or billing work are
not one atomic operation.

The most important gap is a real typed model that can support both visual and
code truth. The safest first slice is STU-F0-001 through STU-F1-001: stable
project identity, one typed screen graph, one accessible data component with all
required states, deterministic source generation, source diff, local preview,
tests, export, and forced-crash recovery. Use temporary test state and a sandbox;
do not touch a production project or state path.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Identity | Stable project, graph, node, environment, and release IDs independent of folder paths | Architecture decision and move proof |
| Source truth | Readable repository files plus typed graph; ownership boundary is explicit per file | Architecture review |
| Round trip | Supported constructs only; unsupported code is read-only and never silently rewritten | Adapter conformance proof |
| Generation | Deterministic, diff-first, buildable without Studio | Engineering review |
| Agents | Isolated worktree and temporary state; exact lease; no self-approval, merge, or release | Safety review |
| Secrets/signing | Separate user-owned broker performs exact operation; values and private keys never exported | Security review |
| Release | One canonical release room runs real fresh gates; skipped means blocked | Founder-approved release policy change |
| Proof | Aether preferred through neutral interface; complete local fallback | Provider-neutrality review |
| Delete | 30-day recovery bin; releases, Git, audit, and legal holds have stated retention | Retention decision |
| Accessibility | WCAG 2.2 AA target for Studio and generated defaults | Accessibility review |
| First slice | Typed screen → deterministic source → preview → tests → export → crash recovery | Product decision with equal foundation proof |
| Managed hosting, marketplace, autonomous release | Off until separate G3/G4 and founder/security/business gates | Founder approval |

## 21. Definition of category-defining done

- [ ] Real users complete all eight journeys on serious applications.
- [ ] Visual, graph, source, runtime, tests, and release evidence remain connected.
- [ ] Generated projects are readable and work on a clean machine without Studio.
- [ ] Unsupported code is preserved and every round-trip boundary is honest.
- [ ] Agents are isolated, least-authority, stoppable, reviewable, and never self-approve.
- [ ] Secrets and signing keys never enter source, chat, logs, proof, or agents.
- [ ] Release status comes only from fresh real Python, Rust, web, desktop,
  browser, packaging, migration, security, and accessibility checks in scope.
- [ ] Independent proof ties source and approvals to the shipped artefact.
- [ ] WCAG 2.2 AA scope passes Studio and generated-product human review.
- [ ] Windows, macOS, Linux, web, offline, CLI, local process, and container limits are proven.
- [ ] The 30-day bin, permanent delete, migration, rollback, restore, and complete exit work.
- [ ] External security, architecture, accessibility, supply-chain, and AI-governance reviews close.
- [ ] The product clearly states what generation, tests, and proof do not guarantee.
