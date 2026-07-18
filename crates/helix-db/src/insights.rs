//! HelixInsights dataset + metric persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub source_type: String,
    pub schema_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDef {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub dataset_id: Uuid,
    pub name: String,
    pub unit: String,
    pub aggregation: String,
    pub expression: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub id: Uuid,
    pub metric_id: Uuid,
    pub tenant_id: TenantId,
    pub value: f64,
    pub dimensions: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateResult {
    pub value: Option<f64>,
    pub count: i64,
}

#[derive(Clone)]
pub struct InsightsRepo {
    pool: PgPool,
}

impl InsightsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_datasets(&self, tenant_id: TenantId) -> HelixResult<Vec<Dataset>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            description: String,
            source_type: String,
            schema_json: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, source_type, schema_json, created_at
            FROM insights.datasets
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights list datasets: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Dataset {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                name: r.name,
                description: r.description,
                source_type: r.source_type,
                schema_json: r.schema_json,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create_dataset(
        &self,
        tenant_id: TenantId,
        name: &str,
        description: &str,
        source_type: &str,
        schema_json: serde_json::Value,
    ) -> HelixResult<Dataset> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let source = if source_type.trim().is_empty() {
            "manual"
        } else {
            source_type.trim()
        };
        sqlx::query(
            r#"
            INSERT INTO insights.datasets
                (id, tenant_id, name, description, source_type, schema_json, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$7)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(source)
        .bind(&schema_json)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights create dataset: {e}")))?;
        Ok(Dataset {
            id,
            tenant_id,
            name: name.into(),
            description: description.into(),
            source_type: source.into(),
            schema_json,
            created_at,
        })
    }

    pub async fn get_dataset(
        &self,
        tenant_id: TenantId,
        dataset_id: Uuid,
    ) -> HelixResult<Option<Dataset>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            description: String,
            source_type: String,
            schema_json: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, source_type, schema_json, created_at
            FROM insights.datasets
            WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(dataset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights get dataset: {e}")))?;
        Ok(row.map(|r| Dataset {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            name: r.name,
            description: r.description,
            source_type: r.source_type,
            schema_json: r.schema_json,
            created_at: r.created_at,
        }))
    }

    pub async fn soft_delete_dataset(
        &self,
        tenant_id: TenantId,
        dataset_id: Uuid,
    ) -> HelixResult<Dataset> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            name: String,
            description: String,
            source_type: String,
            schema_json: serde_json::Value,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            UPDATE insights.datasets
            SET deleted_at = now(), updated_at = now()
            WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL
            RETURNING id, tenant_id, name, description, source_type, schema_json, created_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(dataset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights soft delete dataset: {e}")))?;
        row.map(|r| Dataset {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            name: r.name,
            description: r.description,
            source_type: r.source_type,
            schema_json: r.schema_json,
            created_at: r.created_at,
        })
        .ok_or_else(|| HelixError::not_found("dataset not found"))
    }

    pub async fn list_metrics(
        &self,
        tenant_id: TenantId,
        dataset_id: Uuid,
    ) -> HelixResult<Vec<MetricDef>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            dataset_id: Uuid,
            name: String,
            unit: String,
            aggregation: String,
            expression: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, dataset_id, name, unit, aggregation, expression, created_at
            FROM insights.metrics
            WHERE tenant_id = $1 AND dataset_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(dataset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights list metrics: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| MetricDef {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                dataset_id: r.dataset_id,
                name: r.name,
                unit: r.unit,
                aggregation: r.aggregation,
                expression: r.expression,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn list_metrics_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<MetricDef>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            dataset_id: Uuid,
            name: String,
            unit: String,
            aggregation: String,
            expression: String,
            created_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, dataset_id, name, unit, aggregation, expression, created_at
            FROM insights.metrics
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights list tenant metrics: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| MetricDef {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                dataset_id: r.dataset_id,
                name: r.name,
                unit: r.unit,
                aggregation: r.aggregation,
                expression: r.expression,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create_metric(
        &self,
        tenant_id: TenantId,
        dataset_id: Uuid,
        name: &str,
        unit: &str,
        aggregation: &str,
        expression: &str,
    ) -> HelixResult<MetricDef> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let unit = if unit.trim().is_empty() {
            "count"
        } else {
            unit.trim()
        };
        let aggregation = if aggregation.trim().is_empty() {
            "sum"
        } else {
            aggregation.trim()
        };
        // The dataset existence check is part of the INSERT itself: a dataset
        // deleted between a separate check and insert cannot leak children.
        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"
            INSERT INTO insights.metrics
                (id, tenant_id, dataset_id, name, unit, aggregation, expression, created_at)
            SELECT $1, $2, $3, $4, $5, $6, $7, $8
            FROM insights.datasets d
            WHERE d.tenant_id = $2 AND d.id = $3 AND d.deleted_at IS NULL
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(dataset_id)
        .bind(name)
        .bind(unit)
        .bind(aggregation)
        .bind(expression)
        .bind(created_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights create metric: {e}")))?;
        if inserted.is_none() {
            return Err(HelixError::not_found("dataset not found"));
        }
        Ok(MetricDef {
            id,
            tenant_id,
            dataset_id,
            name: name.into(),
            unit: unit.into(),
            aggregation: aggregation.into(),
            expression: expression.into(),
            created_at,
        })
    }

    pub async fn get_metric(
        &self,
        tenant_id: TenantId,
        metric_id: Uuid,
    ) -> HelixResult<Option<MetricDef>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            dataset_id: Uuid,
            name: String,
            unit: String,
            aggregation: String,
            expression: String,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, dataset_id, name, unit, aggregation, expression, created_at
            FROM insights.metrics
            WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(metric_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights get metric: {e}")))?;
        Ok(row.map(|r| MetricDef {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            dataset_id: r.dataset_id,
            name: r.name,
            unit: r.unit,
            aggregation: r.aggregation,
            expression: r.expression,
            created_at: r.created_at,
        }))
    }

    pub async fn soft_delete_metric(
        &self,
        tenant_id: TenantId,
        metric_id: Uuid,
    ) -> HelixResult<MetricDef> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            dataset_id: Uuid,
            name: String,
            unit: String,
            aggregation: String,
            expression: String,
            created_at: DateTime<Utc>,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            UPDATE insights.metrics
            SET deleted_at = now()
            WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL
            RETURNING id, tenant_id, dataset_id, name, unit, aggregation, expression, created_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(metric_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights soft delete metric: {e}")))?;
        row.map(|r| MetricDef {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            dataset_id: r.dataset_id,
            name: r.name,
            unit: r.unit,
            aggregation: r.aggregation,
            expression: r.expression,
            created_at: r.created_at,
        })
        .ok_or_else(|| HelixError::not_found("metric not found"))
    }

    pub async fn record_point(
        &self,
        tenant_id: TenantId,
        metric_id: Uuid,
        value: f64,
        dimensions: serde_json::Value,
    ) -> HelixResult<MetricPoint> {
        let id = Uuid::now_v7();
        let recorded_at = Utc::now();
        // The metric existence check is part of the INSERT itself: a metric
        // deleted between a separate check and insert cannot leak points.
        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"
            INSERT INTO insights.metric_points
                (id, metric_id, tenant_id, value, dimensions, recorded_at)
            SELECT $1, $2, $3, $4, $5, $6
            FROM insights.metrics m
            WHERE m.tenant_id = $3 AND m.id = $2 AND m.deleted_at IS NULL
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(metric_id)
        .bind(tenant_id.as_uuid())
        .bind(value)
        .bind(&dimensions)
        .bind(recorded_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("insights record point: {e}")))?;
        if inserted.is_none() {
            return Err(HelixError::not_found("metric not found"));
        }
        Ok(MetricPoint {
            id,
            metric_id,
            tenant_id,
            value,
            dimensions,
            recorded_at,
        })
    }

    pub async fn list_points(
        &self,
        tenant_id: TenantId,
        metric_id: Uuid,
        limit: i64,
    ) -> HelixResult<Vec<MetricPoint>> {
        self.list_points_filtered(tenant_id, metric_id, None, None, None, limit)
            .await
    }

    pub async fn list_points_filtered(
        &self,
        tenant_id: TenantId,
        metric_id: Uuid,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        dimensions: Option<&serde_json::Value>,
        limit: i64,
    ) -> HelixResult<Vec<MetricPoint>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            metric_id: Uuid,
            tenant_id: Uuid,
            value: f64,
            dimensions: serde_json::Value,
            recorded_at: DateTime<Utc>,
        }
        let lim = limit.clamp(1, 500);

        let mut builder = QueryBuilder::new(
            "SELECT id, metric_id, tenant_id, value, dimensions, recorded_at FROM insights.metric_points WHERE tenant_id = ",
        );
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND metric_id = ");
        builder.push_bind(metric_id);
        if let Some(f) = from {
            builder.push(" AND recorded_at >= ");
            builder.push_bind(f);
        }
        if let Some(t) = to {
            builder.push(" AND recorded_at <= ");
            builder.push_bind(t);
        }
        if let Some(d) = dimensions {
            builder.push(" AND dimensions @> ");
            builder.push_bind(d);
        }
        builder.push(" ORDER BY recorded_at DESC LIMIT ");
        builder.push_bind(lim);

        let rows: Vec<Row> = builder
            .build_query_as::<Row>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("insights list points: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| MetricPoint {
                id: r.id,
                metric_id: r.metric_id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                value: r.value,
                dimensions: r.dimensions,
                recorded_at: r.recorded_at,
            })
            .collect())
    }

    pub async fn aggregate_points(
        &self,
        tenant_id: TenantId,
        metric_id: Uuid,
        aggregation: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        dimensions: Option<&serde_json::Value>,
    ) -> HelixResult<AggregateResult> {
        let agg = match aggregation.to_ascii_lowercase().as_str() {
            "sum" => "SUM(value)",
            "avg" => "AVG(value)",
            "min" => "MIN(value)",
            "max" => "MAX(value)",
            "count" => "COUNT(*)::float8",
            other => {
                return Err(HelixError::validation(format!(
                    "unsupported aggregation: {other}"
                )))
            }
        };

        let mut builder = QueryBuilder::new(format!(
            "SELECT {agg} AS value, COUNT(*) AS count FROM insights.metric_points WHERE tenant_id = "
        ));
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND metric_id = ");
        builder.push_bind(metric_id);
        if let Some(f) = from {
            builder.push(" AND recorded_at >= ");
            builder.push_bind(f);
        }
        if let Some(t) = to {
            builder.push(" AND recorded_at <= ");
            builder.push_bind(t);
        }
        if let Some(d) = dimensions {
            builder.push(" AND dimensions @> ");
            builder.push_bind(d);
        }

        #[derive(sqlx::FromRow)]
        struct Row {
            value: Option<f64>,
            count: Option<i64>,
        }
        let row: Row = builder
            .build_query_as::<Row>()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("insights aggregate points: {e}")))?;
        Ok(AggregateResult {
            value: row.value,
            count: row.count.unwrap_or(0),
        })
    }
}
