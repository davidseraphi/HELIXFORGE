# HelixAnvil — native, inspectable, agent-ready development environment

```yaml
product: HelixAnvil
catalog_order: standalone
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [software developers, systems engineers, researchers, agent operators]
deployment: [local, self-hosted]
platforms: [windows, macos, linux]
```

> [!NOTE]
> **CANONICAL HOME CHOSEN — IMPLEMENTATION STILL PORTFOLIO-LAST**
>
> The founder has chosen `C:\Users\divin\PROJECTS\HELIXFORGE\projects\helix-anvil`
> as the canonical HelixAnvil home. The intended external
> `C:\Users\divin\PROJECTS\HELIXANVIL` is abandoned.
>
> Sequencing remains **portfolio-last**: Anvil implementation waits until the
> HelixForge monorepo endgame (including HelixPulse) is complete, unless the
> founder explicitly changes sequencing. Until activation, agents MUST NOT create,
> move, merge, rename, delete, or implement Anvil code. Read-only planning and
> specification review are allowed.

## 1. Category claim

HelixAnvil is a fast native development environment where code, running systems,
agents, tests, decisions, and proof share one inspectable local model instead of
being hidden behind separate tools and chat boxes.

## 2. Five-year destination

The useful product is a beautiful native editor with projects, search, language
tools, debugging, terminals, tasks, source control, extensions, accessibility,
and reliable recovery. The category advantage is a causal work graph: every
change can show why it happened, what requested it, what ran, what changed, what
passed, what failed, and what remains uncertain. The frontier is a multi-agent
workshop where bounded agents can explore, propose, implement, test, and review
in isolated workspaces while the human sees progress and keeps final authority.

## 3. Users and hard jobs

- Developers need speed and flow and fear losing work or context.
- Systems engineers need deep runtime truth and fear an IDE that hides processes.
- Researchers need code, data, notebooks, and evidence together and fear irreproducible work.
- Agent operators need narrow authority and fear silent, broad machine changes.
- Extension authors need stable contracts and fear one vendor-controlled ecosystem.

## 4. Product laws

1. The editor kernel, layout, rendering, input, and desktop shell are native; no
   Electron or browser editor is the product core.
2. A user can edit, search, diff, build, and recover without an account or network.
3. No tool or agent changes a file without a visible transaction and undo path.
4. Slow work shows process, stage, output, last signal, and stop state.
5. Crashes must not lose acknowledged edits.
6. Extensions and agents receive explicit capabilities, not full machine trust.
7. Keyboard, screen reader, high contrast, reduced motion, and scaling are core features.
8. The project format is open and folder paths are not identity.
9. Advanced power is progressively revealed; basic editing stays calm.

## 5. Scope boundaries

Anvil owns the native editor kernel, desktop shell, workspaces, local tool
hosting, language and debug clients, process views, extension runtime, and
human-agent workbench. HelixCode owns hosted repositories, pull requests, shared
CI, issues, and remote collaboration. Anvil can use HelixCode but works fully
with local Git and other providers. It does not create its own cloud identity,
billing, vault, or general project-management system.

As a standalone product, Anvil embeds HelixCore-compatible logical contracts
for stable identity, policy, capabilities, audit, jobs, objects, recovery, and
proof. This is a compatibility boundary, not a runtime dependency on another
HelixForge product. Anvil owns its native operating-system adapters for files,
windows, input, accessibility, processes, terminals, notifications, clipboard,
and secure storage. HelixCore does not own or implement those adapters.

## 6. Signature experiences

