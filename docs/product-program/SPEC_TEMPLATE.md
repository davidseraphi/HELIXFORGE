# HelixForge category-defining product specification template

This template is the required shape for every target-state product sheet in
`docs/product-program/specs/`.

The sheets describe a five-year destination. They are not claims about the
current code. Every sheet must keep `Current truth` separate from `Target
state` so an agent cannot turn a future ambition into a false completion
report.

## Required header

```yaml
product: <canonical name>
catalog_order: <number or standalone>
status: target-state-spec
horizon: 60 months
current_maturity: <scaffold|prototype|alpha>
primary_users: [<roles>]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

One precise sentence describing the category the product creates or wins.
It must name the user outcome, not only the technology.

## 2. Five-year destination

Describe what becomes possible after five years of successful work. Separate:

- the useful product;
- the category-defining advantage;
- the frontier capability;
- what remains under human authority.

## 3. Users and hard jobs

Name primary and secondary users. For each user, list the hard jobs they need
to complete and the failure they fear most.

## 4. Product laws

Five to ten rules that no implementation may break. Include sovereignty,
portability, proof, accessibility, human authority, and honest failure.

## 5. Scope boundaries

List what the product owns, what another HelixForge product owns, and what the
product will not attempt. High-stakes products must state that advice and
automation do not replace licensed or accountable human decisions.

## 6. Signature experiences

Describe at least six complete user journeys. Each journey must include:

1. entry point;
2. visible progress;
3. human decision points;
4. completion proof;
5. failure and recovery;
6. export or portability path.

## 7. Capability map

Use stable capability identifiers. Group them as:

- `F0` foundation;
- `F1` useful product;
- `F2` category leader;
- `F3` advanced category leadership;
- `F4` frontier network.

Each capability needs inputs, outputs, invariants, authority level, evidence,
failure state, and at least one testable acceptance statement.

Capability identifiers are public program contracts. After publication, an ID
cannot be renamed or reused without a Decision Log entry, a compatibility alias,
a migration for stored references, and proof that old packets and evidence still
resolve to the same meaning.

## 8. Domain model

Define the important records, their ownership, lifecycle, version rules,
retention rules, and relationships. Do not use a generic `metadata` field as a
substitute for a real domain model.

## 9. System architecture

Name the domain engine, application services, adapters, storage, event flows,
background work, offline behaviour, and extension points. Reuse HelixCore for
identity, policy, audit, capabilities, jobs, objects, billing, and operations.

## 10. Agent and automation contract

For every agent role define:

- what it may read;
- what tools it may call;
- what it may draft;
- what requires approval;
- what it can never do;
- how a user sees progress;
- how the result is checked;
- how the action is stopped or reversed.

No sheet may use “AI-powered” as a capability definition.

## 11. Trust, safety, and privacy

Define access control, tenant separation, sensitive-data classes, encryption,
consent, purpose limits, data residency, deletion, legal hold, misuse controls,
and incident recovery. Delete means a recoverable 30-day bin by default;
permanent deletion is a separate, explicit, audited act.

## 12. Proof and audit

Define what evidence is created for important actions, how evidence is signed,
what can be independently checked, and what the evidence does not prove.
Aether is the preferred proof and capability provider through provider-neutral
interfaces. HelixForge must retain a local fallback so one external project is
never a hard runtime dependency.

## 13. UX system

Define the main surfaces, navigation model, progressive reveal, keyboard and
touch use, accessibility, slow-work feedback, completion notifications,
selection and move confirmation, undo, recovery, empty states, and plain-language
error handling. Nothing long-running may look frozen or complete silently.

## 14. Interoperability and standards

Name only standards verified from official sources. Keep standards behind
adapters when versions or jurisdictions can change. Record why each standard
is used and what information could be lost during import or export.

## 15. Cross-platform contract

The same core behaviour must be tested on Windows, macOS, and Linux. Browser,
desktop, CLI, container, and offline support must state clear limits. Platform
features must have capability detection and a safe fallback.

## 16. Reliability and performance budgets

Define service objectives, data-loss budget, recovery targets, offline limits,
job cancellation, idempotency, concurrency, scale targets, and graceful
degradation. Do not use a percentage without naming the measured event and
time window.

## 17. Success measures

Include user outcome, trust, reliability, accessibility, portability, and
business measures. Avoid vanity counts such as raw agent calls or lines of code.

## 18. Delivery plan

Use these gates:

- `G0 — Truthful foundation` (0–6 months)
- `G1 — Useful single-player product` (6–18 months)
- `G2 — Trusted team product` (18–30 months)
- `G3 — Category leader` (30–42 months)
- `G4 — Frontier network` (42–60 months)

Every gate must have build, test, safety, UX, cross-platform, migration, and
operator proof. A gate closes only from fresh checks.

## 19. Current truth and gap

Record what is present in the live source, what is only a scaffold, the most
important gap, and the safest first vertical slice. Never copy completion
claims from status documents without checking the implementation.

## 20. Decisions locked for Kimi

Use a table with `Question`, `Locked default`, and `Change requires`. Resolve
common design choices so Kimi does not guess. Any genuinely founder-only choice
must be marked and must not block safe foundation work.

## 21. Definition of category-defining done

End with a checklist describing the full 60-month destination. It must include
independent proof, real user journeys, accessibility, portability, recovery,
security review, and honest limits.
