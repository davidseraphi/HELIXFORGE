---
product: HelixCode
catalog_order: 02
status: target-state-spec
horizon: 60 months
current_maturity: alpha
primary_users: [individual developer, software team, reviewer, release owner, security engineer, platform engineer]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
---

# 1. Category claim

HelixCode is a sovereign software forge where humans and bounded agents can understand, change, test, review, prove, recover, and ship code without surrendering repositories or execution authority.

# 2. Five-year destination

- **Useful product:** A fast editor, Git forge, issues, reviews, pipelines, terminals, debugging, environments, packages, releases, and agent work live in one coherent product.
- **Category-defining advantage:** Every meaningful change carries its intent, authority, checks, provenance, review, artifact, and honest limits from request to release.
- **Frontier capability:** Teams can run a policy-bound network of human and agent builders across local and remote machines while preserving exact custody and proof.
- **Human authority:** People approve requirements, capability grants, protected-branch changes, security exceptions, releases, deployment, and permanent deletion.

# 3. Users and hard jobs

| User | Hard jobs | Failure feared most |
|---|---|---|
| Individual developer | Open, understand, edit, run, debug, commit, and recover code | Tooling damages the working tree or traps the repository |
| Software team | Plan, review, merge, coordinate, and keep branch truth clear | A merge passes without the required review or tests |
| Reviewer | Understand intent and risk, inspect proof, reproduce checks | Large agent changes hide a dangerous mistake |
| Release owner | Build the exact candidate across targets and prove what shipped | Tested source differs from the shipped artifact |
| Security engineer | Control commands, secrets, dependencies, runners, and webhooks | Untrusted code escapes its sandbox or receives a secret |
| Platform engineer | Run scalable, repeatable development and CI environments | A platform difference makes production fail after green CI |

# 4. Product laws

1. The repository stays valid Git and remains usable with normal Git tools.
2. HelixCode never changes, discards, resets, or publishes user work without a visible plan and authority.
3. A path is a location, not repository, workspace, agent, or proof identity.
4. Agent work happens in an isolated branch or work cell with exact tools and capabilities.
5. Protected branch, release, deployment, secret, and destructive actions require human authority.
6. Tests, reviews, and release gates run fresh against the exact source and artifact.
7. Slow clone, index, build, test, debug, agent, package, and deploy work shows real progress and a completion notification.
8. Delete moves repositories, branches, workspaces, artifacts, and environments to a recoverable 30-day bin where technically possible.
9. Security checks state covered scope, skipped checks, and limits. Green never means risk-free.
10. Windows, macOS, and Linux are equal product targets.
11. Extensions and external tools run with declared permissions and can be disabled without losing owned code.
12. High-stakes domain code still needs an accountable qualified human decision.

# 5. Scope boundaries

**HelixCode owns:** source repositories, refs, worktrees, code search, editor state, language and debug adapters, terminals, issues, pull requests, reviews, branch policy, developer environments, pipelines, artifacts, packages, releases, code agents, supply-chain evidence, repository recovery, and portable forge export.

**HelixCore owns:** identity, tenant policy, capabilities, durable jobs, object custody, shared proof, billing, common operations, and stable project identity. HelixCollab owns general documents. HelixFlow owns reusable cross-product automation.

**HelixCode will not attempt:** to replace Git; to be a general container platform; to let an agent self-merge or self-release; to guarantee code has no defect; to make one model vendor required; or to replace licensed security, safety, legal, financial, or medical review.

# 6. Signature experiences

