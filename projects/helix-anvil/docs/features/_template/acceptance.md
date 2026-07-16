# NNN — <Feature Title> · Acceptance

<!-- INSTRUCTIONS (delete before use)
  This document is the objective bar for "this packet is done" AND the EVIDENCE
  ledger that proves it. It must be completable by a reviewer (human or AI)
  without re-interviewing the implementer.

  Substrate v2.2: this file is keyed to the Requirement/Scenario blocks in
  requirements.md — ONE checklist row + ONE evidence block per `#### Scenario`.
  Precedence: acceptance.md is PROOF, not a behavior authority. The behavior
  authority is requirements.md (in-flight delta) and, once promoted on `done`,
  specs/<capability>/spec.md (ratified current). Do not restate behavior here.

  STRUCTURE:
    Section 1 — Scenario checklist: one row per Requirement/Scenario.
    Section 2 — Evidence: paste-ready command output proving each Scenario.
    Section 3 — Review requirements: which tiers ran.
    Section 4 — Trust-but-verify primitives: which primitives apply + evidence.

  USAGE:
    - Copy each Requirement/Scenario name from requirements.md, one row per Scenario.
    - Mark [x] only when the Scenario is demonstrably true AND evidence is pasted
      in Section 2.
    - "Demonstrably true" means: a deterministic command (test, lint, schema
      validator, hash check, curl) returned the expected output — NOT "it looked
      right in the browser."
-->

---

## Section 1 — Scenario checklist

<!-- One row per `##### Scenario` from requirements.md.
  Format: - [ ] <Requirement name> / <Scenario name> — <one-line expected outcome>
  Mark [x] only when evidence is pasted in Section 2. -->

- [ ] <Requirement name> / <Scenario name> — ...
- [ ] <Requirement name> / <Scenario name> — ...

---

## Section 2 — Evidence

<!-- For each Scenario, paste the exact command and its output below.
  Use the format shown. Do NOT summarize or paraphrase — paste verbatim.
  If a Scenario is verified by a screenshot, describe what the screenshot
  shows and attach a path reference. -->

### <Requirement name> / <Scenario name>

```
# Command run:
<paste command here>

# Output:
<paste output verbatim here>
```

<!-- Add one block per Scenario. -->

---

## Section 3 — Review requirements

<!-- Check off each tier only after it has actually run and findings have been
  absorbed or documented. Tier 3 is mandatory when: the packet touches a gated
  path (check REVIEW.md), the diff exceeds the project's line-count threshold
  (default 500 LOC), or the packet touches signing/auth/payment/migration paths.
-->

- [ ] **Tier 0** (deterministic linters: ruff, mypy, bandit, semgrep, etc.) — required
  ```
  # Command:
  <project Tier 0 command>
  # Result: PASS / findings list
  ```
- [ ] **Tier 1** (`/review`) — required on every push
  - Findings: [none | list findings absorbed or documented with DJ reference]
- [ ] **Tier 2** (`/review-pr`) — [required if substantial | n/a — reason]
  - Findings: [none | list]
- [ ] **Tier 3** (`/ultrareview`) — [required — gated path: <name> | required — diff > 500 LOC | n/a — reason]
  - Findings: [none | list findings absorbed or documented]

---

## Section 4 — Trust-but-verify primitives

<!-- Copy the applicable primitives from design.md. For each, paste evidence.
  Delete primitives that do not apply to this packet; explain why briefly. -->

### (h) UI-feature runtime-render assertion
<!-- Applies if this packet introduces or modifies a UI surface.
  Verify: render via the framework's test client; grep response for the
  interactive element selector AND any client-behavior glue (script tag,
  binding). -->
[applies / does not apply — reason]
```
# Command:
# Output:
```

### (i) AST-scoped contract-grep
<!-- Applies if this packet adds new contracts to a verified set (event_kind,
  allowlist, ACL, registry, schema). Verify: read the verifying function body
  (AST-scoped, not file-wide grep); confirm the new contract appears in the
  dispatch structure, not just in a docstring. -->
[applies / does not apply — reason]
```
# Command:
# Output:
```

### (j) Handler-chain return-code-propagation
<!-- Applies if this packet introduces a handler that calls a function with a
  meaningful return code. Verify: read the dispatch source; locate rc-capture,
  rc-check, and downstream side-effect lines; confirm rc-check occurs BEFORE
  the side-effect. -->
[applies / does not apply — reason]
```
# Evidence (direct read — not regex):
```

### (l) Happy-path signing roundtrip
<!-- Applies if this packet touches signing-chain code or introduces a new
  event_kind requiring signing. Verify: generate a real keypair in tmp_path,
  configure emitter with private key, dispatch a real event, call the verify
  function with verify_signatures=True, assert clean. -->
[applies / does not apply — reason]
```
# Command:
# Output:
```

### (m) Sovereignty-constraint cross-check
<!-- Applies if this packet introduces a new external network source (URL in
  HTML/JS/server config), new filesystem source outside the project tree, or
  new outbound-call helper. Verify: cross-check against the project's
  sovereignty allow-list (docs/sovereignty-constraints.yaml or equivalent). -->
[applies / does not apply — reason]
```
# Allow-list path:
# New source(s) introduced:
# Cross-check result:
```

### (n) Production-state-isolation gate
<!-- Applies if this packet introduces or modifies a tool that writes to
  repo-tree state (state/, _meta/, registry/, or any persistent path).
  Verify: (a) tool accepts a path-override flag AND (b) all test invocations
  pass it explicitly to tmp_path. -->
[applies / does not apply — reason]
```
# Path-override flag name:
# Test invocation example:
# tmp_path confirmed: yes / no
```
