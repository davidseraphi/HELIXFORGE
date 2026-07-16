# HelixCapital — continuous-close finance with proof

```yaml
product: HelixCapital
catalog_order: 7
status: target-state-spec
horizon: 60 months
current_maturity: prototype
primary_users: [business owners, finance teams, accountants, treasury operators, auditors]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

> **Target-state rule:** Sections 1–18 and 20–21 are planned capability. Only
> Section 19 describes the implementation that exists today.

## 1. Category claim

HelixCapital is a proof-first finance workspace that keeps every business number
traceable from source transaction through approval, double-entry posting,
reconciliation, close, statement, and later correction.

## 2. Five-year destination

The useful product is a local-first accounting and treasury workspace for a
small business or finance team: chart of accounts, journals, receivables,
payables, bank imports, reconciliation, close, statements, budgets, and cash
plans. The category-defining advantage is a continuous close where each reported
number carries its source, policy, approval, reconciliation, and proof. The
frontier capability is a consented capital network where organisations can share
verified facts, request finance, and settle through regulated providers without
giving one platform custody of books or credentials. Accountable humans keep
authority over postings, period close, payment approval, tax treatment,
reporting policy, forecasts used for action, and all movement of money.

## 3. Users and hard jobs

- **Business owners** need to know cash, obligations, and performance now. They
  fear confident reports built from missing or duplicated transactions.
- **Finance teams and accountants** need a correct book and fast close. They fear
  silent edits, currency errors, and unreconciled balances.
- **Treasury operators** need safe cash plans and payment controls. They fear an
  agent or compromised account moving money.
- **Department owners** need simple evidence and approval flows. They fear
  finance work that is invisible until a deadline is missed.
- **Auditors and boards** need an independent trail from statements to sources.
  They fear a ledger that can be rewritten after review.

## 4. Product laws

1. Every posted journal balances by currency and accounting unit.
2. Posted entries are never edited or deleted; corrections use linked entries.
3. Currency is explicit and exact; conversion uses a named rate and source.
4. Source documents, approvals, postings, reconciliations, and reports stay linked.
5. No agent may post, close a period, approve itself, or move money.
6. A forecast is visibly separate from the accounting book and reported fact.
7. Jurisdiction and reporting rules live in versioned adapters and policy packs.
8. Provider data is reconciled; it never silently overwrites the book.
9. Long imports, matching, close, and reports show real progress and exceptions.
10. Unknown, late, disputed, and unreconciled values remain honest states.

## 5. Scope boundaries

HelixCapital owns accounting entities, chart of accounts, journals, source
documents, receivables, payables, reconciliation, close, statements, budgets,
cash scenarios, treasury approval, and reporting exports. HelixCore owns
identity, policy, audit, capabilities, jobs, objects, billing, and operations.
HelixCommerce owns customer order and commerce fulfilment truth. HelixInsights
owns general analytics. Regulated banks and payment providers hold funds and
execute transfers through adapters.

HelixCapital is not a bank, broker, auditor, tax authority, or licensed financial
adviser. It does not promise legal, tax, audit, lending, or investment compliance.
Licensed and accountable people remain responsible for regulated decisions,
filings, assurance, and movement of money.

## 6. Signature experiences

1. **Create a trustworthy book.** **Entry:** an accountant creates or imports an
   entity and chart. **Visible progress:** accounts, currencies, opening balances,
   mappings, and errors are counted. **Human decision:** the accountant approves
   policy and opening entries. **Completion proof:** balances, approvals, and
   source hashes form a signed opening packet. **Failure and recovery:** invalid
   rows stay quarantined and import can resume. **Export:** chart, entries,
   policies, and proof are portable.
2. **Capture and post a transaction.** **Entry:** a user uploads a document or
   starts a journal. **Visible progress:** extraction, duplicate, account, tax,
   currency, and approval checks show. **Human decision:** a named poster approves
   the final journal. **Completion proof:** source, journal, lines, actor, and
   policy commit together. **Failure and recovery:** a crash produces no partial
   posting and retry returns one result. **Export:** source and journal packet are
   downloadable.
3. **Reconcile a bank account.** **Entry:** finance imports a consented feed or
   statement. **Visible progress:** matched, suggested, split, missing, duplicate,
   and difference totals update live. **Human decision:** a reviewer approves
   matches and exceptions. **Completion proof:** source balance, book balance,
   items, rules, and approval reconcile. **Failure and recovery:** an adapter
   outage leaves a resumable import and does not mark reconciliation complete.
   **Export:** statement, matches, and difference report are portable.
4. **Pay with dual control.** **Entry:** an approved payable becomes a payment
   proposal. **Visible progress:** beneficiary, amount, currency, bank, limits,
   approvals, provider state, and reconciliation are separate. **Human decision:**
   required people approve in the external or local control flow. **Completion
   proof:** consent and provider evidence link to the payable and later bank line.
   **Failure and recovery:** unknown provider state blocks retry until reconciled.
   **Export:** payment and approval proof are portable.
5. **Close a period.** **Entry:** a controller opens the close room. **Visible
   progress:** every task, owner, exception, reconciliation, adjustment, and check
   has a real state. **Human decision:** an authorised controller closes and may
   later reopen through a separate approval. **Completion proof:** close packet
   pins ledger, policy, statements, checks, and signatures. **Failure and
   recovery:** failed checks keep the period open. **Export:** the full close
   binder is portable.
6. **Explain a reported number.** **Entry:** an owner selects a statement line.
   **Visible progress:** the system loads dimensions, journals, source documents,
   reconciliations, and policy. **Human decision:** an accountant can annotate or
   correct by a new entry. **Completion proof:** the trace ends at signed source
   and balance checks. **Failure and recovery:** missing source is shown as a gap.
   **Export:** one line's evidence graph can be shared with a reviewer.
7. **Plan cash without changing truth.** **Entry:** a user copies actuals into a
   forecast scenario. **Visible progress:** assumptions, scheduled items,
   uncertainty, versions, and runs show. **Human decision:** people approve any
   action based on the forecast. **Completion proof:** actuals and assumptions
   remain separate and reproducible. **Failure and recovery:** stale rates or
   missing data mark the scenario incomplete. **Export:** assumptions, model,
   inputs, outputs, and limits are portable.

## 7. Capability map

### F0 — foundation

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| CAP-F0-001 | G0 | Entity, reporting unit, currency | Stable accounting identity | Identity is independent of path, bank, and deployment | Owner creates; finance approves opening | Identity and policy event | `draft` | WHEN a workspace moves folders, entity and ledger IDs SHALL remain unchanged. |
| CAP-F0-002 | G0 | Account facts and hierarchy | Versioned chart of accounts | Account identity never changes; disabled accounts remain referential | Accountant approves | Chart diff and approval | `invalid_chart` | WHEN an account is retired, old entries SHALL still resolve to it. |
| CAP-F0-003 | G0 | Journal and exact money lines | Posted journal plus event | Debits equal credits by currency; post and event are atomic | Named human poster | Lines, balance check, signature | `draft`, `rejected`, `not_committed` | WHEN any crash occurs during post, recovery SHALL show zero or one balanced journal, never partial lines. |
| CAP-F0-004 | G0 | Command and idempotency key | One durable result | Retry cannot duplicate journal, source, or provider action | Service policy | Request and replay record | `unknown_external_state` | WHEN the same posting command is retried, the system SHALL return the first journal ID. |

### F1 — useful product

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| CAP-F1-001 | G1 | File, email attachment, or adapter record | Source document version | Original bytes are immutable; duplicate checks are visible | User imports; reviewer classifies sensitive data | Hash, origin, extraction | `quarantined` | WHEN the same invoice arrives twice, the system SHALL flag a possible duplicate before posting. |
| CAP-F1-002 | G1 | Source, policy, account suggestion | Journal draft and approval route | Agent suggestion cannot become posted entry | Preparer drafts; separate poster approves by policy | Suggestion, edits, approvals | `needs_approval` | WHEN preparer and approver must differ, one identity SHALL not complete both actions. |
| CAP-F1-003 | G1 | Counterparty, due item, settlement | Receivable or payable lifecycle | Open amount cannot be negative; currency cannot change | Finance user; write-off needs approval | Source, allocations, reasons | `overdue`, `disputed`, `settled` | WHEN a partial settlement arrives, the system SHALL preserve original, paid, and open amounts exactly. |
| CAP-F1-004 | G1 | Ledger and report policy | Trial balance and statements | Every total reconciles to posted ledger version | Accountant prepares; controller publishes | Ledger hash, calculation, policy | `unbalanced` or `draft` | WHEN a report is generated, its totals SHALL tie to the pinned trial balance. |

### F2 — category leader

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| CAP-F2-001 | G2 | Bank statement/feed and book items | Reconciliation | Provider rows never overwrite entries; closing difference is explicit | Reviewer approves matches | Import, match rule, approval | `difference_open` | WHEN difference is non-zero, the account SHALL not be marked reconciled. |
| CAP-F2-002 | G2 | Period, tasks, checks, approvers | Close packet | Failed check blocks close; reopen is a new event | Controller closes; separate policy may approve reopen | Tasks, checks, ledger hash, signatures | `close_blocked` | WHEN an unreconciled material account exists, close SHALL be rejected with its owner. |
| CAP-F2-003 | G2 | Approved payable, limits, provider | Payment proposal and observations | Product does not export secret; unknown state blocks duplicate | Human approvers and regulated provider | Consents, provider refs, reconciliation | `approval_waiting`, `checking` | WHEN provider response is lost, retry SHALL not create a second transfer. |
| CAP-F2-004 | G3 | Ledger, dimensions, evidence | Explainable statement graph | Every reported fact reaches journal and source or names the gap | Reviewer reads; accountant corrects by new entry | Calculation and provenance graph | `evidence_gap` | WHEN a statement line is selected, the system SHALL list all contributing entries and unresolved gaps. |

### F3 — advanced category leadership

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| CAP-F3-001 | G3 | Actuals, assumptions, schedules | Cash forecast with ranges | Forecast never posts to book and shows uncertainty | Finance drafts; human approves actions | Inputs, model, back-test | `stale` or `inconclusive` | WHEN an exchange rate is stale, the scenario SHALL be marked incomplete before use. |
| CAP-F3-002 | G3 | Approved facts and reporting taxonomy | Machine-readable report | Each tag maps to ledger and reporting policy | Accountant approves; licensed filing remains external | Tags, validation, approval | `taxonomy_error` | WHEN a required fact cannot map, export SHALL fail or name the gap, never invent a value. |
| CAP-F3-003 | G3 | Consented verified financial facts | Selective finance packet | Only approved facts leave; raw books stay with owner | Owner and finance approver | Disclosure manifest and signatures | `denied` | WHEN one lender request is revoked, its binding SHALL stop without deleting shared source records. |
| CAP-F3-004 | G3 | Approved payment or financing contract | Network settlement workflow | Regulated providers move value; every state is reconciled | Multi-party human and provider controls | Contract, approvals, provider events | `counterparty_unknown` | WHEN one network party is offline, no party SHALL report final settlement without required signed states. |

### F4 — frontier network

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| CAP-F4-001 | G4 | Owner-selected ledger facts, report policies, recipient request, purpose, duration | Verifiable selective finance disclosure across independent parties | Raw books, private documents, and unrelated facts stay with the owner; disclosed facts retain ledger and policy links | Entity owner and finance approver choose every fact; recipient cannot expand access | Request, disclosure manifest, fact proofs, recipient use, expiry, revocation | `denied`, `fact_stale`, or `policy_mismatch` | WHEN a recipient asks beyond the approved purpose or period, the network SHALL deny access and record a signed metadata-only event. |
| CAP-F4-002 | G4 | Approved commercial or financing contract, exact amounts/currencies, party approvals, regulated-provider capabilities | Multi-party settlement workflow with reconciled signed states | HelixCapital never takes custody; no party reports final value movement before required provider and book evidence | Each entity approves its obligation; regulated providers move value; agents cannot approve or submit alone | Contract, approvals, provider observations, journals, reconciliations, exceptions | `party_unavailable`, `provider_unknown`, or `unreconciled` | WHEN one provider result is unknown, every party SHALL keep settlement pending and retries SHALL not create a second movement. |
| CAP-F4-003 | G4 | Owner-approved fact subscriptions, close packets, materiality and notification policy | Continuous assurance feed of signed changed facts and open exceptions | It is not an audit opinion, credit score, or autonomous filing; each recipient sees only subscribed facts and known gaps | Entity owner and accountant publish; reviewer interprets; licensed assurance stays external | Fact versions, change proof, exceptions, recipient acknowledgements, revocations | `evidence_gap` or `subscription_revoked` | WHEN a source fact changes or loses reconciliation, the network SHALL mark the prior assurance view stale and notify authorised recipients. |

## 8. Domain model

`AccountingEntity`, `ReportingUnit`, `Book`, `Currency`, `ExchangeRate`,
`FiscalPeriod`, `Account`, `AccountVersion`, `Dimension`, `SourceDocument`,
`Counterparty`, `Journal`, `JournalLine`, `CorrectionLink`, `ApprovalPolicy`,
`Approval`, `Receivable`, `Payable`, `Allocation`, `BankConnection`,
`BankStatement`, `BankTransaction`, `Match`, `Reconciliation`, `CloseTask`,
`CloseCheck`, `ClosePacket`, `Statement`, `StatementFact`, `TaxonomyMapping`,
`Budget`, `ForecastScenario`, `PaymentProposal`, `ProviderObservation`, and
`DisclosurePacket` are explicit records. Posted journals, provider observations,
approvals, and close packets are append-only. Account, policy, mapping, and
forecast meaning use versions. Retention is set by entity, jurisdiction profile,
record class, and legal hold. Generic metadata cannot replace account,
counterparty, tax, currency, due date, approval, or provenance fields.

## 9. System architecture

- A Rust finance kernel owns exact-money types, double-entry validation, account
  lifecycles, posting, allocation, reconciliation, close, and report contracts.
- PostgreSQL is the durable ledger. Object storage holds source documents,
  statements, taxonomies, and signed close binders.
- Domain writes, idempotency results, and outbox events commit in one database
  transaction. Provider observations are append-only.
- Sandboxed durable workers run extraction, import, matching, report generation,
  taxonomy validation, forecasts, exports, and recovery checks.
- Versioned adapters isolate banks, payments, commerce, payroll, tax, and
  reporting regimes. Each adapter states authority, supported operations, and
  semantic loss.
- HelixCore supplies identity, policy, audit, capabilities, jobs, objects,
  billing, operations, notifications, stable project identity, and recovery.
- Offline mode supports preparation and review. Posting or payment while offline
  requires an explicit pre-authorised policy and normally stays queued.

## 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Visible progress, check, stop, reverse |
|---|---|---|---|---|---|
| Bookkeeping agent | Approved sources, chart, policies; extraction and match tools | Journal and allocation drafts | Every posting under policy | Post, create its own approval, or alter source | Shows source, confidence, checks; balanced draft test; discard or correct safely. |
| Reconciliation agent | One account's book and imported statement | Match and exception proposals | Final reconciliation and new journal | Hide difference or create provider evidence | Live matched/difference totals; duplicate tests; cancel keeps prior approved work. |
| Close agent | Period tasks, ledger, checks | Task plan, adjustment drafts, binder | Adjustment, close, reopen | Mark a failed check passed | Streams each check; controller signs; reopen is a separate reversible event. |
| Treasury agent | Approved payables, cash limits, provider capability | Payment batch and cash plan | Beneficiary, batch, money movement | Retrieve secrets, approve, or submit outside exact lease | Shows limits and provider states; dual control; revoke lease stops future calls. |
| Finance analyst | Posted facts and approved scenarios | Forecasts and explanations | Publish or action from forecast | Change the book or label estimate as actual | Shows assumptions, ranges, back-tests; scenario is deletable without book impact. |

Agents receive exact capabilities for one entity, account, purpose, amount,
operation, and time. Sensitive signing is brokered without exporting private
keys. No agent can retrieve credential values.

## 11. Trust, safety, and privacy

Tenant and accounting-entity separation is enforced in database and object
storage. Role, duty separation, amount limit, purpose, account, jurisdiction,
and exact capability apply together. Financial, payroll, banking, identity, tax,
and beneficial-owner data have distinct sensitivity classes. Encryption is
required in transit and at rest; signing and bank credentials remain in the
user-owned broker. Data residency is a deployment policy checked before every
external job.

Delete sends drafts, imports, scenarios, mappings, and allowed attachments to a
recoverable 30-day bin. Posted journals, approvals, reconciliations, statements,
payments, and legally required documents follow retention and legal hold; they
are corrected or access-limited, not erased casually. Permanent deletion is a
separate, re-authenticated, explicit, audited act. Controls address account
takeover, approval collusion, duplicate payment, altered bank files, forged
webhooks, invoice fraud, prompt injection, malicious documents, report
manipulation, and bulk export. Incident recovery freezes risky actions, revokes
leases, rotates bindings, reconciles every external provider, restores signed
state, and tells users which balances remain uncertain.

## 12. Proof and audit

Proof covers source hash and origin, extraction, duplicate check, policy version,
journal and exact lines, preparer, approver, posting time, correction link,
statement import, match decision, reconciliation, close check, ledger root,
statement calculation, taxonomy mapping, payment consent, provider observation,
and later settlement. Independent tools can check signatures, double entry,
idempotency, state transitions, report arithmetic, source links, and disclosed
taxonomy validation. Evidence cannot prove that an invoice was genuine, an
accounting judgement was correct, a bank event was honest, or a report complies
with a jurisdiction unless the proper accountable reviewer says so.

Aether is the preferred proof and capability provider behind neutral interfaces.
A local signer, verifier, lease service, and signed export remain the fallback.

## 13. UX system

Main surfaces are Home, Inbox, Book, Sales, Purchases, Banking, Reconcile, Close,
Reports, Cash, Payments, Evidence, and Recovery. A role-based home shows work,
exceptions, cash freshness, and open decisions. Progressive reveal keeps simple
entry forms calm while exposing lines, currency, tax, dimensions, policy,
provider messages, and signatures when needed. All core tasks target
[WCAG 2.2 Level AA](https://www.w3.org/TR/WCAG22/) with keyboard, touch, screen
reader, zoom, non-colour states, and accessible data tables.

Imports, matching, report builds, close, payment checks, and migrations show
named stages, real counts, last signal, owners, elapsed time, and cancellation.
Completion leaves a durable activity item plus optional device notice. Selected
entries have a visible check and total. Moving a line, match, close task, or
source shows impact before commit; posted entries cannot be dragged into a new
meaning. Draft changes have undo. Delete enters Recovery. Errors state whether
the book, provider, or payment changed and give the safest next step.

## 14. Interoperability and standards

- [ISO 4217](https://www.iso.org/iso-4217-currency-codes.html) defines standard
  currency codes and minor units. Private or non-standard units require explicit
  local definitions and never masquerade as an ISO currency.
- [ISO 20022](https://www.iso20022.org/iso-20022) is an adapter family for
  financial message meaning and schemas. Message sets and external code sets are
  version-pinned; local bank rules may lose or change detail.
- [XBRL](https://www.xbrl.org/the-standard/what/the-standard-for-reporting/) is
  used for machine-readable financial and regulatory reporting. A valid XBRL
  document is not proof that its accounting is correct.
- The [IFRS Accounting Taxonomy](https://www.ifrs.org/issued-standards/ifrs-taxonomy/)
  is an optional, versioned reporting profile for entities that lawfully use
  IFRS. It is never a universal accounting-policy default.
- The [UK Open Banking Read/Write API](https://openbankinguk.github.io/read-write-api-site3/)
  is one jurisdiction adapter for consented account and payment access. It is not
  the global bank interface and cannot shape the canonical ledger.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) sets the accessibility target.

Every import/export previews lost dimensions, currencies, exchange rates, tax
facts, identifiers, approvals, source links, and signatures. Version upgrades
need conformance tests and a rollback path.

## 15. Cross-platform contract

Exact-money, double-entry, posting, reconciliation, close, report, signature,
migration, and recovery fixtures run on Windows, macOS, and Linux. The web client
supports full team preparation, approval, and review. Desktop adds offline work,
local documents, secure broker access, and notifications. The CLI supports
import, validate, reconcile, report, export, backup, and verify. Containers run
self-hosted API and workers. Offline mode clearly blocks or queues actions that
need current balances, approval, or a provider. OS secure storage, notifications,
PDF rendering, and hardware signing use capability detection with file, in-app,
or user-owned broker fallback.

## 16. Reliability and performance budgets

- Acknowledged posted journals, approvals, reconciliations, closes, payment
  observations, and audit events have zero allowed data loss in crash tests.
- Posting 1,000-line journals completes under 500 ms at p95 over a rolling 30-day
  window in the supported managed profile, excluding source upload.
- Trial balance for 10 million journal lines completes under 5 seconds at p95,
  with a clearly visible older snapshot available during rebuild.
- Long work creates a durable stage within 2 seconds and heartbeats no less often
  than every 5 seconds while local work is active.
- Local cancellation is accepted within 2 seconds and stops work within 30
  seconds; external provider work stays `cancel_requested` until confirmed.
- Posting, import, payment, and provider-event commands are idempotent for their
  full legal/provider retry period and never less than 90 days.
- Concurrent close and posting cannot cross a closed period boundary; one action
  wins under a serialised policy and the other receives a clear conflict.
- Offline preparation supports 30 days and 50 GB; posting and payment leases
  expire visibly and cannot be extended by an offline client.
- Managed committed ledger recovery point is zero and recovery time target is 1
  hour; self-hosted documented recovery target is 4 hours.
- Loss of forecast models, Aether, bank feed, notifications, or report taxonomy
  does not block the core book, manual import, local proof, or backup.

## 17. Success measures

- Unbalanced posted journals, duplicate payments, and silent changed entries;
  target zero in each rolling quarter.
- Median days and human hours from period end to a signed close.
- Share of material balance-sheet accounts reconciled on time.
- Share of statement facts traceable to source without an evidence gap.
- Median time to resolve an unknown bank or payment state.
- Forecast error reported by horizon and cash category, never one vanity score.
- Independent close packets that validate on another supported OS.
- Accessibility task success and serious issue count across entry, reconcile,
  close, and report journeys.
- Restore, backup, legal hold, and disaster drill success.
- Retained paying entities and reduced close/review cost, not journal or agent
  call counts.

## 18. Delivery plan

| Gate | Build | Test | Safety | UX | Cross-platform | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Stable entities, exact money, double-entry, atomic ledger/outbox, recovery | Arithmetic, currency, crash, concurrency, signature tests | Duty separation, tenant rules, secret broker, threat model | Honest journal and long-work states | Rust and packaging CI on Windows, macOS, Linux | Dry-run current-account/journal importer | Fresh post, forced crash, restore, and independent verify |
| **G1 — Useful single-player product (6–18 months)** | Sources, journals, receivables, payables, trial balance, statements | Full source-to-statement journeys | Approval, duplicate, export checks | Accessible inbox, book, reports | Web, desktop, CLI, container, offline prep | Chart, opening balance, document migrations | Fresh month book and report on each OS |
| **G2 — Trusted team product (18–30 months)** | Banking, reconciliation, close, payment proposals, roles | Race, feed, dual-control, close tests | External penetration and finance-control review | Reconcile and close rooms with real progress | Provider and degraded-network matrix | Bank/role/policy migration and rollback | Fresh reconciliation, close, incident, recovery drill |
| **G3 — Category leader (30–42 months)** | Explainable statements, forecasts, adapter kit | Scale, back-test, adapter conformance | Model, fraud, jurisdiction boundary review | Evidence graph and uncertainty tests | Mixed deployment and hardware-signing proof | Verified live finance-system migration | External accountant, accessibility, and security review |
| **G4 — Frontier network (42–60 months)** | Build CAP-F4-001 selective disclosures, CAP-F4-002 regulated multi-party settlement, and CAP-F4-003 continuous assurance feeds | Exact-money, disclosure-purpose, replay, double-movement, provider-unknown, reconciliation, stale-fact, partition, and revocation tests | Independent legal, accounting, payments, privacy, security, and ethics review; no custody, credit decision, or audit opinion | Owner-controlled disclose, approve, pending/unknown, reconcile, assurance-gap, revoke, and exit journeys | Mixed Windows/macOS/Linux entity and reviewer nodes prove local books, provider degradation, signed facts, and offline denial | Add/remove a recipient or provider, revoke exact bindings, and preserve book identity plus shared obligations | Independent disclose, settle in provider sandbox, reconcile, mark stale, revoke, remove-node, disaster-recover, and verify exercise covering all F4 evidence |

Each gate closes only from fresh release-candidate checks. A skipped bank sandbox,
stale close binder, or structurally valid but untraced report does not pass.

## 19. Current truth and gap

The live Rust source has real accounts and double-entry journals. It rejects an
unbalanced journal and uses a database transaction to write lines and update
balances. That is a meaningful finance prototype. It has no currency-compatibility
rule across accounts and journal lines, source documents, correction model,
reconciliation, close, statements, approvals, bank feeds, treasury controls,
product UI, or domain test suite. The service shares the repository's current
application-state compile failure. Audit and billing work is not atomic with the
domain transaction.

The most important gap is a ledger that is balanced but not yet fully truthful
about currency, approval, correction, and evidence. The safest first slice is
CAP-F0-001 through CAP-F1-002: exact same-currency posting, immutable correction,
separate prepare/approve roles, one source document, atomic event, and forced
crash recovery. All tests must use temporary state and non-production credentials.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Identity | Stable entity, account, source, journal, and period IDs independent of paths/providers | Architecture decision and migration proof |
| Money | Exact decimal or integer minor units with explicit ISO currency; no float | Finance architecture review |
| Ledger | Append-only posted journals; corrections are linked new journals | Founder and accounting architecture decision |
| Write integrity | Journal, lines, idempotency, balance updates, and outbox commit together | Founder-approved integrity exception |
| Approval | Preparer and approver duties follow policy; agents cannot fill either final authority alone | Finance-control review |
| Money movement | Regulated provider plus human control; disabled by default | Founder, legal, security, and provider approval |
| Reporting rules | Versioned jurisdiction and taxonomy adapters; no universal IFRS default | Licensed accounting review |
| Proof provider | Aether preferred through neutral interface; local fallback mandatory | Provider-neutrality review |
| Delete | 30-day bin for allowed drafts; posted/retained records use correction, hold, and policy | Legal-retention decision |
| Accessibility | WCAG 2.2 AA target including complex tables and non-colour states | Accessibility review |
| First slice | Source → approved exact journal → atomic event → crash recovery | Product decision with equal integrity proof |
| Lending, investing, custody, autonomous tax filing | Disabled and separate founder/legal feature gates | Founder approval |

## 21. Definition of category-defining done

- [ ] All seven finance journeys run with real source and provider failure cases.
- [ ] Every posted journal balances by currency and is append-only.
- [ ] Every reported fact traces to source, policy, approval, and reconciliation.
- [ ] Agents cannot post, self-approve, close, retrieve secrets, or move money.
- [ ] Retries, crashes, races, and out-of-order events cannot duplicate value.
- [ ] Independent close and disclosure packets validate without the live server.
- [ ] Jurisdiction standards stay versioned adapters with honest mapping losses.
- [ ] Forecasts stay separate from facts and show uncertainty and back-tests.
- [ ] WCAG 2.2 AA scope passes for tables, forms, review, and evidence graphs.
- [ ] Windows, macOS, Linux, web, offline, CLI, and container limits are proven.
- [ ] The 30-day bin, correction, legal hold, backup, restore, and incident drills work.
- [ ] External security, privacy, accounting, legal, and accessibility reviews close.
- [ ] The product states what financial proof does not establish or assure.
