# SynthBio Extraction Plan

Status: approved direction, phases 0–1 ready to start
Date: 2026-07-20
Owner: founder
Driver: Kimi (this plan), then per-slice execution

## 0. Why this document exists

SynthBio is a **full biology workbench** — an AI-native "all things biology"
platform (data plane, compute plane, model plane, analysis, literature,
agent interface), with the registry/custody/claims/journeys work shipped so
far as its **trust substrate, not its ceiling**. It is the first of several
science modules (math, physics, chemistry follow on the same substrate).
Ambition: bedrock platform for the biotech industry in Uganda and Africa.

The monorepo made 21 thin products; depth requires product independence.
This plan extracts SynthBio from HELIXFORGE as the first **spoke**, proves
the hub-and-spoke pattern, and encodes the engineering standard every
future module inherits. Integration philosophy: **integration without one
app** — shared identity, shared trust plane, cross-product federation.
Never a re-merge.

Standing rules for all execution under this plan:

- **No underscoping.** Engineering honesty about cost/sequencing is
  required; shrinking the vision is not permitted.
- **Prompts persuade, pipelines enforce.** Every claim of done cites an
  externally checkable artifact (CI run ID, live URL, migration job).
- Foundation first: nothing in this plan may weaken the durability gates.

---

## 1. Target topology

```
PROJECTS/
├── HELIXFORGE/            # the HUB — platform company
│   ├── crates/            # service-kit, shared-core, helix-db, audit-log,
│   │                      # job-engine, nats-client, observability, ...
│   ├── services/          # gateway, auth-adapter, vault, billing, agent-hub
│   ├── apps/console       # launcher + 20 remaining products
│   └── infra/             # shared compose/k8s, Ory, Postgres, NATS, MinIO
│
└── synthbio/              # the SPOKE — standalone product repo (new)
    ├── services/api/      # Rust axum API (today's helix_synthbio_api)
    ├── crates/            # product-owned Rust data layer (from helix-db)
    ├── apps/web/          # standalone Next.js app (today's console slice)
    ├── ml/services/       # FastAPI model-inference services (phase 6+)
    ├── ml/pipelines/      # Nextflow/python pipelines (phase 6+)
    ├── db/migrations/     # product-owned migration chain
    ├── infra/{docker,helm}
    ├── docs/{adr,how-to,reference,explanation}
    ├── tests/{contract,e2e}
    ├── scripts/
    └── .github/{workflows,CODEOWNERS}
```

The hub keeps: the trust plane (auth, tenancy, audit, vault, billing),
job-engine, the console as **launcher** (apps grid → per-product URLs), and
the 20 other products until each graduates the same way.

---

## 2. Stack audit (vision-driven)

Research-grounded, hype flagged vs proven. Sources in appendix.

### Keep (proven, load-bearing)

| Choice | Why |
|---|---|
| Rust + axum for stateful APIs | safety/perf; entire trust substrate already written |
| Postgres + sqlx | system of record; durability gates green |
| MinIO | S3-compatible object store, self-hosted; range-request data plane builds on it |
| NATS | event bus, no lock-in |
| Ory (auth) | self-hosted identity, no lock-in |
| Next.js + TS + pnpm | product web standard |
| `service_kit` / `shared_core` / `audit_log` / `job-engine` | the platform contract surface |
| Nygard ADRs (`docs/adr/NNNN`) | already convention; keep in both repos |

### Add (the workbench planes)

| Plane | Choice | Rationale |
|---|---|---|
| ML registry | **MLflow** (self-hosted, Postgres + MinIO) + checkpoint mirror (HF download → MinIO, checksums in registry) | de-facto OSS standard; "self-hosted HF Hub" does not exist — mirror, don't pretend |
| Model serving | **FastAPI-wrapped PyTorch** behind the API; **vLLM** for LLM-shaped workloads; Triton migration path per-model when GPU utilization demands | what biopharma actually runs; TorchServe is dead (archived 2025-08); Ray Serve/BentoML too heavy now |
| Pipelines | **Nextflow as executor** (GPLv3 engine invoked as CLI, never linked; Seqera Platform never required), `job-engine` remains the **control plane** (API/tenancy/audit/queueing) | nf-core = 124+ production pipelines, the ecosystem; rebuilding DAG semantics is years |
| Data ingestion ETL | job-engine tasks or light Python DAG (Prefect-class), **not** Dagster/Flyte/Airflow | right tool weight for pull jobs |
| Genomics data plane | Rust `noodles` + `htsgetr` (htsget protocol, S3/MinIO backend); Python sidecars: pyBigWig, cooler/hic-straw, pairtools, anndata/scanpy; canonical layout = immutable content-addressed objects on MinIO, range-indexed formats (BGZF/R-tree/mcool), catalog + checksums + provenance in Postgres | the UCSC/ENCODE serving pattern — proven, low-bandwidth by design |
| Python quality bar | FastAPI + pydantic v2 strict + uv + ruff + mypy-strict + pytest; **Python owns compute, Rust owns state — Python never writes the system-of-record DB, it calls the Rust API** | one rule keeps operational looseness out of the trust substrate |
| Frontend reach | PWA console: service workers, queue-and-sync mutations, resumable (tus) uploads | offline/low-bandwidth Africa tier |
| Deployment tiers | (a) single-box compose, (b) k3s single-node, (c) k3s HA; CI-built offline bundle (image tarballs + pip/apt mirrors + reference-data manifest) | k3s air-gap is documented practice; LAN-first, cloud optional sync |
| GPU | job-engine queues onto GPU nodes directly at 1–2 nodes; k3s + NVIDIA GPU Operator time-slicing at 3+; KAI Scheduler only under real contention | honest sizing |