| Journey | Entry | Visible progress | Human decision | Completion proof | Failure and recovery | Portability |
|---|---|---|---|---|---|---|
| Open and edit now | Open a folder or choose a stable project identity with no sign-in. | Launch card shows identity resolution, file scan, recovery check, service state, and first editable file. | User chooses trust level before tasks, extensions, or agents can run. | Session receipt links project identity, opened resource version, and ready time. | Missing tools do not block editing; file conflicts open a side-by-side recovery choice. | The same project identity can bind to a different path on another OS. |
| Never lose work | Continue after app, service, or machine failure with dirty buffers and running work. | Recovery view lists every journaled edit, selection, view, task, and last confirmed process state. | User accepts all, selects items, compares with disk, or discards only after preview. | Recovered buffer versions match the acknowledged edit journal and record every choice. | Corrupt entries are isolated; the last valid checkpoint opens read-only while repair stays reversible. | Recovery package validates and opens on all three systems. |
| Understand a codebase | Select a symbol, file, failure, test, or question. | Search, index, source, runtime, test, history, and freshness steps remain visible. | User chooses which source-backed path to follow and whether stale derived data may be used. | Answer or graph links every claim to source version, query, runtime signal, and known limit. | Failed services show stale/offline state; direct text and file navigation continue. | Open formats and adapter reports keep the model usable without one provider. |
| Run and debug | Start a named task, test, terminal command, or debug profile. | Exact command, environment names, owner, process tree, output, resources, ports, checks, and stop state stream live. | User approves sensitive capabilities, external targets, and any escalation after a failed stop. | Result links exit codes, process lifetimes, diagnostics, tests, debug actions, and artifacts. | Stop reaches all owned children or names the exact survivor; restart uses the last safe profile. | Profiles map tools and paths explicitly per OS; unavailable tools get a clear report. |
| Delegate safely | Select a goal, files, tools, time, compute, and approval points. | Agent card shows scope, plan, current action, files touched, command, elapsed time, last signal, checks, and blocked reason. | Human approves the lease, plan, selected risky steps, and final acceptance by section. | Proof links intent, lease, checkpoints, edits, commands, checks, reviews, limits, and approvals. | Expiry or stop revokes future tool use; isolated edits remain previewable and can be discarded together. | The intent, change set, and proof use provider-neutral formats. |
| Review proof | Open a completed or stopped human, tool, or agent change set. | Semantic diff, tests, screenshots, logs, risks, uncertainty, comments, and independent checks load by section. | Reviewer accepts, rejects, edits, or restores each section and confirms any release action separately. | Accepted transaction records exact sections, reviewers, checks, artifact hashes, and remaining limits. | Rejection changes no source; partial acceptance rebuilds a clean change set and preserves rejected evidence. | Exported proof verifies offline without Anvil or the original agent provider. |
| Move machines | Export a project home and open it on another supported OS. | Identity, settings, layouts, tasks, evidence, allowed extensions, tool mapping, and validation show progress. | User approves excluded private data, path mappings, tool substitutions, and final activation. | Clean host restores the same logical identity and matches the portable-home manifest. | Import stays in a disposable target until checks pass; rollback removes it and leaves source unchanged. | Windows, macOS, and Linux routes are tested in both directions. |

## 7. Capability map

