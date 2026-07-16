//! Environment-driven configuration shared by all services.
//!
//! Secrets are loaded from process env (locally via
//! `~/Desktop/.keys/helixforge/.env.local` — never from the repo tree).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use crate::error::{HelixError, HelixResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub service_name: String,
    pub environment: String,
    pub listen_addr: SocketAddr,
    pub database_url: String,
    pub db_pool: DbPoolConfig,
    pub nats_url: String,
    /// NATS production credentials / TLS (Phase 6).
    pub nats_creds_file: Option<String>,
    pub nats_jwt: Option<String>,
    pub nats_nkey: Option<String>,
    pub nats_tls_ca_file: Option<String>,
    pub nats_tls_cert_file: Option<String>,
    pub nats_tls_key_file: Option<String>,
    pub nats_require_tls: bool,
    pub nats_tls_first: bool,
    pub nats_retry_on_initial_connect: bool,
    pub nats_connection_timeout_secs: u64,
    pub nats_max_reconnects: u32,
    pub nats_js_retry_attempts: u32,
    pub nats_js_retry_backoff_ms: u64,
    pub nats_js_core_fallback: bool,
    pub minio_endpoint: String,
    pub minio_bucket: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_region: String,
    pub ory_kratos_public: String,
    pub ory_kratos_admin: String,
    pub ory_hydra_public: String,
    pub ory_hydra_admin: String,
    pub otlp_endpoint: Option<String>,
    pub otlp_sample_rate: f64,
    pub log_json: bool,
    pub data_residency_region: String,
    /// Residency values accepted from identity providers. Defaults to [data_residency_region].
    pub residency_allowlist: Vec<String>,
    /// Master key material for vault AES-GCM (from env in production).
    pub vault_master_key: String,
    /// Optional explicit KEK override for the local software HSM endpoints.
    pub vault_kek: Option<String>,
    /// Max HTTP body size in bytes (default 2 MiB).
    pub max_body_bytes: usize,
    /// Soft rate limit requests/second per client key (0 = disabled).
    pub rate_limit_rps: u32,
    /// Reject weak/default vault master key outside local/dev.
    pub enforce_strong_secrets: bool,
    pub endpoints: ServiceEndpoints,
    /// Gateway product upstream discovery (Phase 6).
    pub product_upstream_template: Option<String>,
    pub product_upstream_base: Option<String>,
    pub product_overrides: HashMap<String, String>,
    pub proxy_timeout_secs: u64,
    pub proxy_connect_timeout_secs: u64,
    pub proxy_body_limit_bytes: usize,
    pub proxy_retry_count: u32,
    pub proxy_retry_backoff_ms: u64,
    /// Dev / operator controls (centralised in Phase 6).
    pub allow_dev_headers: bool,
    pub dev_login_token: Option<String>,
    pub dev_platform: bool,
    pub dev_master_key: Option<String>,
    /// True when HELIX_LOCAL_DEV_UNSAFE=1 was accepted (local/dev only).
    pub local_dev_unsafe: bool,
    /// CORS policy (non-local explicit allowlist).
    pub cors_origins: Option<String>,
    pub cors_allow_credentials: bool,
    /// Rate-limit backend selection.
    pub rate_limit_backend: String,
    pub rate_limit_pg_fallback: bool,
    /// Audit operator controls.
    pub audit_hmac_secret: Option<String>,
    pub audit_operator_key: Option<String>,
    /// Payment provider configuration.
    pub payment_provider: String,
    pub stripe_secret_key: Option<String>,
    pub stripe_force_sim: bool,
    /// Webhook signature verification.
    pub webhook_secret: Option<String>,
    pub webhook_allow_unsigned: bool,
    /// Vault KMS mode and remote URL.
    pub kms_mode: String,
    pub kms_url: String,
    pub kms_fallback: bool,
    /// Hydra basic-auth credentials for introspection / admin endpoints.
    pub hydra_introspect_client_id: Option<String>,
    pub hydra_introspect_client_secret: Option<String>,
    pub hydra_admin_client_id: Option<String>,
    pub hydra_admin_client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    pub gateway: String,
    pub agent_hub: String,
    pub vault: String,
    pub billing: String,
    pub observability: String,
    pub auth_adapter: String,
}

impl Default for ServiceEndpoints {
    fn default() -> Self {
        Self {
            gateway: "http://127.0.0.1:8080".into(),
            agent_hub: "http://127.0.0.1:8081".into(),
            vault: "http://127.0.0.1:8082".into(),
            billing: "http://127.0.0.1:8083".into(),
            observability: "http://127.0.0.1:8084".into(),
            auth_adapter: "http://127.0.0.1:8085".into(),
        }
    }
}

