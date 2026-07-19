# Next action

## Active: HELIXNETWORK-DURABILITY — eighth product through the gate

Prove the Foundation Integrity durability gate on HelixNetwork: fresh
crash, concurrency, and restore, verified locally and in CI. Eighth
product (after `helix-collab`, `helix-capital`, `helix-commerce`,
`helix-flow`, `helix-insights`, `helix-edu`, `helix-well`).

Goal doc: `docs/goals/HELIXNETWORK_DURABILITY.md`.

### Scope

`request_connection` ran its checks (profiles active, pair not blocked, no
existing row) in separate statements from the insert/revive write — a
profile deactivated or a pair blocked in between would silently accept the
request. This packet makes the whole check-then-act sequence one
transaction with the profile rows locked, and proves the gate.

### Definition of done

1. `NetworkRepo::request_connection` runs profile checks (locked
   `FOR UPDATE`), the blocked-pair check, the existing-row check, and the
   insert or revive update in one transaction.
2. Ignored Postgres integration tests `concurrent_accepts_single_winner`
   and `concurrent_requests_same_pair` pass locally and in CI.
3. `scripts/helix_network_durability.ps1` proves lifecycle, forced-kill
   survival, and schema restore roundtrip.
4. `network-durability` CI job in `.github/workflows/ci.yml`.
5. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Next action

Implement the packet, verify locally, push, and watch CI to green.
