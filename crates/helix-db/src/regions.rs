//! Multi-region registry and write-affinity helpers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionRecord {
    pub code: String,
    pub name: String,
    pub is_primary: bool,
    pub write_enabled: bool,
    pub read_enabled: bool,
    pub endpoint_hint: Option<String>,
    pub status: String,
    pub lag_seconds: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RegionRepo {
    pool: PgPool,
}

impl RegionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> HelixResult<Vec<RegionRecord>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            code: String,
            name: String,
            is_primary: bool,
            write_enabled: bool,
            read_enabled: bool,
            endpoint_hint: Option<String>,
            status: String,
            lag_seconds: i32,
            updated_at: DateTime<Utc>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT code, name, is_primary, write_enabled, read_enabled, endpoint_hint,
                   status, lag_seconds, updated_at
            FROM helix_core.regions
            ORDER BY is_primary DESC, code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("regions list: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| RegionRecord {
                code: r.code,
                name: r.name,
                is_primary: r.is_primary,
                write_enabled: r.write_enabled,
                read_enabled: r.read_enabled,
                endpoint_hint: r.endpoint_hint,
                status: r.status,
                lag_seconds: r.lag_seconds,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn get(&self, code: &str) -> HelixResult<Option<RegionRecord>> {
        Ok(self.list().await?.into_iter().find(|r| r.code == code))
    }

    pub async fn upsert_status(
        &self,
        code: &str,
        status: &str,
        lag_seconds: i32,
        write_enabled: bool,
        read_enabled: bool,
    ) -> HelixResult<RegionRecord> {
        sqlx::query(
            r#"
            UPDATE helix_core.regions
            SET status = $2, lag_seconds = $3, write_enabled = $4, read_enabled = $5, updated_at = now()
            WHERE code = $1
            "#,
        )
        .bind(code)
        .bind(status)
        .bind(lag_seconds)
        .bind(write_enabled)
        .bind(read_enabled)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("region status: {e}")))?;
        self.get(code)
            .await?
            .ok_or_else(|| HelixError::not_found(format!("region {code}")))
    }

    pub async fn primary(&self) -> HelixResult<Option<RegionRecord>> {
        Ok(self.list().await?.into_iter().find(|r| r.is_primary))
    }

    /// Write affinity: service region must allow writes; principal residency must match
    /// unless service region is `local` or principal has Platform.
    pub async fn assert_write_allowed(
        &self,
        service_region: &str,
        principal_residency: &str,
        is_platform: bool,
    ) -> HelixResult<()> {
        if service_region == "local" {
            return Ok(());
        }
        let region = self
            .get(service_region)
            .await?
            .ok_or_else(|| HelixError::unavailable(format!("unknown region {service_region}")))?;
        if !region.write_enabled || region.status == "offline" {
            return Err(HelixError::unavailable(format!(
                "region {service_region} not accepting writes (status={})",
                region.status
            )));
        }
        if !is_platform && principal_residency != service_region && principal_residency != "local" {
            return Err(HelixError::forbidden(format!(
                "write affinity: principal residency {principal_residency} cannot write in {service_region}"
            )));
        }
        Ok(())
    }
}