/// Postgres connection pool tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
    pub test_before_acquire: bool,
}

impl Default for DbPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(1800)),
            test_before_acquire: true,
        }
    }
}

impl CoreConfig {
    /// Load config for a named service. `port` is the default listen port.
    pub fn from_env(service_name: &str, default_port: u16) -> HelixResult<Self> {
        // Best-effort: load dotenv if present (CI / container) — production
        // secrets come from the process environment / secret mounts only.
        let _ = dotenvy::dotenv();

        let port: u16 = env_or_parse("PORT", default_port)?;
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let listen_addr: SocketAddr = format!("{host}:{port}")
            .parse()
            .map_err(|e| HelixError::validation(format!("invalid HOST/PORT: {e}")))?;

        let environment = env_or("HELIX_ENV", "local");
        let is_local = environment == "local" || environment == "dev";
        let allow_local_defaults = local_dev_defaults_allowed(&environment);

        let endpoints = ServiceEndpoints {
            gateway: env_or_local(
                "HELIX_GATEWAY_URL",
                "http://127.0.0.1:8080",
                allow_local_defaults,
            )?,
            agent_hub: env_or_local(
                "HELIX_AGENT_HUB_URL",
                "http://127.0.0.1:8081",
                allow_local_defaults,
            )?,
            vault: env_or_local(
                "HELIX_VAULT_URL",
                "http://127.0.0.1:8082",
                allow_local_defaults,
            )?,
            billing: env_or_local(
                "HELIX_BILLING_URL",
                "http://127.0.0.1:8083",
                allow_local_defaults,
            )?,
            observability: env_or_local(
                "HELIX_OBSERVABILITY_URL",
                "http://127.0.0.1:8084",
                allow_local_defaults,
            )?,
            auth_adapter: env_or_local(
                "HELIX_AUTH_ADAPTER_URL",
                "http://127.0.0.1:8085",
                allow_local_defaults,
            )?,
        };

        let product_upstream_template = env::var("HELIX_PRODUCT_HOST")
            .ok()
            .filter(|s| !s.is_empty());
        let product_upstream_base = env::var("HELIX_PRODUCT_UPSTREAM_BASE")
            .ok()
            .filter(|s| !s.is_empty());
        let product_overrides: HashMap<String, String> = env::vars()
            .filter(|(k, _)| k.starts_with("HELIX_UPSTREAM_"))
            .map(|(k, v)| {
                (
                    k.trim_start_matches("HELIX_UPSTREAM_")
                        .to_ascii_lowercase()
                        .replace('_', "-"),
                    v,
                )
            })
            .collect();

        let proxy_timeout_secs = env_or_parse("HELIX_PROXY_TIMEOUT_SECS", 30u64)?;
        let proxy_connect_timeout_secs = env_or_parse("HELIX_PROXY_CONNECT_TIMEOUT_SECS", 5u64)?;
        let proxy_body_limit_bytes =
            env_or_parse("HELIX_PROXY_BODY_LIMIT_BYTES", 32usize * 1024 * 1024)?;
        let proxy_retry_count = env_or_parse("HELIX_PROXY_RETRY_COUNT", 1u32)?;
        let proxy_retry_backoff_ms = env_or_parse("HELIX_PROXY_RETRY_BACKOFF_MS", 40u64)?;

        let nats_url = env_or_local("NATS_URL", "nats://127.0.0.1:4222", allow_local_defaults)?;
        let nats_creds_file = nonempty_env("NATS_CREDS_FILE");
        let nats_jwt = nonempty_env("NATS_JWT");
        let nats_nkey = nonempty_env("NATS_NKEY");
        let nats_tls_ca_file = nonempty_env("NATS_TLS_CA_FILE");
        let nats_tls_cert_file = nonempty_env("NATS_TLS_CERT_FILE");
        let nats_tls_key_file = nonempty_env("NATS_TLS_KEY_FILE");
        let nats_require_tls = env_bool("NATS_REQUIRE_TLS", false);
        let nats_tls_first = env_bool("NATS_TLS_FIRST", false);
        let nats_retry_on_initial_connect = env_bool("NATS_RETRY_ON_INITIAL_CONNECT", false);
        let nats_connection_timeout_secs = env_or_parse("NATS_CONNECTION_TIMEOUT_SECS", 5u64)?;
        let nats_max_reconnects = env_or_parse("NATS_MAX_RECONNECTS", 0u32)?;
        let nats_js_retry_attempts = env_or_parse("NATS_JS_RETRY_ATTEMPTS", 5u32)?;
        let nats_js_retry_backoff_ms = env_or_parse("NATS_JS_RETRY_BACKOFF_MS", 500u64)?;
        let nats_js_core_fallback = env_bool("HELIX_NATS_JS_FALLBACK", false);

        let db_pool = DbPoolConfig {
            max_connections: env_or_parse("HELIX_DB_POOL_MAX_CONNECTIONS", 10u32)?,
            min_connections: env_or_parse("HELIX_DB_POOL_MIN_CONNECTIONS", 2u32)?,
            acquire_timeout: Duration::from_secs(env_or_parse(
                "HELIX_DB_POOL_ACQUIRE_TIMEOUT_SECS",
                5u64,
            )?),
            idle_timeout: env::var("HELIX_DB_POOL_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs),
            max_lifetime: env::var("HELIX_DB_POOL_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs),
            test_before_acquire: env_bool("HELIX_DB_POOL_TEST_BEFORE_ACQUIRE", true),
        };

        let vault_master_key = env_or_local(
            "HELIX_VAULT_MASTER_KEY",
            "helixforge-local-dev-master-key-change-me",
            allow_local_defaults,
        )?;
        let enforce_strong_secrets = !is_local;
        if enforce_strong_secrets
            && (vault_master_key.contains("change-me")
                || vault_master_key.contains("local-dev")
                || vault_master_key.len() < 24)
        {
            return Err(HelixError::validation(
                "HELIX_VAULT_MASTER_KEY must be strong outside local/dev (len>=24, non-default)",
            ));
        }

        let max_body_bytes: usize = env_or_parse("HELIX_MAX_BODY_BYTES", 2usize * 1024 * 1024)?;
        let rate_limit_rps: u32 =
            env_or_parse("HELIX_RATE_LIMIT_RPS", if is_local { 0u32 } else { 100u32 })?;

        let data_residency_region =
            env_or_local("HELIX_DATA_RESIDENCY", "local", allow_local_defaults)?;
        let residency_allowlist: Vec<String> = env::var("HELIX_RESIDENCY_ALLOWLIST")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
            .unwrap_or_else(|| vec![data_residency_region.clone()]);

        let allow_dev_headers = env_bool("HELIX_ALLOW_DEV_HEADERS", false);
        let dev_login_token = nonempty_env("HELIX_DEV_LOGIN_TOKEN");
        let dev_platform = env_bool("HELIX_DEV_PLATFORM", false);
        let dev_master_key = nonempty_env("HELIX_DEV_MASTER_KEY");
        let cors_origins = nonempty_env("HELIX_CORS_ORIGINS");
        let cors_allow_credentials = env_bool("HELIX_CORS_ALLOW_CREDENTIALS", false);
        let rate_limit_backend = env_or("HELIX_RATE_LIMIT_BACKEND", "postgres");
        let rate_limit_pg_fallback = env_bool("HELIX_RATE_LIMIT_PG_FALLBACK", false);
        let audit_hmac_secret = nonempty_env("HELIX_AUDIT_HMAC_SECRET");
        let audit_operator_key = nonempty_env("HELIX_AUDIT_OPERATOR_KEY");
        let payment_provider =
            env_or_local("HELIX_PAYMENT_PROVIDER", "local_sim", allow_local_defaults)?;
        let stripe_secret_key = nonempty_env("STRIPE_SECRET_KEY");
        let stripe_force_sim = env_bool("HELIX_PAYMENT_STRIPE_FORCE_SIM", false);
        let webhook_secret = nonempty_env("HELIX_WEBHOOK_SECRET");
        let webhook_allow_unsigned = env_bool("HELIX_WEBHOOK_ALLOW_UNSIGNED", false);
        let kms_mode = env_or_local("HELIX_VAULT_KMS_MODE", "local", allow_local_defaults)?;
        let kms_url = env_or_local(
            "HELIX_VAULT_KMS_URL",
            "http://127.0.0.1:8082",
            allow_local_defaults,
        )?;
        let kms_fallback = env_bool("HELIX_VAULT_KMS_FALLBACK", false);

        Ok(Self {
            service_name: service_name.into(),
            environment,
            listen_addr,
            database_url: env_or_local(
                "DATABASE_URL",
                "postgres://helix:helix@127.0.0.1:55432/helixforge",
                allow_local_defaults,
            )?,
            db_pool,
            nats_url,
            nats_creds_file,
            nats_jwt,
            nats_nkey,
            nats_tls_ca_file,
            nats_tls_cert_file,
            nats_tls_key_file,
            nats_require_tls,
            nats_tls_first,
            nats_retry_on_initial_connect,
            nats_connection_timeout_secs,
            nats_max_reconnects,
            nats_js_retry_attempts,
            nats_js_retry_backoff_ms,
            nats_js_core_fallback,
            minio_endpoint: env_or_local(
                "MINIO_ENDPOINT",
                "http://127.0.0.1:9000",
                allow_local_defaults,
            )?,
            minio_bucket: env_or_local("MINIO_BUCKET", "helixforge", allow_local_defaults)?,
            minio_access_key: env_or_local("MINIO_ACCESS_KEY", "helixminio", allow_local_defaults)?,
            minio_secret_key: env_or_local(
                "MINIO_SECRET_KEY",
                "helixminio_secret",
                allow_local_defaults,
            )?,
            minio_region: env_or_local("MINIO_REGION", "us-east-1", allow_local_defaults)?,
            ory_kratos_public: env_or_local(
                "KRATOS_PUBLIC_URL",
                "http://127.0.0.1:4433",
                allow_local_defaults,
            )?,
            ory_kratos_admin: env_or_local(
                "KRATOS_ADMIN_URL",
                "http://127.0.0.1:4434",
                allow_local_defaults,
            )?,
            ory_hydra_public: env_or_local(
                "HYDRA_PUBLIC_URL",
                "http://127.0.0.1:4444",
                allow_local_defaults,
            )?,
            ory_hydra_admin: env_or_local(
                "HYDRA_ADMIN_URL",
                "http://127.0.0.1:4445",
                allow_local_defaults,
            )?,
            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            otlp_sample_rate: env_or_parse("HELIX_OTLP_SAMPLE_RATE", 1.0f64)?,
            log_json: env_bool("LOG_JSON", false),
            data_residency_region: data_residency_region.clone(),
            residency_allowlist,
            vault_master_key,
            vault_kek: nonempty_env("HELIX_VAULT_KEK"),
            max_body_bytes,
            rate_limit_rps,
            enforce_strong_secrets,
            endpoints,
            product_upstream_template,
            product_upstream_base,
            product_overrides,
            proxy_timeout_secs,
            proxy_connect_timeout_secs,
            proxy_body_limit_bytes,
            proxy_retry_count,
            proxy_retry_backoff_ms,
            allow_dev_headers,
            dev_login_token,
            dev_platform,
            dev_master_key,
            local_dev_unsafe: allow_local_defaults,
            cors_origins,
            cors_allow_credentials,
            rate_limit_backend,
            rate_limit_pg_fallback,
            audit_hmac_secret,
            audit_operator_key,
            payment_provider,
            stripe_secret_key,
            stripe_force_sim,
            webhook_secret,
            webhook_allow_unsigned,
            kms_mode,
            kms_url,
            kms_fallback,
            hydra_introspect_client_id: nonempty_env("HELIX_HYDRA_INTROSPECT_CLIENT_ID"),
            hydra_introspect_client_secret: nonempty_env("HELIX_HYDRA_INTROSPECT_CLIENT_SECRET"),
            hydra_admin_client_id: nonempty_env("HELIX_HYDRA_ADMIN_CLIENT_ID"),
            hydra_admin_client_secret: nonempty_env("HELIX_HYDRA_ADMIN_CLIENT_SECRET"),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.into())
}

fn env_or_parse<T: FromStr>(key: &str, default: T) -> HelixResult<T> {
    match env::var(key) {
        Ok(v) => v
            .parse()
            .map_err(|_| HelixError::validation(format!("invalid {key}={v}"))),
        Err(_) => Ok(default),
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

fn nonempty_env(key: &str) -> Option<String> {
    env::var(key).ok().filter(|s| !s.is_empty())
}

/// Local/dev defaults are only permitted when the environment is local/dev AND
/// the operator has explicitly opted in via HELIX_LOCAL_DEV_UNSAFE=1.
fn local_dev_defaults_allowed(environment: &str) -> bool {
    let env_ok = environment == "local" || environment == "dev";
    let flag = env_bool("HELIX_LOCAL_DEV_UNSAFE", false);
    env_ok && flag
}

/// Require an explicit value outside local/dev. When `allow_local` is true the
/// default is used but a loud warning is emitted so it cannot be missed.
fn env_or_local(key: &str, default: &str, allow_local: bool) -> HelixResult<String> {
    match env::var(key) {
        Ok(v) if !v.is_empty() => Ok(v),
        _ if allow_local => {
            local_dev_warn(key, default);
            Ok(default.into())
        }
        _ => Err(HelixError::internal(format!(
            "{key} is required; set it explicitly or enable HELIX_LOCAL_DEV_UNSAFE=1 for local/dev only"
        ))),
    }
}

fn local_dev_warn(key: &str, default: &str) {
    eprintln!(
        "SECURITY WARNING: {key} is unset. Using local-dev default '{default}'. \
         NEVER set HELIX_LOCAL_DEV_UNSAFE=1 in production."
    );
}