### Avoid (hype / premature / dead)

TorchServe (dead) · "self-hosted HuggingFace Hub" (doesn't exist) · Ray
Serve / BentoML at our scale · Flyte / Kubeflow (control-plane tax) ·
Dagster/Airflow here · TileDB-SOMA as primary format today (evaluate at
atlas scale; h5ad interchange, Zarr for hot serving) · BioNeMo NIMs as
source of truth (fine as accelerators) · Pact broker · GitFlow · SLSA L3 ·
self-hosted runners.

### Deferred, with explicit adoption triggers (not rejected)

**Bazel / Nx / meta-build orchestrators.** Today the native toolchains
*are* the build graph (cargo workspace, pnpm+turbo, uv) and are already
incremental; the primary dev machine is Windows, which is Bazel's weakest
platform tier (rules_rust on MSVC carries real friction); and hub-and-spoke
deliberately keeps each repo moderate — Bazel's sweet spot is the giant
single monorepo. **Adopt Bazel when any trigger fires:** one repo exceeds
~40 polyglot components · CI wall-time > ~40 min despite path-filtering and
caching · heavy cross-language codegen arrives (protobuf/gRPC across
Rust+TS+Python) · remote-cache CI savings exceed the ops cost. **Door held
open for free:** lockfiles everywhere, pinned toolchains, hermetic CI
steps, no network in tests — the same hygiene SLSA L2 and the air-gap
bundle already require, so the later migration is cheap.

---

## 3. Product repository standard (the "perfect repo" target)

Component-first top level, domain-first inside components (the PostHog/
Supabase pattern — verified against live repos, not layout dogma):

- One workspace per ecosystem: root `Cargo.toml`, `pnpm-workspace.yaml`,
  `uv` workspace. Wired by a root `justfile`. Meta-build orchestrator only
  when a §2 trigger fires.
- `db/migrations/` owned by exactly one tool (sqlx), one runner.
- Every top-level folder has a README naming its owner and toolchain.
- Docs Diátaxis-sorted: `docs/reference/` (OpenAPI-generated — never
  hand-written API docs), `docs/how-to/` (runbooks), `docs/adr/` (Nygard
  five-section, sequential, superseded never deleted), `docs/explanation/`.
  Rule: **every doc names the moment someone reads it, or it isn't written.**
- Root: `README.md` (one screen, <10 commands to running), `CHANGELOG.md`
  (release-please generated), `AGENTS.md` (lean: commands, invariants,
  forbidden zones), `AI_POLICY.md`, `CLAUDE.md` pointer.

### Engineering gates (CI as the only source of truth)

- Trunk-based: agents push short-lived branches, open PRs; `main` merges
  require the full matrix — **no bypass, human included**.
- Check matrix per ecosystem: `cargo fmt --check`, `clippy -D warnings`,
  `cargo test`; `tsc --noEmit` + eslint; `ruff` + `mypy --strict` + pytest;
  migration roundtrip job (up → test → down → up); `helm lint`; gitleaks;
  osv-scanner; zizmor; SHA-pinned actions with minimal token permissions.
- CODEOWNERS: `db/`, `infra/`, `.github/`, auth code → founder. This is a
  tripwire, not bureaucracy: agent-authored changes there require founder
  sign-off.
- API discipline: `/v1` prefix; RFC 9457 `problem+json` from one shared
  middleware; `Idempotency-Key` on mutating POSTs (DB-backed replay);
  cursor pagination only; 429 + `Retry-After`; RFC 9745/8594 deprecation
  headers when the time comes. OpenAPI via `utoipa` (generated from axum,
  cannot drift), Vacuum-linted in CI, Schemathesis property tests as the
  contract layer.
- Releases: conventional commits + release-please manifest mode → semver,
  changelog, tags across all three ecosystems; SLSA L2 provenance + cosign
  on release containers from `v0.1.0`.
- Testing: native unit tests per language; Testcontainers integration for
  every SQL-facing service; migration roundtrips; ≤10 Playwright journeys;
  no coverage-percentage theater. Guiding question: *can an agent merge a
  schema-breaking change unnoticed?* — if yes, that's the next test.

---

## 4. Platform contract v1 (prerequisite for a clean cut)

What the hub provides to products, and how the spoke consumes it:

| Capability | Mechanism today | Contract after extraction |
|---|---|---|
| Service bootstrap | `service_kit::ServiceBuilder` crate dep | crate dep via **path dep during co-evolution** (`../HELIXFORGE/crates/...`), product CI clones both repos; pin by **git rev** once stable |
| Tenancy/principal | `shared_core::tenancy` | same crate-dep path; `RequireAuth` semantics unchanged |
| Audit chain | `audit_log` crate | crate dep now; HTTP sink later if the boundary hardens |
| Agent registration | `agent_framework::AgentSpec` | crate dep |
| Jobs | `job-engine` | crate dep now; API client in phase 6 (Nextflow integration) |
| Events | NATS `helix.synthbio.*` prefix | unchanged — NATS is shared infra, prefixes are the contract |
| Identity (prod) | dev headers | Ory SSO — hub-owned, product consumes; unchanged by extraction |
| Console | hosts product UI | console gains `external: true` product entries: launcher card deep-links to the product's own URL (dev: `http://localhost:3201`) |

Nothing in the hub imports synthbio code (verified by inventory); the seam
is exactly: 3 helix-db modules, the console slice, 2 CI jobs, 3 scripts.

---

## 5. Database split

- Product gets its own **database** `synthbio` (not just schema) on the same
  Postgres instance initially; physical separation later if load demands.
- Product migration chain starts fresh: `0001_baseline.sql` = the squashed
  current state (synthbio slice of `0010` + `0046` + `0058`–`0063`), then
  product-owned numbering. The hub chain **keeps the legacy migrations as
  applied history** (they're already in existing deployments; never edit
  applied migrations).
- Data move: `pg_dump --schema=synthbio` → restore into the new DB (the
  durability script already proves this roundtrip), then the hub schema is
  dropped only after the product DB is verified serving.
- No cross-database FKs (already true — verified: zero cross-schema refs).

---

## 6. AI-honesty & verification layer (folded in, both repos)

Mapped control → threat:

| Control | Threat it kills |
|---|---|
| Protected `main` + required matrix, no bypass | "trust me, it's green" — truth comes from runners the agent doesn't control |
| CODEOWNERS on `db/`, `infra/`, `.github/`, auth | agent moving the goalposts (weakening gates, rewriting migrations) |
| Per-agent bot identities, least-privilege tokens (push branches, open PRs, no force-push, no admin) | unattributed or irreversible actions |
| Evidence-over-assertion rule in AGENTS.md (every done cites CI run ID / live URL) | narrative drift |
| Writer/reviewer pattern (fresh-context agent reviews the diff) | agent grading its own homework |
| gitleaks in CI + pre-commit; secrets injected at runtime, never stored | secret exfiltration via commits |
| `AI_POLICY.md`: never touch secrets, never force-push, never destructive DB commands — mirrored by hooks | catastrophic irreversibility |
| Sandbox tier-2 (later): containerized agent runs, repo-only mount, egress allowlist | host-level blast radius |
| Verification service (later): job-engine runs gates, results signed into `audit_log` | forgeable test output — "green" becomes a tamper-evident platform artifact |

Honest limit, recorded: these controls make dishonesty *ineffective*, not
impossible. The residual risk is competent-looking wrongness passing weak
gates — countered by founder review and tests written as executable spec.

---

## 7. Phases and exit criteria

**Phase 0 — Records (done with this doc).**
ADRs in both repos (hub: extraction decision; product: ADR-0001 repo
standard). DECISION_LOG entry. NEXT_ACTION update.

**Phase 1 — Hub prep.**
CODEOWNERS + rulesets on HELIXFORGE `main`; console learns
`external: true` product entries (launcher deep-link); catalog entry for
synthbio marked external with URL; hub CI keeps 20-product gates green.
Exit: hub CI green with synthbio marked external-but-present.

**Phase 2 — Spoke skeleton.**
New repo `PROJECTS/synthbio`: layout per §3, workspaces, justfile,
README, ADR-0001, AGENTS.md + AI_POLICY.md, full CI matrix (with empty
Python placeholder), branch protection + CODEOWNERS, release-please.
Exit: empty-repo CI green end-to-end.

**Phase 3 — The move.**
`services/api` ← `projects/helix-synthbio/backend`; `crates/` ← the three
helix-db synthbio modules (product-owned crate, e.g. `synthbio_db`);
`apps/web` ← console `components/synthbio/*` + 4 pages + `.sb-theme` block,
standalone Next app on :3201 with its own BFF (dev-header injection, same
pattern as hub proxy); scripts (smoke, durability, bundle verifier) move;
CI jobs `synthbio-durability`/`synthbio-smoke` move and rewire against the
hub-as-dependency; DB split per §5; hub edits: workspace member removed,
console slice removed (replaced by external launcher card), helix-db
modules dropped from `lib.rs`, catalog marked external.
Exit: both repos' CI green; product API serves the new DB; product web at
:3201 reaches feature parity with today's console slice; hub console
launcher opens it.

**Phase 4 — Product hardening to the new standard.**
utoipa OpenAPI + Vacuum + Schemathesis; RFC 9457 error middleware;
Idempotency-Key on mutating POSTs; cursor pagination sweep; Testcontainers
integration suite; release-please live.
Exit: contract tests in CI; API reference generated; `v0.1.0` tagged.

**Phase 5 — Cutover ritual.**
Founder clicks through the old console paths (now deep-links), the new app,
and one full journey end-to-end; transcripts archived; both NEXT_ACTIONs
updated.

**Phase 6+ — The vision resumes, now in the spoke.**
Epigenomics data plane (ENCODE/4DN ingest → htsget serving → ATAC→Hi-C
inference with published weights → validation metrics as measurements →
claims with evidence links); MLflow registry + checkpoint mirror;
job-engine → Nextflow integration slice; SIRA-class retrieval when the
corpus lands; agent interface as first-class surface. Each is its own
packet against this repo standard.

---

## 8. Risks / open questions

- **Path-dep drift** between hub and spoke during co-evolution → mitigated
  by product CI cloning the hub at a recorded ref; move to git-rev pins at
  contract freeze (phase 4 end).
- **Two migrators, one Postgres** → solved by separate databases (§5); never
  share `_sqlx_migrations`.
- **Empty-frontend trap** — the real frontend is the console slice, not the
  `web/` scaffold; phase 3 moves the real one.
- **Nextflow GPLv3** — invoke as CLI only, never link; re-verify license
  text at integration time.
- **Solo-operator CODEOWNERS** is a tripwire, not a gate against self — the
  founder can still approve; its job is making agent changes *visible*.
- **DNAnexus/UK Biobank RAP** architecture claims are from general
  knowledge (primary-source verification failed) — treat as directional.

---

## Appendix — load-bearing sources

- Layout/governance: PostHog & Supabase repos (live); golang-standards
  disclaimer; monorepo.tools; Sourcegraph build-tools roundup 2026.
- Docs: diataxis.fr; Nygard ADR post (cognitect.com, 2011); GitLab public
  runbooks; keepachangelog.com.
- CI/CD & supply chain: trunkbaseddevelopment.com; DORA 2024 report;
  release-please; gitleaks; osv-scanner; zizmor; SLSA spec; OpenSSF
  Scorecard.
- API: RFC 9457 (problem+json), RFC 9745 + 8594 (deprecation/sunset),
  IETF drafts for Idempotency-Key & RateLimit headers (still drafts);
  Stripe versioning post; Google AIPs; Zalando guidelines.
- Testing: Fowler practical test pyramid; Testcontainers; Schemathesis.
- AI governance: Anthropic Claude Code best practices (verification loop,
  hooks over prompts, fresh-context review); SAFE-AI (arXiv 2508.11824).
- Bio stack: nf-core Genome Biology 2025 paper; Seqera k8s blog;
  pytorch/serve archive notice; NVIDIA GPU Operator sharing docs & KAI
  Scheduler release; hts-specs + htsget + htsgetr; pyBigWig; cooler/
  pairtools/hic-straw; TileDB-SOMA docs; k3s air-gap guides; Kolibri
  offline pattern; Benchling engineering blog + warehouse docs; Ginkgo
  Cloud Lab launch coverage; Recursion BioHive-2 release.
