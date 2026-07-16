# Bug Operating System — HelixAnvil

Project: **HelixAnvil** (standalone native IDE). Customize surface taxonomy when product code lands; lifecycle rules remain portfolio doctrine.

# Bug Operating System — portfolio canonical (v4)

Status: canonical upstream (`~/shared/bug-os/`)
SPEC_VERSION: 4.0.0
Last updated: 2026-07-03

This is the single source of truth for how every project in the portfolio finds, fixes,
prevents, and learns from bugs. Projects **copy** this file to
`docs/quality/BUG_OPERATING_SYSTEM.md` and customize only the marked project-specific slots
(surface taxonomy, gate command list, seed classes). They MUST NOT rewrite the lifecycle,
severity meanings, evidence standard, or the reproduce-before-patch rule. Lineage:
DOCUMENT_OS → AETHER → Investment OS → this upstream (v4 merges the best of all three).

## The rule

No broad "hunt every bug" claims without evidence. **Every material bug has a reproducible
packet, a named owner surface, a root-cause statement, a regression gate, and closure
evidence.** AI agents generate packets and propose fixes; **deterministic gates remain the
authority — no agent self-certifies closure.**

The closed loop:

```text
report -> reproduce -> localize -> fix -> regression gate -> release proof -> learning update
```

## The packet (7 files under `docs/bugs/BUG-NNNN-<slug>/`)

Numbering is 4-digit sequential per project; the slug is kebab-case. `docs/bugs/BUGS.md` is the
quick-index registry ("the packet is the authority; this file is the quick index").

| File | Holds |
|---|---|
| `bug.json` | Machine-readable metadata (schema: `docs/quality/bug-packet.schema.json`) |
| `report.md` | Symptom / Impact / Environment / First Seen / Resolution |
| `repro.md` | Status / Steps / Expected / Actual / Evidence / Reproduction Notes |
| `root_cause.md` | What failed / Why gates missed it / Owning surface / Preventing invariant / Primitive(s) |
| `fix_plan.md` | Scope / Files expected to change / Non-goals / Risk / Rollback |
| `regression_tests.md` | Required gates / Manual proof / Automated proof / Not covered |
| `evidence.md` | Before / After / Commands / Closure decision / Closure attestation |

## Severity (user/product impact, NOT implementation difficulty)

| Severity | Meaning | Default gate |
|---|---|---|
| S0 | Data loss; security/sensitivity exposure (secret in tree, auth/isolation bypass, data-to-untrusted leak); audit-trail integrity break; money/state corruption; destructive writes | Stop feature work; fix or isolate immediately |
| S1 | A core-loop stage broken; a gated-path defect; a governance/safety gate silently not enforcing; an agent acting outside its boundary | Fix before continuing adjacent features |
| S2 | Important workflow degraded; visible correctness/perf/accessibility problem; a contract drift on a non-core surface | Packet and schedule in the current/next quality batch |
| S3 | Annoyance, polish, narrow edge case | Packet or combine into a gardening batch |

> PROJECT SLOT: refine the S0/S1 examples to your real failure history — but never weaken the
> tiers themselves.

## Priority (scheduling — orthogonal to severity)

| Priority | Meaning |
|---|---|
| P0 | Act now |
| P1 | Current quality batch |
| P2 | Next planned batch |
| P3 | Backlog |

A serious bug is lower priority only with a documented mitigation.

## Bug classes (`bug.json.bug_class`, optional routing)

- **`product`** — everything else (default).
- **`data-integrity`** — persistence/isolation/audit/money correctness. Gate: integration or
  security test (+ a lint/semgrep rule where applicable).
- **`contract`** — FE/BE or API/MCP contract drift. Gate: a generated-types compile guard
  and/or a contract test.
- **`ai-behavior`** — an agent/model produced wrong, ungoverned, or out-of-boundary output.
  **Regression gate MUST be an eval case (a path/marker containing `ai_eval`)** — a
  deterministic test cannot regress a probabilistic behavior class. The validator BLOCKS
  `verified` without one.
- **`security`** — auth, recusal, sensitivity-class, secret handling.
- **`performance`** — a measured regression against a stated budget.

> PROJECT SLOT: you MAY narrow this enum in your copied schema to your real classes.

## Required evidence

Before patching, every bug needs ≥1 reproducible evidence type: a failing deterministic test;
a runtime trace/log; a fixture or golden diff; a performance measurement; a source-level
invariant violation; a failing eval case; or an explicit "cannot reproduce yet" note with the
attempted environment.

