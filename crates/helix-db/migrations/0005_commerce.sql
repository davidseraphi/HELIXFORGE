-- HelixCommerce durable catalog + orders (reuse helix_core tenancy)
CREATE SCHEMA IF NOT EXISTS commerce;

CREATE TABLE IF NOT EXISTS commerce.products (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    sku TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    price_cents BIGINT NOT NULL CHECK (price_cents >= 0),
    currency TEXT NOT NULL DEFAULT 'USD',
    inventory INT NOT NULL DEFAULT 0 CHECK (inventory >= 0),
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, sku)
);

CREATE INDEX IF NOT EXISTS commerce_products_tenant_idx ON commerce.products (tenant_id);
CREATE INDEX IF NOT EXISTS commerce_products_status_idx ON commerce.products (tenant_id, status);

CREATE TABLE IF NOT EXISTS commerce.orders (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    currency TEXT NOT NULL DEFAULT 'USD',
    total_cents BIGINT NOT NULL DEFAULT 0 CHECK (total_cents >= 0),
    customer_email TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS commerce_orders_tenant_idx ON commerce.orders (tenant_id);
CREATE INDEX IF NOT EXISTS commerce_orders_status_idx ON commerce.orders (tenant_id, status);

CREATE TABLE IF NOT EXISTS commerce.order_items (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL REFERENCES commerce.orders(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    product_id UUID NOT NULL REFERENCES commerce.products(id),
    sku TEXT NOT NULL,
    name TEXT NOT NULL,
    quantity INT NOT NULL CHECK (quantity >= 1),
    unit_price_cents BIGINT NOT NULL CHECK (unit_price_cents >= 0),
    line_total_cents BIGINT NOT NULL CHECK (line_total_cents >= 0)
);

CREATE INDEX IF NOT EXISTS commerce_order_items_order_idx ON commerce.order_items (order_id);
CREATE INDEX IF NOT EXISTS commerce_order_items_tenant_idx ON commerce.order_items (tenant_id);
