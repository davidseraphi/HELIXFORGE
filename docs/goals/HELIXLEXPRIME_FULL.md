# HELIXLEXPRIME-FULL

Move HelixLex Prime from thin durable scaffold to full second-wave depth
(catalog order 12, port `8112`).

## Scope

This packet closes the core matter + filing lifecycle gaps in the current
scaffold: a matter open/close lifecycle with a close guard, a filing
file/withdraw lifecycle, and a lex summary report.

## Definition of done

1. Migration `0047_lex_depth.sql` adds:
   - `opened_at`, `closed_at`, `deleted_at` lifecycle columns on `lex.matters`
   - `updated_at`, `filed_at`, `withdrawn_at`, `deleted_at` columns on
     `lex.filings`
   - Backfill of legacy scaffold filing status `open` to `draft`
   - Partial active-matter and active-filing indexes
2. `LexRepo` gains:
   - `update_matter`, `open_matter`, `close_matter` (rejected while draft
     filings remain), `reopen_matter`, `soft_delete_matter`,
     `restore_matter`
   - `update_filing`, `file_filing`, `withdraw_filing`,
     `soft_delete_filing`, `restore_filing` (all verified against the parent
     matter)
   - `get_lex_summary` (per-matter filing counts by status)
3. Domain APIs:
   - `PATCH /v1/matters/{id}`
   - `POST /v1/matters/{id}/open`
   - `POST /v1/matters/{id}/close`
   - `POST /v1/matters/{id}/reopen`
   - `POST /v1/matters/{id}/delete`
   - `POST /v1/matters/{id}/restore`
   - `PATCH /v1/matters/{id}/filings/{filing_id}`
   - `POST /v1/matters/{id}/filings/{filing_id}/file`
   - `POST /v1/matters/{id}/filings/{filing_id}/withdraw`
   - `POST /v1/matters/{id}/filings/{filing_id}/delete`
   - `POST /v1/matters/{id}/filings/{filing_id}/restore`
   - `GET /v1/reports/lex-summary`
4. `GET /v1/domain/status` returns `phase: wave2_w12` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for matter/filing status transitions
   - ignored Postgres integration test for the close guard, matter
     lifecycle, filing lifecycle, and lex summary
7. `scripts/helix_lex_prime_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29646308966`
- Smoke script: `scripts/helix_lex_prime_smoke.ps1`
- Unit tests: `2/2 PASS` (plus one ignored Postgres integration test)

## Out of scope

- Matter authority model, legal knowledge graph, deadline engine, filing
  integration, citation checker.
- Web UI changes.