| Journey | Entry point | Visible progress | Human decision | Completion proof | Failure and recovery | Export or portability |
|---|---|---|---|---|---|---|
| Import or create a repository | New repository | Scan, object validation, default branch, index, policy, and environment phases | Choose ownership, visibility, policy, and remote links | Repository inventory, object checks, and registration receipt | Source remains untouched; partial import is discarded or resumed | Standard Git clone plus forge metadata export |
| Understand and edit safely | Repository workspace | Index state, diagnostics, unsaved files, background checks, connection state | Developer chooses edits and commit boundary | Diff, diagnostics, test results, and signed commit context | Crash restores unsaved buffer; conflicting disk change gets a merge view | Working tree and standard Git history |
| Delegate a bounded code task | Agent work cell | Plan, sources read, files changed, commands, tests, elapsed time, heartbeat | Approve scope/capabilities and accept or reject patch | Patch, transcript metadata, checks, limitations, and reviewer decision | Stop revokes leases; partial branch remains separate; undo never touches user branch | Export patch and proof bundle |
| Review and merge a pull request | Pull request | Review state, changed risk areas, checks, unresolved comments, merge readiness | Required humans approve; release owner handles exception | Merge receipt bound to exact head, policy, reviews, and checks | Head change invalidates approval; failed merge keeps both refs | Standard Git refs plus review metadata |
| Debug a failing program | Run and Debug | Adapter launch, process state, threads, stack, variables, output, resource use | User approves execution and any sensitive capability | Debug session record and chosen fix diff | Stop kills child tree; crash leaves no hidden process; workspace restores | Export launch config and redacted session report |
| Build and release across platforms | Release center | Queue, runner, steps, logs, artifacts, signatures, Windows/macOS/Linux matrix | Named release owner approves exact candidate | Source-to-artifact map, SBOM, checks, signatures, and release receipt | Any unknown blocks release; retry is idempotent; rollback package ready | Standard packages, images, source, SBOM, and proof |
| Recover deleted or damaged work | Recovery | Reflog/object scan, bin expiry, dependencies, restore preview | Choose restore target or permanent purge | Restored refs and integrity check | Conflict restores to a new name; corrupt object is isolated | Standard Git bundle before purge |
| Move the forge to another home | Operations, Portability | Repositories, LFS/objects, issues, reviews, policies, runners, keys, verification | Approve cutover after target rehearsal | Complete inventory and target health proof | Source remains authoritative until cutover; one-action rollback | Git bundles plus open forge manifest |

# 7. Capability map

## F0 — Foundation

### CODE-F0-001 — Git-faithful repository service

- **Gate:** G0. **Inputs:** Git objects, refs, pack requests, identity, policy. **Outputs:** valid repositories, reads, commits, fetch, and push.
- **Invariants:** object integrity; atomic ref updates; no silent history rewrite. **Authority:** scoped contributor writes; reader fetches.
- **Evidence:** old/new ref, actor, policy, object set, and checks. **Failure:** invalid object or stale ref blocks update.
- **Acceptance:** repositories round-trip through standard Git clients with identical reachable object hashes.

### CODE-F0-002 — Tenant and repository policy

- **Gate:** G0. **Inputs:** subject, tenant, repository, branch, action, protection policy. **Outputs:** allow or deny with reason.
- **Invariants:** storage and application both bind tenant; admin cannot bypass without explicit break-glass. **Authority:** owners publish policy.
- **Evidence:** sensitive policy decisions and break-glass events. **Failure:** unavailable policy fails closed for writes.
- **Acceptance:** adversarial tests cannot read, push, review, or run against another tenant’s repository.

### CODE-F0-003 — Isolated execution cell

- **Gate:** G0. **Inputs:** source snapshot, command contract, limits, capabilities, network policy. **Outputs:** logs, exit status, artifacts, resource record.
- **Invariants:** least authority; no host or sibling workspace mutation; secret use is brokered. **Authority:** user approves risky execution.
- **Evidence:** image/runtime facts, command, capability uses, outputs, and limits. **Failure:** breach signal stops and quarantines the cell.
- **Acceptance:** escape, fork-bomb, network, filesystem, and secret-exfiltration tests fail safely on all supported hosts.

### CODE-F0-004 — Recoverable source lifecycle

- **Gate:** G0. **Inputs:** delete, restore, retention, legal hold. **Outputs:** tombstone, 30-day bin, restored ref/repository, or purge.
- **Invariants:** standard Git recovery remains available; purge is separate. **Authority:** owner purges.
- **Evidence:** impact, object reachability, and lifecycle events. **Failure:** shared object or dependency prevents unsafe purge.
- **Acceptance:** a repository deleted on day 1 restores with refs, issues, policies, and review links on day 29.

## F1 — Useful product

### CODE-F1-001 — Cross-platform code workspace