| ID / gate | Input | Output | Invariant | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|
| HA-F0-01 / G0 | Unicode text, file version, selections, and ordered edit transactions | Versioned buffer, persistent undo, and crash journal | Positions are Unicode-correct; a transaction is whole; every acknowledged edit is recoverable | Human edits directly; a tool or agent needs a narrow buffer lease | Transaction IDs, before/after hashes, journal flush, origin, and recovery trace | Conflict opens a new comparison state; corrupt journal tail is isolated after the last valid entry | One million model-based edits and 10,000 forced crash points lose zero acknowledged edits on each OS |
| HA-F0-02 / G0 | Buffer version, layout, font metrics, scale, theme, input, IME, and accessibility settings | Native pixels, hit-test map, input result, and accessibility tree | Visual, input, and accessibility outputs refer to the same buffer version | User owns settings; platform adapter has only required window and input access | Frame ID, buffer version, input event, accessibility snapshot, timing, and golden fixture | Renderer or font failure enters native safe mode without closing the buffer | Golden render, keyboard, IME, screen-reader, 200% text, and reduced-motion fixtures pass on all three OSs |
| HA-F0-03 / G0 | Stable identity, candidate roots, file events, workspace state, and delete request | Bound workspace, conflict alert, recovery point, or 30-day bin item | Project identity never depends on a folder path; delete never becomes immediate purge | Owner binds roots, trusts tools, confirms delete, and approves restore | Identity record, root mapping, watcher sequence, conflict preview, tombstone, and restore event | Moved root can be rebound; watcher gaps trigger rescan; conflict never overwrites both versions | Move and rename 1,000 projects across roots, replay 100,000 file events, and restore every fixture from the bin |
| HA-F1-01 / G1 | Buffer versions, cursors, folds, snippets, structural edits, diffs, and merge inputs | One transactional edit result, conflict set, history entry, and undo point | Multi-part edits are atomic; large-file mode declares disabled features; merge never hides a conflict | Human or leased tool proposes; human controls conflict resolution | Edit graph, origin, affected ranges, mode, conflict choices, and undo result | Invalid range or stale version rejects the whole edit; failed merge leaves all inputs | One million randomized multi-cursor, undo/redo, diff, and merge cases match the reference model |
| HA-F1-02 / G1 | Query or language request, project scope, document versions, adapter capability, and cancellation | Search stream, symbol, diagnostic, format edit, rename, reference, or code action | Every result carries source version and freshness; an adapter edit still uses the text transaction boundary | Read tools need project scope; edits require explicit file capability and preview | Query, adapter/version, files read, result versions, cancellation, and applied transaction | Crashed or stale adapter becomes offline/stale; text search and direct editing continue | Million-file search yields first useful result under budget; 10,000 stale/cancel cases apply zero unreviewed edits |
| HA-F1-03 / G1 | Named task, exact command, environment names, working root, limits, and owner | Owned terminal/process tree, durable output, checks, result, and completion notice | Every child has one owner; stop covers descendants; secret values never enter logs or proof | Human starts normal tasks; tools and agents need command, path, network, time, and compute grants | Redacted launch manifest, process IDs, output positions, resources, ports, exit codes, and stop trace | Failed launch has no orphan; failed stop names the survivor and requires human escalation | 10,000 start/stop/crash cases leave zero silent child processes and 100% emit a durable completion state |
| HA-F2-01 / G2 | Debug profile, executable, source map, breakpoints, adapter capabilities, and process grant | Owned debug session with threads, stacks, variables, memory views, and actions | Debug actions apply only to the named session and target; displayed state shows freshness | User approves target attach and any write-to-process capability | Target identity, adapter version, commands, pauses, reads/writes, and session end | Adapter crash preserves editor and target ownership; stale state is labeled and cannot be acted on | 1,000 adapter restarts and target exits preserve ownership, labels, and editor state on all three OSs |
| HA-F2-02 / G2 | Repository identity, worktree, selected hunks, operation, signer, remote, and provider capability | Local change, commit, branch, review, signed artifact, or remote result | Unselected content is untouched; local editing works without a provider; push and signing are separate grants | Human selects changes and approves commit, sign, push, merge, or destructive history action separately | Before/after object IDs, selection, command, signer identity, remote response, and review | Conflict pauses with all versions; remote failure keeps local objects; rejected signing exports no key | 10,000 partial-stage, conflict, offline, signing, and retry cases preserve unselected content and exact authority |
| HA-F2-03 / G2 | Signed extension package, manifest capabilities, quotas, project trust, and version | Isolated extension process, contributed commands/views, and explicit status | Deny by default; extension failure cannot corrupt the editor kernel or widen its manifest | User approves install and per-project capabilities; publisher cannot self-approve | Package hash, signer, manifest, grants, uses, resources, crashes, update, and removal | Crash or quota breach disables only the extension and offers safe restart or removal | 100,000 denied capability cases and 1,000 crash loops show zero out-of-manifest access or buffer loss |
| HA-F3-01 / G3 | Versioned source, symbols, tests, runtime signals, history, decisions, and evidence | Navigable codebase graph with source-backed explanations and freshness | Derived links never replace source truth; every claim exposes version, source, and uncertainty | User selects indexed roots and may remove a source or derived capability | Ingest positions, source hashes, link type, freshness, query, cited nodes, and removals | Missing source removes or marks dependent links stale; unsupported answer becomes unknown | Delete and rebuild the graph, then match 10,000 golden links and explanations with no orphan claim |
| HA-F3-02 / G3 | Human goal, project identity, files, tools, time, compute, network, approvals, and stop policy | Isolated agent intent, lease, checkpoints, change set, checks, review, and rollback | Agent cannot widen its lease, edit outside isolation, approve itself, or hide work | Human grants scope and approves gated steps; reviewer is separate from builder | Intent, lease, tool calls, files, commands, progress signals, checks, review, expiry, and revocation | Expiry or stop blocks future tools; failed work remains isolated and discardable | 100,000 boundary cases and 10,000 stop/retry cases show zero scope escape, silent edit, or lost human work |
| HA-F3-03 / G3 | Request, plan, edits, commands, outputs, checks, screenshots, approvals, limits, and reviews | Verifiable proof bundle and section-level acceptance record | Observed evidence and agent statements stay distinct; omitted limits cannot be called complete | Evidence owner chooses disclosure; reviewer accepts sections; release authority acts separately | Source hashes, timestamps, tool identities, check results, reviewer signatures, and export manifest | Missing or stale evidence lowers state to partial/unknown and blocks a verified completion claim | 1,000 tamper, omission, stale, partial-accept, and offline-verification fixtures produce the correct proof state |
| HA-F4-01 / G4 | Multiple intents, workspaces, resources, edit ranges, dependencies, priorities, and leases | Collision-free schedule, explicit conflict, merge proposal, or paused work | Overlapping effects never silently overwrite; one resource cannot have incompatible active owners | Human sets priority and resolves semantic conflict; scheduler may pause within policy | Ownership graph, overlap detection, schedule choice, checkpoints, messages, and resolution | Uncertain overlap pauses affected work; unaffected work continues; reversal returns to saved checkpoints | 100 agents and 10,000 overlapping operations complete with zero silent overwrite in 1,000 deterministic campaigns |
| HA-F4-02 / G4 | Approved source, deployed version, traces, metrics, logs, processes, network metadata, and time window | Read-only live-system graph with causal hypotheses and freshness | Observation cannot change production; source version and deployed version remain distinct | Production owner grants exact read sources and time; no agent grants itself access | Grant, queries, source timestamps, redaction, join logic, result nodes, and revocation | Missing signals become unknown; revoked access stops collection; local editor stays usable | 10,000 stale, missing, revoked, and version-skew cases make zero production change and no false-fresh claim |
| HA-F4-03 / G4 | Stable project identity, open project-home manifest, settings, layouts, tasks, evidence, extension grants, and tool mappings | Governed portable development home or exact incompatibility report | Raw secrets and machine-bound tokens never export; identity and history survive path and OS change | Owner chooses contents and target grants; destination user activates tools and extensions | Manifest, hashes, exclusions, mappings, validations, imports, activation, and rollback | Import remains disposable until valid; unsupported items are disabled and explained | All 30 directed Windows/macOS/Linux routes restore identity and logical state with zero secret fixture leakage |

