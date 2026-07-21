# Next action

## Latest: SYNTHBIO EXTRACTED — hub-and-spoke is live

Phases 1–3 of `docs/architecture/SYNTHBIO_EXTRACTION_PLAN.md` are done:

- **Phase 1 (hub prep):** CODEOWNERS tripwires, `main` protected (required
  checks, admins enforced), console learned external products. PR #1.
- **Phase 2 (spoke skeleton):** `PROJECTS/synthbio` — component-first
  layout, three ecosystems (Rust/TS/Python), full CI standard,
  release-please, shadow Bazel, AGENTS.md + AI_POLICY.md. Private repo
  `davidseraphi/synthbio`, `main` protected.
- **Phase 3 (the move):** API + `synthbio_db` data layer + migration chain
  (`0001_baseline`) + standalone web app. Two-pool topology
  (`DATABASE_URL` hub trust plane / `SYNTHBIO_DATABASE_URL` product
  domain). Data copied with exact parity; product API on :8111, web on
  :3201, hub console deep-links (307). Hub-slim merged (PR #2, `6563ddc`).
- **Phase 4 (hardening):** in flight on `phase4-hardening` — RFC 9457
  errors, Idempotency-Key, cursor pagination, OpenAPI + contract CI.

## Active goal

**Phase 5 — cutover ritual (founder).** Checklist:
`PROJECTS/synthbio/docs/how-to/cutover.md`. Top item: **fix GitHub
billing** — Actions on the private synthbio repo are blocked ("recent
account payments have failed / spending limit"); product CI is staged to
go green the moment it's fixed. Then ratify review requirements, then
(optionally) drop the legacy hub `synthbio` schema.

## Next action

Founder runs the phase-5 checklist. Agent resume point after sign-off:
phase 6 — epigenomics data plane in the spoke (ENCODE/4DN ingest →
htsget serving → ATAC→Hi-C inference → validation as measurements).

---

## Archive: HELIXSYNTHBIO parity program + JOURNEYS engine (pre-extraction)

**Benchling parity (7 slices, commits `02b2d50`…`891653d`)**

- registry (0058): DSN accessions, immutable versions (DB triggers),
  risk review with CAS `expected_state`
- inventory (0059): SMP accessions, custody serialized `FOR UPDATE`
- measurements (0060): MSR + accept/reject verdicts
- claims (0061): CLM + evidence links + ELN notes
- signatures (0062): approval locks decision; `locked_at` on risk_cases
- client parity UI: `sb-theme`, SequenceMap (linear+circular),
  translate.ts full codon table, motifs.ts, LineageGraph
- CI green on run `29707873561`

**Journeys engine — the intent-first rethink (commits `4b98e63`,
`6bee1b5` (fmt), `d2b8e31` (UI))**

- `0063_synthbio_journeys.sql`: `synthbio.journeys` + `journey_stages`
- `RegistryRepo`: create/demo/set_route/link_stage_target/refresh/detail;
  guarded route choice (409), build-must-derive-from-design (422),
  auto-completing risk/test/evidence stages, teacher strings on read
- API: `/v1/journeys*`, `/v1/pathways`; 14 ignored tests green
- UI: Journeys first rail tab; seven-stage pipeline viz with per-stage
  guidance + actions; one-click demo journey