- **Gate:** G1. **Inputs:** repository, worktree, editor state, files, user settings. **Outputs:** responsive editor with safe unsaved state.
- **Invariants:** disk and buffer divergence is visible; no editor format destroys content. **Authority:** developer saves and commits.
- **Evidence:** save/restore events only where useful, not keystroke surveillance. **Failure:** crash recovery opens a reviewable buffer.
- **Acceptance:** edit, search, split, move tab, restart, and recover journeys pass on Windows, macOS, Linux, and web.

### CODE-F1-002 — Language and debug intelligence

- **Gate:** G1. **Inputs:** source, language server, debug adapter, build facts. **Outputs:** diagnostics, navigation, completion, stack, variables.
- **Invariants:** adapter capability is detected; stale result is labelled. **Authority:** user controls process launch and evaluation.
- **Evidence:** adapter version, request scope, and session outcome. **Failure:** adapter loss leaves plain editing usable.
- **Acceptance:** unsupported language/debug features degrade clearly without blocking file access.

### CODE-F1-003 — Durable pipelines and artifacts

- **Gate:** G1. **Inputs:** versioned pipeline, exact source, runner, capability lease, idempotency key. **Outputs:** step states, logs, artifacts, checks.
- **Invariants:** source cannot change under a run; retry cannot duplicate release side effects. **Authority:** users trigger; protected deployment needs approval.
- **Evidence:** runner facts, step commands, inputs, outputs, artifacts, and signatures. **Failure:** worker loss resumes or fails honestly.
- **Acceptance:** killing a runner at every step boundary produces one terminal state and no false success.

### CODE-F1-004 — Review and protected merge

- **Gate:** G1. **Inputs:** exact head/base, diff, policy, checks, reviews, risk classification. **Outputs:** merge, rejection, or blocked state.
- **Invariants:** head change invalidates approval; required checks are fresh. **Authority:** required reviewers and merge owner.
- **Evidence:** review comments, approvals, check receipts, and merge ref update. **Failure:** race at merge cannot bypass protection.
- **Acceptance:** concurrent push during merge either updates the reviewed head atomically or blocks merge.

## F2 — Category leader

### CODE-F2-001 — Agent work cells

- **Gate:** G2. **Inputs:** task contract, selected source, tool policy, capability lease, budget. **Outputs:** separate branch, patch, checks, evidence.
- **Invariants:** agent cannot self-approve, self-merge, expand scope, or retrieve secrets. **Authority:** human owns every consequential transition.
- **Evidence:** plan, reads, tool calls, changes, checks, resource use, and limitations. **Failure:** stop revokes capabilities and preserves partial work.
- **Acceptance:** an agent completes a multi-file task while denied access to an unrelated repository and ungranted network target.

### CODE-F2-002 — Supply-chain proof

- **Gate:** G2. **Inputs:** source, locks, toolchains, dependencies, build cell, artifact. **Outputs:** SBOM, provenance, signatures, policy result.
- **Invariants:** artifact hash binds all proof; missing input is unknown, not passed. **Authority:** release owner accepts exceptions.
- **Evidence:** machine-readable bill of materials and source-to-artifact lineage. **Failure:** post-build artifact change invalidates release.
- **Acceptance:** an offline verifier checks artifact integrity, builder identity, source, dependency inventory, and named tests.

### CODE-F2-003 — Full forge collaboration

- **Gate:** G2. **Inputs:** issues, plans, branches, pull requests, reviews, discussions, ownership. **Outputs:** linked delivery graph.
- **Invariants:** issue closure does not imply code success; author and reviewer roles are explicit. **Authority:** team policy controls workflow.
- **Evidence:** relationship graph and state transitions. **Failure:** broken link is visible and repairable.
- **Acceptance:** a release traces back to its approved requirement, code changes, reviews, checks, and remaining risks.

### CODE-F2-004 — Reproducible development environments

- **Gate:** G2. **Inputs:** environment contract, toolchain locks, source, host facts, cache policy. **Outputs:** local or remote work cell.
- **Invariants:** same contract has measured equivalence; cache never changes semantic output. **Authority:** owner approves privileged features.
- **Evidence:** resolved image/tools, host capability, and comparison result. **Failure:** unsupported host gives exact missing capability.
- **Acceptance:** reference projects build and test with matching results on all three supported systems and Linux CI cells.

