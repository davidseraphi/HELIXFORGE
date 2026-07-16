---
product: HelixCollab
catalog_order: 01
status: target-state-spec
horizon: 60 months
current_maturity: alpha
primary_users: [individual creator, project team, editor, reviewer, knowledge steward, external guest]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
---

# 1. Category claim

HelixCollab is a sovereign shared thinking space where people and approved agents can create living knowledge together, remain productive offline, prove important decisions, and leave with a complete usable copy.

# 2. Five-year destination

- **Useful product:** Documents, structured pages, files, comments, tasks, decisions, meetings, and team knowledge live in one calm workspace with strong search and offline editing.
- **Category-defining advantage:** Collaboration stays understandable and portable even when work spans people, agents, organisations, devices, and long periods of time.
- **Frontier capability:** A privacy-preserving knowledge network can share selected facts and proofs across sovereign homes without central ownership.
- **Human authority:** People choose membership, publication, accepted changes, final decisions, retention, agent permissions, and permanent deletion.

# 3. Users and hard jobs

| User | Hard jobs | Failure feared most |
|---|---|---|
| Individual creator | Capture, organise, edit, recover, and export knowledge | Work disappears or becomes trapped in a service |
| Project team | Co-edit, review, decide, hand off, and find the latest truth | Conflicts or hidden changes destroy trust |
| Editor or reviewer | Suggest precise changes and preserve authorship | An accepted change cannot be traced or reversed |
| Knowledge steward | Set structure, retention, access, and quality rules | Private or outdated material spreads silently |
| External guest | Work in a narrow shared space without joining the whole tenant | Guest access reaches unrelated work |
| Accountable leader | Approve decisions and see unresolved risk | An agent summary is mistaken for a human decision |

# 4. Product laws

1. The user owns the workspace and can export documents, structure, attachments, history, and proof.
2. Offline work is normal. Sync never hides a conflict or silently discards a valid edit.
3. Every person, device, agent, document, and decision has stable identity independent of a folder.
4. Encryption state, sharing state, residency, and last successful sync are always visible.
5. Agents suggest or draft. They do not approve their own changes, invite people, publish, or expand access.
6. Important decisions preserve options considered, authority, evidence, and stated uncertainty.
7. Delete enters a 30-day recoverable bin by default.
8. Slow import, export, encryption, sync, indexing, and agent work shows live phases and completion.
9. Private content does not enter prompts, logs, indexes, or federation without an exact approved purpose.
10. Accessibility and plain language are part of correctness.
11. A collaborator leaving a group loses future access without destroying shared work.
12. The system states when encryption, proof, identity, or delivery assurance is reduced.

# 5. Scope boundaries

**HelixCollab owns:** workspaces, spaces, folders, documents, structured blocks, revisions, comments, mentions, presence, decisions, attachments, sharing, offline replicas, sync conflicts, workspace search, knowledge views, collaboration agents, recovery, and portable collaboration bundles.

**HelixCore owns:** identity, policy decisions, tenant isolation, capabilities, jobs, object custody, billing, common proof, operations, and the shared shell. HelixCode owns source repositories and code review. HelixFlow owns reusable automation.

**HelixCollab will not attempt:** to replace licensed legal, medical, financial, or records professionals; to claim that group agreement makes a statement true; to train on tenant content by default; or to make federation required for ordinary collaboration.

# 6. Signature experiences

