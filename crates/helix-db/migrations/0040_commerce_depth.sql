-- HelixCommerce W3 depth: order cancel tracking + query support
CREATE SCHEMA IF NOT EXISTS commerce;

ALTER TABLE commerce.orders
    ADD COLUMN IF NOT EXISTS cancelled_at TIMESTAMPTZ;

-- Fast lookups for pending orders that may be cancelled/fulfilled
CREATE INDEX IF NOT EXISTS commerce_orders_pending_idx
    ON commerce.orders (tenant_id, created_at DESC)
    WHERE status = 'pending';

-- Active product list used during order creation
CREATE INDEX IF NOT EXISTS commerce_products_active_idx
    ON commerce.products (tenant_id, sku)
    WHERE status = 'active';