## F3 — Advanced category leadership

### CODE-F3-001 — Verified development graph

- **Gate:** G3. **Inputs:** requirements, decisions, source, reviews, checks, artifacts, incidents. **Outputs:** queryable proof graph.
- **Invariants:** every edge has origin and confidence; absence remains visible. **Authority:** humans ratify requirement and exception edges.
- **Evidence:** signed graph snapshots and verifier reports. **Failure:** stale proof cannot attach to changed source.
- **Acceptance:** an auditor answers why an artifact exists and what remains unproven without trusting the forge database.

### CODE-F3-002 — Multi-agent engineering studio

- **Gate:** G3. **Inputs:** decomposed tasks, separate work cells, shared contracts, review topology, budget. **Outputs:** competing or complementary patches and synthesis.
- **Invariants:** agents do not approve each other as the final authority; merge is conflict-aware. **Authority:** human chooses synthesis.
- **Evidence:** task lineage, independent checks, disagreement, and final decision. **Failure:** collision pauses before shared-state mutation.
- **Acceptance:** three agents work in parallel without sharing write state and produce a human-reviewable combined change.

### CODE-F3-003 — Sovereign build mesh

- **Gate:** G4. **Inputs:** signed job, target matrix, data/residency policy, runner capabilities. **Outputs:** placed builds and comparable artifacts.
- **Invariants:** source and secrets stay within allowed custody; untrusted runner output is quarantined. **Authority:** owner sets trust.
- **Evidence:** runner attestation, placement reason, artifacts, and reproducibility comparison. **Failure:** no compliant runner means no build.
- **Acceptance:** independent homes contribute Windows, macOS, and Linux builds while one verifier checks the complete release.

## F4 — Frontier network

### CODE-F4-001 — Cross-home change exchange

- **Gate:** G4. **Inputs:** stable repository identity, exact base revision, signed change bundle, intent, checks, review policy, and destination trust. **Outputs:** imported review branch, verified diff, local checks, review decision, or rejection.
- **Invariants:** an incoming bundle never writes directly to a protected branch, carries no raw secret, and cannot grant itself merge authority. **Authority:** the destination owner chooses trust, reviewers, checks, and merge.
- **Evidence:** source home, base and head hashes, intent, patch inventory, signatures, checks, comments, decision, and final local revision. **Failure:** base mismatch, missing artifact, invalid signature, or policy conflict quarantines the bundle without changing the working tree.
- **Acceptance:** one home exports a multi-file change, a second home verifies and reviews it against the named base, rejects one section, merges the accepted remainder locally, and preserves a complete cross-home proof trail.

### CODE-F4-002 — Independent release assembly

- **Gate:** G4. **Inputs:** exact source revision, signed build contract, target matrix, independent runner policies, artifacts, and required agreement rule. **Outputs:** comparable artifacts, difference report, verified release candidate, or blocked assembly.
- **Invariants:** no single runner can declare the release valid; each artifact stays bound to source, toolchain, platform, and runner. **Authority:** the release owner chooses approved runners, the agreement threshold, exceptions, and final release.
- **Evidence:** runner attestations, environment facts, artifact hashes, reproducibility class, disagreements, reviews, exception approvals, and release signature. **Failure:** missing target, unexplained artifact difference, malicious runner, or insufficient agreement blocks release and quarantines affected output.
- **Acceptance:** three independent homes build the Windows, macOS, and Linux matrix; a seeded malicious artifact is rejected, and the remaining matching or explicitly equivalent artifacts produce one independently verifiable release.

# 8. Domain model

