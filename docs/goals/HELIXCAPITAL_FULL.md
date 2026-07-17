# HELIXCAPITAL-FULL

Move HelixCapital from durable scaffold to full second-wave depth (catalog
order 7, port `8107`).

## Scope

This packet closes the core account + journal lifecycle gaps in the current
scaffold: account lifecycle management, journal voiding with balance reversal,
trial-balance reporting, and durable balance snapshots.

## Definition of done

1. Migration `0042_capital_depth.sql` adds:
   - `closed_at`, `deleted_at` lifecycle columns on `capital.accounts`
   - `voided_at`, `void_reason` columns on `capital.journals`
   - `is_reversal` marker on `capital.journal_lines`
   - Partial active-account index, status indexes, kind/currency index
   - `capital.account_balance_history` snapshot table
2. `CapitalRepo` gains:
   - `update_account`
   - `close_account` (balance must be zero)
   - `reopen_account`
   - `soft_delete_account` (rejected if journal lines exist)
   - `void_journal` (transactional balance reversal + reversal lines)
   - `get_trial_balance`
   - `record_balance_snapshot`
3. Domain APIs:
   - `PATCH /v1/accounts/{id}`
   - `POST /v1/accounts/{id}/close`
   - `POST /v1/accounts/{id}/reopen`
   - `POST /v1/accounts/{id}/delete`
   - `POST /v1/journals/{id}/void`
   - `GET /v1/reports/trial-balance`
   - `POST /v1/reports/balance-snapshot`
4. `GET /v1/domain/status` returns `phase: wave2_w7` and capability planes.
5. Audit + metering + NATS on all mutating operations.
6. Unit/integration tests:
   - in-process validation tests for unbalanced sides, invalid side, zero amount
   - ignored Postgres integration test for account lifecycle + journal void + trial balance
7. `scripts/helix_capital_smoke.ps1` passes locally and in CI.
8. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29621350739`
- Smoke script: `scripts/helix_capital_smoke.ps1`
- Unit tests: `4/4 PASS` (plus one ignored Postgres integration test)

## Out of scope

- Multi-currency balancing / FX conversion.
- Chart-of-accounts hierarchy, cost centers, budgets.
- Bank feeds, reconciliation, tax codes, payroll, fixed assets.
- Web UI changes.