After patching, every bug needs closure evidence: the original reproduction no longer fails;
≥1 regression gate exists (or a written reason why not); affected canonical docs / feature
packets are updated if scope, commands, or acceptance changed.

## Lifecycle

| State | Meaning |
|---|---|
| reported | Observed, not yet reproduced |
| reproduced | Repro evidence exists |
| localized | Probable root cause + owner surface named |
| fixing | Implementation in progress |
| fixed | Fix exists and local proof passes |
| verified | Regression gate + closure evidence pass |
| deferred | Accepted risk with reason, owner, and revisit trigger |

**`verified` means the packet contains evidence — never "the app looks fine."**

## Root-cause standard

A root-cause note must answer:

1. What failed?
2. Why did existing gates miss it?
3. What invariant would have caught it earlier?
4. What new test, lint, eval, smoke, or review checklist now catches the class?

Tie Q4 to machine-checkable enforcement: name the **trust-but-verify primitive(s) (a)-(n)**
(portfolio doctrine, `~/shared/AGENTS.md`) and/or the **Tier-0 runner / lint / semgrep rule**
in `bug.json.root_cause_primitives`. After two failed fix attempts, stop patching and read the
owning code, dependency source, runtime logs, and lowest-layer repro before a third attempt
(portfolio doctrine).

## Closure attestation (dual mode)

Closure binds `verified` to proof, not a self-asserted flag. Two modes; the validator warns
when a `verified` packet has neither.

- **Mode 1 — git-anchored (portable default; works in any git repo):**
  `closure_attestation = {commit_sha, verified_at, gate_run_ref}` — the commit carrying the
  fix and where the passing gate output lives (an `evidence.md` section, a CI run URL, or a
  command-transcript path).
- **Mode 2 — signed-ledger (strong; where a signed run-ledger exists, e.g. AETHER, or Genesis
  post-ATTEST-004):** `closure_attestation = {ledger_event_id, signed, key_id, verified_at}` —
  closure bound to a signed run-ledger event, cryptographically verifiable.

Prefer Mode 1 everywhere by default; use Mode 2 wherever the project has a signed ledger.

## Audit-intake lane (optional, from DOCUMENT_OS — for projects that receive raw audits)

When a project takes in bulk read-only audit reports (e.g. an AI review dump), do NOT turn each
finding into a packet. Instead:

- Land raw reports read-only under `docs/bugs/audit-intake/`, each SHA-256-hashed so re-pasting
  the same report is deduped, not re-triaged.
- Track candidate families in `docs/bugs/AUDIT_CANDIDATE_BACKLOG.md` as `AUD-*` entries.
- **Promotion rule — promote an `AUD-*` candidate to a real `BUG-NNNN` packet ONLY when** it has
  a failing test, a source-localized invariant, a clear feature blocker, or a shared root cause
  across several findings. Everything else stays a candidate. This keeps the packet registry
  meaningful instead of flooded.

See `AUDIT_INTAKE.md` in this upstream for the lane's files and the backlog template.

## Gate integration

Wire the validator (`tools/quality/validate_bug_packets.py`) into the project's **Tier-0**
blocking gate — either the context/drift check or a pre-commit hook — so packet integrity
blocks on every check. Deterministic regression tests join the normal test command; eval-case
gates join the project's `ai_eval` runner. Gated-path bugs additionally require the project's
highest-tier review before the fix merges. **Zero packets is a clean pass.**

> PROJECT SLOT: list your actual gate commands in `docs/quality/quality-gate.<project>.json`
> (copy `quality-gate.schema.json` conventions from a sibling project).

## Gardening rule

Every quality batch starts by reading `docs/bugs/BUGS.md`, then the packet for every
S0/S1/P0/P1 item. Do not continue broad feature work while an S0 or unmitigated S1 bug is newly
reproduced. Validate packets with `python tools/quality/validate_bug_packets.py` (also runs
inside the Tier-0 gate).

## Agent handoff rule

> Before fixing a bug, read `docs/quality/BUG_OPERATING_SYSTEM.md` and the relevant
> `docs/bugs/BUG-*/` packet. Do not patch until the bug is reproduced or the packet records why
> reproduction is blocked. After the fix, update the packet evidence and run the configured
> quality gates. AI agents generate packets and propose fixes; deterministic gates remain the
> authority.

## Filing a bug fast

Use the machine-wide `/file-bug` skill (`~/.claude/skills/file-bug/`): it copies the template,
allocates the next `BUG-NNNN`, updates `docs/bugs/BUGS.md`, and runs the validator — so filing
is one step, not a manual copy + number hunt.