| Record | Owner and relationships | Lifecycle and version rules | Retention |
|---|---|---|---|
| Repository | Tenant project; contains refs, policies, issues, reviews | Stable ID independent of storage path | 30-day bin before purge |
| GitObject | Repository object graph; may be shared by reachability | Immutable by hash | While reachable, held, or in recovery |
| RefUpdate | Ref, old/new object, actor, policy, result | Append-only atomic attempt | Repository audit retention |
| Workspace | User/repository worktree and editor state | Versioned environment and branch binding | 30-day bin |
| ChangeSet | Exact base, head, diff, intent, author | Immutable when submitted | Repository history |
| Issue | Repository problem or request linked to decisions/change sets | State changes retained | Project policy |
| PullRequest | Base/head, reviews, checks, merge policy | Head revisions create review epochs | Repository history |
| Review | Reviewer, exact head, findings, decision | New review after head change | Repository history |
| PipelineDefinition | Repository-owned job graph | Immutable published versions | All used versions |
| PipelineRun | Exact source plus step attempts and artifacts | Durable state machine; retries linked | Operational policy plus release proof |
| Artifact | Content-addressed output linked to run and source | Immutable; promotion changes channel, not bytes | Release policy |
| Environment | Toolchain, image, mounts, network, capability contract | Versioned and resolved per host | Used versions retained |
| AgentJob | Task, branch, work cell, authority, budget, result | Durable attempts; no in-place history rewrite | Project policy |
| SecretBinding | Capability name to broker policy, never value | Versioned, revocable, expiring | Metadata retained |
| Release | Candidate, artifacts, approvals, channels, rollback | Immutable release; withdrawal is new state | Long-term |
| RecoveryItem | Deleted repo/ref/workspace/artifact with expiry | Restore, expire, or purge | 30 days default |

# 9. System architecture

- **Domain engine:** Git object/ref service, repository policy, review/merge engine, pipeline scheduler, environment resolver, agent work-cell manager, artifact/provenance service, and recovery engine.
- **Application services:** Git smart transport, forge API, editor backend, language/debug bridge, terminal broker, runner coordinator, package/release service, webhook service, search index, and desktop/web shell.
- **Adapters:** normal Git clients, language servers, debug adapters, local processes, OCI-compatible runtimes, CI runners, package registries, issue importers, HelixCore capability/proof/jobs, and Aether.
- **Storage:** standard Git object stores; tenant-enforced relational forge data; encrypted object storage for artifacts and logs; local worktrees owned by the user.
- **Event flow:** ref update, policy decision, and proof outbox commit atomically where storage permits; pipeline and agent transitions are durable and idempotent.
- **Background work:** clone/fetch, indexing, builds, tests, scans, agent work, packaging, signing, replication, export, and recovery use HelixCore jobs.
- **Offline behaviour:** local Git, editing, search cache, diffs, commits, local tests, and proof verification remain usable. Server reviews, shared policy, remote capabilities, and publication wait for connection.
- **Extension points:** languages, debuggers, runners, scanners, agents, package formats, issue providers, webhooks, proof providers, and custody backends.
- **Dependencies:** HelixCore for shared identity, policy, capabilities, jobs, objects, proof, and operations. Git remains the portable source substrate. Aether is optional.

# 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Progress, check, stop, reverse |
|---|---|---|---|---|---|
| Code explorer | Selected repository and read-only analysis tools | Map, explanation, risk list | None for private draft | Modify files, run commands, or read secrets | Files and phases shown; citations checked; stop closes lease |
| Patch builder | Approved files, isolated work cell, exact command and capability set | Branch, patch, tests, notes | Any merge or scope expansion | Touch user branch, self-approve, retrieve credentials | Diff/test timeline; fresh checks; cancel cell; delete agent branch |
| Reviewer | Exact change, policy, tests, proof, read-only tools | Findings and review decision proposal | Human submits final approval where policy requires | Modify reviewed branch or hide failed checks | Review checklist; reproduce; stop; supersede review |
| Release builder | Exact candidate, locked runners, package/sign tools | Artifacts and release report | Signing, channel promotion, deployment | Use changed source or simulated provider | Platform matrix; verifier; cancel before promotion; rollback release |
| Dependency steward | Manifests, advisories, compatibility tests | Upgrade branch and migration note | Major upgrade, licence exception, release | Auto-merge breaking/security-sensitive change | Dependency graph; fresh tests; stop; revert branch |
| Incident assistant | Redacted logs, proof, affected refs/artifacts | Timeline and containment plan | Revocation, rollback, public notice | Erase evidence or change history | Live tasks; independent checks; stop; restore known-good release |

# 11. Trust, safety, and privacy

