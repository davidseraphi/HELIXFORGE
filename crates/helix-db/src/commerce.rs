//! HelixCommerce product catalog + order persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub currency: String,
    pub inventory: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub status: String,
    pub currency: String,
    pub total_cents: i64,
    pub customer_email: String,
    pub metadata: serde_json::Value,
    pub items: Vec<OrderItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub line_total_cents: i64,
}

#[derive(Debug, Clone)]
pub struct OrderLineInput {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(sqlx::FromRow)]
struct ProductRow {
    id: Uuid,
    tenant_id: Uuid,
    sku: String,
    name: String,
    description: String,
    price_cents: i64,
    currency: String,
    inventory: i32,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl ProductRow {
    fn into_product(self) -> Product {
        Product {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            sku: self.sku,
            name: self.name,
            description: self.description,
            price_cents: self.price_cents,
            currency: self.currency,
            inventory: self.inventory,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
        }
    }
}

const PRODUCT_SELECT: &str = r#"
    SELECT id, tenant_id, sku, name, description, price_cents, currency,
           inventory, status, metadata, created_at
    FROM commerce.products
"#;

#[derive(Clone)]
pub struct CommerceRepo {
    pool: PgPool,
}

impl CommerceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_products(&self, tenant_id: TenantId) -> HelixResult<Vec<Product>> {
        let rows: Vec<ProductRow> = sqlx::query_as(&format!(
            "{PRODUCT_SELECT} WHERE tenant_id = $1 ORDER BY created_at DESC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce list products: {e}")))?;
        Ok(rows.into_iter().map(ProductRow::into_product).collect())
    }

    pub async fn create_product(
        &self,
        tenant_id: TenantId,
        sku: &str,
        name: &str,
        description: &str,
        price_cents: i64,
        currency: &str,
        inventory: i32,
        metadata: serde_json::Value,
    ) -> HelixResult<Product> {
        if price_cents < 0 {
            return Err(HelixError::validation("price_cents must be >= 0"));
        }
        if inventory < 0 {
            return Err(HelixError::validation("inventory must be >= 0"));
        }
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let currency = if currency.trim().is_empty() {
            "USD"
        } else {
            currency.trim()
        };
        sqlx::query(
            r#"
            INSERT INTO commerce.products
                (id, tenant_id, sku, name, description, price_cents, currency,
                 inventory, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'active',$9,$10,$10)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(sku)
        .bind(name)
        .bind(description)
        .bind(price_cents)
        .bind(currency)
        .bind(inventory)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce create product: {e}")))?;
        Ok(Product {
            id,
            tenant_id,
            sku: sku.into(),
            name: name.into(),
            description: description.into(),
            price_cents,
            currency: currency.into(),
            inventory,
            status: "active".into(),
            metadata,
            created_at,
        })
    }

    pub async fn get_product(
        &self,
        tenant_id: TenantId,
        product_id: Uuid,
    ) -> HelixResult<Option<Product>> {
        let row: Option<ProductRow> = sqlx::query_as(&format!(
            "{PRODUCT_SELECT} WHERE tenant_id = $1 AND id = $2"
        ))
        .bind(tenant_id.as_uuid())
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce get product: {e}")))?;
        Ok(row.map(ProductRow::into_product))
    }

    pub async fn update_product(
        &self,
        tenant_id: TenantId,
        product_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        price_cents: Option<i64>,
        inventory_delta: Option<i32>,
        status: Option<String>,
    ) -> HelixResult<Product> {
        if let Some(p) = price_cents {
            if p < 0 {
                return Err(HelixError::validation("price_cents must be >= 0"));
            }
        }

        let mut builder = sqlx::QueryBuilder::new("UPDATE commerce.products SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(n) = name {
            builder.push(", name = ");
            builder.push_bind(n);
        }
        if let Some(d) = description {
            builder.push(", description = ");
            builder.push_bind(d);
        }
        if let Some(p) = price_cents {
            builder.push(", price_cents = ");
            builder.push_bind(p);
        }
        if let Some(delta) = inventory_delta {
            builder.push(", inventory = inventory + ");
            builder.push_bind(delta);
        }
        if let Some(s) = status {
            builder.push(", status = ");
            builder.push_bind(s);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(product_id);
        builder.push(" RETURNING id, tenant_id, sku, name, description, price_cents, currency, inventory, status, metadata, created_at");

        let row: Option<ProductRow> = builder
            .build_query_as::<ProductRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("commerce update product: {e}")))?;

        row.map(ProductRow::into_product)
            .ok_or_else(|| HelixError::not_found("product not found"))
    }

    pub async fn cancel_order(&self, tenant_id: TenantId, order_id: Uuid) -> HelixResult<Order> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("commerce begin: {e}")))?;

        let row: Option<String> = sqlx::query_scalar(
            "SELECT status FROM commerce.orders WHERE tenant_id = $1 AND id = $2 FOR UPDATE",
        )
        .bind(tenant_id.as_uuid())
        .bind(order_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce lock order: {e}")))?;

        let status = row.ok_or_else(|| HelixError::not_found("order not found"))?;
        if status != "pending" {
            return Err(HelixError::validation(format!(
                "cannot cancel order with status {}",
                status
            )));
        }

        let cancelled_at = Utc::now();
        sqlx::query(
            "UPDATE commerce.orders SET status = 'cancelled', cancelled_at = $1, updated_at = $1 WHERE tenant_id = $2 AND id = $3",
        )
        .bind(cancelled_at)
        .bind(tenant_id.as_uuid())
        .bind(order_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce cancel order: {e}")))?;

        let items = self.load_items(order_id).await?;
        for item in &items {
            sqlx::query(
                "UPDATE commerce.products SET inventory = inventory + $1, updated_at = $2 WHERE tenant_id = $3 AND id = $4",
            )
            .bind(item.quantity)
            .bind(cancelled_at)
            .bind(tenant_id.as_uuid())
            .bind(item.product_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("commerce restore inventory: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("commerce commit cancel: {e}")))?;

        let order = self
            .get_order(tenant_id, order_id)
            .await?
            .ok_or_else(|| HelixError::not_found("order not found"))?;
        Ok(order)
    }

    pub async fn list_orders(&self, tenant_id: TenantId) -> HelixResult<Vec<Order>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            status: String,
            currency: String,
            total_cents: i64,
            customer_email: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, status, currency, total_cents, customer_email, metadata, created_at
            FROM commerce.orders
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce list orders: {e}")))?;

        let mut orders = Vec::with_capacity(rows.len());
        for r in rows {
            let items = self.load_items(r.id).await?;
            orders.push(Order {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                status: r.status,
                currency: r.currency,
                total_cents: r.total_cents,
                customer_email: r.customer_email,
                metadata: r.metadata,
                items,
                created_at: r.created_at,
            });
        }
        Ok(orders)
    }

    pub async fn get_order(
        &self,
        tenant_id: TenantId,
        order_id: Uuid,
    ) -> HelixResult<Option<Order>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            status: String,
            currency: String,
            total_cents: i64,
            customer_email: String,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, status, currency, total_cents, customer_email, metadata, created_at
            FROM commerce.orders
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce get order: {e}")))?;
        let Some(r) = row else {
            return Ok(None);
        };
        let items = self.load_items(r.id).await?;
        Ok(Some(Order {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            status: r.status,
            currency: r.currency,
            total_cents: r.total_cents,
            customer_email: r.customer_email,
            metadata: r.metadata,
            items,
            created_at: r.created_at,
        }))
    }

    /// Create an order: validates products, decrements inventory, inserts order + lines.
    pub async fn create_order(
        &self,
        tenant_id: TenantId,
        customer_email: &str,
        lines: &[OrderLineInput],
        metadata: serde_json::Value,
    ) -> HelixResult<Order> {
        if lines.is_empty() {
            return Err(HelixError::validation("order requires at least one line"));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("commerce begin: {e}")))?;

        let mut resolved: Vec<(Product, i32)> = Vec::with_capacity(lines.len());
        let mut total_cents: i64 = 0;
        let mut currency: Option<String> = None;

        for line in lines {
            if line.quantity < 1 {
                return Err(HelixError::validation("quantity must be >= 1"));
            }
            let row: Option<ProductRow> = sqlx::query_as(&format!(
                "{PRODUCT_SELECT} WHERE tenant_id = $1 AND id = $2 FOR UPDATE"
            ))
            .bind(tenant_id.as_uuid())
            .bind(line.product_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("commerce lock product: {e}")))?;

            let product = row
                .map(ProductRow::into_product)
                .ok_or_else(|| HelixError::not_found(format!("product {}", line.product_id)))?;

            if product.status != "active" {
                return Err(HelixError::validation(format!(
                    "product {} is not active",
                    product.sku
                )));
            }
            if product.inventory < line.quantity {
                return Err(HelixError::validation(format!(
                    "insufficient inventory for {}",
                    product.sku
                )));
            }

            match currency.as_ref() {
                None => currency = Some(product.currency.clone()),
                Some(c) if c == &product.currency => {}
                Some(c) => {
                    return Err(HelixError::validation(format!(
                        "mixed currency in order: {} and {}",
                        c, product.currency
                    )));
                }
            }

            let line_total = product
                .price_cents
                .checked_mul(line.quantity as i64)
                .ok_or_else(|| HelixError::validation("line total overflow"))?;
            total_cents = total_cents
                .checked_add(line_total)
                .ok_or_else(|| HelixError::validation("order total overflow"))?;
            resolved.push((product, line.quantity));
        }

        let order_id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO commerce.orders
                (id, tenant_id, status, currency, total_cents, customer_email, metadata, created_at, updated_at)
            VALUES ($1,$2,'pending',$3,$4,$5,$6,$7,$7)
            "#,
        )
        .bind(order_id)
        .bind(tenant_id.as_uuid())
        .bind(currency.as_deref().unwrap_or("USD"))
        .bind(total_cents)
        .bind(customer_email)
        .bind(&metadata)
        .bind(created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce create order: {e}")))?;

        let mut items = Vec::with_capacity(resolved.len());
        for (product, quantity) in resolved {
            let item_id = Uuid::now_v7();
            let line_total = product.price_cents * quantity as i64;
            sqlx::query(
                r#"
                INSERT INTO commerce.order_items
                    (id, order_id, tenant_id, product_id, sku, name, quantity,
                     unit_price_cents, line_total_cents)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                "#,
            )
            .bind(item_id)
            .bind(order_id)
            .bind(tenant_id.as_uuid())
            .bind(product.id)
            .bind(&product.sku)
            .bind(&product.name)
            .bind(quantity)
            .bind(product.price_cents)
            .bind(line_total)
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("commerce create order item: {e}")))?;

            sqlx::query(
                r#"
                UPDATE commerce.products
                SET inventory = inventory - $1, updated_at = $2
                WHERE id = $3 AND tenant_id = $4
                "#,
            )
            .bind(quantity)
            .bind(created_at)
            .bind(product.id)
            .bind(tenant_id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("commerce decrement inventory: {e}")))?;

            items.push(OrderItem {
                id: item_id,
                order_id,
                product_id: product.id,
                sku: product.sku,
                name: product.name,
                quantity,
                unit_price_cents: product.price_cents,
                line_total_cents: line_total,
            });
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("commerce commit order: {e}")))?;

        Ok(Order {
            id: order_id,
            tenant_id,
            status: "pending".into(),
            currency: currency.unwrap_or_else(|| "USD".to_string()),
            total_cents,
            customer_email: customer_email.into(),
            metadata,
            items,
            created_at,
        })
    }

    async fn load_items(&self, order_id: Uuid) -> HelixResult<Vec<OrderItem>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            order_id: Uuid,
            product_id: Uuid,
            sku: String,
            name: String,
            quantity: i32,
            unit_price_cents: i64,
            line_total_cents: i64,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, order_id, product_id, sku, name, quantity, unit_price_cents, line_total_cents
            FROM commerce.order_items
            WHERE order_id = $1
            ORDER BY sku
            "#,
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("commerce load items: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| OrderItem {
                id: r.id,
                order_id: r.order_id,
                product_id: r.product_id,
                sku: r.sku,
                name: r.name,
                quantity: r.quantity,
                unit_price_cents: r.unit_price_cents,
                line_total_cents: r.line_total_cents,
            })
            .collect())
    }
}