## 8. Domain model

Paths locate resources but never define project identity. Every edit belongs to
one transaction and one origin: human, tool, import, merge, or named agent.
Ephemeral records are labeled; durable records have explicit retention and delete
behaviour.

| Group and records | Owner | Lifecycle | Version rule | Retention and delete | Relationships |
|---|---|---|---|---|---|
| Identity and workspace: `ProjectIdentity`, root binding, `Workspace`, `Resource`, trust decision | Project owner controls identity, roots, and trust | Proposed, bound, trusted/read-only, moved, archived, removed | Stable identity is immutable; root bindings and trust are versioned events | Removing a root does not delete identity; archive enters the 30-day bin; permanent purge needs separate approval | Project identity owns workspaces; workspaces bind resources by logical ID and current path |
| Editing and recovery: `Buffer`, `BufferVersion`, `EditTransaction`, `SelectionSet`, `RecoveryPoint`, conflict | Buffer owner is the active user; leased tools act through a named origin | Open, dirty, journaled, saved/conflicted, recovered, closed | Immutable buffer versions; transactions link exact before and after versions | Journal remains through save and recovery window; discard is previewed; recovery items stay 30 days | Buffer binds one resource; transaction creates versions; recovery point covers buffers and views |
| View and layout: `View`, `Layout`, panel, focus, accessibility snapshot, theme | User profile owns layout and access settings | Created, active, hidden, saved, restored, retired | Layout and settings are immutable revisions with device-specific overrides | Retired layouts can be restored for 30 days; ephemeral frame data is not retained | Views reference buffers or tools; layout places views; accessibility snapshot binds one frame/version |
| Execution: `Task`, `Process`, process edge, `TerminalSession`, output segment, completion notice | Human or approved lease owns a launch; supervisor owns child lifetime tracking | Draft, approved, starting, running, stopping, completed/failed/orphan-alerted | Task definition and launch are separate versions; output is append-only by position | Redacted logs follow project policy; process metadata remains with proof; user may delete disposable output through the bin | Task launch creates process tree and terminal; notices link final state and checks |
| Language and debug: adapter, `Diagnostic`, `Symbol`, search result, `DebugSession`, breakpoint, stack snapshot | Project owner selects adapters; session owner controls target actions | Discovering, ready, stale, offline, restarting; debug proposed, attached, paused, ended | Every result binds document or deployed version and adapter version | Derived indexes and diagnostics are rebuildable; debug memory snapshots require explicit retention | Adapters read resources; diagnostics and symbols point to versions; debug session owns target and snapshots |
| Change and review: `ChangeSet`, diff section, source-control operation, `Review`, approval, release intent | Change author owns draft; independent reviewer owns acceptance; release authority owns release | Draft, isolated, checked, in review, partly accepted, accepted/rejected, applied, reverted | Change-set and review versions are immutable; section decisions create new acceptance versions | Rejected work remains recoverable for policy window; revert creates a new transaction rather than erasing history | Change set groups edit transactions; review binds evidence; accepted sections create a clean applied set |
| Extension and agent: `Extension`, `CapabilityManifest`, grant, `AgentIntent`, `AgentLease`, `Checkpoint` | User owns grants; extension/agent owns no authority beyond active lease | Requested, denied/granted, active, paused, expired/revoked, removed | Manifests, grants, intents, and checkpoints are immutable signed versions | Removal revokes bindings without deleting shared providers; values are never retained; metadata follows audit policy | Extension or agent binds project, tools, resources, quota, purpose, time, and checkpoints |
| Evidence and portability: `EvidenceItem`, `Decision`, `ProofBundle`, project-home manifest, import run | Evidence owner controls disclosure; destination owner controls activation | Captured, checked, signed, verified/partial, exported, imported, superseded | Content-addressed evidence and append-only decisions; bundle manifest binds exact versions | Proof follows declared policy and legal hold; raw secrets are excluded; failed import target is disposable | Proof links identity, edits, work, checks, approvals, and limits; home manifest references portable logical records |

## 9. System architecture