- Access combines tenant, repository, branch, action, actor, environment, and purpose. Database and storage policy enforce tenant boundaries.
- Sensitive classes include private source, personal data, credentials, signing operations, release artifacts, vulnerability reports, and incident evidence.
- Secrets are capabilities, not environment dumps. A broker performs exact approved use for one workload and never returns values to agents.
- Untrusted code runs in a bounded work cell with filesystem, process, network, resource, and time limits. Host escape testing blocks release.
- Source, logs, caches, artifacts, and backups obey residency. External models receive only explicitly approved context.
- Webhooks use verified destinations, signed delivery, replay control, private-address protection, and an observable retry/dead-letter path.
- Delete enters the 30-day bin. Git reachability, shared objects, packages, releases, and legal hold are checked before purge.
- Break-glass is time-bound, two-person for protected production actions, visibly active, and followed by review.
- Incident recovery freezes affected capabilities, preserves evidence, rotates bindings, restores a known-good artifact, and reruns release gates.

# 12. Proof and audit

Important repository creation, access, ref update, branch-policy, review, merge, pipeline, agent, secret-use, package, signing, release, deployment, recovery, and break-glass actions create canonical signed events.

The release proof bundle binds requirement and decision references, source commit, dependency locks, environment, runner, commands, checks, review head, artifacts, SBOM, signatures, approvals, and rollback artifact. Independent verification can establish integrity, relationship, named checks, and authority records. It cannot prove code is bug-free, requirements are complete, or a test covers every behaviour.

Aether is the preferred external proof and capability adapter. Local HelixCore signing, verification, and brokering keep normal development available without it.

# 13. UX system

