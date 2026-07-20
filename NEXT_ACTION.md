# Next action

## Latest: HELIXSYNTHBIO parity program + JOURNEYS engine closed

SynthBio is no longer a thin app. Two programs landed on top of the
durability gate:

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

- `0063_synthbio_journeys.sql`: `synthbio.journeys` (JRN accession,
  pathway_key, route_choice, status, current_stage) + `journey_stages`
- `RegistryRepo`: `create_journey`, `demo_journey` (lavender balm,
  end-to-end), `set_route` (single guarded choice, 409 on re-choice),
  `link_stage_target` (build sample must derive from the journey's
  design — 422 otherwise), `refresh_journey` (auto-completes
  risk/test/evidence), `journey_detail` (refreshes on every read;
  stages zipped with live checks — the "teacher" missing strings)
- API: `/v1/journeys*`, `/v1/pathways`; 14 ignored tests green incl.
  `journey_full_walk` + `journey_demo_end_to_end`
- UI: Journeys is the first rail tab of SynthBio; detail page
  `/products/helix-synthbio/journeys/[id]` — seven-stage pipeline viz,
  per-stage guidance + action forms, one-click demo journey

### Active goal

**SYNTHBIO EXTRACTION (hub-and-spoke).** SynthBio graduates out of the
monorepo into `PROJECTS/synthbio` as the first standalone product repo.
Canonical plan: `docs/architecture/SYNTHBIO_EXTRACTION_PLAN.md`.
Decision recorded: DECISION_LOG 2026-07-20.

### Next action

Phase 1 — Hub prep: CODEOWNERS + branch rulesets on HELIXFORGE `main`;
console learns `external: true` product entries (launcher deep-link);
synthbio catalog entry marked external. Exit: hub CI green with synthbio
marked external-but-present. Then phases 2–5 per the plan; phase 6 resumes
the vision roadmap (epigenomics data plane first) inside the spoke.
