use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use parking_lot::Mutex;
use shared_core::tenancy::{Principal, Scope};
use shared_core::HelixError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::context::AppState;
use crate::error_map::ApiError;

/// Extract request id from header or mint a new one.
pub fn request_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("req:{}", Uuid::now_v7()))
}

/// Resolve session token from explicit headers or Kratos session cookie.
pub fn session_token_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers
        .get("x-session-token")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Some(v.to_string());
    }
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        let t = auth.trim();
        if let Some(rest) = t
            .strip_prefix("Bearer ")
            .or_else(|| t.strip_prefix("bearer "))
        {
            let rest = rest.trim();
            // API keys use hk_live_ prefix — not session tokens.
            if rest.starts_with("hk_live_") {
                return None;
            }
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    cookie_value(headers, "ory_kratos_session").or_else(|| cookie_value(headers, "helix_session"))
}

fn api_key_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers
        .get("x-helix-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Some(v.to_string());
    }
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        let t = auth.trim();
        if let Some(rest) = t
            .strip_prefix("Bearer ")
            .or_else(|| t.strip_prefix("bearer "))
        {
            let rest = rest.trim();
            if rest.starts_with("hk_live_") {
                return Some(rest.to_string());
            }
        }
    }
    None
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie = headers.get("cookie")?.to_str().ok()?;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            if k.trim() == name {
                let v = v.trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Axum extractor that resolves a zero-trust [`Principal`].
pub struct RequireAuth(pub Principal);

impl FromRequestParts<AppState> for RequireAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Prefer service API keys for machine principals.
        if let Some(raw_key) = api_key_from_headers(&parts.headers) {
            if let Some(store) = state.clients.api_keys.as_ref() {
                match store.resolve(&raw_key).await {
                    Ok(Some(mut p)) => {
                        // Align residency with service region for machine keys.
                        p.residency_region = state.clients.config.data_residency_region.clone();
                        if let Some(tenants) = state.clients.tenants.as_ref() {
                            if !tenants.is_active(p.tenant_id).await.map_err(ApiError)? {
                                return Err(ApiError(HelixError::forbidden("tenant suspended")));
                            }
                        }
                        return Ok(RequireAuth(p));
                    }
                    Ok(None) => {
                        return Err(ApiError(HelixError::unauthorized("invalid api key")));
                    }
                    Err(e) => return Err(ApiError(e)),
                }
            }
            return Err(ApiError(HelixError::unavailable(
                "api keys require Postgres",
            )));
        }

        let session = session_token_from_headers(&parts.headers);

        let dev_user = parts
            .headers
            .get("x-helix-dev-user")
            .and_then(|v| v.to_str().ok());

        let mut principal = state
            .clients
            .auth
            .resolve(session.as_deref(), dev_user)
            .await
            .map_err(ApiError)?;

        // Dev header escalation is gated by the canonical auth-client flag and an
        // optional operator master key. It never applies outside local+flag.
        if state.clients.auth.dev_headers_allowed() {
            let master_key_env = state.clients.config.dev_master_key.clone();
            let master_key_header = parts
                .headers
                .get("x-helix-dev-master-key")
                .and_then(|v| v.to_str().ok());
            let master_valid = master_key_env.as_deref().is_none_or(|expected| {
                let provided = master_key_header.unwrap_or("");
                let expected_bytes = expected.as_bytes();
                let provided_bytes = provided.as_bytes();
                let len_ok = expected_bytes.len() == provided_bytes.len();
                let eq = expected_bytes.ct_eq(provided_bytes);
                len_ok && eq.into()
            });

            if let Some(raw) = parts
                .headers
                .get("x-helix-dev-scopes")
                .and_then(|v| v.to_str().ok())
            {
                if let Some(mut scopes) = Scope::parse_list(raw) {
                    let wants_elevated = scopes
                        .iter()
                        .any(|s| matches!(s, Scope::Platform | Scope::Admin));
                    if wants_elevated && !master_valid {
                        scopes.retain(|s| !matches!(s, Scope::Platform | Scope::Admin));
                        tracing::warn!("dev-scope escalation to Platform/Admin ignored: master key missing/invalid");
                    }
                    if !scopes.is_empty() {
                        principal = principal.with_scopes(scopes.clone());
                        let _ = state
                            .clients
                            .audit
                            .append(audit_log::AuditEvent {
                                tenant_id: Some(principal.tenant_id),
                                actor: shared_core::tenancy::Actor::User {
                                    user_id: principal.user_id,
                                    tenant_id: principal.tenant_id,
                                },
                                action: "auth.dev_scope_override".into(),
                                resource_type: "principal".into(),
                                resource_id: principal.session_id.clone().unwrap_or_default(),
                                metadata: serde_json::json!({
                                    "scopes": scopes.iter().map(Scope::as_str).collect::<Vec<_>>(),
                                    "via_header": "x-helix-dev-scopes",
                                }),
                                residency_region: principal.residency_region.clone(),
                            })
                            .await;
                    }
                }
            }
            if master_valid {
                if let Some(region) = parts
                    .headers
                    .get("x-helix-dev-residency")
                    .and_then(|v| v.to_str().ok())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    principal = principal.with_residency(region);
                    let _ = state
                        .clients
                        .audit
                        .append(audit_log::AuditEvent {
                            tenant_id: Some(principal.tenant_id),
                            actor: shared_core::tenancy::Actor::User {
                                user_id: principal.user_id,
                                tenant_id: principal.tenant_id,
                            },
                            action: "auth.dev_residency_override".into(),
                            resource_type: "principal".into(),
                            resource_id: principal.session_id.clone().unwrap_or_default(),
                            metadata: serde_json::json!({"residency": region}),
                            residency_region: principal.residency_region.clone(),
                        })
                        .await;
                }
            }
        }

        if principal.residency_region != state.clients.config.data_residency_region
            && state.clients.config.data_residency_region != "local"
        {
            return Err(ApiError(HelixError::forbidden(
                "principal residency does not match service region",
            )));
        }

        // Tenant lifecycle gate. Unknown tenants: auto-provision on local only.
        if let Some(tenants) = state.clients.tenants.as_ref() {
            let active = tenants
                .is_active(principal.tenant_id)
                .await
                .map_err(ApiError)?;
            if !active {
                let exists = tenants
                    .get(principal.tenant_id)
                    .await
                    .map_err(ApiError)?
                    .is_some();
                if !exists && state.clients.config.environment == "local" {
                    let _ = tenants
                        .create(
                            principal.tenant_id,
                            &principal.user_id.to_string(),
                            &principal.residency_region,
                            None,
                        )
                        .await;
                } else {
                    return Err(ApiError(HelixError::forbidden(
                        "tenant suspended or not provisioned",
                    )));
                }
            }
        }

        Ok(RequireAuth(principal))
    }
}