- **Main surfaces:** Home, Repositories, Workspace, Search, Source Control, Issues, Pull Requests, Runs, Agents, Releases, Evidence, and Recovery.
- **Navigation:** editor-style activity rail, repository tree, central editor, panels, command palette, breadcrumbs, and context-aware actions.
- **Progressive reveal:** common edit/run/commit actions stay close; branch policy, sandbox, provenance, raw logs, runner facts, and proof open on demand.
- **Keyboard and touch:** full keyboard operation, visible focus and selections, remappable shortcuts, drag alternatives, and touch-safe review/approval.
- **Accessibility:** target [WCAG 2.2](https://www.w3.org/TR/WCAG22/) AA; terminal and editor accessibility modes receive human tests.
- **Slow work:** clone, index, build, test, agent, package, and deploy show queue, runner, phase, step, logs, heartbeat, elapsed time, cancel safety, and completion notification.
- **Selection and move:** checked files/tabs/changes show count and destination. Moving a tab, file, ref, or issue previews impact and supports undo where safe.
- **Destructive actions:** default to archive/bin; branch/repo purge is visually separate and explains Git recovery limits.
- **Errors:** preserve user buffers, keep logs, name the failed layer, distinguish code failure from platform failure, and offer a reproducible next action.

# 14. Interoperability and standards

- [Git protocol version 2](https://git-scm.com/docs/protocol-v2) and standard Git object/ref behaviour preserve normal client access. Forge reviews, issues, and policies require a separate manifest on export.
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
  is used behind language-intelligence adapters. The official site identified
  3.18 as the latest specification when checked on 2026-07-15. Server-specific
  commands and indexes may not transfer.
- [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/) is used behind debugger adapters. Runtime launch, custody, and sandbox policy remain Helix contracts.
- [Open Container Initiative specifications](https://specs.opencontainers.org/) are used for portable build images and runtime bundles where supported. Host kernel, hardware, and security behaviour can differ and must be reported.
- [SPDX specification](https://spdx.github.io/spdx-spec/) is an SBOM and software provenance exchange format. Helix review, authority, and test proof use linked companion records.
- [JSON Schema 2020-12](https://json-schema.org/draft/2020-12) defines versioned pipeline, environment, agent-task, and proof contracts.

# 15. Cross-platform contract

- Repository service, CLI, editor core, Git transport, search, pipeline contracts, proof verification, and recovery pass CI on Windows, macOS, and Linux.
- Native desktop packages are built, installed, upgraded, rolled back, and uninstalled on all three systems before release.
- Browser mode supports editing, reviews, agents, runs, evidence, and recovery; local terminal/debug/file access is capability-detected and clearly limited.
- Linux containers are one runner type. Native Windows and macOS jobs use native runners where target truth matters.
- Path case, separators, permissions, symlinks, line endings, executable bits, process trees, and filesystem watchers have explicit compatibility tests.
- Offline mode preserves local Git and editor operation. Remote policy state is shown with age; protected publication waits for fresh authority.

# 16. Reliability and performance budgets

| Measure | Budget |
|---|---|
| Git data loss | 0 acknowledged object or ref updates lost per rolling 30 days |
| Ref atomicity | 100% of protected ref updates are compare-and-swap checked in each release test corpus |
| Editor restore | Unsaved buffers checkpoint within 2 seconds and restore after forced exit on reference projects |
| Workspace open | p95 under 3 seconds for a 100,000-file indexed repository over each rolling 24 hours after warm metadata load |
| Search | First useful result under 500 ms p95 over each rolling 24 hours for reference corpus |
| Pipeline state | Initial state within 1 second; runner heartbeat at least every 10 seconds |
| Cancellation | Acknowledged within 2 seconds; child process tree stopped within 30 seconds or exact blocker shown |
| Release integrity | 0 releases whose artifact hash differs from the approved candidate proof |
| Control availability | At least 99.95% eligible forge API success per rolling 30 days |
| Recovery | Restore one repository from the bin within 5 minutes; full reference forge within 4 hours |
| Idempotency | Replayed push, webhook, pipeline, payment, or release command creates one effect within a 24-hour retry window |
| Scale | 100,000 repositories per estate, 10,000 concurrent work cells, and 1 million pipeline steps per day after measured proof |

# 17. Success measures

- 90% of developers import, edit, run, commit, review, recover, and export without support.
- 100% of protected merges bind the exact reviewed head and required fresh checks.
- Zero confirmed cross-tenant repository or secret disclosures per rolling 12 months.
- At least 95% of agent patches are accepted, revised, or rejected through visible diff review rather than direct mutation.
- At least 95% of accidental repository or branch deletions reported within 30 days are restored.
- All critical editor, review, release, and recovery journeys pass keyboard and screen-reader checks.
- A reference forge moves between supported deployments within one working day with no Git object loss.
- Fewer than 2% of long runs complete without notification.
- Teams reduce median request-to-reviewed-change time without increasing escaped high-severity defects.

# 18. Delivery plan

| Gate | Build | Test and safety | UX | Cross-platform | Migration and operator proof |
|---|---|---|---|---|---|
| G0 — Truthful foundation (0–6 months) | Full compile; Git fidelity; tenant policy; sandbox baseline; durable pipelines; 30-day bin | Standard Git corpus, tenant attacks, sandbox escapes, crash/replay tests | Honest editor/run/recovery states | Windows, macOS, Linux build matrix | Import current repos and restore drill |
| G1 — Useful single-player product (6–18 months) | Editor, LSP/DAP, terminal, search, pipelines, review, artifacts | Buffer recovery, adapter degradation, merge races, accessibility | Complete workspace, PR, run, evidence, recovery | Signed desktop packages and browser proof | Clean-machine repository and metadata export |
| G2 — Trusted team product (18–30 months) | Agent cells, supply-chain proof, collaboration graph, reproducible environments | Secret non-disclosure, agent authority, SBOM, provider failover | Agent, policy, release, incident views | Native and container runner equivalence | Team forge migration and rollback |
| G3 — Category leader (30–42 months) | Verified development graph and multi-agent studio | Independent proof and collision tests | Graph exploration and agent synthesis | Offline verifier everywhere | Aether/local fallback exercise |
| G4 — Frontier network (42–60 months) | Sovereign build mesh, cross-home change exchange, and independent release assembly | Malicious runner, base mismatch, bundle quarantine, partition, residency, agreement, and reproducibility tests | Trust, placement, incoming-change review, disagreement, and release controls | Independent cross-home Windows, macOS, and Linux release matrix | Change exchange, bad-runner rejection, mesh exit, and disaster exercise |

A gate closes only from fresh build, test, security, UX, platform, migration, and operator proof for the exact candidate.

# 19. Current truth and gap

**Present in live source:** a large Rust backend with Git object/read code, smart HTTP routes, repository/file/tree/search/commit APIs, workspaces, pipelines, artifacts, agent jobs, issues, pull requests, reviews, branch protection, webhooks, ACLs, LSP bridge and routes, terminal policy, sandbox/container policy modules, DAP routes, sealed objects, MLS group code, deploy keys, quotas, settings, and break-glass records. Several policy and protocol modules have unit tests. The Next.js interface is a 1,656-line editor workspace with Monaco, split groups, tabs, command palette, quick open, search, source control, pipeline, agent, collaboration, terminal, debug, extension, and settings surfaces. Electron packaging output exists for Windows.

**Scaffold or unproven today:** the whole workspace does not compile and therefore has no full integration proof. The registered code agents still use the shared deterministic echo/catalog/time/tenant tools, not a reasoning or durable coding-agent loop. Source contains many high-risk execution and network surfaces, but the root integration and end-to-end suites are empty. The current UI is concentrated in one large page. A Windows unpacked Electron build exists, but macOS and Linux packages and a three-OS CI matrix do not. The root toolchain is Windows-host pinned, Docker is Linux-only, and most operator scripts favour PowerShell/CMD.

**Most important gap:** HelixCode has unusually broad forge code but does not yet prove that repository truth, sandbox boundaries, protected merges, agent authority, and release artifacts stay correct together.

**Safest first vertical slice:** import one normal Git repository, edit and recover a buffer, run one isolated three-OS pipeline, delegate one bounded agent patch, review exact head, merge under protection, build signed artifacts, delete/restore the repository, and export everything with independent proof.

# 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Source substrate | Standard Git objects, refs, and protocol remain canonical | Founder and architecture approval |
| Repository identity | Stable Helix project/repository ID, not path or remote URL | Architecture decision |
| Agent workspace | Separate isolated work cell and branch | Security review |
| Agent authority | No self-approval, merge, release, deploy, or raw secret access | Founder approval |
| Secret handling | Exact brokered capability for one workload; values never returned | Founder approval |
| Protected merge | Atomic exact-head check plus fresh required reviews and gates | No exception outside audited break-glass |
| Delete | 30-day recoverable bin with Git recovery; purge separate | Founder approval |
| Pipelines | Durable state machine, idempotency, heartbeat, cancellation | Architecture decision |
| Release truth | Exact source-to-artifact binding; skipped is not passed | No exception |
| Environments | Versioned contract behind native and OCI adapters | Architecture decision |
| Editor | Monaco-based current shell may evolve, but standard files and Git remain portable | Founder approval for replacement |
| Shipping UI | One code workspace and forge shell | Founder approval |
| Aether | Preferred proof/capability adapter with local fallback | Founder approval |
| Cross-platform | Native Windows, macOS, Linux plus browser gates | Founder approval |
| Extensions | Deny by default; signed package, declared permissions, easy disable | Security review |
| Founder-only choices | Hosted runner pricing and public extension marketplace | Founder decision; does not block G0 |

# 21. Definition of category-defining done

- [ ] Standard Git clients can clone, fetch, push, recover, and leave without HelixCode.
- [ ] Real developers complete all signature journeys on Windows, macOS, Linux, and web.
- [ ] Protected merges cannot race past exact-head reviews or fresh checks.
- [ ] Work cells pass filesystem, process, network, resource, and secret-boundary review.
- [ ] Agents are bounded, visible, stoppable, reversible, and never self-approving.
- [ ] Every long clone, index, build, test, agent, package, and deployment shows progress and notifies completion.
- [ ] Repository, branch, workspace, artifact, and environment deletion follows the 30-day recovery contract.
- [ ] Independent proof links requirement, code, review, checks, artifact, release, and known limits.
- [ ] Native packaging, upgrade, rollback, backup, restore, migration, and offline verification meet budgets.
- [ ] Critical editor and forge journeys meet WCAG 2.2 AA and pass assistive-technology review.
- [ ] Supply-chain exports use open formats and work on a clean verifier.
- [ ] Security review closes all critical and high findings or records a named, time-bound exception.
- [ ] Aether, any model vendor, and any runner provider can be removed without losing owned repositories.
- [ ] The product states that green tests and signatures do not prove defect-free or requirement-complete software.
