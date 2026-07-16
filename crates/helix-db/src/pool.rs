//! Postgres pool + embedded sqlx migrations.

use shared_core::{DbPoolConfig, HelixError, HelixResult};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::{info, warn};

pub type DbPool = PgPool;

#[derive(Debug, Clone)]
pub struct DbStatus {
    pub connected: bool,
    pub migrated: bool,
    pub detail: String,
}

/// Connect and run migrations. Returns error if Postgres is unreachable.
pub async fn connect_and_migrate(database_url: &str) -> HelixResult<DbPool> {
    connect_and_migrate_with_config(database_url, &DbPoolConfig::default()).await
}

/// Connect and run migrations with explicit pool tuning.
pub async fn connect_and_migrate_with_config(
    database_url: &str,
    cfg: &DbPoolConfig,
) -> HelixResult<DbPool> {
    let mut opts = PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .min_connections(cfg.min_connections)
        .acquire_timeout(cfg.acquire_timeout)
        .test_before_acquire(cfg.test_before_acquire);
    if let Some(idle) = cfg.idle_timeout {
        opts = opts.idle_timeout(idle);
    }
    if let Some(lifetime) = cfg.max_lifetime {
        opts = opts.max_lifetime(lifetime);
    }

    let pool = opts
        .connect(database_url)
        .await
        .map_err(|e| HelixError::dependency(format!("postgres connect: {e}")))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| HelixError::dependency(format!("postgres migrate: {e}")))?;

    info!("postgres connected and migrations applied");
    Ok(pool)
}

/// Best-effort connect: returns None when Postgres is down (local boot without docker).
pub async fn try_connect_and_migrate(database_url: &str) -> (Option<DbPool>, DbStatus) {
    try_connect_and_migrate_with_config(database_url, &DbPoolConfig::default()).await
}

/// Best-effort connect with explicit pool tuning.
pub async fn try_connect_and_migrate_with_config(
    database_url: &str,
    cfg: &DbPoolConfig,
) -> (Option<DbPool>, DbStatus) {
    match connect_and_migrate_with_config(database_url, cfg).await {
        Ok(pool) => (
            Some(pool),
            DbStatus {
                connected: true,
                migrated: true,
                detail: "ok".into(),
            },
        ),
        Err(err) => {
            warn!(error = %err, "postgres unavailable — durable stores disabled");
            (
                None,
                DbStatus {
                    connected: false,
                    migrated: false,
                    detail: err.message,
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn try_connect_fails_soft_on_bad_url() {
        let (pool, status) =
            try_connect_and_migrate("postgres://helix:helix@127.0.0.1:1/nope").await;
        assert!(pool.is_none());
        assert!(!status.connected);
    }

    #[test]
    fn db_pool_config_default_sane() {
        let cfg = DbPoolConfig::default();
        assert_eq!(cfg.max_connections, 10);
        assert_eq!(cfg.min_connections, 2);
        assert!(cfg.test_before_acquire);
    }
}