- A Rust editor kernel owns text, positions, transactions, undo, journaling, and file sync.
- A native platform layer provides windows, input, accessibility, clipboard,
  menus, drag/drop, notifications, rendering, and secure storage adapters.
- An embedded HelixCore-compatible contract layer supplies the logical shapes
  for identity, policy, capabilities, audit, jobs, objects, recovery, and proof.
  It is implemented locally and does not require a HelixCore service to run.
- Anvil-owned native adapters translate those logical contracts to each OS for
  files, windows, input, accessibility, processes, terminals, notifications,
  clipboard, and secure storage. These adapters never move into HelixCore.
- Services for search, language, debug, Git, tasks, and extensions run out of
  process and can restart without losing editor state.
- A process supervisor owns every child and records its real lifetime.
- A local event graph joins code, runtime, agent, review, and evidence state.
- Provider contracts connect HelixCode, Aether, Git hosts, models, and remote
  compute without making any one provider mandatory.

## 10. Agent and automation contract

Each agent card shows goal, scope, current action, files touched, command,
elapsed time, last fresh signal, checks, blocked reason, and safe stop.

| Agent | Reads | Tools | Drafts | Approval | Forbidden | Checking | Stop or reversal |
|---|---|---|---|---|---|---|---|
| Explorer | Approved roots, versioned indexes, symbols, tests, history, and existing evidence | Read-only search, navigation, index query, and evidence citation | Codebase map, findings, questions, and source-backed explanation | Human grants roots and source types; no approval lets it edit | Read outside roots, retrieve raw secrets, run commands, edit files, or present stale derived data as fresh | Resolve paths safely, cite versions, label freshness, and separate observed from inferred | Revoke lease; drafts remain notes and make no project change |
| Planner | Explorer evidence, requirement, constraints, capability policy, prior decisions, and current changes | Read-only dependency graph, task splitter, risk model, and plan validator | Ordered plan, file/tool scope, checkpoints, approvals, tests, rollback, and unknowns | Human accepts or narrows the plan before builder work | Edit, run tools, widen scope, approve its plan, or hide an unresolved choice | Confirm every step has authority, evidence, acceptance, and reversal | Reject or replace the plan; no implementation state exists to undo |
| Builder | Approved plan, named files, relevant source, and prior accepted checkpoints | Transactional editor and only the named build tools inside an isolated change set | Edits, generated artifacts, progress checkpoints, and implementation notes | Lease grants exact paths/tools; human approves gated actions and final handoff | Edit outside isolation, modify production state, start unowned services, read secret values, merge, or self-approve | Re-read changed files, run scoped checks, show diff and generated-artifact provenance | Stop revokes tools and keeps the isolated set; discard removes all builder effects without touching accepted work |
| Tester | Accepted candidate change, test contract, fixtures, limits, and disposable targets | Named test runners, native UI harness, process supervisor, and temporary state | Results, failures, screenshots, logs, performance trace, and reproduction steps | Human or plan approves commands, compute, network, devices, and duration | Change product source, use production state, hide failed checks, or turn a skipped test into a pass | Verify command, environment names, collection count, exit code, artifacts, and fresh timestamp | Cancel kills owned process tree; partial results remain labeled incomplete and disposable state is removed |
| Reviewer | Request, plan, diff, checks, proof, limits, and applicable contracts | Read-only semantic diff, proof verifier, fixture viewer, and comment writer | Section findings, risk rank, missing checks, accept/reject proposal, and uncertainty | Independent human accepts or dismisses findings with reason; reviewer cannot apply changes | Edit the candidate, approve its own work, suppress evidence, or call partial proof complete | Re-run fresh named checks, trace claims to source, and verify unresolved findings remain visible | Stop leaves candidate unchanged; rejection returns selected sections to the builder or human |
| Release Operator | Accepted change set, independent review, artifact hashes, target, signer identity, migration, and rollback plan | Reproducible package, signer broker, target adapter, migration verifier, and release monitor | Release intent, signed artifacts, deployment preview, completion record, and rollback evidence | Human grants target, signing operation, publish, and migration cutover separately | Export private keys, change source, select a new target, skip gates, or publish from an unreviewed set | Rebuild from clean source, verify all gates and hashes, dry-run migration, and confirm target identity | Cancel before publish has no external effect; failed release keeps prior version active and follows the approved rollback |

Agents never read raw secret values, bypass path boundaries, start an unowned
server, widen their lease, hide output, or treat silence as progress.

## 11. Trust, safety, and privacy

Workspace trust is per project identity and capability, not one global yes/no.
New folders start read-only until the user approves tools. Path checks resolve
links and platform-specific paths before access. Terminals, tasks, debuggers,
extensions, and agents use separate capability manifests. Secrets are brokered
to one approved process and never enter chat, logs, evidence, or project memory.
Delete uses a 30-day bin; Git-aware deletion also previews tracked state. Signed
releases, production access, and permanent deletion require re-authentication.

