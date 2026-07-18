# HELIXCOMMERCE-DURABILITY

Prove the Foundation Integrity durability gate on HelixCommerce: fresh crash,
concurrency, and restore, verified locally and in CI. Third product through
the gate (after `helix-collab`, `helix-capital`).

## Scope

Commerce order/cancel paths are already transactional with `FOR UPDATE`
locks and status guards, and the oversell race is already proven by the
existing `two_buyers_cannot_oversell_last_unit` test. This packet adds the
cancel-side race proof, the crash proof, and the restore proof.

## Definition of done

1. New ignored Postgres integration test (run in CI):
   - `concurrent_cancels_single_winner` — N concurrent cancels of one order
     produce exactly one success; the rest fail with Validation; inventory
     is restored exactly once.
2. `scripts/helix_commerce_durability.ps1`:
   - create product, order, cancel, verify inventory consistency
   - acknowledge an order, force-kill the API, restart, and verify the
     order and the inventory reservation are present and consistent
   - `pg_dump` of the `commerce` schema restores into a scratch database
     with equal product/order/item counts and equal content hashes
3. `commerce-durability` CI job in `.github/workflows/ci.yml` that also runs
   the ignored integration tests (`cargo test -p helix_commerce_api --
   --ignored`); the same ignored-test step is added to the existing
   `capital-durability` job so its race proofs run in CI too.
4. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29664024211` (**HelixCommerce durability gate** job green;
  ignored integration tests also run in CI in both durability jobs)
- Proof script: `scripts/helix_commerce_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on commerce writes.
- Idempotency keys on order creation.
- Durability gates for other products (each needs its own named packet).