| Journey | Entry point | Visible progress | Human decision | Completion proof | Failure and recovery | Export or portability |
|---|---|---|---|---|---|---|
| Create a workspace and first page | New workspace | Structure, local cache, access, and encryption setup | Choose owner, privacy, residency, and template | Workspace manifest and created-page event | Resume setup from checkpoint; incomplete workspace remains private | Export workspace descriptor and page |
| Co-edit a proposal | Open document | Presence, typing, connection, sync lag, saved state, and revision markers | Accept or reject suggestions and resolve comments | Revision with author/device attribution and accepted-decision record | Conflict view keeps both valid branches; restore any prior revision | Export current document plus chosen history |
| Work offline and reconnect | Offline banner | Local save count, pending changes, reconnect, merge phases | Choose resolution for meaning-level conflicts | Merge receipt naming inputs and outcome | Never discard a branch; retry sync idempotently | Local portable draft remains usable without server |
| Invite a narrow external guest | Share panel | Scope preview, expiry, device verification, delivery state | Owner approves document/space, role, and duration | Signed invitation and accepted membership | Revoke immediately; expired link cannot reopen | Guest can export only content explicitly allowed |
| Review and approve a decision | Decision block | Draft, evidence, reviewers, open objections, approval state | Named accountable human accepts, rejects, or asks for more proof | Decision record bound to exact revision and evidence | Reopen creates a new decision version; old decision remains visible | Export decision packet in open form |
| Import a large knowledge set | Import center | Scan, mapping, duplicates, unsupported content, item progress | Confirm mappings and losses before commit | Import report with source hashes and loss list | Cancel leaves source untouched; partial import rolls back or stays as reviewable draft | Keep original files beside converted forms |
| Delete and restore a space | Space menu or Recovery | Dependency preview, bin state, days remaining | Confirm bin move; elevated approval for purge | Tombstone, recovery inventory, restore receipt | Restore names conflicts before applying | Export before purge |
| Ask an agent to improve a document | Agent panel | Scope, sources, plan, current step, changes, checks | Approve capability use and each publish/accept action | Suggestion set with sources, diff, checks, and limitations | Stop immediately; discard or revert agent branch | Export suggestions separately from accepted document |

# 7. Capability map

## F0 — Foundation

### COL-F0-001 — Versioned document graph

- **Gate:** G0. **Inputs:** workspace, document structure, content operations, actors. **Outputs:** stable documents, blocks, revisions, and links.
- **Invariants:** no overwrite without a revision; tenant and workspace ownership are explicit. **Authority:** editors change; viewers read.
- **Evidence:** canonical revision and actor/device receipt. **Failure:** stale update returns a merge case, not silent last-write-wins.
- **Acceptance:** two edits from the same base preserve both changes or create a visible conflict.

### COL-F0-002 — Local replica and deterministic sync

- **Gate:** G0. **Inputs:** local operation log and remote operation log. **Outputs:** merged state, acknowledgements, pending queue.
- **Invariants:** replay is idempotent; offline data remains readable. **Authority:** device acts only for its authenticated member.
- **Evidence:** sync range, device, operation hashes, conflict outcome. **Failure:** damaged queue is quarantined and recoverable.
- **Acceptance:** disconnecting during every sync phase causes no accepted edit loss.

### COL-F0-003 — Workspace access and device identity

- **Gate:** G0. **Inputs:** HelixCore subject, device key, membership, resource policy. **Outputs:** exact read, comment, edit, share, or owner decision.
- **Invariants:** no caller-selected tenant or role; guest scope cannot expand itself. **Authority:** owners and delegated stewards manage membership.
- **Evidence:** share, accept, deny, revoke, and device events. **Failure:** policy or identity outage blocks protected writes.
- **Acceptance:** a guest invited to one document cannot list its parent workspace.

### COL-F0-004 — Recoverable content lifecycle

- **Gate:** G0. **Inputs:** delete, restore, expiry, legal hold. **Outputs:** tombstone, 30-day bin entry, restored record, or audited purge.
- **Invariants:** links and identity survive restore; permanent deletion is separate. **Authority:** editor may bin own content; owner purges.
- **Evidence:** impact preview and lifecycle events. **Failure:** attachment or child failure blocks final purge.
- **Acceptance:** a deleted document restores on day 29 with comments, links, and revision history.

## F1 — Useful product

### COL-F1-001 — Rich, structured authoring