## 12. Proof and audit

Proof links the user request, project identity, accepted plan, capability lease,
edit transactions, commands, processes, tests, diagnostics, visual checks,
reviews, approvals, release artifacts, and known limits. It distinguishes
observed runtime evidence from an agent statement. Aether is the preferred proof
and capability provider; Anvil includes a local signed evidence viewer and exporter.

## 13. UX system

The shell has Workbench, Search, Source, Run, Debug, Test, Agents, Evidence,
Recovery, and System views. The center stays focused on the current artifact.
Rare controls appear when context requires them. Every move or bulk edit shows
the selected count and destination. Undo remains close to the action. Slow work
has persistent activity cards and desktop completion notices. Errors say what
happened, what was preserved, and the safest next action. The UI supports full
keyboard use, screen readers, 200% text scaling, reduced motion, and color-independent states.

## 14. Interoperability and standards

- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
  is the language-tool adapter, verified from its official site on 2026-07-15.
- [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/)
  is the debugger adapter, verified from its official site on 2026-07-15.
- Git remains a source-control adapter; Anvil's edit and evidence models do not
  depend on Git being present.

Protocol extensions are namespaced, capability-negotiated, and recorded. Unknown
messages cannot crash the editor kernel.

## 15. Cross-platform contract

Windows, macOS, and Linux are equal release targets. Each merge runs native edit,
render, input, accessibility, file-watch, terminal, process, language, debug,
installer, update, crash, and recovery tests. Platform behaviour sits behind a
documented adapter with capability detection. There is no WSL requirement. A
project home exported on one system opens on the other two with a clear report
for unavailable tools or path mappings.

## 16. Reliability and performance budgets

Targets use published reference hardware, datasets, and native runners. Each
release records raw results, build identity, operating system, and any approved
exception.

| Contract | Numeric target and window | Required proof | Degraded behaviour |
|---|---|---|---|
| Edit durability | Zero acknowledged edits lost across 1,000,000 model edits and 10,000 forced crash points per release on each OS | Edit receipts, journal positions, crash schedule, recovered buffer hashes, and selections | Open the last valid version read-only and isolate a corrupt journal tail |
| Transaction atomicity | Zero partly applied multi-buffer transactions across 100,000 partial-write, stale-version, and restart cases per release | Before/after versions, transaction ID, flush result, and recovery trace | Reject the whole transaction or expose an explicit conflict; never guess |
| Idempotency | Repeating a tool, task, or agent action with the same key for 24 hours returns the original logical result in all 1,000,000 duplicate cases | Action key, command or edit hash, retained result, and expiry rule | After expiry, require a new preview and approval rather than repeat silently |
| Concurrent work | At G4, 100 agents and 10,000 overlapping operations complete 1,000 campaigns with zero silent overwrite or unowned effect | Ownership graph, collision decisions, checkpoints, diffs, and final hashes | Pause uncertain overlaps while unrelated buffers and agents continue |
| Offline work | Edit, search, local source control, tasks, terminals, local debug, recovery, and proof export work for 72 hours with network disabled | Seven native journeys, no-network trace, and offline artifact verification | Remote providers show unavailable; local work never waits on them |
| Input and rendering | Input-to-render p95 below 16 ms for normal files and below 50 ms in declared large-file mode over a 30-minute run | Raw frame/input trace, dropped-frame count, file profile, display scale, and host | Reduce decoration and derived views before text input or accessibility |
| Launch | Cold launch to first editable local file p95 below 500 ms; warm p95 below 200 ms across 100 launches per OS | Native launch trace from process start to editable buffer and accessibility-ready state | Open the buffer first and mark slow services as starting, stale, or offline |
| Search scale | First useful result p95 below 150 ms and complete indexed query below 10 seconds for a one-million-file corpus | Corpus manifest, query set, index state, result stream, and timing histogram | Stream partial results, permit cancel, and label excluded or stale sources |
| Process ownership | At least 99.99% of 10,000 normal stop cases end the full owned local tree within 2 seconds; every survivor is named within 3 seconds | Parent-child graph, stop signals, exit state, ports, resources, and escalation choice | Keep editor responsive, block false completion, and ask the human before escalation |
| Service isolation | Across 1,000 crash loops per service, zero buffer closes or acknowledged edit loss; service status changes within 1 second | Service lifetime, restart count, buffer checks, capability grants, and UI state | Search, language, debug, Git, task, or extension feature becomes stale/offline alone |
| Recovery | All 30 directed Windows/macOS/Linux recovery-package routes validate before activation each release | Signed manifest, hashes, schema version, import checks, recovery choices, and activation | Keep source and current workspace active; failed import remains disposable |
| Cancel and reversal | 100% of 10,000 agent, search, task, test, migration, and import cancellations stop at a safe boundary or name the exact bounded delay within 1 second | Cancel receipt, process state, checkpoint, resource release, and resulting diff | User keeps control; incomplete output stays labeled and no partial change is accepted |
| Resource pressure | At 80% of configured memory, CPU queue, disk, or output budget, background work throttles; normal text input still meets its p95 target in 100 pressure runs | Resource trace, scheduler decisions, input latency, evictions, and recovery | Pause indexes, previews, agents, and history enrichment before editing or recovery |
| Update and rollback | Every release passes 100 update, failed-update, and rollback cycles per OS with project identity and acknowledged edits unchanged | Old/new package hashes, migration preview, recovery package, health checks, and rollback result | Failed update keeps or restores the prior signed version and explains preserved state |
| Packaging | Installer, update, uninstall, file association, accessibility, notifications, terminals, and recovery pass on clean current and prior supported OS images | Signed native package report, clean-machine video/trace, and residue check | Unsupported host is blocked before mutation and receives a portable-data path |

