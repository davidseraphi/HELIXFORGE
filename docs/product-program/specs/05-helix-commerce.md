# HelixCommerce — sovereign commerce operations

```yaml
product: HelixCommerce
catalog_order: 5
status: target-state-spec
horizon: 60 months
current_maturity: prototype
primary_users: [independent merchants, commerce teams, buyers, fulfilment operators, finance reviewers]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

> **Target-state rule:** Sections 1–18 and 20–21 define the planned product.
> Section 19 is the only statement of current implementation truth.

## 1. Category claim

HelixCommerce is a merchant-owned commerce operating system that keeps catalog,
orders, customer promises, money movement, and proof portable across channels
and payment or fulfilment providers.

## 2. Five-year destination

The useful product is a complete catalog, inventory, checkout, order, payment,
fulfilment, return, and customer-service workspace for one merchant. The
category-defining advantage is a single verifiable promise ledger: what was
offered, what the buyer accepted, what stock and money were reserved, what was
delivered, and how any problem was resolved. The frontier capability is a
federated market where independent merchants can share discovery and logistics
without giving one platform custody of their business identity or customer
history. Humans keep authority over prices, promotions, refunds, disputes,
restricted products, supplier terms, and movement of money.

## 3. Users and hard jobs

- **Independent merchants** need to sell through several channels without
  losing ownership of their catalog and customers. They fear platform lock-in.
- **Commerce teams** need correct stock, price, tax, and order state. They fear
  overselling or charging the wrong amount.
- **Buyers** need a clear promise, safe payment, and honest recovery. They fear
  hidden fees, dark patterns, and lost refunds.
- **Fulfilment operators** need pick, pack, ship, collect, and return work with
  real status. They fear silent state changes and duplicate work.
- **Finance and compliance reviewers** need exact money and approval records.
  They fear a report that cannot be tied back to orders and provider events.

## 4. Product laws

1. An accepted order preserves the exact offer, currency, tax, and terms seen.
2. Money uses exact decimal or integer minor units, never floating-point values.
3. Different currencies are never added without an explicit exchange record.
4. Inventory reservation and order acceptance commit atomically.
5. Payment provider state is evidence, not the internal order state itself.
6. No agent may change a live price, issue money, or accept a dispute alone.
7. Buyers never face a preselected purchase, hidden fee, or fake urgency signal.
8. Merchants can export their products, orders, customers, and proof at any time.
9. Every slow payment, fulfilment, or import shows real progress and last signal.
10. A failed or uncertain external call is never shown as success.

## 5. Scope boundaries

HelixCommerce owns sellable products, offers, channels, inventory, carts,
checkout, orders, payment intents, fulfilment, returns, refunds, disputes,
customers, and service cases. HelixCore owns identity, policy, audit,
capabilities, jobs, objects, billing, and operations. HelixCapital owns the
accounting book and financial statements. HelixInsights owns general analytics.
External regulated providers move money, calculate jurisdiction-specific tax,
or carry parcels through adapters.

Commerce is not a bank, card network, tax authority, customs broker, or legal
adviser. It does not hold funds unless a separately licensed deployment is
approved. Agents and forecasts do not replace merchant, finance, legal, or
regulated-provider decisions.

## 6. Signature experiences

1. **Publish a product.** **Entry:** a merchant creates or imports a product.
   **Visible progress:** media, identifier, price, stock, policy, and channel
   checks show separately. **Human decision:** a merchant approves the offer and
   publish time. **Completion proof:** the signed product and offer version lists
   every channel result. **Failure and recovery:** failed channels stay failed
   and can retry without duplicate offers. **Export:** product, media, offer,
   identifiers, and mappings are portable.
2. **Buy with a clear promise.** **Entry:** a buyer opens a product or shared
   cart link. **Visible progress:** price, tax estimate, delivery, stock, and each
   checkout stage remain visible. **Human decision:** the buyer explicitly
   accepts the final total and terms. **Completion proof:** a receipt records the
   accepted offer and provider references. **Failure and recovery:** uncertain
   payment enters `checking`, never `failed` or `paid` by guess. **Export:** the
   buyer can download receipt, terms, and status history.
3. **Reserve scarce stock.** **Entry:** checkout requests a reservation.
   **Visible progress:** held quantity and expiry are shown to buyer and operator.
   **Human decision:** the merchant may approve an exception, never hidden
   oversell. **Completion proof:** reservation and order event share one commit.
   **Failure and recovery:** expired or lost reservations release exactly once.
   **Export:** the inventory movement ledger is portable.
4. **Fulfil an order.** **Entry:** an operator opens the work queue. **Visible
   progress:** pick, pack, label, handoff, delivery, and exceptions show real
   timestamps. **Human decision:** substitutions and address changes require
   approval. **Completion proof:** item, parcel, actor, and carrier evidence link
   to the order. **Failure and recovery:** a carrier outage leaves a retryable
   handoff, not a shipped state. **Export:** packing slip, shipment, and proof use
   standard or documented formats.
5. **Return and refund fairly.** **Entry:** a buyer or operator opens a return
   request. **Visible progress:** eligibility, evidence, return travel, inspection,
   and refund stages are shown. **Human decision:** policy exceptions and money
   release follow thresholds and approvals. **Completion proof:** reason,
   approval, provider result, and inventory result are linked. **Failure and
   recovery:** a stuck refund is reconciled without issuing a duplicate.
   **Export:** the case and refund proof are downloadable.
6. **Reconcile money.** **Entry:** finance imports provider settlements.
   **Visible progress:** matches, fees, currency, missing events, and differences
   update live. **Human decision:** a reviewer approves exceptions and posting to
   HelixCapital. **Completion proof:** every line maps to orders and provider
   evidence. **Failure and recovery:** unmatched lines remain open and rerunnable.
   **Export:** settlement, mapping, and reconciliation report are portable.
7. **Move to another provider or deployment.** **Entry:** an owner starts a
   portability plan. **Visible progress:** records, media, secrets bindings,
   redirects, and validation are counted. **Human decision:** the owner approves
   cutover and rollback window. **Completion proof:** source and destination
   totals, hashes, and sample journeys match. **Failure and recovery:** the old
   path remains available until verified cutover. **Export:** no proprietary
   provider identifier is the only identity for a business record.

## 7. Capability map

### F0 — foundation

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| COM-F0-001 | G0 | Product facts and identifiers | Versioned product | Stable ID is provider and path independent | Merchant owns; reviewer may verify | Actor, hash, version | `draft` or `invalid` | WHEN a product changes after an order, the order SHALL retain the accepted product snapshot. |
| COM-F0-002 | G0 | Amount and ISO currency | Exact money value | Currency is required; arithmetic is checked; no floats | Domain engine | Calculation trace | `invalid_money` | WHEN two currencies differ, the system SHALL reject addition unless an approved exchange record is supplied. |
| COM-F0-003 | G0 | Stock request and order intent | Reservation plus order event | Stock cannot go below policy floor; write and event are atomic | Buyer requests; merchant sets policy | Transaction and item movements | `unavailable` or `not_committed` | WHEN two buyers race for one unit, at most one accepted order SHALL exist. |
| COM-F0-004 | G0 | External command and idempotency key | One durable result | Retry cannot duplicate charge, refund, shipment, or event | Service policy | Request, response hash, replay result | `unknown_external_state` | WHEN a reply is lost, retry SHALL return or reconcile the first result without a second external action. |

### F1 — useful product

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| COM-F1-001 | G1 | Product, price, terms, channel | Offer version | Published offer is immutable for accepted carts | Merchant approves | Preview, approval, channel result | `partially_published` | WHEN one channel fails, successful channels SHALL stay named and retry SHALL not duplicate them. |
| COM-F1-002 | G1 | Items, buyer context, delivery | Cart and final quote | Total names subtotal, tax, discount, delivery, and currency | Buyer approves checkout | Quote version and consent | `expired_quote` | WHEN any total changes, the buyer SHALL see and accept the new total before payment. |
| COM-F1-003 | G1 | Approved quote | Order state machine | Only named transitions are legal | Buyer places; merchant may cancel by policy | State events and reason | `waiting`, `cancelled`, `exception` | WHEN a duplicate place-order command arrives, the system SHALL return the same order. |
| COM-F1-004 | G1 | Order and provider adapter | Payment intent and observations | Provider secret is brokered; uncertain state stays uncertain | Buyer authorises; refunds need merchant rules | Provider refs, consent, callbacks | `checking`, `failed`, `refunded` | WHEN callbacks arrive out of order, the final state SHALL follow the provider sequence rules and preserve all observations. |

### F2 — category leader

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| COM-F2-001 | G2 | Order items and stock locations | Fulfilment plan and parcels | Item quantity is neither lost nor doubled | Operator approves substitutions | Pick, pack, handoff evidence | `blocked` or `carrier_unknown` | WHEN carrier handoff fails, the order SHALL remain ready-to-handoff and SHALL NOT become shipped. |
| COM-F2-002 | G2 | Return reason, policy, items | Return and inspection | Returned quantity cannot exceed fulfilled quantity | Buyer requests; exception needs human | Photos, policy version, inspection | `ineligible` or `disputed` | WHEN policy denies a return, the buyer SHALL receive the rule, evidence path, and appeal path. |
| COM-F2-003 | G2 | Approved return or service decision | Refund intent | Refund total cannot exceed captured total by currency | Human threshold approval | Approval and provider result | `refund_checking` | WHEN a refund response is lost, reconciliation SHALL prevent a second refund. |
| COM-F2-004 | G3 | Orders and settlement lines | Reconciliation and finance export | Every posted amount maps to source and currency | Finance reviewer approves posting | Match rules, exceptions, signature | `unmatched` | WHEN settlement totals differ, the system SHALL block close and list exact differences. |

### F3 — advanced category leadership

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| COM-F3-001 | G3 | Merchant catalog and channel contracts | Multi-channel projection | Canonical product remains merchant owned | Merchant selects fields/channels | Mapping and loss report | `mapping_required` | WHEN a channel cannot express a field, the preview SHALL name the loss before publish. |
| COM-F3-002 | G3 | Buyer intent and approved merchant offers | Explainable discovery result | Paid placement and relevance are separate labels | Buyer controls filters; merchant cannot self-rank | Ranking inputs and reasons | `insufficient_match` | WHEN an item is sponsored, every view SHALL label it and show the non-paid ordering option. |
| COM-F3-003 | G3 | Merchant grants and network order | Federated order contract | Each party keeps custody; shared states are signed | Both sides approve terms | Offer, acceptance, events, disputes | `counterparty_unavailable` | WHEN a node leaves, each party SHALL retain a verifiable order and recovery path. |
| COM-F3-004 | G3 | Supply-chain evidence | Promise and provenance view | Unknown claims stay unknown; evidence is not marketing text | Merchant publishes; reviewer may challenge | Signed source events and gaps | `unverified_claim` | WHEN a claim lacks evidence, the buyer view SHALL mark it unverified rather than hide it. |

### F4 — frontier network

| ID | Gate | Inputs | Outputs | Invariants | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|---|
| COM-F4-001 | G4 | Buyer-controlled intent, merchant-approved offers, disclosure and ranking policies | Federated discovery results with per-node reasons and paid labels | Buyer profile and private graph stay local; paid influence never changes the unadvertised order silently | Buyer controls fields and filters; each merchant publishes an offer; network governance cannot self-rank | Query grants, offer versions, node reasons, paid status, result digest | `insufficient_match` or `node_untrusted` | WHEN a result is paid or a node cannot explain its match, the network SHALL label it and provide the non-paid or excluded view. |
| COM-F4-002 | G4 | Signed offer, buyer acceptance, inventory promise, regulated payment reference, fulfilment contracts | Multi-party order state shared across independent nodes | No central node owns identity, funds, or all records; final state requires the named signed party states | Buyer, merchant, fulfilment party, and regulated provider approve their own acts | Offer, consent, reservation, provider observations, handoffs, exceptions, signatures | `party_unavailable` or `state_disputed` | WHEN one party is offline or disputes state, no node SHALL report the order finally settled or delivered without required evidence. |
| COM-F4-003 | G4 | Merchant-approved product provenance, fulfilment outcomes, returns, and contextual trust statements | Portable promise-and-trust graph for selected products and counterparties | Unknown claims stay unknown; there is no universal merchant or buyer score; leaving one node does not erase shared orders | Each evidence issuer signs; buyer and merchant choose disclosure; human reviewer handles disputes | Source events, issuer, scope, expiry, dispute, revocation, export proof | `unverified_claim` or `trust_disputed` | WHEN an issuer revokes or a merchant leaves, the graph SHALL update future trust views while preserving verifiable historic order evidence. |

## 8. Domain model

`Merchant`, `Store`, and `Channel` own `Product`, `ProductVersion`, `Variant`,
`Identifier`, `MediaAsset`, `Offer`, `Price`, `TaxQuote`, `Promotion`,
`InventoryLocation`, `StockItem`, `InventoryMovement`, `Reservation`, `Cart`,
`Quote`, `Customer`, `Address`, `Consent`, `Order`, `OrderLine`, `PaymentIntent`,
`PaymentObservation`, `Capture`, `Fulfilment`, `Parcel`, `Shipment`, `Return`,
`Inspection`, `Refund`, `Dispute`, `Settlement`, `Reconciliation`, and
`ServiceCase`. Money always has amount, currency, and rounding rule. Accepted
offers and order lines are immutable snapshots. Customer, address, and payment
references use purpose and retention limits. Provider IDs are alternate keys,
not the canonical identity.

## 9. System architecture

- A Rust commerce engine owns money, inventory, quote, order, payment,
  fulfilment, return, refund, and dispute state machines.
- PostgreSQL is the durable source; object storage holds media, documents, and
  evidence. An embedded local profile supports one merchant offline.
- A transactional outbox commits domain changes and audit events together.
- Durable workers handle channel sync, payment checks, carrier work, imports,
  exports, and reconciliation with idempotency keys.
- Provider-neutral adapters isolate payment, tax, carrier, marketplace, and
  messaging systems. A capability manifest states countries, currencies,
  operations, limits, and data loss.
- HelixCore supplies identity, policy, audit, capabilities, jobs, objects,
  billing, operations, stable project identity, and recovery services.
- Offline selling is limited to pre-declared stock and payment modes; conflicts
  stop acceptance rather than inventing availability.

## 10. Agent and automation contract

| Role | May read and call | May draft | Approval required | Never allowed | Visible progress, check, stop, reverse |
|---|---|---|---|---|---|
| Catalog agent | Approved product and channel data; image and mapping tools | Descriptions, mappings, offer drafts | Publish, price, legal claim | Invent facts, lower stock, or expose private data | Shows field-by-field source and channel checks; drafts undo by version. |
| Merchandising agent | Sales, stock, approved rules | Promotion and assortment proposals | Every live price or promotion | Use protected traits, fake scarcity, or dark patterns | Shows expected effect and uncertainty; human signs; promotion can be stopped. |
| Service agent | One authorised customer case and order | Replies, return plan, refund request | Policy exception, refund, dispute acceptance | Reveal other customers or issue money | Streams case stages; rule engine checks; actions reverse through linked events. |
| Operations agent | Assigned orders, stock, carrier capability | Pick waves and fulfilment plan | Substitution, address change, carrier spend above limit | Mark work done without evidence | Live queue and scans verify; cancel leaves safe state. |
| Reconciliation agent | Orders, provider observations, settlements | Match proposals and finance export | Exceptions and posting | Create or alter provider evidence | Shows unmatched lines; totals and duplicate tests check; export may be rejected. |

All agents use exact, time-bound leases and never receive raw credentials.
Money movement, price publication, restricted goods, and dispute decisions always
cross named human or provider controls.

## 11. Trust, safety, and privacy

Tenant separation is enforced in the database and object store. Resource,
role, purpose, channel, and exact capability control access. Payment values stay
with regulated providers; HelixCommerce stores tokens or references through a
user-owned broker, not raw card data. Personal data is classed by purpose and
retention. Encryption is required in transit and at rest. Buyers see and can
withdraw optional consent without blocking the core purchase.

Delete places products, drafts, customers where law permits, and service cases
in a recoverable 30-day bin. Orders and financial records follow legal-retention
rules and may be hidden from normal use without being destroyed. Permanent
deletion is a separate, re-authenticated, audited act and is blocked by legal
hold. Controls cover account takeover, coupon abuse, stock racing, refund abuse,
webhook forgery, replay, malicious imports, prompt injection, counterfeit claims,
and bulk customer export. Incident recovery can stop checkout, revoke adapters,
freeze risky actions, rotate bindings, reconcile external state, and notify
affected users without claiming more certainty than exists.

## 12. Proof and audit

Proof records the product and offer seen, final quote, buyer consent, stock
reservation, order transition, provider request and observation, fulfilment
scan, return evidence, approval, refund, settlement match, actor, and software
version. Signed metadata and content hashes make alteration detectable. An
independent verifier can check state transitions, arithmetic, signatures,
duplicate prevention, and links to externally supplied evidence. It cannot prove
that goods were real, a carrier was honest, a tax result was legally correct, or
a buyer freely consented beyond the evidence captured.

Aether is preferred for proof and capability brokering through neutral
interfaces. The local fallback provides signing, verification, leases, and
audit, and remains fully usable if Aether is absent.

## 13. UX system

The main surfaces are Home, Catalog, Inventory, Orders, Fulfilment, Customers,
Returns, Money, Channels, Evidence, and Recovery. The buyer flow uses one clear
primary action, shows the final total before consent, and never preselects paid
extras. Merchant basic views show work and exceptions first; advanced views
reveal provider, policy, accounting, and evidence details. All core flows target
[WCAG 2.2 Level AA](https://www.w3.org/TR/WCAG22/) and work by keyboard, touch,
screen reader, and 400% zoom.

Imports, channel publishing, payment checks, fulfilment, refund, and migration
show named stages, real counts, last provider signal, elapsed time, and cancel or
pause state. Completion leaves a durable activity item plus optional desktop
notice. Selected rows show clear checks. Moving stock, order lines, or work
previews the impact and asks only when meaning or custody changes. Safe actions
have undo; deletion uses Recovery. Empty states teach the first useful action.
Errors say whether money, stock, provider, or local data changed and name the
next safe step.

## 14. Interoperability and standards

- [ISO 4217](https://www.iso.org/iso-4217-currency-codes.html) supplies currency
  codes and minor-unit meaning. The maintenance data is versioned; unsupported
  or private currencies require an explicit local definition.
- [OASIS UBL 2.3](https://docs.oasis-open.org/ubl/UBL-2.3.html) supports order,
  invoice, fulfilment, and supply-chain document exchange. A UBL export may not
  carry every Helix approval, channel state, or proof link.
- [GS1 Web Vocabulary](https://ref.gs1.org/voc/) maps portable product, party,
  offer, and consumer-facing facts. It is an adapter; GS1 identifiers are not
  invented for products that do not have them.
- [W3C Payment Request API](https://www.w3.org/TR/payment-request/) can provide a
  browser checkout adapter. It is only a user-interface bridge, not a payment
  rail or proof of payment, and a normal accessible form remains the fallback.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) sets the accessibility target.

Standards are pinned per adapter and verified with conformance fixtures. Import
and export reports show lost discounts, taxes, units, identifiers, state,
signatures, policy, and local extensions before data is accepted.

## 15. Cross-platform contract

The commerce engine, exact-money arithmetic, order state machine, migrations,
proof, and recovery run the same fixtures on Windows, macOS, and Linux. Browser
checkout and operator web views work without desktop APIs. Desktop adds offline
catalog work, local devices, print, scanner, and notification adapters. The CLI
supports import, sync, reconcile, export, and verify. Containers support server,
workers, and self-hosted operations. Offline order capture is disabled unless
stock and payment rules declare it safe. Printers, scanners, secure storage, and
notifications use capability detection with manual entry, file export, or
visible in-app fallback.

## 16. Reliability and performance budgets

- Acknowledged order, reservation, payment-observation, refund, and ledger writes
  have zero allowed data loss in forced-crash tests.
- Checkout quote and reserve operations finish under 500 ms at p95 over a rolling
  30-day window, excluding external provider time, at 100 orders per second.
- A provider call shows its first durable state within 2 seconds; active local
  jobs heartbeat at least every 5 seconds.
- Local cancellation is accepted within 2 seconds and stops a worker within 30
  seconds; external work remains `cancel_requested` until confirmed.
- Idempotency keys for orders, payment, refunds, shipments, and webhooks are kept
  for the relevant provider retry window and never less than 30 days.
- Inventory serialisation prevents negative stock under 1,000 concurrent buyers
  for the same stock item in the supported managed profile.
- Offline mode supports 7 days or 10,000 queued catalog changes; it accepts no
  unsafe stock or payment action after its declared lease expires.
- Managed committed metadata has recovery point zero and 1-hour recovery time;
  self-hosted documented recovery target is 4 hours.
- If search, recommendations, Aether, notifications, or one channel is down,
  direct catalog, order history, local proof, and unaffected channels continue.

## 17. Success measures

- Order promise accuracy: accepted item, total, and delivery compared with the
  final outcome over each rolling 30-day window.
- Oversell, double-charge, double-refund, and false-shipped incidents; target zero.
- Median time to resolve an uncertain payment or fulfilment state.
- Share of returns completed inside the stated policy time.
- Independent order bundles that validate on another supported system.
- Merchant time and cost to move a full store to another provider.
- Buyer completion without dark patterns, plus accessibility task success and
  serious issue counts for the declared WCAG scope.
- Recovery-bin restore success and disaster-recovery drill time.
- Merchant retention, profitable completed orders, and lower support effort,
  not product, click, or agent-call counts.

## 18. Delivery plan

| Gate | Build | Test | Safety | UX | Cross-platform | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| **G0 — Truthful foundation (0–6 months)** | Exact money, stable IDs, atomic inventory/order ledger, jobs, recovery | Arithmetic, race, idempotency, crash, signature tests | Tenant policy, secret broker, threat model | Honest order and provider states | Rust and packaging CI on Windows, macOS, Linux | Dry-run importer for current products/orders | Fresh install, race, crash, restore, verify record |
| **G1 — Useful single-player product (6–18 months)** | Catalog, inventory, cart, quote, order, one payment and fulfilment adapter | Full purchase, failure, return fixtures | Consent, webhook, export, restricted-action checks | Accessible buyer and merchant journeys | Web, desktop, CLI, container, offline catalog | Product/order mapping with rollback | Real sandbox purchase and independent receipt check |
| **G2 — Trusted team product (18–30 months)** | Roles, returns, refunds, service, fulfilment, two providers | Multi-user, duplicate, reconciliation tests | Refund approvals and penetration review | Work queues, slow-state feedback, notices | Devices and degraded-network matrix | Provider and role migrations | Fresh refund, outage, recovery, and team audit drill |
| **G3 — Category leader (30–42 months)** | Multi-channel, settlement, Capital export, portable store | Adapter conformance and scale tests | Fraud, model, tax-boundary review | Cross-channel impact and portability UX | Store move across mixed OS deployments | Verified live store cutover and rollback | External accounting and accessibility review |
| **G4 — Frontier network (42–60 months)** | Build COM-F4-001 buyer-controlled discovery, COM-F4-002 multi-party orders, and COM-F4-003 portable promise/trust graphs | Paid-ranking, privacy, replay, partition, double-value, party-loss, dispute, revocation, and malicious-node tests | Independent marketplace, payment-boundary, privacy, fraud, and competition review; no central custody | Buyer-controlled intent, clear paid status, multi-party progress, dispute, recovery, and exit journeys | Mixed Windows/macOS/Linux merchant nodes plus browser/desktop buyers prove local custody and degraded operation | Add/remove a merchant, carrier, or provider; revoke bindings without identity, order, or shared-secret loss | Independent discover, order, pay-sandbox, fulfil, dispute, revoke, exit, disaster-recover, and verify exercise covering all F4 evidence |

Every gate closes only from fresh release-candidate builds, tests, safety checks,
journeys, migrations, and operator evidence. A skipped provider test is visible
and cannot be counted as a pass.

## 19. Current truth and gap

The live Rust source has meaningful `products`, inventory counts, `orders`, and
order items. Order creation locks product rows, checks available stock, uses
checked arithmetic, and writes order work in a database transaction. This is the
strongest of these seven early domain prototypes. It is still far from a product:
there is no cart, payment, refund, return, fulfilment, seller workspace, buyer
UI, or automated domain test suite. Mixed-currency items are not rejected; the
current order can label the sum with the last product's currency. The service
also has the shared application-state compile failure, and domain writes are not
atomic with audit or billing events.

The highest-risk gap is exact multi-currency order truth. The safest first slice
is COM-F0-002 through COM-F1-003: reject mixed-currency carts, reserve stock and
accept one order atomically, prove a two-buyer race, show an accessible final
quote, and recover after forced crashes using temporary test state only.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Identity | Stable merchant, product, offer, order, and customer IDs independent of paths/providers | Architecture decision and migration proof |
| Money | Exact decimal/integer minor units plus ISO currency; no float | Finance architecture review |
| Inventory/order write | One database transaction plus transactional outbox | Founder-approved integrity exception |
| Payment custody | External regulated provider; brokered references, no raw card data | Legal, security, and founder approval |
| Provider design | Neutral adapters with capability and loss manifests | Architecture review |
| Agent authority | Draft and propose only; no live price, charge, refund, or dispute decision | Safety and founder approval |
| Proof provider | Aether preferred; local signed fallback mandatory | Provider-neutrality review |
| Delete | 30-day recovery bin where lawful; financial retention and legal hold win | Legal-retention decision |
| Accessibility | WCAG 2.2 AA target; no dark patterns | Accessibility and ethics review |
| Platform | Windows, macOS, Linux, web, CLI, container; no OS-only critical path | Founder scope decision |
| First slice | Exact quote plus atomic stock/order and crash proof | Product decision with equal risk closure |
| Federation, custody, lending | Disabled until G4 and separate legal/safety gates | Founder approval |

## 21. Definition of category-defining done

- [ ] All seven signature journeys work with real providers and failure cases.
- [ ] Accepted offers, totals, currencies, stock, and customer consent are exact.
- [ ] Races, retries, crashes, and out-of-order callbacks cannot duplicate value.
- [ ] Buyers never face hidden fees, fake scarcity, or forced optional consent.
- [ ] Humans control prices, refunds, disputes, restricted goods, and money.
- [ ] Independent proof validates orders without trusting the live server.
- [ ] A merchant can move its store and revoke old provider bindings safely.
- [ ] Agents use exact leases, never see secrets, and cannot self-approve.
- [ ] WCAG 2.2 AA scope and buyer/merchant accessibility journeys pass.
- [ ] Windows, macOS, Linux, web, offline, CLI, and container limits are proven.
- [ ] The 30-day bin, legal holds, retention, permanent deletion, and restore work.
- [ ] External security, privacy, finance, commerce, and accessibility reviews close.
- [ ] The product says clearly what external provider evidence does not prove.
