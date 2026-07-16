# 011 — Foundation Integrity umbrella

## Status

Ratified by founder on 2026-07-15. This umbrella is a map and contract, not one
implementation change. `011.1`–`011.7` are now closed. Foundation Integrity G0
is complete pending the skipped live restore roundtrip evidence from CI.

## Scope

HelixCore **G0 — Truthful foundation**. The umbrella divides the following
foundation items into narrow child packets, each with its own allowed paths,
forbidden paths, proof, and founder activation gate:

1. Repository boundary and preservation plan
2. Complete clean build and formatting
3. Native Windows, macOS, and Linux CI design
4. Stable identity that does not depend on folder paths
5. Safe registration, membership, tenant separation, and per-resource access
6. Exact capability broker contracts, with secret values hidden from agents
7. All-or-nothing domain, audit, outbox, and idempotency writes
8. Durable jobs with real process ownership, cancellation, crash recovery, and visible progress
9. Truthful readiness and release gates that execute fresh checks
10. 30-day recovery bin, restore, permanent-delete authority, and policy exceptions
11. Backup plus clean restore proof
12. One canonical product shell and semantic state system
13. Package, installer, migration, export, and restore proof on all supported systems

## Child packets

| Packet | Title | Activation |
|---|---|---|
| 011.1 | Repository boundary, clean build, and native CI design | Closed |
| 011.2 | Stable identity, safe registration, tenant separation, and per-resource access | Closed |
| 011.3 | Atomic writes, durable visible jobs, truthful readiness, and fresh release gates | Closed |
| 011.4 | Recovery bin, restore, permanent-delete authority, and policy exceptions | Closed |
| 011.5 | Backup plus clean restore proof | Closed |
| 011.6 | One canonical product shell and semantic state system | Closed |
| 011.7 | Package, installer, migration, export, and restore proof on all supported systems | Closed |

Remaining foundation items (11–13) will become child packets `011.5`–`011.7`
after `011.4` is closed.

## Allowed edits at the umbrella level

- Program documents: `PROJECT_STATE.json`, `NEXT_ACTION.md`, `DECISION_LOG.md`,
  `AGENTS.md`, `BUILD_SPEC.md`.
- Product-program index: `docs/product-program/PROGRAM_MANIFEST.json`.
- This umbrella directory and the child-packet directories under
  `docs/features/`.

## Forbidden edits at the umbrella level

- No product source code.
- No migrations that change existing data.
- No secret values, `.env` files, or in-repo credentials.
- No Git mutations, service starts, or runtime-state changes except through an
  activated child packet.

## Cross-cutting acceptance

- The system SHALL close each child packet with fresh commands run against the
  exact candidate before the next child packet is activated.
- The system SHALL record command, environment, timestamps, exit status,
  artifact hashes, and skipped checks with reasons for every gate.
- The system SHALL state what the foundation does and does not yet prove.
