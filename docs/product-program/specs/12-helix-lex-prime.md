# HelixLexPrime — source-grounded legal work and regulatory system

```yaml
product: HelixLexPrime
catalog_order: 12
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [lawyers, legal operations teams, compliance leads, regulated organizations]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixLexPrime is a human-led legal work system where every material statement can
be traced to a dated source, jurisdiction, version, reasoning step, reviewer, and
decision, joining research, matters, obligations, drafting, review, deadlines,
and proof without pretending that software is a lawyer or a court.

## 2. Five-year destination

The useful product is a private matter room with source capture, citation checking,
obligation tracking, drafting, review, deadlines, and recoverable records. The
leading product maintains a living legal knowledge graph that explains which rule
applies, when, where, to whom, and why. The frontier product can monitor authorized
sources, show possible changes, simulate policy effects, and draft response options.
It never creates legal authority, gives unqualified final advice, accepts a contract,
files with a court or regulator, or sends a binding notice without a named human.

## 3. Users and hard jobs

- A lawyer needs fast research but fears fabricated authority, stale law, and lost
  exceptions.
- A legal operations team needs one matter history across documents, tasks, people,
  and systems.
- A compliance lead needs obligations mapped to evidence and fears a false green
  status.
- A business owner needs plain options, costs, deadlines, and uncertainty before a
  decision.
- A reviewer needs to see exact sources, edits, conflicts, and who accepted the work.
- A regulated organization needs privilege, confidentiality, retention, legal hold,
  and jurisdiction controls that survive export and migration.

## 4. Product laws

1. Every legal statement names its source, jurisdiction, effective time, and
   confidence or is clearly labeled an unsupported draft.
2. Primary authority is distinct from commentary, internal policy, facts, and model
   output.
3. A source update never silently rewrites earlier advice or work product.
4. Agents may research and draft; only an authorized human may advise, accept,
   submit, sign, notify, or waive a right.
5. Deadlines show rule, trigger event, calendar, timezone, calculation, reviewer,
   and uncertainty.
6. Privilege and confidentiality labels restrict access, search, export, and model
   use.
7. A missing source, failed fetch, or disputed interpretation is visible, not green.
8. A complete export includes sources, versions, decisions, proof, and limitations.
9. Critical journeys are accessible, portable, and independently verifiable across
   platforms; no model, source vendor, or proof provider is a hard dependency.

## 5. Scope boundaries

LexPrime owns source-grounded legal research, matter records, obligation maps,
deadline calculations, drafting, redlines, review, approval routing, evidence, and
authorized monitoring. HelixCore owns shared identity, policy, audit, capabilities,
jobs, objects, billing, and operations; Aether is reached through a provider-neutral
proof interface with a local fallback. It does not replace a licensed professional, a court filing
service, an e-signature authority, a document management system required by a firm,
or a regulator's official register. It does not practice law on its own. Submission,
signature, settlement, contract acceptance, disclosure, privilege waiver, and final
legal advice remain outside agent authority and require local professional judgment.

## 6. Signature experiences

1. **Open a defensible matter.** Entry: an authorized user selects jurisdiction,
   client, purpose, confidentiality, and retention. Progress: conflicts, authority,
   required facts, and missing controls appear. Human decision: a matter owner
   accepts scope and access. Proof: identity, terms, and approvals are signed.
   Failure and recovery: an incomplete conflict or authority check blocks active
   work without losing the draft. Export: a matter-opening manifest.
2. **Research a legal question.** Entry: a lawyer states the question, facts, date,
   and jurisdiction. Progress: source search, retrieval, validation, reading, and
   conflict checks stream visibly. Human decision: the lawyer selects authority and
   approves the analysis. Proof: exact source versions, passages, treatment, and
   reasoning links are kept. Failure and recovery: unavailable sources remain
   `unverified`; the run resumes from durable retrieval. Export: a cited research
   bundle.
3. **Turn rules into obligations.** Entry: a team imports approved laws, contracts,
   or policies. Progress: actors, duties, rights, conditions, exceptions, dates, and
   evidence requirements are extracted for review. Human decision: counsel accepts
   each obligation. Proof: every obligation links to the source span and review.
   Failure and recovery: uncertain clauses enter a queue, not the control register.
   Export: an obligation matrix with provenance.
4. **Draft and review a legal document.** Entry: counsel selects a matter, template,
   and approved positions. Progress: drafting, source checks, defined terms,
   conflicts, and redlines remain visible. Human decision: counsel accepts every
   material clause and final version. Proof: source, prompt, tool, edit, and approval
   history are kept. Failure and recovery: interrupted work returns to the last
   durable version; deleted drafts enter the bin. Export: clean copy, redline, and
   drafting record.
5. **Calculate and guard a deadline.** Entry: a verified trigger event is recorded.
   Progress: candidate rules, calendars, exclusions, timezones, and review state are
   shown. Human decision: a lawyer confirms the controlling rule and date. Proof:
   the full calculation is replayable. Failure and recovery: any missing fact marks
   the deadline `needs review` and raises a human alert. Export: a deadline proof
   card and calendar event draft.
6. **Respond to a legal change.** Entry: an authorized source version changes.
   Progress: identity, effective date, affected obligations, matters, and controls
   are analyzed. Human decision: counsel accepts the impact and response plan.
   Proof: old and new text, diff, reasoning, and decisions remain linked. Failure
   and recovery: a bad feed is quarantined and cannot change status. Export: a
   change-impact packet.

## 7. Capability map

F0 is foundation, F1 is the useful product, F2 is the trusted-team product, F3 is
advanced category leadership, and F4 is the frontier network. Every row inherits this full contract: its invariants are the
product laws and typed truth boundaries; authority is the exact named human and
policy in Sections 10–11; evidence is input/output hashes, versions, actor, decision,
and ledger event; failure is a durable `blocked`, `failed`, `unverified`, `unknown`,
or quarantined state with retry or recovery; and test acceptance includes denial,
failure, recovery, and cross-platform cases in addition to the row's named check.
The row names its domain-specific inputs, output, and strongest acceptance test.

| ID | First gate | Capability contract |
|---|---|---|
| HLP-F0-01 | G0 | **Legal source identity.** Inputs are authorized documents or official URIs; outputs are immutable source versions with jurisdiction, issuer, dates, hierarchy, language, and hash. Unverified identity is explicit. Acceptance: fixtures detect version, citation, and source substitution errors. |
| HLP-F0-02 | G0 | **Matter and authority boundary.** Inputs are client, purpose, role, confidentiality, and jurisdiction; output is a scoped matter with exact access. No work crosses matters by default. Acceptance: tenant, privilege, and purpose-denial tests pass. |
| HLP-F0-03 | G0 | **Atomic legal event ledger.** Inputs are source, draft, review, decision, and deadline events; output is one ordered, append-only history. Acceptance: concurrent writes, crash recovery, and idempotent replay lose no acknowledged event. |
| HLP-F1-01 | G1 | **Source-grounded research.** Inputs are a scoped question and approved sources; output is an analysis graph with quotations, treatment, conflicts, and unknowns. Agents cannot mark it final. Acceptance: every material statement has a resolvable source or an unsupported label. |
| HLP-F1-02 | G1 | **Citation and authority validator.** Inputs are citations and source versions; output is identity, date, hierarchy, treatment, and mismatch status. Acceptance: stale, reversed, missing, and wrong-jurisdiction fixtures fail visibly. |
| HLP-F1-03 | G1 | **Versioned drafting and redline.** Inputs are approved facts, positions, templates, and sources; output is an immutable draft chain and semantic redline. Acceptance: no agent edit can bypass review or alter accepted text silently. |
| HLP-F2-01 | G2 | **Obligation graph.** Inputs are reviewed clauses and rules; output is actors, duties, rights, conditions, exceptions, controls, evidence, and dates. Acceptance: each active obligation links to exact source spans and human acceptance. |
| HLP-F2-02 | G2 | **Deadline engine.** Inputs are verified triggers, rules, calendars, timezone, service facts, and exceptions; output is candidate dates and a replayable calculation. Acceptance: missing facts cannot produce a confirmed date. |
| HLP-F2-03 | G2 | **Review and approval room.** Inputs are drafts, issues, redlines, and authority policy; output is a signed decision chain. Acceptance: only a role with exact matter authority may accept a material action. |
| HLP-F3-01 | G3 | **Regulatory change monitor.** Inputs are allowlisted official feeds and known source versions; output is a quarantined change candidate and impact graph. It never changes obligations automatically. Acceptance: feed failure or identity drift cannot create a legal-status update. |
| HLP-F3-02 | G3 | **Control and evidence map.** Inputs are accepted obligations, controls, tests, and evidence; output is `supported`, `partial`, `failed`, or `unknown`. Acceptance: evidence expiry or source change removes green status until review. |
| HLP-F3-03 | G3 | **Portable matter proof.** Inputs are authorized sources, facts, work product, decisions, and access/redaction policy; output is a verifiable bundle. Acceptance: an independent verifier confirms hashes and omissions from the redaction manifest. |
| HLP-F3-04 | G3 | **Bounded legal scenario studio.** Inputs are human-approved facts, rule versions, interpretations, and policy choices; output is labeled options and sensitivity, not advice. Acceptance: the studio cannot select or execute an option. |
| HLP-F3-05 | G3 | **Cross-jurisdiction rule map.** Inputs are reviewed local mappings; output shows common concepts and material differences without forcing false equivalence. Acceptance: every mapping names reviewer, date, source, and loss. |
| HLP-F3-06 | G3 | **Institutional legal memory.** Inputs are approved, non-privileged reusable patterns; output is a permissioned knowledge graph. Acceptance: matter removal revokes its bindings, and privileged content never enters general retrieval. |
| HLP-F4-01 | G4 | **Sovereign public-law source mesh.** **Input:** allowlisted official public-source manifests, signed change candidates, jurisdiction, hierarchy, and reviewer policy from independent nodes. **Output:** a revocable cross-node graph of source identity, treatment, conflicts, and freshness. **Invariant:** private, privileged, and matter-specific data never enter the mesh, and no source event automatically changes a legal conclusion, obligation, filing, or advice. **Authority:** each jurisdiction's named human legal-source steward approves publication, mapping, and withdrawal; there is no global legal authority. **Evidence:** source hashes, fetch identity, reviewer, mapping version, conflict, use, expiry, and revocation events. **Failure:** source drift, disputed treatment, stale review, or withdrawal quarantines the affected edge as unverified. **Acceptance:** five independent nodes process 10,000 source-change, conflict, join, and revoke cases with zero matter-data disclosure or automatic legal-status change. |
| HLP-F4-02 | G4 | **Multi-party obligation evidence room.** **Input:** each party's human-accepted obligations, controls, redacted proof, disclosure rules, and purpose. **Output:** a side-by-side map of agreements, differences, missing proof, and human questions. **Invariant:** every party keeps its own legal interpretation and approval; the room cannot form a contract, waive a right, select a position, send a filing, or execute an obligation. **Authority:** each party's named counsel or authorized reviewer approves its disclosure and any accepted mapping. **Evidence:** source spans, redaction manifest, party approvals, mapping decisions, dissent, access, and withdrawal. **Failure:** privilege risk, missing authority, incompatible meaning, or revoked disclosure blocks that comparison and preserves other parties' state. **Acceptance:** 1,000 synthetic multi-party cases preserve every local veto, detect all seeded meaning conflicts, and create zero external legal action. |
| HLP-F4-03 | G4 | **Human-governed legal methods benchmark network.** **Input:** approved public or hypothetical corpora, scoped tasks, jurisdiction labels, scoring rules, model versions, and prohibited-use policy. **Output:** reproducible results for citation support, source identity, uncertainty, redaction, and authority-boundary compliance. **Invariant:** benchmark output is not matter advice, legal ranking, or professional approval, and privileged material cannot be reused as a general corpus. **Authority:** an independent human methods and professional-responsibility board approves datasets, tasks, publication, and removal. **Evidence:** corpus and runner hashes, consent or public basis, scores, failures, reviewer attestations, and removals. **Failure:** contaminated data, source loss, leakage, or unrepeatable results invalidate the affected run and published comparison. **Acceptance:** 100 clean-machine benchmark runs across all three OSs reproduce declared scores within tolerance and reject 10,000 seeded citation, privilege, and authority violations. |

## 8. Domain model

| Record | Ownership, lifecycle, and relationships |
|---|---|
| `Client` / `Matter` | Stable identity, responsible professional, purpose, jurisdiction, access, confidentiality, retention, and legal holds. |
| `LegalSource` / `SourceVersion` | Issuer, authority type, jurisdiction, dates, language, official locator, content hash, and prior/superseding links. |
| `Fact` / `FactVersion` | Source, owner, disputed state, materiality, confidentiality, and links to issues; never silently promoted from model output. |
| `Issue` / `Position` / `ReasoningLink` | Question, candidate rule, application, conflict, conclusion, limits, reviewer, and decision state. |
| `Obligation` / `Exception` / `Control` | Actor, action, object, condition, jurisdiction, effective period, source span, owner, evidence, and review. |
| `TriggerEvent` / `DeadlineCalculation` | Verified event, rule, calendar, exclusions, timezone, candidate result, uncertainty, and confirmation. |
| `Draft` / `ClauseVersion` / `Redline` | Immutable text versions, defined terms, source links, author/agent actions, comments, and approvals. |
| `SubmissionIntent` / `Approval` | Proposed external action, exact payload hash, destination, authority, expiry, and human decision; execution is separate. |
| `RecoveryItem` / `LegalHold` | Deleted work, restore window, purge block, authority, and signed lifecycle events. |

## 9. System architecture

- A Rust legal kernel enforces source identity, matter isolation, state machines,
  authority, deadlines, and append-only events.
- PostgreSQL stores domain records with database-enforced tenant and matter access.
  Encrypted object storage keeps sources and work product by content hash.
- A source gateway retrieves only allowlisted resources, records fetch time and
  response identity, and quarantines change before use.
- A citation and rule engine parses adapters but keeps one provider-neutral internal
  graph. Unsupported meaning is reported.
- Sandboxed agents receive read or draft capabilities for one matter and purpose.
  They have no filing, signature, email-send, settlement, or contract-accept tool.
- A user-owned broker holds external credentials and grants one approved process a
  narrow lease. Aether provides preferred proof, with a local verifier fallback.
- HelixCore supplies shared identity, policy, audit, capabilities, jobs, objects,
  billing, and operations behind domain interfaces; LexPrime retains legal truth.
- The event flow is request, matter-authority check, source/domain validation, atomic
  record plus event, projection, notification, and proof. Background work uses durable
  HelixCore jobs, idempotent checkpoints, visible progress, and explicit cancellation.
- Offline source review, drafting, recovery, and verification use a local encrypted
  store. Versioned source, citation, rule, calendar, and proof adapters are
  contract-tested extension points; none may add an external legal action.

## 10. Agent and automation contract

| Role | May do | Must not do |
|---|---|---|
| Research assistant | Find, compare, summarize, and cite approved sources | Invent authority, hide conflict, or call output final advice |
| Obligation analyst | Draft duties, rights, exceptions, and evidence links | Activate an obligation without human acceptance |
| Drafting assistant | Produce clauses and redlines from approved positions | Accept terms, sign, waive, send, or file |
| Deadline clerk | Calculate candidate dates and show every input | Confirm a date when any controlling fact is missing |
| Change monitor | Detect and explain source changes | Update legal status or controls automatically |

Agents work inside exact matter, purpose, source, data, tool, time, and spend leases.
The interface shows stage, elapsed time, sources checked, last signal, uncertainty,
and any human decision needed. Revocation ends future access. A remote cancellation
stays pending until the provider confirms it. Every draft enters named professional
review; citation, schema, policy, and evidence checks validate the result. Reversal
restores a prior version or revokes authority without rewriting history.

## 11. Trust, safety, and privacy

| Safety case | Prevention, human authority, proof, and recovery |
|---|---|
| Fabricated or stale authority | Official-source identity, version capture, citation resolution, conflict checks, and `unverified` state. A lawyer accepts the source set and analysis. |
| Unauthorized legal action | No agent filing, signature, send, acceptance, settlement, or waiver capability. The exact payload and destination require human approval in an external action system. |
| Privilege or confidentiality leak | Matter isolation, purpose binding, least privilege, local custody, protected search, redacted notifications, export review, and metadata-safe audit. |
| Wrong jurisdiction or date | Every source, obligation, and conclusion carries jurisdiction and temporal scope. Mismatch blocks confirmed status. |
| Missed deadline | Dual-source rule review, visible inputs, timezone/calendar tests, human confirmation, escalation, and independent reminders. The product never guarantees a court date. |
| Unsafe deletion | Draft deletion enters a 30-day bin where lawful. Legal hold, professional duty, regulation, litigation, or retention policy blocks purge. Permanent deletion requires re-authentication, impact preview, named authority, and signed proof. |

Legal and privacy threat models, professional-responsibility review, jurisdictional
adapters, abuse tests, and recovery drills are release gates. The system uses clear
notices that it supports, but does not replace, qualified legal judgment. Tenant and
matter separation are enforced in the database and object layer. Data is encrypted
in transit and at rest, residency follows matter policy, and incident recovery can
quarantine access, revoke leases, preserve privilege-safe evidence, and restore a
reviewed state.

## 12. Proof and audit

Proof records the question, facts used, source identity and version, retrieval time,
citations, reasoning graph, agent and tool versions, edits, rejected alternatives,
human decisions, deadline inputs, external action intents, and known limits. Audit
metadata never copies privileged text when an identifier and hash are enough. A
redaction manifest lists every excluded class. Aether is the preferred signed proof
layer; a local signed bundle and offline verifier are mandatory. Proof establishes
provenance and process, not that advice is correct or a court will agree.

## 13. UX system

The primary surfaces are Home, Matters, Sources, Research, Obligations, Drafts,
Deadlines, Reviews, Evidence, and Recovery. Each view leads with matter, jurisdiction,
effective date, authority level, current state, next human decision, and last signal.
Source passages open beside the statement they support. Detail reveals progressively:
plain summary first, then rule, application, conflict, and full provenance. Long
research shows live sources checked and does not use fake percent complete. Completion,
failure, source drift, approaching deadline, and approval needs create private in-app
and optional desktop notices. Moving a clause previews changed references and defined
terms. Delete shows hold status and restore date before confirmation. Reversible
edits offer undo; empty states explain the first safe action; plain-language errors
state what happened, what remains safe, and how to recover. Keyboard and touch paths
have the same matter and authority checks.

## 14. Interoperability and standards

All links below were verified from the official body on 2026-07-15.

- [Akoma Ntoso 1.0](https://www.oasis-open.org/standard/akn-v1-0/) is an adapter for
  structured legislative, regulatory, and judicial documents. Loss caveat: its
  metadata does not decide local authority, treatment, or applicability.
- [LegalRuleML 1.0](https://www.oasis-open.org/standard/legalrulemlv1-0/) can exchange
  reviewed formal rules and defeasible logic. Loss caveat: formal encoding cannot
  replace interpretation, facts, professional judgment, or local procedure.
- The [European Legislation Identifier](https://op.europa.eu/en/web/eu-vocabularies/eli)
  identifies and describes European legislation. Loss caveat: it applies only where
  an official publisher implements it and is not a global authority scheme.
- [W3C PROV-O](https://www.w3.org/TR/prov-o/) is the provenance export adapter.
  Loss caveat: privilege, professional authority, legal hold, and domain-specific
  reasoning need Helix extensions.

Standards are adapters, not the source of legal truth. Each adapter records version,
jurisdiction, unsupported meaning, and validation result.

## 15. Cross-platform contract

Windows, macOS, and Linux pass the same matter-isolation, source-identity, citation,
deadline, drafting, redaction, crash-recovery, and export fixtures. The same six
journeys pass in two current web engines. Offline mode supports private sources,
drafting, review, deadline replay, and proof verification. Managed mode may monitor
sources at scale but cannot weaken matter custody or human authority. Install,
upgrade, backup, restore, and uninstall tests use disposable state. Accessibility
includes keyboard-complete source review, redlines, tables, and deadline flows. The
CLI and container surfaces support administration, import/export, and fresh checks
only; they cannot send, sign, accept, waive, settle, or file. Optional platform
features use capability detection and a safe fallback.

## 16. Reliability and performance budgets

- Acknowledged source, approval, and deadline events have RPO 0 under process crash
  and concurrent writers in every release test.
- In each calendar month, 99.95% of authorized matter metadata reads complete without
  server error; source-provider failure is reported separately.
- A 100,000-document matter opens to a useful index in p95 under 2 seconds on the
  reference workstation; a cited passage opens in p95 under 500 ms when local.
- Long research emits a source or reasoning signal at least every 10 seconds or shows
  `no recent signal` after 30 seconds.
- Local cancellation is acknowledged within 2 seconds and reaches a checkpoint
  within 30 seconds. External fetches remain `cancel requested` until confirmed.
- Deadline recalculation after a verified source or trigger change completes within
  60 seconds for 100,000 active obligations and never changes a confirmed date
  without review.
- Metadata RTO is 30 minutes and encrypted object RTO is 4 hours in the supported
  self-hosted profile, tested quarterly.
- Create, import, fetch, and calculation requests use idempotency keys retained for
  at least 24 hours; a duplicate returns the original durable result.
- Offline mode cannot monitor official sources or perform external action. Unsynced
  work stays visible. If an optional source or model fails, local research, drafting,
  review, recovery, and export remain in a named degraded state.

## 17. Success measures

Measure material statements with verified primary sources, citation defects caught
before review, obligations with accepted source spans, deadline changes detected and
confirmed, time from change to reviewed impact, unauthorized actions blocked,
privilege leakage tests passed, accessible journey completion, cross-platform export
validation, independent bundle validation, recovery drill success,
and reviewer time to find uncertainty. Do not measure generated words, matters opened,
or agent activity as proof of legal quality. Business measures are renewal after a
verified matter journey, support burden per active team, and cost per reviewed,
independently valid work product.

## 18. Delivery plan

- **G0 — Truthful foundation (0–6 months):** freshly prove shared service startup; replace generic records
  with source identity, matter authority, and an atomic ledger; add disposable-state
  tests, three-platform CI, and truthful capability status.
- **G1 — Useful single-player product (6–18 months):** ship grounded research, citation validation, versioned
  drafting, review UX, private notifications, accessibility, and recovery.
- **G2 — Trusted team product (18–30 months):** add obligation graph, deadline engine, approval room,
  source adapters, redaction controls, and portable matter proof.
- **G3 — Category leader (30–42 months):** add authorized change monitoring, control evidence,
  bounded scenario work, reviewed cross-jurisdiction maps, institutional memory,
  team operations, external privacy review, and jurisdiction adapter contracts.
- **G4 — Frontier network (42–60 months):** ship HLP-F4-01 the sovereign public-law
  source mesh, HLP-F4-02 the multi-party obligation evidence room, and HLP-F4-03
  the legal methods benchmark only after professional-responsibility and founder
  gates. Fresh G4 proof requires five independently governed nodes, 10,000 source
  and revocation cases, 1,000 synthetic multi-party reconciliations, 100 reproducible
  benchmark runs, zero privileged or matter-data disclosure, and zero autonomous
  legal conclusion, advice, contract, filing, waiver, or external action.

Every gate runs fresh Rust and web builds, unit and integration tests, legal-domain
fixtures, six end-to-end journeys, accessibility, authorization, source-loss,
deadline, migration, recovery, redaction, security, Windows/macOS/Linux packaging,
and browser checks. A stored green badge cannot satisfy a gate.

## 19. Current truth and gap

The live source is a generated scaffold. Its backend exposes generic `matters` and
`filings` create/list/get records with title, body, status, and metadata. There is
no source identity, matter authority model, legal knowledge graph, deadline engine,
filing integration, citation checker, or legal-domain test. The assistant has only
echo and product-catalog tools. The web folder has `package.json` but no product UI.
The live backend now applies route state and calls the shared graceful-shutdown server
helper; the earlier startup defect is repaired in source, but this spec-only pass did
not run a fresh build. The product must not imply that any filing exists merely
because a generic table is named `filings`. The first honest slice is HLP-F0-01
through HLP-F0-03 plus fresh build and CI proof.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Internal truth | Versioned source and legal-work graph | Architecture decision and migration proof |
| Legal authority | Named qualified human for the matter | Governance and professional review |
| Agent role | Research, draft, compare, and flag only | Founder and safety decision |
| External action | No agent send, sign, accept, waive, settle, or file | Founder, legal, and security approval |
| Missing evidence | `unverified` or `needs review`, never green | Trust review |
| Privilege | Matter isolation and purpose-bound access | Legal/privacy review |
| Deadline | Candidate until human confirmation | Professional review |
| Delete | 30-day bin where lawful; hold and duty override purge | Retention decision |
| Proof | Aether preferred, offline signed bundle required | Architecture decision |

## 21. Definition of category-defining done

- [ ] Every material legal statement resolves to a dated, jurisdiction-bound source
  or is visibly unsupported.
- [ ] A source update creates a reviewable impact, never a silent rewrite.
- [ ] No agent can advise finally, accept, sign, waive, settle, send, or file.
- [ ] Matters, privilege, confidentiality, retention, and legal holds survive export
  and migration.
- [ ] Deadline calculations are replayable and cannot hide missing facts.
- [ ] A reviewer can see facts, rules, application, conflicts, edits, and decisions
  without reading an agent transcript.
- [ ] Independent bundles validate and state what they do not prove.
- [ ] Windows, macOS, Linux, web, offline, accessibility, recovery, privacy, and
  security gates pass from fresh source.
- [ ] Qualified legal, privacy, and security reviewers accept the product safety case.