- **Gate:** G1. **Inputs:** text, blocks, tables, media, references, templates. **Outputs:** accessible document and revision.
- **Invariants:** semantic structure survives visual editing; no format mode silently loses content. **Authority:** editor controls content.
- **Evidence:** revision, conversion warnings, and accessibility results. **Failure:** unsupported block remains preserved and labelled.
- **Acceptance:** keyboard-only users create, reorder, link, and export all required block types.

### COL-F1-002 — Review, comment, and decision workflow

- **Gate:** G1. **Inputs:** anchored comment, suggestion, evidence, reviewer, due state. **Outputs:** resolved thread or versioned decision.
- **Invariants:** resolution does not erase discussion; approval binds an exact revision. **Authority:** designated approver decides.
- **Evidence:** thread history and decision receipt. **Failure:** moved text keeps a recoverable anchor or shows orphan state.
- **Acceptance:** independent export reconstructs who proposed, reviewed, approved, and changed what.

### COL-F1-003 — Presence and visible slow work

- **Gate:** G1. **Inputs:** connection, cursor, typing, sync, import, export, index, agent-job state. **Outputs:** live presence and durable progress.
- **Invariants:** unknown duration never uses fake percentage; stale presence expires. **Authority:** users choose presence visibility.
- **Evidence:** job transitions, not every keystroke. **Failure:** disconnection becomes offline state within the budget.
- **Acceptance:** a 30-minute export never appears frozen and sends a completion notification.

### COL-F1-004 — Portable workspace bundle

- **Gate:** G1. **Inputs:** selected spaces, history choice, attachments, access metadata. **Outputs:** open manifest plus files and proof.
- **Invariants:** export is useful without HelixCollab; secret keys are not included by default. **Authority:** owner controls scope.
- **Evidence:** source-to-export inventory and losses. **Failure:** export with missing items cannot report complete.
- **Acceptance:** a clean machine renders exported documents and verifies inventory without network access.

## F2 — Category leader

### COL-F2-001 — Private group encryption

- **Gate:** G2. **Inputs:** verified devices, group membership, key epochs, encrypted operations. **Outputs:** group-readable content and rotation.
- **Invariants:** server cannot read protected content; removal blocks future epochs. **Authority:** group owners change membership.
- **Evidence:** metadata-only epoch and device events. **Failure:** lost-key recovery needs approved recovery policy and never fabricates access.
- **Acceptance:** a revoked device cannot decrypt a document update created after revocation.

### COL-F2-002 — Meaning-aware knowledge map

- **Gate:** G2. **Inputs:** user-approved documents, links, claims, decisions, terms, provenance. **Outputs:** navigable map and explainable search.
- **Invariants:** source and confidence travel with every derived relation; private scope is preserved. **Authority:** steward curates canonical terms.
- **Evidence:** source references and derivation version. **Failure:** uncertain matches are suggestions, not facts.
- **Acceptance:** every map edge opens the exact source passage or states that it is manually asserted.

### COL-F2-003 — Cross-organisation collaboration space

- **Gate:** G2. **Inputs:** two sovereign homes, explicit shared-space contract, member mappings. **Outputs:** narrow shared replica and joint proof.
- **Invariants:** neither side gains parent access; removal stops future sync. **Authority:** accountable owner on each side approves.
- **Evidence:** bilateral contract, scope, sync receipts, revocations. **Failure:** partition leaves both sides readable and marks divergence.
- **Acceptance:** either organisation can leave and export its allowed record without deleting the other side’s history.

### COL-F2-004 — Knowledge quality and retention

- **Gate:** G2. **Inputs:** ownership, review date, evidence age, policy, usage. **Outputs:** stale, conflicted, superseded, or trusted status.
- **Invariants:** quality labels are explainable; low confidence never hides content. **Authority:** steward sets policy; owner confirms disposal.
- **Evidence:** rule, source facts, and human override. **Failure:** unavailable check becomes unknown.
- **Acceptance:** no expired canonical page stays silently marked current.

## F3 — Advanced category leadership

### COL-F3-001 — Proof-carrying publication