## 17. Success measures

| Outcome | Threshold and time window | Measurement |
|---|---|---|
| Daily-use value | By the end of each 90-day G1 pilot, at least 80% of enrolled developers use Anvil on 4 of 5 working days without needing another editor for basic local work | Consent-based active-day and task-completion study, not keystroke surveillance |
| Edit trust | Zero lost acknowledged edits in every release suite and quarterly 1,000,000-edit fault campaign | Journal receipt-to-recovery reconciliation |
| Responsiveness | At least 99% of weekly reference runs meet launch and input p95 budgets; any two consecutive misses block release | Native performance history with confidence range |
| Process truth | At least 99.99% of 10,000 stop cases end within 2 seconds and 100% of survivors are named within 3 seconds per release | Process-tree oracle and port/resource check |
| Failure understanding | At least 90% of developers identify failed stage, preserved work, and next safe action within 5 minutes in a quarterly 20-person study | Timed failed-build and crashed-service journey study |
| Accessible completion | All seven signature journeys pass keyboard, screen-reader, 200% text, high-contrast, reduced-motion, and color-independent checks each release | Native assistive-technology journey report on all three OSs |
| Agent control | In every quarterly 10,000-case campaign, zero agent scope escapes or silent edits; at least 95% of users can stop and fully discard a change set within 30 seconds | Boundary harness plus a 20-person control study |
| Review usefulness | During each 90-day G3 pilot, at least 80% of accepted agent changes are accepted by section and fewer than 2% need rollback for a defect the proof claimed was checked | Section decisions, rollback cause, and verified-check classification |
| Extension isolation | Zero editor-kernel failures or edit loss across 1,000 extension crash loops per release | Native isolation trace and buffer digest |
| Portable home | All 30 directed cross-OS routes restore identity and logical state each release; at least 95% of pilot moves finish without manual file repair | Clean-machine verifier and 90-day pilot support record |
| Recovery bin | 100% of 1,000 eligible delete fixtures restore within the 30-day window; 100% of permanent purges require separate authority | Tombstone, restore, expiry, approval, and purge events |
| Support burden | By the end of a 90-day G1 pilot, fewer than 1 support incident per active developer-month concerns hidden progress, lost work, or an unowned process | Support classification linked to journey and product state |

Extension count, agent message count, and time spent inside the product are not
success measures.

## 18. Delivery plan

**No gate may begin implementation until the founder records the canonical
Anvil project location.** The rows below are future release contracts, not proof
that the current scaffold has these capabilities.

