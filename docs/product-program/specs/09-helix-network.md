# HelixNetwork — a consent-first opportunity network

```yaml
product: HelixNetwork
catalog_order: 9
status: target-state-spec
horizon: 60 months
current_maturity: prototype
primary_users: [professionals, creators, teams, communities, opportunity owners, trust reviewers]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

> **Target-state rule:** Sections 1–18 and 20–21 define the intended product.
> Section 19 is the current live-source assessment.

## 1. Category claim

HelixNetwork is a user-owned opportunity network that helps people form trusted
relationships and complete real work through consent, explainable matching, and
portable proof instead of engagement pressure.

## 2. Five-year destination

The useful product provides a private or public profile, connections,
opportunities, matching, proposals, messages, collaboration handoff, moderation,
and portable reputation. The category-defining advantage is an intent graph:
people state what they seek, offer, permit, and have proved, while every match
explains why it exists and what information was used. The frontier capability is
a federation of independent communities where identity, relationships, and
verified contributions move under user control. Humans keep authority over
profile claims, connection, disclosure, messages sent in their name, opportunity
selection, moderation outcomes, reputation, and entry into any federation.

## 3. Users and hard jobs

- **Professionals and creators** need relevant people and work without building
  life on one platform. They fear spam, ranking manipulation, and lock-in.
- **Teams and opportunity owners** need credible candidates and fair review.
  They fear false claims, bias, and automated rejection with no appeal.
- **Communities** need healthy membership and local rules. They fear harassment,
  abuse waves, and a central network overruling their values.
- **New or less visible people** need a path based on fit and proof, not follower
  count. They fear popularity becoming the only signal.
- **Trust and moderation reviewers** need context, evidence, proportional action,
  and appeal. They fear both unchecked harm and opaque punishment.

## 4. Product laws

1. Connection and disclosure require consent; a follow is not ownership.
2. The product optimises for useful outcomes, not time, outrage, or infinite feed.
3. Matches show reasons, used fields, paid influence, uncertainty, and controls.
4. Agents may draft outreach but never send, connect, or apply without approval.
5. Reputation is plural, contextual, evidence-linked, and appealable.
6. Private fields, contacts, and social graphs are denied by default.
7. Communities can set rules but cannot silently rewrite a person's identity.
8. Users can export and leave without losing their canonical identity or proof.
9. Federation and imports treat remote content as untrusted input.
10. Slow matching, delivery, verification, and moderation show real progress.

## 5. Scope boundaries

HelixNetwork owns profiles, intent, relationships, opportunities, matches,
proposals, consented messages, community membership, contextual reputation,
moderation, federation, and network export. HelixCollab owns deep project
collaboration. HelixEdu owns learning and credentials. HelixCommerce owns orders
and marketplace transactions. HelixCore owns identity, policy, audit,
capabilities, jobs, objects, billing, and operations.

Network is not a public court, universal identity authority, hiring decision
system, credit score, background-check service, or advertising-surveillance
feed. Agents and rankings cannot make employment, access, safety, legal, or other
high-stakes decisions. Accountable humans and lawful processes retain authority.

## 6. Signature experiences

1. **Create a portable profile.** **Entry:** a person starts private and chooses
   what to publish. **Visible progress:** identity, fields, evidence, audience,
   accessibility, and preview checks show. **Human decision:** the person
   approves every public or community view. **Completion proof:** a signed profile
   version records claims and sources. **Failure and recovery:** failed imports
   preserve a draft and loss report. **Export:** profile, evidence links, and
   audience rules are portable.
2. **Form a consented connection.** **Entry:** one person requests a named
   relationship for a purpose. **Visible progress:** requested scope, message,
   expiry, and response state are clear. **Human decision:** the recipient
   accepts, narrows, declines, blocks, or reports. **Completion proof:** both
   sides receive the same signed relationship state. **Failure and recovery:**
   duplicate or crossed requests settle once without auto-accept. **Export:**
   each person can export their side of the relationship.
3. **Publish an opportunity.** **Entry:** an owner creates a job, project, event,
   grant, request, or collaboration. **Visible progress:** requirements, pay or
   exchange, dates, audience, policy, and fairness checks show. **Human decision:**
   the owner approves publication and reviewers. **Completion proof:** the exact
   opportunity version remains attached to every proposal. **Failure and
   recovery:** channel failures can retry without duplicate posts. **Export:**
   opportunity and responses are portable.
4. **Find a meaningful match.** **Entry:** a user asks for people or opportunities
   under chosen filters. **Visible progress:** sources, stages, excluded fields,
   and match reasons are visible. **Human decision:** the user saves, dismisses,
   contacts, or changes controls. **Completion proof:** each match records its
   reasons and model/rule version. **Failure and recovery:** weak evidence returns
   no match or low confidence, not a false best choice. **Export:** the user can
   export matches and explanations.
5. **Reach out without spam.** **Entry:** a person opens one profile or match.
   **Visible progress:** recipient policy, shared context, draft, delivery, and
   read state are separate. **Human decision:** the sender approves the exact
   message and the recipient controls reply or block. **Completion proof:** sent
   content and consent state are linked. **Failure and recovery:** delivery retry
   cannot send twice. **Export:** participants can export their conversation.
6. **Prove relevant work.** **Entry:** a person attaches selected evidence or a
   credential to a claim or proposal. **Visible progress:** source, signature,
   status, disclosure, and verification checks show. **Human decision:** the
   person chooses each claim; the receiver decides what weight it has.
   **Completion proof:** an independent verification report records limits.
   **Failure and recovery:** invalid or revoked proof is marked, not erased.
   **Export:** proof remains usable outside HelixNetwork.
7. **Handle harm with due process.** **Entry:** a user blocks, mutes, or reports
   content or conduct. **Visible progress:** immediate protection, review queue,
   evidence, policy, decision, notice, and appeal are separate. **Human decision:**
   trained people own serious sanctions and appeals. **Completion proof:** policy
   version, evidence, action, reasons, and reviewer are recorded. **Failure and
   recovery:** urgent protection works even if review is delayed. **Export:** a
   user can obtain their report and decision where safe and lawful.
8. **Move or federate a community.** **Entry:** an owner plans export, import, or
   federation with another node. **Visible progress:** identities, relationships,
   opportunities, media, policies, blocks, and verification are counted.
   **Human decision:** users choose which bindings and public content move.
   **Completion proof:** source and destination hashes, totals, and sample
   journeys reconcile. **Failure and recovery:** rollback leaves old identities
   and blocks working. **Export:** leaving a node does not delete shared proof or
   other users' data.

## 7. Capability map

### F0 — foundation

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| NET-F0-001 | G0 | Person or organisation and recovery method | Stable network identity | Identity is path, host, and vendor independent | Subject owns; organisation delegates roles | Creation and binding events | `locked` | WHEN an account moves host, the canonical identity SHALL remain and old bindings can be revoked. |
| NET-F0-002 | G0 | Field, audience, purpose, expiry | Disclosure policy | Deny by default; private field cannot leak through search or match | Subject grants | Grant, use, denial, revocation | `denied` | WHEN a field is private, search, export, and matching SHALL not reveal or infer it without a grant. |
| NET-F0-003 | G0 | Relationship request and response | Signed relationship state | Acceptance is explicit; crossed requests do not auto-connect | Recipient decides | Requests, response, state | `declined`, `blocked`, `expired` | WHEN two requests cross, neither SHALL count as acceptance without one explicit response. |
| NET-F0-004 | G0 | Domain command and idempotency key | Record plus event | Record, outbox, and replay result commit together | Domain policy | Transaction and replay proof | `not_committed` | WHEN a crash occurs during connect or publish, recovery SHALL show zero or one complete action. |

### F1 — useful product

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| NET-F1-001 | G1 | Claims, evidence, field audiences | Versioned profile views | Audience views are generated from one canonical profile | Subject publishes | Preview, version, source links | `draft` or `invalid` | WHEN a field audience narrows, every wider cached view SHALL be invalidated before success. |
| NET-F1-002 | G1 | Opportunity facts, terms, reviewers | Opportunity version | Terms seen by an applicant are immutable | Owner approves | Version, fairness checks, publish result | `draft` or `closed` | WHEN terms change, existing proposals SHALL retain the prior version and receive notice. |
| NET-F1-003 | G1 | User intent and allowed profile fields | Explained match | Protected/private fields are excluded; paid influence is separate | User chooses controls | Inputs, reasons, rule/model version | `low_confidence` | WHEN a match is shown, the user SHALL see at least one reason and all used field classes. |
| NET-F1-004 | G1 | Approved sender, recipient policy, message | One message and delivery timeline | Agent draft cannot send; retry cannot duplicate | Sender approves; recipient controls inbox | Content hash, approval, delivery states | `blocked` or `delivery_unknown` | WHEN delivery acknowledgement is lost, retry SHALL not create a second visible message. |

### F2 — category leader

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| NET-F2-001 | G2 | Opportunity and selected profile/evidence | Proposal lifecycle | Only user-selected fields leave; opportunity version is pinned | Applicant submits; owner reviews | Disclosure, proposal, decisions | `withdrawn` or `needs_review` | WHEN a proposal is withdrawn, future review SHALL stop while the audit record remains. |
| NET-F2-002 | G2 | Claim and portable evidence | Verification report | Invalid proof stays visible as invalid; verifier weight is separate | Subject discloses; receiver judges | Signature/status checks and limits | `invalid` or `revoked` | WHEN a credential is revoked, the profile SHALL update status without deleting the historic claim. |
| NET-F2-003 | G2 | Context, evidence, policy version | Moderation case and action | Block/mute is immediate; serious sanction has human review and appeal | User protects self; trained moderator sanctions | Evidence, policy, reason, appeal | `review_delayed` or `appealed` | WHEN a user blocks another, new direct delivery SHALL stop before moderation completes. |
| NET-F2-004 | G3 | Contribution context and verified outcome | Contextual reputation statement | No universal score; source and right of reply are required | Contributor consents; issuer signs | Outcome, issuer, expiry, dispute | `disputed` | WHEN reputation is displayed, the system SHALL show context, issuer, date, and dispute state. |

### F3 — advanced category leadership

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| NET-F3-001 | G3 | Community rules, membership, local identity | Governed community space | Community policy cannot alter canonical person identity | Community admins; user accepts rules | Policy, membership, actions | `suspended` or `left` | WHEN a user leaves, community access ends without deleting the user's external identity or proof. |
| NET-F3-002 | G3 | Node capabilities and user-approved objects | Federated delivery | Remote objects are untrusted, signed where available, and policy checked | Node owner and each subject | Delivery, validation, denial | `remote_untrusted` | WHEN a remote object fails schema or policy, it SHALL be quarantined and not rendered as trusted content. |
| NET-F3-003 | G3 | Portable identifier, claims, relationships | Host migration | No host owns canonical identity; relationship consent is preserved or re-requested | Subject approves | Move manifest, redirects, revocations | `partial_move` | WHEN migration cannot preserve consent semantics, the affected relationship SHALL require new consent. |
| NET-F3-004 | G3 | Cross-node intent and allowed proof | Federated opportunity match | Raw private graph stays with custodian; reasons remain explainable | Users and nodes grant exact purpose | Query, grants, match proof | `insufficient_evidence` | WHEN a node revokes matching access, new matches SHALL stop without deleting other nodes' shared objects. |

### F4 — frontier network

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| NET-F4-001 | G4 | User-controlled intent, exact private-field grants, node capability manifests, approved opportunities | Privacy-preserving cross-node match with local reasons and no central raw graph | Private graph and protected traits remain local; every result is explainable; paid influence is separate | Each user chooses fields and action; each node approves query purpose; no agent contacts or applies | Grants, local query proofs, used field classes, reasons, paid status, result digest | `insufficient_match`, `privacy_floor_failed`, or `node_untrusted` | WHEN a node cannot prove allowed fields and a human-readable reason, the network SHALL exclude its result and record the denial. |
| NET-F4-002 | G4 | Stable identity, user-selected relationships, contextual reputation, blocks, proof, destination policy | Portable continuity bundle and verified host transition | No host owns identity; blocks and consent are preserved or re-requested; reputation never becomes one universal score | Subject approves every moved object; counterparties approve any consent that cannot transfer; issuers control their statements | Move manifest, relationship decisions, proof status, block tests, redirects, revocations | `partial_move`, `consent_required`, or `issuer_unavailable` | WHEN a host leaves or fails, the subject SHALL recover identity and proof while every uncertain relationship stays disconnected until new consent. |
| NET-F4-003 | G4 | Community policies, shared opportunity contract, selected member proof, moderation and appeal agreements | Governed multi-community opportunity consortium with signed outcomes | No central feed or authority overrides local policy; sanctions stay contextual; participation and exit remain voluntary | Each community approves policy and reviewers; users approve disclosure and proposal; humans decide moderation | Policy versions, grants, proposals, reviewer decisions, outcomes, appeals, exit events | `policy_conflict`, `review_blocked`, or `consortium_partitioned` | WHEN community policies conflict on a required action, the consortium SHALL pause that action and show the conflict rather than choose silently. |

## 8. Domain model

`NetworkIdentity`, `Person`, `Organisation`, `Profile`, `ProfileVersion`,
`ProfileField`, `Audience`, `Intent`, `Offer`, `Claim`, `EvidenceReference`,
`RelationshipType`, `ConnectionRequest`, `Relationship`, `Block`, `Mute`,
`Opportunity`, `OpportunityVersion`, `Match`, `MatchReason`, `Proposal`,
`Conversation`, `Message`, `DeliveryObservation`, `Community`, `Membership`,
`CommunityPolicy`, `ReputationStatement`, `VerificationReport`, `Report`,
`ModerationCase`, `Action`, `Appeal`, `FederationNode`, `RemoteObject`, and
`MigrationManifest` are explicit. Profile and opportunity meaning use immutable
versions. Relationships require both parties and state. Reputation has issuer,
context, evidence, expiry, and dispute. Personal data has subject, audience,
purpose, retention, and residency. Generic metadata does not replace these rules.

## 9. System architecture

- A Rust network engine validates identity bindings, profile views,
  relationships, opportunity, proposal, moderation, and federation lifecycles.
- Application services handle search, explainable matching, messaging,
  verification, community governance, export, import, and recovery.
- PostgreSQL stores durable network state; object storage holds media and proof;
  an embedded local profile supports private drafts and portable identity.
- A transactional outbox commits domain state and events together. Delivery is
  retried with idempotent inbox and outbox records.
- Search and matching indexes are rebuildable projections, never authority for
  profile, consent, or relationship truth.
- Versioned adapters isolate credential systems, messaging, collaboration,
  moderation support, and federation protocols.
- HelixCore supplies identity primitives, policy, audit, capabilities, jobs,
  objects, billing, operations, notifications, stable project identity, and recovery.
- Offline mode supports drafts, cached approved objects, local proof, blocks,
  and queued actions; current consent is required before external delivery.

## 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Visible progress, check, stop, reverse |
|---|---|---|---|---|---|
| Profile agent | Subject-approved facts and evidence; writing/accessibility tools | Profile text and audience previews | Every publish or disclosure | Invent claim, widen audience, or scrape contacts | Shows source per claim; subject edits/discards; version rollback works. |
| Match agent | User intent and allowed indexed fields | Match list and explanation | Contact, apply, or share more data | Use protected/private traits or optimise engagement | Shows fields/reasons and uncertainty; user can dismiss/reset; model can be disabled. |
| Outreach agent | One approved match and recipient policy | Message or proposal draft | Every send and application | Send bulk messages, connect, or impersonate | Streams checks and delivery after approval; revoke stops future work; duplicate test runs. |
| Trust assistant | Case evidence and current policy | Case summary and action options | Serious sanction and appeal decision | Decide guilt, reveal reporter, or change policy | Shows evidence gaps and conflicts; human signs; appeal can reverse action by new event. |
| Federation agent | Approved objects and exact remote capability | Delivery plan and mapping | New node, new data class, broad backfill | Retrieve secrets or trust remote input by default | Shows object counts and denials; quarantine checks; cancel stops queue safely. |

Agents use exact, time-limited leases. They can see capability metadata and ask
for access, but cannot retrieve secret values or approve their own request.

## 11. Trust, safety, and privacy

Access combines tenant, subject, audience, relationship, purpose, resource,
field, community, time, and exact capability. The database, object store, search
index, caches, and exports apply the same policy. Sensitive identity, contact,
location, social graph, message, block, report, protected-trait, and child data
use separate classes. Encryption is required in transit and at rest. Public
profile fields are explicit and previewed. Data residency and federation region
are policy choices.

Delete moves drafts, messages where policy allows, opportunities, profile
versions, and community objects to a recoverable 30-day bin. Shared messages,
moderation evidence, legal holds, and another person's records follow stated
retention and cannot be silently removed from their custody. Permanent deletion
is separate, re-authenticated, explicit, and audited. Controls cover stalking,
doxxing, spam, grooming, impersonation, coordinated abuse, discriminatory
matching, scraping, malicious links, prompt injection, fake credentials,
federation floods, and moderator misuse. Incident recovery can block delivery,
revoke nodes and leases, preserve appeals, rotate bindings, rebuild projections,
and notify users without exposing reporters.

## 12. Proof and audit

Proof covers identity binding, profile and opportunity versions, disclosure,
connection request and response, match fields and reason, agent draft and human
send approval, delivery state, evidence verification, community policy,
moderation evidence, action, appeal, federation validation, and migration. An
independent verifier can check signatures, object hashes, consent timing,
credential status, state transitions, and disclosed policy. Proof cannot show
that a person is honest, a match is fair in every context, a reputation claim is
true beyond its issuer, or moderation was morally correct.

Aether is the preferred proof and capability provider through neutral
interfaces. A local signer, verifier, capability lease service, and portable
audit bundle keep the network usable and reviewable without Aether.

## 13. UX system

Main surfaces are Home, Intent, Discover, Connections, Opportunities, Messages,
Communities, Proof, Trust, Settings, and Recovery. Home is finite and task-based,
not an infinite engagement feed. Every match shows Why this, used fields, paid
status, controls, and hide/report actions. Progressive reveal keeps profile and
opportunity work simple while exposing consent, proof, policy, ranking, delivery,
and federation details when asked. The product targets
[WCAG 2.2 Level AA](https://www.w3.org/TR/WCAG22/) across keyboard, touch, screen
reader, zoom, reduced motion, and non-colour states.

Search, matching, verification, migration, federation delivery, and moderation
show real stages, counts, queue, last signal, and cancel or pause. Completion
leaves a durable activity item and optional desktop notice. Selected profiles or
opportunities have clear checks. Moving content between audiences, communities,
or proposals previews who gains access and asks for consent. Safe drafts have
undo; delete uses Recovery. Empty states help the user state intent, not follow
random accounts. Errors preserve drafts, say whether anything was sent, and
offer block, retry, appeal, or human support as appropriate.

## 14. Interoperability and standards

- [W3C ActivityPub](https://www.w3.org/TR/activitypub/) is a federation adapter
  for client-to-server and server-to-server social activities based on
  ActivityStreams 2.0. Its broad object model does not carry every Helix consent,
  proof, moderation, or opportunity rule.
- [IETF WebFinger RFC 7033](https://www.rfc-editor.org/info/rfc7033/) discovers
  information about an account or other URI over HTTPS. Published discovery data
  is easy to correlate, so only minimum public links are returned.
- [W3C DID Core 1.0](https://www.w3.org/TR/did-core/) is an optional adapter for
  portable identifiers and verification methods. A DID does not prove a real
  identity or replace recovery, consent, or moderation.
- [W3C Verifiable Credentials Data Model 2.0](https://www.w3.org/TR/vc-data-model-2.0/)
  supports portable claims. Selective disclosure, issuer trust, status, and
  privacy remain explicit product concerns.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) sets the accessibility target.

Standards are versioned adapters. Import previews lost audience, relationship,
block, moderation, signature, proof-status, rich-content, and extension data.
Remote HTML and links are sanitised; remote signatures do not create local trust.

## 15. Cross-platform contract

Identity, consent, relationship, opportunity, matching reason, proof, moderation,
migration, and recovery fixtures run on Windows, macOS, and Linux. Browser mode
supports full connected use. Desktop adds private local drafts, offline cache,
user-owned broker, files, and notifications. The CLI supports import, export,
verify, block, node health, and recovery, not all social interaction. Containers
support self-hosted nodes and workers. Offline mode supports reading approved
cache, drafting, blocking, and queued actions; sending waits for fresh policy and
consent. Secure storage, notifications, camera, contacts, and deep links use
capability detection with manual file, in-app, or browser fallback.

## 16. Reliability and performance budgets

- Acknowledged identity, disclosure, relationship, block, proposal, moderation,
  and event writes have zero allowed data loss in forced-crash tests.
- Profile and opportunity reads complete under 300 ms at p95 over a rolling
  30-day window for a node with 10 million active objects.
- Explainable top-100 matching completes under 2 seconds at p95 for the supported
  managed profile and names stale-index age.
- Long work creates a durable stage within 2 seconds and has a local heartbeat
  no older than 5 seconds while active.
- Local cancellation is accepted within 2 seconds and stops work within 30
  seconds; remote delivery remains `cancel_requested` until confirmed.
- Message, proposal, relationship, federation, and moderation commands are
  idempotent for at least 30 days.
- Block takes effect for local new delivery within 2 seconds; reachable remote
  delivery receives revocation within 60 seconds and otherwise shows pending.
- Offline cache supports 30 days or 50 GB; private cache and expiry are visible.
- Managed committed metadata has recovery point zero and 1-hour recovery time;
  self-hosted documented recovery target is 4 hours.
- If models, Aether, search index, federation, or notifications fail, direct
  profile, known connections, blocks, local proof, export, and recovery continue.

## 17. Success measures

- Share of matches users rate relevant after seeing and controlling the reasons.
- Useful outcomes completed: accepted collaborations, filled opportunities, and
  verified contributions, not time or feed views.
- Unapproved agent messages, connections, applications, or disclosures; target zero.
- Spam, harassment, block-bypass, and duplicate-delivery incidents per quarter.
- Median time from serious report to protection, decision, and resolved appeal.
- Profiles, opportunities, relationships, and proof moved to another node with
  named loss and no canonical identity change.
- Accessibility task success and serious issue counts across profile, matching,
  messaging, opportunity, and trust flows.
- Independent proof, 30-day restore, node revoke, and disaster drill success.
- Sustainable paid communities and opportunity tools without surveillance ads,
  not follower, message, or agent-call counts.

## 18. Delivery plan

| Gate | Build | Test | Safety | UX | Cross-platform | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Stable identity, disclosure, relationship state, atomic event ledger, blocks, recovery | Consent, cross-request, crash, idempotency, signature tests | Tenant policy, abuse threat model, secret broker | Private profile, clear states, Recovery | Rust and packaging CI on Windows, macOS, Linux | Dry-run current profile/connection/opportunity importer | Fresh connect, block, crash, restore, verify |
| **G1 — Useful single-player product (6–18 months)** | Profiles, opportunities, explainable match, messages, proposals | Complete opportunity and failure journeys | Spam, protected-field, export checks | Finite accessible discovery and inbox | Web, desktop, CLI, container, offline drafts | Identity/profile mappings and rollback | Fresh real opportunity path on each OS |
| **G2 — Trusted team product (18–30 months)** | Communities, verification, moderation, appeals, roles | Abuse, race, permission, appeal tests | External trust/safety and privacy review | Protection, review queue, notices | Degraded network and device matrix | Community/policy migrations | Fresh abuse, appeal, incident, and recovery drill |
| **G3 — Category leader (30–42 months)** | Contextual reputation, portable profiles, adapter kit | Fairness, explanation, scale, conformance | Ranking, bias, moderator-power review | Intent and proof comprehension tests | Mixed deployment and portable-proof matrix | Verified move between two deployments | External user, accessibility, security review |
| **G4 — Frontier network (42–60 months)** | Build NET-F4-001 private cross-node matching, NET-F4-002 portable continuity, and NET-F4-003 governed opportunity consortia | Protected-field, paid-ranking, replay, block-bypass, consent, migration, policy-conflict, appeal, partition, revoke, and malicious-node tests | Independent federation, privacy, trust/safety, child-safety, fairness, competition, security, and legal review | User-controlled match, explain, disclose, block, migrate, appeal, consortium conflict, leave, and recover journeys | Mixed Windows/macOS/Linux nodes plus offline clients prove local graphs, block enforcement, signed delivery, and safe degradation | Remove/move a node or community, revoke selected relationships, and preserve identity, blocks, proof, and other communities | Independent match, connect, block, migrate, govern, appeal, partition, exit, disaster-recover, and verify exercise covering all F4 evidence |

Every gate needs fresh release-candidate proof. A federation test with only one
trusted node or a ranking review without protected-field checks cannot close it.

## 19. Current truth and gap

The live Rust source has real profiles, connection request and accept actions,
and opportunities. Connection acceptance checks that the caller owns the
receiving profile. This is a meaningful early backend prototype. It has no
stable portable identity model, field audience policy, search, matching,
explanations, messaging, proposals, verification, moderation, reputation,
federation, product UI, or domain test suite. The service has the repository's
shared application-state compile problem. Domain changes and audit or billing
events are not one atomic operation.

The most important gap is consent and privacy across profile fields and
relationships. The safest first slice is NET-F0-001 through NET-F1-001: stable
identity, private-by-default field policy enforced in database and search
projection, explicit connection acceptance, block, profile preview, export, and
forced-crash recovery. Use temporary test state only.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Identity | Stable network identity independent of folder, host, provider, and handle | Architecture decision and migration proof |
| Privacy | Field-level audience and purpose, deny by default, enforced in storage/index/export | Independent privacy review |
| Relationship | Explicit recipient acceptance; crossed requests never auto-connect | Product and trust review |
| Product goal | Useful outcomes and consent, not engagement or infinite feed | Founder decision |
| Matching | Explain reasons, used field classes, paid influence, uncertainty, and controls | Fairness review |
| Agent authority | Draft only; every send, connect, proposal, and disclosure needs human approval | Safety review |
| Proof provider | Aether preferred through neutral interface; local fallback required | Provider-neutrality review |
| Secrets | User-owned capability broker; agents never receive values | Security review |
| Delete | 30-day bin with shared-record, moderation, and legal-hold limits | Legal/trust decision |
| Standards | ActivityPub, WebFinger, DID, and VC behind versioned adapters | Federation review |
| First slice | Private profile → explicit connection → block → export → crash recovery | Product decision with equal privacy proof |
| Federation, hiring ranking, advertising | Disabled until separate G4/founder/ethics gates | Founder approval |

## 21. Definition of category-defining done

- [ ] People complete all eight journeys without hidden disclosure or automation.
- [ ] Profiles, relationships, matches, and reputation show consent and context.
- [ ] No agent sends, connects, applies, discloses, moderates, or self-approves.
- [ ] Matching is explainable, controllable, paid influence labelled, and reviewed.
- [ ] Blocks protect immediately and serious moderation supports evidence and appeal.
- [ ] Atomic writes, retries, partitions, and crash recovery lose no acknowledged state.
- [ ] Independent proof and credentials verify without the live node or Aether.
- [ ] Users move hosts and revoke bindings without losing canonical identity.
- [ ] WCAG 2.2 AA scope passes all major social and trust journeys.
- [ ] Windows, macOS, Linux, web, offline, CLI, container, and federation limits are proven.
- [ ] The 30-day bin, shared-record retention, legal hold, restore, and exit work.
- [ ] External security, privacy, trust/safety, fairness, legal, and accessibility reviews close.
- [ ] The product states what identity, proof, matching, and reputation do not prove.
