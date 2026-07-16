# HelixForge — agent operating layer

Vendor-neutral. Defer behavioral doctrine to `~/shared/AGENTS.md`.

## Read order (resume)

1. `NEXT_ACTION.md` — next smallest safe scope
2. `PROJECT_STATE.json` — machine state
3. `BUILD_SPEC.md` — stack + commands
4. `VISION.md` — north star
5. `docs/product-program/README.md` — category-defining target program
6. `docs/product-program/SHARED_PRODUCT_CONTRACT.md` — rules every product keeps
7. `docs/product-program/GLOSSARY.md` — plain-English domain terms
8. the active product sheet under `docs/product-program/specs/`
9. `docs/architecture/overview.md`
10. `DECISION_LOG.md` — read the full log; current category-program decisions
    are at the top and older build history follows

## Commands

```bash
# Infra
docker compose up -d postgres nats minio minio-init

# Rust
cargo test --workspace
cargo run -p gateway
cargo run -p helix_collab_api
cargo clippy --workspace --all-targets

# JS
pnpm install
pnpm --filter @helixforge/console dev
```

## Architecture invariants

- **All product APIs** start via `service_kit` and reuse HelixCore clients.
- **Secrets** only in `~/Desktop/.keys/helixforge/.env.local` — never in-repo `.env`.
- **Audit**: security actions go through `audit_log` hash chain.
- **No vendor lock-in**: Ory, Postgres, MinIO, NATS, self-hosted K8s.
- Long-running dev servers run in **user-owned CMD**, not AI background shells.

## Transcript archive (machine)

Sessions for this cwd are archived under:

`C:\Users\divin\TRANSCRIPTS\HELIXFORGE\`

(per tool: `grok\`, `claude\`, `codex\`, `cursor\`, `kimi\`, …)

Scheduled task: **AI Transcript Archiver** every 5 minutes.  
Details: `docs/runbooks/transcript-archive.md`.

## Active goal

**FOUNDATION-INTEGRITY-011.2** — implement stable identity, safe registration,
tenant separation, and per-resource access. In plain project terms: make sure
sign-ups and project creation generate their own stable IDs (not folder paths),
cannot hijack an existing tenant or project, receive only least-privilege
permissions by default, and are stopped by the database from touching another
tenant's data. Child packet `011.1` is closed; `011.3` is documented but not
activated.

Canonical program: `docs/product-program/README.md`.  
Kimi execution contract: `docs/product-program/KIMI_MASTER_BUILD_PROMPT.md`.

**Foundation first.** Do not resume a thin product depth pass until the founder
activates `011.2` and that child packet closes.
Target-state sheets are ambitions, not completion claims.

## Compaction checklist

- [ ] What changed since last session?
- [ ] Are crates still compiling (`cargo test -p shared_core`)?
- [ ] Is docker-compose healthy?
- [ ] Any new secrets needed in `.keys/helixforge`?
- [ ] Update `NEXT_ACTION.md` before ending.
- [ ] Transcripts still landing under `TRANSCRIPTS\HELIXFORGE` (archiver task running)?

## Product ports

Core: 8080–8085 · Products: 8101–8121 (see `shared_core::PRODUCT_CATALOG`).
Product **21 HelixPulse** (8121) is scaffold-only — full cluster **after** products 1–20.
HelixAnvil canonical home is `projects/helix-anvil` inside this monorepo; it remains
**portfolio-last** and is not activated for implementation until the monorepo
endgame is reached or the founder explicitly changes sequencing.
