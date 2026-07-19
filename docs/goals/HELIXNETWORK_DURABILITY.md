# HELIXNETWORK-DURABILITY

Prove the Foundation Integrity durability gate on HelixNetwork: fresh
crash, concurrency, and restore, verified locally and in CI. Eighth product
through the gate (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`).

## Scope

`request_connection` ran its checks (profiles active, pair not blocked, no
existing row) in separate statements from the insert/revive write — a
profile deactivated or a pair blocked in between would silently accept the
request. This packet makes the whole check-then-act sequence one
transaction with the profile rows locked, and proves the gate.

## Definition of done

1. `NetworkRepo::request_connection` runs profile checks (locked
   `FOR UPDATE`), the blocked-pair check, the existing-row check, and the
   insert or revive update in one transaction.
2. New ignored Postgres integration tests (run in the
   `network-durability` CI job):
   - `concurrent_accepts_single_winner` — N concurrent accepts of one
     connection produce exactly one success; the rest are rejected; the
     connection ends accepted.
   - `concurrent_requests_same_pair` — N concurrent requests for the same
     ordered pair produce exactly one success; the rest conflict; exactly
     one connection row exists.
3. `scripts/helix_network_durability.ps1`:
   - create profiles, request, accept, verify connection
   - acknowledge a connection, force-kill the API, restart, and verify the
     connection is fully present
   - `pg_dump` of the `network` schema restores into a scratch database
     with equal profile/connection/opportunity counts and equal content
     hashes
4. `network-durability` CI job in `.github/workflows/ci.yml` running the
   ignored integration tests and the proof script.
5. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29668195166` (**HelixNetwork durability gate** job green)
- Proof script: `scripts/helix_network_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on network writes.
- Durability gates for other products (each needs its own named packet).
