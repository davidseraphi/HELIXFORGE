DROP INDEX IF EXISTS commerce_order_status_history_tenant_idx;
DROP INDEX IF EXISTS commerce_order_status_history_order_idx;
DROP TABLE IF EXISTS commerce.order_status_history;
DROP INDEX IF EXISTS commerce_orders_active_idx;
DROP INDEX IF EXISTS commerce_products_active_idx;
ALTER TABLE commerce.orders DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE commerce.products DROP COLUMN IF EXISTS deleted_at;