- **Gate:** G3. **Inputs:** exact revision, evidence, approvals, limitations, publication target. **Outputs:** signed publication bundle.
- **Invariants:** published claim and source revision are inseparable. **Authority:** named publisher approves.
- **Evidence:** independent verifier report. **Failure:** changed content invalidates the publication receipt.
- **Acceptance:** a recipient verifies integrity and sees claims that remain unproven.

### COL-F3-002 — Human-agent collaboration studio

- **Gate:** G3. **Inputs:** bounded workspace context, task, capability lease, review policy. **Outputs:** agent branches, suggestions, checks.
- **Invariants:** agents cannot self-approve or expand scope. **Authority:** human accepts every consequential merge.
- **Evidence:** plan, sources, tool uses, diff, checks, and approval. **Failure:** stop revokes the lease and preserves partial draft separately.
- **Acceptance:** an agent completes a research-to-proposal journey without hidden sources or direct publication.

### COL-F3-003 — Sovereign knowledge federation

- **Gate:** G4. **Inputs:** selected public or partner objects, federation policy, trust roots. **Outputs:** distributed discovery and updates.
- **Invariants:** private-by-default; deletion and correction propagate with proof; central service optional. **Authority:** publishers choose audience.
- **Evidence:** signed origin, delivery, correction, and revocation. **Failure:** untrusted remote data remains isolated.
- **Acceptance:** three independent homes exchange a corrected publication while each preserves its own trust decision.

## F4 — Frontier network

### COL-F4-001 — Private federated discovery

- **Gate:** G4. **Inputs:** user query, approved local indexes, disclosure policy, trust bindings, and proof request. **Outputs:** source-linked summaries, allowed result pointers, denials, and freshness state without a central content copy.
- **Invariants:** private text, hidden membership, and secret search terms never leave their allowed home; every result keeps origin and confidence. **Authority:** each home owner chooses searchable collections, audiences, fields, and expiry.
- **Evidence:** query purpose, policy version, responding homes, disclosed fields, source hashes, denials, and result-verification report. **Failure:** an offline or untrusted home returns unavailable or denied and is never replaced with guessed content.
- **Acceptance:** three homes answer one shared query, one refuses under local policy, and the combined view exposes only approved fields while every result opens its exact source or proof pointer.

### COL-F4-002 — Cross-home decision assembly

- **Gate:** G4. **Inputs:** shared-space contract, exact proposal revision, participating homes, local reviewers, evidence, and decision rule. **Outputs:** signed joint decision bundle with approvals, objections, unresolved points, and local adoption state.
- **Invariants:** no home rewrites another home’s record or treats silence as approval; disagreement remains visible. **Authority:** each accountable owner signs for its own home and controls local adoption or withdrawal.
- **Evidence:** proposal hash, local review trails, signatures, objections, final rule result, adoption receipts, corrections, and exits. **Failure:** revision drift, missing required approval, or partition leaves the decision pending and local histories readable.
- **Acceptance:** three organisations ratify one proposal, preserve one minority objection, and later let one organisation leave with its allowed record while the remaining decision and proof still verify.

# 8. Domain model

| Record | Owner and relationships | Lifecycle and version rules | Retention |
|---|---|---|---|
| Workspace | Tenant-owned; contains spaces, folders, documents, policies | Stable ID; settings are versioned | 30-day bin after closure |
| Space | Workspace boundary for members, encryption, and residency | Membership and policy versions are immutable once effective | 30-day bin |
| Folder | Workspace structure; parent and child links | Stable ID; moves create events | Follows workspace |
| Document | Space or workspace-owned living object | Stable ID; content changes create revisions | 30-day bin plus policy |
| Block | Typed semantic unit inside a document | Position and content are operations; schema version carried | Follows document |
| Revision | Exact document state or operation range | Immutable and addressable | Retention policy, protected when cited |
| Replica | Device plus sync cursor and pending operations | One state per device/document; key epoch bound | Until device revocation plus recovery period |
| Conflict | Competing valid meanings or structures | Open, resolved, or intentionally forked | Preserved with resolution |
| CommentThread | Anchored to document range or block | Edit history retained; resolved is state, not deletion | Follows document |
| Decision | Question, options, evidence, approver, outcome, uncertainty | New version reopens; old decision immutable | Long-term record |
| Membership | Subject/device role in workspace or space | Effective-time versions and revocation | Metadata retained |
| EncryptionGroup | Members, devices, epoch, recovery policy | Append-only epoch changes | Metadata retained; secret state device-held |
| Attachment | Object reference, media facts, owner, scan state | Content-addressed revisions | 30-day bin |
| ImportJob | Source inventory, mapping, losses, result refs | Durable state machine | Report retained |
| Publication | Exact revision plus claims, proof, audience | Correction supersedes; never rewrites history | Publication policy |