/// Inject `x-request-id` + security headers; emit OTLP span when configured.
pub async fn inject_request_id(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let rid = request_id_from_headers(req.headers());
    let traceparent = req
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let started = Instant::now();
    let mut res = next.run(req).await;
    if let Ok(val) = HeaderValue::from_str(&rid) {
        res.headers_mut().insert("x-request-id", val);
    }
    // Security headers (enterprise baseline)
    res.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    res.headers_mut()
        .insert("x-frame-options", HeaderValue::from_static("DENY"));
    res.headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    res.headers_mut().insert(
        "permissions-policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );
    if observability::otlp_enabled() {
        let ms = started.elapsed().as_millis() as u64;
        let ok = res.status().is_success() || res.status().is_informational();
        let name = format!("HTTP {method} {path}");
        observability::emit_span("helix-http", &name, ms, ok, traceparent.as_deref());
    }
    res
}

/// Fixed-window rate limiter. In-process by default; optional shared Postgres backend
/// for multi-replica (HELIX_RATE_LIMIT_BACKEND=postgres + pool).
#[derive(Clone, Default)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    rps: u32,
    /// When set, check also writes through to helix_core.rate_buckets.
    pg: Option<sqlx::PgPool>,
    /// Whether silent fallback to in-process rate limiting is allowed on Postgres errors.
    allow_pg_fallback: bool,
}

