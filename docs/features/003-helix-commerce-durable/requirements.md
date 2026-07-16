# 003 — HelixCommerce durable products & orders

### ADDED Requirements

#### Requirement
WHEN Postgres is available HelixCommerce SHALL persist products via
`helix_db::CommerceRepo` after migration `0005_commerce.sql`.

##### Scenario
- GIVEN docker Postgres is healthy
- WHEN `POST /v1/products` creates a SKU with price and inventory
- THEN the row exists in `commerce.products` and audit logs `product.create`

#### Requirement
WHEN an order is created the system SHALL lock product rows, reject insufficient
inventory, write order lines, and decrement inventory in one transaction.

##### Scenario
- GIVEN a product with inventory 10 and price_cents 1999
- WHEN `POST /v1/orders` with quantity 2 succeeds
- THEN order total_cents is 3998 and product inventory becomes 8

#### Requirement
WHEN Postgres is unavailable list endpoints SHALL report `durable: false`.