# 9. System architecture

- **Domain engine:** document operation model, revision service, offline merge, sharing policy, encrypted-group state, decision workflow, and knowledge mapping.
- **Application services:** authoring API, real-time sync service, search/index service, import/export workers, notification service, and agent workspace.
- **Adapters:** HelixCore identity/policy/jobs/objects/proof/capabilities, local file import/export, mail and notification delivery, Aether proof, optional federation, and content converters.
- **Storage:** tenant-enforced relational records; encrypted object storage for attachments and bundles; device-local replica store; search index derived from approved content.
- **Event flow:** accepted operation and audit/outbox commit together; sync distributes ordered operations; index and notifications consume idempotently.
- **Background work:** import, export, indexing, encryption rotation, large merge, proof build, and agent tasks use durable jobs.
- **Offline behaviour:** full local authoring for downloaded scope, visible pending queue, local history, and explicit conflict review. New remote memberships and publication wait for connection.
- **Extension points:** block types, importers, exporters, search engines, notification channels, agents, proof providers, and federation protocols.
- **Dependencies:** HelixCore is required for shared identity and policy; local single-user mode keeps a compatible embedded contract. Aether is optional. No external model is required to read or edit.

# 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Progress, check, stop, reverse |
|---|---|---|---|---|---|
| Writing partner | Selected document and user-approved references; language and structure tools | Rewrite, outline, alternatives, comments | Accepting changes into canonical revision | Invite, publish, delete, or hide uncertainty | Diff and source list; user review; stop lease; discard branch |
| Research assistant | Exact approved search scope and safe external fetch capability | Source notes and claim/evidence table | Adding external data to workspace or publishing a claim | Invent sources or read other spaces | Query and source progress; link checks; cancel; remove imported draft |
| Meeting synthesiser | Consented transcript and named prior decisions | Notes, tasks, unresolved questions | Final minutes and assignments | Record secretly or declare a decision | Processing phases; speaker/source checks; stop; revert draft |
| Knowledge steward assistant | Approved map metadata and quality rules | Duplicate, stale, conflict, and merge suggestions | Canonical merge, retention, or deletion | Lower access controls or erase provenance | Batch progress; rule evidence; pause; undo merge |
| Migration assistant | Export/import inventory and conversion tools | Mapping and loss report | Final import and cutover | Delete source or omit unsupported items silently | Item counts and errors; hash check; stop; rollback imported draft |

# 11. Trust, safety, and privacy

- Workspace and space policy combines tenant, role, document, device, purpose, and time.
- Tenant separation is enforced by HelixCore storage policy. Shared spaces use explicit bilateral contracts.
- Content classes are public, internal, confidential, restricted, end-to-end encrypted, secret material, and proof metadata.
- Search, suggestions, and agents inherit the source access boundary. Derived indexes cannot be broader than their source.
- End-to-end encryption keys stay on approved devices or in user-approved recovery custody. The server stores ciphertext and metadata needed for delivery.
- Consent is required for recording, transcription, external search, model use, and publication.
- Residency follows documents, attachments, indexes, backups, and derived agent artifacts.
- Delete enters a recoverable 30-day bin. Permanent deletion is separately approved and may be delayed by legal hold.
- Misuse controls include guest expiry, link revocation, download policy, rate limits, suspicious sharing alerts, device revocation, and content-report workflows.
- Incident recovery can revoke devices, rotate group epochs, freeze sharing, restore a revision, and show exactly which content may have been exposed.