| Gate | Build | Tests | Safety | UX | Windows / macOS / Linux | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| G0 — truthful foundation | Native text kernel, render/input loop, stable identity, journal, workspace binding, and embedded HelixCore-compatible contracts | One million edit cases; 10,000 crash points; deterministic render, IME, file-watch, and recovery fixtures | Read-only first trust; transactional edits; no raw secret access; location decision recorded before code | Open/edit and never-lose-work journeys work with persistent progress and recovery choices | Native build, unit, render, input, accessibility, filesystem, crash, package smoke, and recovery CI pass on all three | Synthetic fixtures only; identity survives root move; rollback removes the disposable scaffold | Founder observes edit, forced crash, exact recovery, path move, delete, and restore with signed local proof |
| G1 — first-choice local editor | Editing, search, language tools, tasks, terminals, tests, local Git, settings, accessibility, installers, updates, and recovery | Million-file search; 10,000 process stops; 100 update/rollback cycles; all seven local journeys | Child ownership, brokered secrets, 30-day bin, path-safe access, and no account/network requirement | Daily-use pilot meets launch/input budgets and all seven accessibility journeys | Signed installer, update, uninstall, association, notification, terminal, and 72-hour offline suites pass natively | Open project-home import; reversible settings/workspace migration from prior supported release | Ten developers complete a full week of edit, search, run, debug-basic, source, recovery, and move tasks without developer help |
| G2 — trusted extensible environment | Debugging, merge/review, out-of-process extensions, provider-neutral remote adapters, and strong process isolation | 1,000 debug/extension crash loops; 100,000 denied capability cases; 10,000 source-control conflict/offline cases | Deny-by-default extension manifests, target-specific debug grants, split sign/push/merge authority | Crashed services explain preserved state and recovery; extension power and resource use stay inspectable | Debug, Git, extension, process, packaging, accessibility, and recovery matrices pass on all three | Extension and provider import reports permissions and unsupported features before activation | Independent operator installs, limits, crashes, removes, and restores extensions and adapters without edit loss |
| G3 — verifiable agent workbench | Source/runtime codebase model, bounded agent workshop, unified proof, and complete HelixCode/Aether-compatible adapters | 10,000 graph goldens; 100,000 agent boundaries; 10,000 stop/retry cases; 1,000 proof tamper/omission cases | Agents cannot self-grant, self-review, expose values, use production state, or hide work; release authority is separate | Human can see progress, stop, compare, accept by section, undo all, and verify proof offline | Full agent, proof, accessibility, package, resource-pressure, and recovery journeys pass on all three | Existing workspaces migrate in previewed, reversible batches; provider removal leaves open local formats | Twenty pilot users complete delegated change, independent review, partial accept, discard, and rollback with signed proof |
| G4 — collision-safe development home | Multi-agent scheduler, read-only live-system model, and governed portable home | 100-agent collision campaigns; 10,000 stale/revoked live-signal cases; all 30 cross-OS routes; long-run resource tests | No silent overwrite, no production mutation from observation, no global agent authority, and no secret export | Complex work stays calm through progressive reveal, clear ownership, persistent signals, and one-step safe stop | Equal release gates for scheduler, live views, portable home, accessibility, installers, updates, and rollback | Export/import preserves identity, history, proof, settings, and grants while excluded secrets stay excluded | Independent accessibility, reliability, privacy, performance, and usability reviewers approve a founder-observed multi-machine drill |

## 19. Current truth and gap

The intended external `C:\Users\divin\PROJECTS\HELIXANVIL` does not exist. A
nested scaffold exists at
`C:\Users\divin\PROJECTS\HELIXFORGE\projects\helix-anvil`, but the founder has
not authorized it as the canonical implementation location. It contains scaffold
and planning material, not proof of a product editor. There is no proven editor
buffer, native shell, renderer, workspace loader, language client, or product UI.

Implementation is blocked. Kimi and every implementation agent MUST NOT create,
move, merge, rename, or delete either location. After the founder records the
canonical location and what happens to the other scaffold, the first allowed
slice is HA-F0-01: a headless native text kernel with property tests, crash
journal, Unicode fixtures, and no UI claims. HA-F0-02 renders the same kernel on
all three operating systems only after HA-F0-01 passes G0 proof.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Project location | **BLOCKED:** intended external `C:\Users\divin\PROJECTS\HELIXANVIL` is absent; nested `C:\Users\divin\PROJECTS\HELIXFORGE\projects\helix-anvil` exists; neither is authorized for create, move, merge, rename, delete, or implementation | Founder records one canonical location and explicit retain, migrate, or retire instructions for the other |
| UI core | Native Rust desktop; no Electron or Monaco | Founder decision |
| Text model | Persistent chunked text structure plus transactional edit journal | Kernel benchmark and design decision |
| Logical contracts | Embedded HelixCore-compatible identity, policy, capability, audit, job, object, recovery, and proof contracts; no HelixCore runtime dependency | Cross-product architecture decision |
| Native adapters | Anvil owns OS file, window, input, accessibility, process, terminal, notification, clipboard, and secure-storage adapters | Founder and platform architecture decision |
| Services | Out of process; editor survives their failure | Architecture decision |
| First milestone | Headless kernel before visual shell | Founder decision |
| Extension power | Denied by default; manifest capabilities and quotas | Security review |
| Agent edits | Isolated change set with full preview and rollback | Trust review |
| Delete | 30-day bin with Git and dependency preview | Data-policy decision |

## 21. Definition of category-defining done

- [ ] Founder location decision is recorded before any implementation begins.
- [ ] Anvil is a first-choice daily editor before any agent feature is needed.
- [ ] It starts fast, stays responsive, survives crashes, and truly stops its processes.
- [ ] A human can see and control every agent's scope, progress, changes, and proof.
- [ ] Extensions cannot take silent machine-wide authority.
- [ ] Projects move across Windows, macOS, and Linux without losing identity or history.
- [ ] Accessibility, recovery, security, performance, usability, packaging, and
      independent review gates pass on all three operating systems.
