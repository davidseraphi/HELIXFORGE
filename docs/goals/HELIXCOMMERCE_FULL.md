# HELIXCOMMERCE-FULL

Move HelixCommerce from durable scaffold to full second-wave depth (catalog
order 5, port `8105`).

## Scope

This packet closes the highest-risk commerce gap identified in the product
sheet: exact multi-currency order truth and atomic stock/order writes.

## Definition of done

1. Migration `0040_commerce_depth.sql` adds soft-delete columns, active
   indexes, and order-state history/indexes.
2. `CommerceRepo` enforces:
   - mixed-currency carts rejected with a clear error;
   - atomic inventory reservation + order creation in one transaction;
   - order cancel restores inventory and transitions status;
   - product soft-delete and restore.
3. Domain APIs:
   - `POST /v1/products/{id}/delete` and `POST /v1/products/{id}/restore`
   - `POST /v1/orders/{id}/cancel`
   - `POST /v1/orders/{id}/status`
4. `GET /v1/domain/status` returns `phase: wave2_w3` and capability planes.
5. Unit/integration tests:
   - exact money arithmetic
   - mixed-currency rejection
   - two-buyer race for last unit (ignored data-plane test)
6. `scripts/helix_commerce_smoke.ps1` passes locally and in CI.
7. `cargo test --workspace --all-features` and `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed** and CI-proven on `main`.
- Commit: `6bb1a65`
- CI run: `29599963866` (all green, including **HelixCommerce smoke**)

## Out of scope

- Carts, payment intents, fulfilment, returns, refunds, buyer UI, channels,
  reconciliation, and multi-channel publishing.