# 12. Proof and audit

Important membership, sharing, revision, decision, publication, import, export, recovery, device, encryption-epoch, and agent actions produce signed metadata events. High-volume typing operations are grouped into revision receipts so the ledger remains useful.

Proof bundles include the exact revision, operation or source hashes, actor and device identity, policy decision, approvals, checks, export inventory, and limitations. Independent verification can prove integrity, ordering, named approvals, and the relationship between a published bundle and its source revision. It cannot prove that a claim is factually correct or that uncaptured conversation did not happen.

Aether is preferred for stronger proof and capability use through an adapter. A local HelixCore verifier and signer remain available.

# 13. UX system

- **Main surfaces:** Home, Workspace, Document, Review, Decisions, Inbox, Knowledge Map, Agent Work, Evidence, and Recovery.
- **Navigation:** workspace switcher, left content tree, central canvas, right context rail, breadcrumbs, recent work, and command search.
- **Progressive reveal:** the canvas stays calm; presence, comments, history, proof, access, encryption, and raw sync details open as needed.
- **Keyboard and touch:** full editor commands, predictable focus, visible selection, drag alternative, and touch-safe menus.
- **Accessibility:** target [WCAG 2.2](https://www.w3.org/TR/WCAG22/) AA with semantic editor output and human assistive-technology tests.
- **Slow work:** sync, merge, import, export, encryption, indexing, and agent tasks show phases, item counts, heartbeat, elapsed time, pause/cancel, and completion notification.
- **Selection and move:** checked selected items and total count remain visible; move preview names destination, permissions, encryption change, broken links, and undo.
- **Completion:** durable inbox plus optional operating-system notification; proof and next action are one click away.
- **Recovery:** delete explains the 30-day bin; Recovery shows expiry, dependencies, preview, restore, export, and permanent purge.
- **Empty and error states:** offer a safe first action, preserve drafts, explain offline/degraded state, and never erase the working canvas because a side service failed.

# 14. Interoperability and standards

- [The WebSocket Protocol, RFC 6455](https://www.rfc-editor.org/info/rfc6455/) is used behind the real-time transport adapter. Proxy behaviour and offline semantics are Helix contracts and may not transfer.
- [Messaging Layer Security, RFC 9420](https://www.rfc-editor.org/info/rfc9420/) is used behind private group key establishment. Application content format, identity proof, and recovery remain Helix extensions.
- [Web Authentication](https://www.w3.org/TR/webauthn/) is used for device-bound sign-in and sensitive approval. Platform authenticators vary.
- [OpenDocument 1.3](https://docs.oasis-open.org/office/OpenDocument/v1.3/OpenDocument-v1.3-part1-introduction.html) is an import/export target for office documents. Comments, custom blocks, live presence, and proof may have no exact representation and must appear in the loss report.
- [ActivityPub](https://www.w3.org/TR/activitypub/) is an optional public-federation adapter, never the private workspace protocol. Helix decisions, private group state, and proof need extension objects or separate bundles.
- [JSON Schema 2020-12](https://json-schema.org/draft/2020-12) defines structured block and portable bundle contracts. Visual layout may not survive import.

# 15. Cross-platform contract

- Authoring, offline store, sync, import/export, proof verification, keyboard paths, and recovery run in CI on Windows, macOS, and Linux.
- Browser tests cover supported Chromium, Firefox, and WebKit engines. Platform passkey and notification support is capability-detected.
- A desktop shell may add filesystem and background-sync features, but browser users retain full core authoring and portability.
- Offline downloads state their scope and last sync. Unsupported storage limits are shown before large work begins.
- Mobile-width and touch use support reading, commenting, decisions, simple edits, approvals, recovery, and notifications; dense structure tools may progressively reveal.
- A missing native notification, key store, or file watcher falls back safely and reports reduced behaviour.

# 16. Reliability and performance budgets

| Measure | Budget |
|---|---|
| Accepted edit loss | 0 acknowledged operations lost in every rolling 30-day window |
| Local save | p95 under 50 ms over each rolling 24 hours on reference hardware |
| Remote acknowledgement | p95 under 500 ms over each rolling 24 hours within the deployment region |
| Presence freshness | Disconnect or stale state shown within 10 seconds |
| Offline capacity | At least 30 days and 100,000 queued operations for a downloaded workspace before an explicit capacity warning |
| Merge | 99% of non-semantic operation sets merge automatically per rolling 30 days; meaning conflicts remain visible |
| Search | p95 under 750 ms for tenant corpus up to 10 million indexed blocks over each rolling 24 hours |
| Long-work visibility | First state within 1 second; heartbeat at least every 10 seconds; completion notification within 5 seconds |
| Recovery point | 0 for acknowledged revisions; attachment replica lag at most 5 minutes |
| Recovery time | Restore one document under 60 seconds; restore a reference workspace under 4 hours |
| Cancellation | Import/export/agent cancellation acknowledged within 2 seconds and stops at a safe item boundary within 30 seconds |
| Scale | 1,000 concurrent editors in one space and 100,000 members in one tenant after measured load and privacy proof |

# 17. Success measures

- 90% of new users create, share, edit, recover, and export a document without help.
- 99.9% of reconnect sessions finish without manual conflict for non-overlapping edits per rolling 30 days.
- Zero confirmed cross-workspace or cross-tenant disclosures per rolling 12 months.
- At least 95% of review decisions link to the exact revision and named evidence.
- At least 95% of accidental deletions reported within 30 days are restored.
- 100% of critical authoring and review journeys pass keyboard and screen-reader release checks.
- 95% of exported workspaces open on a clean machine with no unexplained missing item.
- Fewer than 2% of long jobs complete without a visible notification.
- Teams report lower time-to-find-current-truth than their prior tool in a controlled adoption study.

# 18. Delivery plan

| Gate | Build | Test and safety | UX | Cross-platform | Migration and operator proof |
|---|---|---|---|---|---|
| G0 — Truthful foundation (0–6 months) | Make workspace compile; stable document model; tenant policy; offline operation log; 30-day bin | Sync crash matrix, tenant access, revision and restore tests | Honest saved/offline/conflict states | Browser and backend matrix on Windows, macOS, Linux | Import existing documents with inventory and rollback |
| G1 — Useful single-player product (6–18 months) | Rich authoring, review, decisions, search, portable bundle, durable jobs | Loss reports, accessibility, long-job and proof tests | Complete authoring, review, inbox, evidence, recovery flows | Desktop/browser limits proven | Clean-machine export restore |
| G2 — Trusted team product (18–30 months) | Private groups, knowledge map, guests, retention, cross-org space | Device revoke, epoch rotate, bilateral isolation, quality-rule tests | Access, encryption, decision, steward views | Passkey/key-store fallbacks | Team migration and key recovery drill |
| G3 — Category leader (30–42 months) | Proof publication and human-agent studio | Agent authority, source integrity, independent verifier | Agent branch and publication review | Offline verifier on all systems | Aether and local fallback exercised |
| G4 — Frontier network (42–60 months) | Sovereign federation, private federated discovery, large shared spaces, and cross-home decision assembly | Partition, malicious peer, disclosure, correction, deletion, revision-drift, denial, and exit tests | Trust, disclosure, federation, joint-decision, objection, and exit controls | Heterogeneous three-home network and offline-verifier exercise | Shared query, joint decision, correction, migration, and one-home exit drill |

Every gate needs fresh build, test, safety, UX, platform, migration, and operator evidence for the exact candidate.

# 19. Current truth and gap

**Present in live source:** a substantial Rust backend with document CRUD, revisions, flags, access lists, sharing, presence, WebSocket routes, CRDT-related code, workspace folders, comments, mentions, activity, document move, device records, attachments, residency, federation endpoints, recovery endpoints, WebAuthn routes, and OpenMLS-based group code. The Next.js client is a large working-surface implementation with rich and Markdown editing, Yjs provider wiring, comments, history, presence, offline storage/merge helpers, client encryption, device keys, passkeys, folders, attachments, sharing, toasts, and focus mode. Several domain modules contain unit tests.

**Scaffold or unproven today:** the complete workspace does not compile, so this product has no full build or end-to-end proof from the audit. Agent registration uses the shared deterministic tool runner rather than a real collaboration agent. The current UI is concentrated in one 2,263-line page, which raises maintainability and testability risk. Source contains direct delete routes, but no proven universal 30-day recovery contract. Encryption, federation, recovery, residency, WebAuthn, and multi-device behaviour have not been proven in a live adversarial integration suite. Root integration and e2e test folders are empty.

**Most important gap:** the source has wide collaboration features but no fresh system proof that offline sync, encryption membership, access control, and recovery remain correct together under crash and concurrency.

**Safest first vertical slice:** one workspace, two users, two devices, one guest, one document, offline concurrent edits, review approval, encrypted sharing, device revoke, 30-day delete/restore, export, and independent revision proof across Windows, macOS, and Linux.

# 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Document identity | Stable IDs and revision graph; never path as identity | Architecture decision and migration proof |
| Edit model | Operation log plus immutable revisions; no silent last-write-wins | Architecture decision |
| Offline | Local-first for downloaded scope with explicit pending queue | Product decision with equivalent continuity proof |
| Conflicts | Preserve both valid meanings and require visible resolution | No exception |
| Delete | 30-day recoverable bin; separate audited purge | Founder approval |
| Sharing | Narrowest resource, role, purpose, and expiry by default | Security review |
| Agents | Suggest on a separate branch; cannot self-approve or publish | Founder approval |
| Encryption | Device-held private group keys behind a versioned adapter | Security review |
| Search and models | Opt-in by content scope; derived data inherits source access | Privacy review |
| Canonical UI | One workspace shell with progressive side rails | Founder approval |
| Slow work | Durable job state, heartbeat, cancel, and completion notification | Product decision with equivalent UX proof |
| Federation | Off by default; public/partner adapter only after G3 proof | Founder approval |
| Export | Open files plus manifest, history choice, proof, and loss report | Architecture decision |
| Aether | Preferred proof/capability adapter with local fallback | Founder approval |
| Cross-platform | Windows, macOS, Linux, web release gates | Founder approval |
| Founder-only choices | Default managed storage region and commercial guest limits | Founder decision; does not block G0 |

# 21. Definition of category-defining done

- [ ] Real users complete every signature journey, including offline and cross-organisation work.
- [ ] No acknowledged edit is lost under crash, partition, replay, or concurrent merge tests.
- [ ] Independent proof binds important decisions and publications to exact revisions.
- [ ] Access and encryption tests prove device removal and tenant separation.
- [ ] Every long sync, import, export, index, encryption, and agent task remains visibly alive and ends with notification.
- [ ] Delete is recoverable for 30 days and permanent purge is explicit.
- [ ] Complete useful exports open on a clean machine and state every conversion loss.
- [ ] Windows, macOS, Linux, major browsers, touch, and offline limits are tested.
- [ ] Critical journeys meet WCAG 2.2 AA and pass human assistive-technology review.
- [ ] Agents remain bounded, reviewable, stoppable, reversible, and unable to retrieve secret values.
- [ ] Backup, restore, device loss, key recovery, migration, federation exit, and incident drills meet budgets.
- [ ] Security and privacy review closes all critical and high findings or records a named, time-bound exception.
- [ ] Aether and every external model/provider can be removed without losing core authoring or access to owned data.
- [ ] The product states what revisions, signatures, consensus, and agent summaries do not prove.
