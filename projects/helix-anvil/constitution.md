# HelixAnvil — Constitution (immutable project principles)

Status: canonical · Last updated: 2026-07-15

These principles are binding on every agent and every change. When a packet,
design, or shortcut conflicts with an Article, the Article wins — or the Article
is amended first (with a `DECISION_LOG.md` entry), never silently violated.

## Article I — Scope discipline
All implementation happens inside a reviewed feature packet with declared
allowed/forbidden edit paths. No packet, no code. Scope changes revise the
packet first; they are never smuggled into an unrelated change.

## Article II — Evidence before claims
Load-bearing factual claims (versions, external APIs, editor-engine capabilities)
are verified against a primary source before they enter the build. Unverified
claims are marked `[NEEDS CLARIFICATION]` and resolved before they become
load-bearing.

## Article III — Secrets stay outside the tree
No credential ever lives in the repository. All secrets live under
`~/Desktop/.keys/helix-anvil/` (or `.keys/_shared/` for cross-project keys).
New credentials are introduced with a paste-ready block, never a vague instruction.

## Article IV — Vendor-neutral by construction
The substrate works for every portfolio vendor (Claude, Codex, Gemini/
Antigravity, Kimi). Canonical instructions live as literal content in `AGENTS.md`;
vendor-specific notes live only in that vendor's shim.

## Article V — Thorough over cheap; ambitious but never vague
At a fork between a structurally-correct path and a cheaper shortcut, the
thorough path is the default. A shortcut requires a named, quantified reason in
`DECISION_LOG.md`. Ambition must resolve into concrete primitives, schemas, and
milestones — never a slogan.

## Article VI — Independent verification
Verification is structurally independent of the work it checks: it re-derives
against tests, rules, or a different model — never just re-reads the same agent's
output. After two consecutive failed fix attempts at the same problem, stop and
root-cause before a third attempt.

## Article VII — Own the editor kernel
HelixAnvil’s identity is a **from-scratch native editor path**. Wrapping Monaco,
Code-OSS, or Electron as the *product* is forbidden without an explicit
constitution amendment. Optional engines (parsers, LSP servers, git libraries)
attach at boundaries; they do not replace the document model.

## Article VIII — Separate from HelixCode
HelixAnvil does not re-implement the HelixForge code forge. Forge features belong
in HelixCode. This project may *consume* forge APIs later; it must not become a
soft fork of HelixForge monorepo product scaffolding.