impl RateLimiter {
    pub fn new(rps: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            rps,
            pg: None,
            allow_pg_fallback: true,
        }
    }

    pub fn with_postgres(rps: u32, pool: sqlx::PgPool, allow_pg_fallback: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            rps,
            pg: Some(pool),
            allow_pg_fallback,
        }
    }

    pub fn check(&self, key: &str) -> bool {
        if self.rps == 0 {
            return true;
        }
        let mut guard = self.inner.lock();
        let now = Instant::now();
        if guard.len() > 10_000 {
            guard.retain(|_, (_, t)| now.duration_since(*t) < Duration::from_secs(5));
        }
        let entry = guard.entry(key.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) >= Duration::from_secs(1) {
            *entry = (1, now);
            return true;
        }
        if entry.0 >= self.rps {
            return false;
        }
        entry.0 += 1;
        true
    }

    /// Shared multi-replica check when Postgres pool configured.
    /// `tenant_id` scopes the bucket key for aggregate tenant limits.
    pub async fn check_async(&self, key: &str, tenant_id: Option<&str>) -> bool {
        if self.rps == 0 {
            return true;
        }
        let scoped_key = match tenant_id {
            Some(t) if !t.is_empty() => format!("{t}:{key}"),
            _ => key.to_string(),
        };
        let Some(pool) = self.pg.as_ref() else {
            return self.check(&scoped_key);
        };
        let window = chrono::Utc::now().timestamp();
        let result = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO helix_core.rate_buckets (bucket_key, window_epoch, count)
            VALUES ($1, $2, 1)
            ON CONFLICT (bucket_key, window_epoch)
            DO UPDATE SET count = helix_core.rate_buckets.count + 1
            RETURNING count
            "#,
        )
        .bind(&scoped_key)
        .bind(window)
        .fetch_one(pool)
        .await;
        match result {
            Ok(count) => (count as u32) <= self.rps,
            Err(e) => {
                if self.allow_pg_fallback {
                    tracing::debug!(error = %e, "pg rate limit failed — local fallback");
                    self.check(&scoped_key)
                } else {
                    tracing::error!(error = %e, key = %scoped_key, "pg rate limit failed — failing closed");
                    false
                }
            }
        }
    }
}

/// Health/readiness/probe endpoints are never rate-limited.
fn rate_limit_skip(path: &str) -> bool {
    matches!(
        path,
        "/healthz" | "/readyz" | "/livez" | "/v1/meta" | "/metrics"
    ) || path.starts_with("/healthz")
        || path.starts_with("/readyz")
}

pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if limiter.rps == 0 {
        return next.run(req).await;
    }
    let path = req.uri().path().to_string();
    if rate_limit_skip(&path) {
        return next.run(req).await;
    }
    let method = req.method().to_string();
    let path_prefix = path.split('/').take(3).collect::<Vec<_>>().join("/");
    let mut tenant_id: Option<&str> = None;
    let key = req
        .headers()
        .get("x-helix-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| {
            tenant_id = None; // tenant resolved from key by the rate limit backend if needed
            format!("{method}:{path_prefix}:k:{}", &s[..s.len().min(20)])
        })
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| {
                    format!(
                        "{method}:{path_prefix}:ip:{}",
                        s.split(',').next().unwrap_or(s).trim()
                    )
                })
        })
        .or_else(|| {
            req.headers()
                .get("x-helix-dev-user")
                .and_then(|v| v.to_str().ok())
                .map(|s| {
                    tenant_id = Some(s.trim());
                    format!("{method}:{path_prefix}:u:{s}")
                })
        })
        .unwrap_or_else(|| format!("{method}:{path_prefix}:anon"));

    if !limiter.check_async(&key, tenant_id).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", "1")],
            r#"{"error":{"code":"rate_limited","message":"rate limit exceeded"}}"#,
        )
            .into_response();
    }
    next.run(req).await
}

/// No-op placeholder for future tower middleware layering.
pub async fn request_id_middleware() {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn parses_bearer_and_cookie() {
        let mut h = HeaderMap::new();
        h.insert("authorization", HeaderValue::from_static("Bearer tok-1"));
        assert_eq!(session_token_from_headers(&h).as_deref(), Some("tok-1"));

        let mut h2 = HeaderMap::new();
        h2.insert(
            "cookie",
            HeaderValue::from_static("a=1; ory_kratos_session=sess-xyz; b=2"),
        );
        assert_eq!(session_token_from_headers(&h2).as_deref(), Some("sess-xyz"));
    }

    #[test]
    fn api_key_not_session() {
        let mut h = HeaderMap::new();
        h.insert(
            "authorization",
            HeaderValue::from_static("Bearer hk_live_abc_secret"),
        );
        assert!(session_token_from_headers(&h).is_none());
        assert_eq!(
            api_key_from_headers(&h).as_deref(),
            Some("hk_live_abc_secret")
        );
    }

    #[test]
    fn rate_limiter_trips() {
        let rl = RateLimiter::new(2);
        assert!(rl.check("a"));
        assert!(rl.check("a"));
        assert!(!rl.check("a"));
    }
}
