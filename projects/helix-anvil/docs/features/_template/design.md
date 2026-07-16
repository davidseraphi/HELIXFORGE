# NNN — <Feature Title> · Design

<!-- INSTRUCTIONS (delete before use)
  This document is the implementation contract. It must be concrete enough
  that a different agent — including a smaller local model — can execute
  the tasks in tasks.md without re-deriving architecture.

  If this design requires deviating from BUILD_SPEC.md (or equivalent
  project-level architecture document), record a Decision Journal entry AND
  update BUILD_SPEC in the same commit — never silently diverge.

  Resolve all [NEEDS CLARIFICATION] markers from requirements.md before
  filling this document.
-->

## Approach

<!-- The chosen implementation approach in plain prose. State WHY this approach
  was chosen (see "Alternatives considered" below). Per portfolio doctrine,
  lead with the thorough option; if a cheaper option is chosen instead, name
  a quantified threshold justifying the override (session LOC budget, deadline,
  blast-radius score — NOT "for simplicity"). -->

## Capability & spec promotion

<!-- Substrate v2.2: name the capability/capabilities this packet's behavior-spec
  delta (requirements.md) promotes into on `done`. For each: is it a NEW capability
  (greenfield specs/<name>/spec.md — copy _spec_template/spec.md) or a MODIFICATION
  of an already-ratified spec (use ADDED/MODIFIED/REMOVED groups; MODIFIED/REMOVED
  must reference Requirements that already exist there)? Set this packet's
  `promotes_to` in status.json + PROJECT_STATE.json.ratified_specs to match. A pure
  refactor that ratifies no standing behavior leaves promotes_to empty and says so. -->

- Target capability(ies): `specs/<capability>/spec.md` — [new | modifies ratified]
- promotes_to: [<capability> ... | none — reason]

## Architecture

<!-- How this feature fits the existing system. Describe the component(s)
  introduced or modified, the data flow end-to-end, and the seams where
  other packets or systems connect. A diagram in ASCII or Mermaid is
  preferred for any non-trivial flow. -->

## Data contracts

<!-- Tables, entities, schema fields, event shapes, or message envelopes
  created or changed. Reference the canonical data model (BUILD_SPEC.md or
  equivalent). Be explicit — these are the seams other packets build against.

  For any signed events: specify the signing tuple (fields that enter the
  signature), the key-id convention, and the verify function contract.
  Trust-but-verify primitive (l) — happy-path signing roundtrip — applies.

  For any new external URL introduced (CDN, API endpoint, webhook): cross-check
  against the project's sovereignty allow-list before committing.
  Trust-but-verify primitive (m) — sovereignty-constraint cross-check — applies.
-->

## Interfaces / contracts

<!-- API endpoints, function signatures, CLI subcommands, UI component props,
  message/event schemas. Include:
  - HTTP: method, path, request shape, response shape, status codes
  - CLI: subcommand name, flags, stdout contract, exit codes
  - Functions: signature, return type, exceptions raised
  - Events/messages: topic, payload schema (JSON Schema or TypeScript type)

  Any contract listed here is a stability boundary — changing it requires
  updating all callers and bumping schema_version in status.json. -->

## Allowed edit paths

<!-- Repeat (and narrow if possible) from requirements.md. This is the build-
  time enforcement boundary. Implementation MUST stay inside these paths. -->

- `docs/features/NNN-<slug>/**`
- ...

## Forbidden edit paths

<!-- Repeat from requirements.md. Add any design-level additions. -->

- `docs/features/**` (other packets)
- Any `.env*` inside the repo
- Production state paths without tmp_path isolation during dev/test
- [Add project-specific paths]

## Test strategy

<!-- Describe what is tested and at what layer. Per portfolio doctrine,
  tests must cover:
  (h) Runtime-render assertion for any new UI surface.
  (i) AST-scoped contract-grep for new contracts added to verified sets
      (allowlists, ACLs, registries, event_kind sets, etc.).
  (j) Handler-chain return-code-propagation direct-read for any new handler.
  (l) Happy-path signing roundtrip for any new signed event type.
  (m) Sovereignty cross-check for any new external network source.
  (n) Production-state-isolation gate: tool accepts path-override flags AND
      all test invocations pass tmp_path explicitly.

  List which primitives apply to this packet. Delete those that do not apply
  and explain why. -->

  Unit tests:
  - ...

  Integration tests:
  - ...

  Manual / exploratory:
  - ...

  Trust-but-verify primitives applicable to this packet:
  - (h) [applies / does not apply — reason]
  - (i) [applies / does not apply — reason]
  - (j) [applies / does not apply — reason]
  - (l) [applies / does not apply — reason]
  - (m) [applies / does not apply — reason]
  - (n) [applies / does not apply — reason]

## Integrations

<!-- External services touched. For each:
  - Service name and what it is used for
  - Credential name and where it lives (`.keys/<project>/.env.local` per doctrine)
  - Whether this is a gated path (triggers mandatory Tier 3 review — check
    REVIEW.md for the project's gated-path list)
  - Sovereignty posture: is the service on the project's allow-list? -->

## Alternatives considered

<!-- Options weighed and the reason the chosen approach wins. Per portfolio
  doctrine, always name which option is more thorough and which is cheaper
  before leading with a recommendation. A cheaper option requires a quantified
  override threshold. -->

| Option | Thoroughness rank | Cost rank | Notes |
|--------|------------------|-----------|-------|
| (A) ... | 1 (most thorough) | 2 | ... |
| (B) ... | 2 | 1 (cheapest) | ... |

Chosen: Option [A/B] because [quantified reason if cheaper; or because it is the most thorough option].

## Risks and mitigations

<!-- For each risk: describe the failure mode, impact (blast radius), and the
  mitigation or monitoring approach. Categories to consider:
  - Data integrity / audit-trail contamination
  - PII / secrets exposure
  - Sovereignty violation (outbound network, external CDN)
  - Gated-path blast radius (auth, payments, data migration)
  - State mutation side-effects during dev/test runs
  - Signing-chain silent failures (pytest-green, production-broken) -->

| Risk | Impact | Mitigation |
|------|--------|------------|
| ... | ... | ... |
