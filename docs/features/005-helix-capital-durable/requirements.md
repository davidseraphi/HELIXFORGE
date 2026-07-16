# 005 — HelixCapital durable accounts & journals

### ADDED Requirements

#### Requirement
WHEN Postgres is available HelixCapital SHALL persist accounts via
`helix_db::CapitalRepo` after migration `0007_capital.sql`.

##### Scenario
- GIVEN docker Postgres is healthy
- WHEN `POST /v1/accounts` creates Cash and Revenue
- THEN both rows exist with balance_cents 0

#### Requirement
WHEN posting a journal the system SHALL reject unbalanced lines and, when
balanced, update account balances in one transaction (debit +, credit −).

##### Scenario
- GIVEN Cash and Revenue accounts
- WHEN `POST /v1/journals` with debit Cash 5000 and credit Revenue 5000
- THEN Cash balance is 5000 and Revenue balance is −5000
