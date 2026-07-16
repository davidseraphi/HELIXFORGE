//! Auth client for Ory Kratos sessions, Hydra OIDC introspection, and gated local dev.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared_core::ids::{TenantId, UserId};
use shared_core::tenancy::{Principal, Scope};
use shared_core::{HelixError, HelixResult};
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub kratos_public_url: String,
    pub kratos_admin_url: String,
    pub hydra_public_url: String,
    pub hydra_admin_url: String,
    /// Only when HELIX_ALLOW_DEV_HEADERS=1 and HELIX_ENV=local.
    pub allow_dev_headers: bool,
    pub default_residency: String,
    /// Optional basic auth for Hydra introspection (RFC 7662).
    pub hydra_introspect_client_id: Option<String>,
    pub hydra_introspect_client_secret: Option<String>,
    /// Optional basic auth for Hydra admin endpoints (e.g. /admin/clients).
    pub hydra_admin_client_id: Option<String>,
    pub hydra_admin_client_secret: Option<String>,
    /// True when HELIX_ENV != "local". Used to fail closed on missing Ory metadata.
    pub strict: bool,
    /// Residency values accepted from identity providers. Defaults to [default_residency].
    pub residency_allowlist: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            kratos_public_url: "http://127.0.0.1:4433".into(),
            kratos_admin_url: "http://127.0.0.1:4434".into(),
            hydra_public_url: "http://127.0.0.1:4444".into(),
            hydra_admin_url: "http://127.0.0.1:4445".into(),
            // Fail-closed by default (Kimi P0).
            allow_dev_headers: false,
            default_residency: "local".into(),
            hydra_introspect_client_id: None,
            hydra_introspect_client_secret: None,
            hydra_admin_client_id: None,
            hydra_admin_client_secret: None,
            strict: false,
            residency_allowlist: vec!["local".into()],
        }
    }
}

impl AuthConfig {
    /// Build from environment. Dev headers require BOTH local env and explicit flag.
    pub fn from_env_flags(environment: &str, default_residency: &str) -> Self {
        let allow = environment == "local"
            && std::env::var("HELIX_ALLOW_DEV_HEADERS")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
        let strict = environment != "local";
        let residency_allowlist: Vec<String> = std::env::var("HELIX_RESIDENCY_ALLOWLIST")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
            .unwrap_or_else(|| vec![default_residency.to_string()]);
        Self {
            kratos_public_url: std::env::var("KRATOS_PUBLIC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4433".into()),
            kratos_admin_url: std::env::var("KRATOS_ADMIN_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4434".into()),
            hydra_public_url: std::env::var("HYDRA_PUBLIC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4444".into()),
            hydra_admin_url: std::env::var("HYDRA_ADMIN_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4445".into()),
            allow_dev_headers: allow,
            default_residency: default_residency.into(),
            hydra_introspect_client_id: std::env::var("HELIX_HYDRA_INTROSPECT_CLIENT_ID").ok(),
            hydra_introspect_client_secret: std::env::var("HELIX_HYDRA_INTROSPECT_CLIENT_SECRET")
                .ok(),
            hydra_admin_client_id: std::env::var("HELIX_HYDRA_ADMIN_CLIENT_ID").ok(),
            hydra_admin_client_secret: std::env::var("HELIX_HYDRA_ADMIN_CLIENT_SECRET").ok(),
            strict,
            residency_allowlist,
        }
    }

    /// Build from the canonical `CoreConfig`. This is the preferred path in Phase 6.
    pub fn from_core_config(cfg: &shared_core::config::CoreConfig) -> Self {
        let strict = cfg.environment != "local";
        Self {
            kratos_public_url: cfg.ory_kratos_public.clone(),
            kratos_admin_url: cfg.ory_kratos_admin.clone(),
            hydra_public_url: cfg.ory_hydra_public.clone(),
            hydra_admin_url: cfg.ory_hydra_admin.clone(),
            allow_dev_headers: cfg.environment == "local" && cfg.allow_dev_headers,
            default_residency: cfg.data_residency_region.clone(),
            hydra_introspect_client_id: cfg.hydra_introspect_client_id.clone(),
            hydra_introspect_client_secret: cfg.hydra_introspect_client_secret.clone(),
            hydra_admin_client_id: cfg.hydra_admin_client_id.clone(),
            hydra_admin_client_secret: cfg.hydra_admin_client_secret.clone(),
            strict,
            residency_allowlist: cfg.residency_allowlist.clone(),
        }
    }
}

#[async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn resolve_session(
        &self,
        session_token: Option<&str>,
        dev_user: Option<&str>,
    ) -> HelixResult<Principal>;
    async fn health(&self) -> HelixResult<AuthHealth>;
    /// Whether dev headers are allowed in this runtime configuration.
    fn dev_headers_allowed(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthHealth {
    pub mode: String,
    pub kratos_reachable: bool,
    #[serde(default)]
    pub hydra_reachable: bool,
    #[serde(default)]
    pub dev_headers_allowed: bool,
}

#[derive(Clone)]
pub struct AuthClient {
    inner: Arc<dyn IdentityProvider>,
}

impl AuthClient {
    pub fn new(provider: Arc<dyn IdentityProvider>) -> Self {
        Self { inner: provider }
    }

    pub fn from_config(cfg: AuthConfig) -> Self {
        Self::new(Arc::new(HybridIdentityProvider::new(cfg)))
    }

    pub fn dev() -> Self {
        Self::new(Arc::new(DevIdentityProvider::default()))
    }

    pub async fn resolve(
        &self,
        session_token: Option<&str>,
        dev_user: Option<&str>,
    ) -> HelixResult<Principal> {
        self.inner.resolve_session(session_token, dev_user).await
    }

    pub async fn health(&self) -> HelixResult<AuthHealth> {
        self.inner.health().await
    }

    pub fn dev_headers_allowed(&self) -> bool {
        self.inner.dev_headers_allowed()
    }
}

/// Local development identity — gated; default scopes are least-privilege.
#[derive(Default)]
pub struct DevIdentityProvider {
    pub residency: String,
}

impl DevIdentityProvider {
    pub fn new(residency: impl Into<String>) -> Self {
        Self {
            residency: residency.into(),
        }
    }
}

fn dev_scopes_for_label(label: &str) -> Vec<Scope> {
    // Platform only with explicit env or ops@ operator label for local smokes.
    let elevate = std::env::var("HELIX_DEV_PLATFORM")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
        || label.starts_with("ops@");
    if elevate {
        vec![
            Scope::Read,
            Scope::Write,
            Scope::Admin,
            Scope::AuditRead,
            Scope::Platform,
        ]
    } else {
        vec![Scope::Read, Scope::Write, Scope::AuditRead]
    }
}

#[async_trait]
impl IdentityProvider for DevIdentityProvider {
    async fn resolve_session(
        &self,
        _session_token: Option<&str>,
        dev_user: Option<&str>,
    ) -> HelixResult<Principal> {
        let label = dev_user.ok_or_else(|| {
            HelixError::unauthorized("x-helix-dev-user required when using dev identity")
        })?;
        debug!(%label, "dev identity resolve");
        let user_id = UserId::from_uuid(uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("helixforge-user:{label}").as_bytes(),
        ));
        let tenant_id = TenantId::from_uuid(uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            b"helixforge-tenant:local-dev",
        ));
        Ok(Principal {
            user_id,
            tenant_id,
            org_id: None,
            scopes: dev_scopes_for_label(label),
            session_id: Some(format!("dev-session:{label}")),
            residency_region: if self.residency.is_empty() {
                "local".into()
            } else {
                self.residency.clone()
            },
        })
    }

    async fn health(&self) -> HelixResult<AuthHealth> {
        Ok(AuthHealth {
            mode: "dev".into(),
            kratos_reachable: false,
            hydra_reachable: false,
            dev_headers_allowed: true,
        })
    }

    fn dev_headers_allowed(&self) -> bool {
        true
    }
}

pub struct HybridIdentityProvider {
    cfg: AuthConfig,
    http: reqwest::Client,
    health_http: reqwest::Client,
    dev: DevIdentityProvider,
}

impl HybridIdentityProvider {
    pub fn new(cfg: AuthConfig) -> Self {
        let residency = cfg.default_residency.clone();
        Self {
            cfg,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
            health_http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(400))
                .build()
                .expect("reqwest health client"),
            dev: DevIdentityProvider::new(residency),
        }
    }

    async fn whoami(&self, session_token: &str) -> HelixResult<KratosSession> {
        let url = format!(
            "{}/sessions/whoami",
            self.cfg.kratos_public_url.trim_end_matches('/')
        );
        let resp = self
            .http
            .get(&url)
            .header("Accept", "application/json")
            .header("X-Session-Token", session_token)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("kratos whoami: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(HelixError::unauthorized(format!(
                "invalid or expired session ({status}): {body}"
            )));
        }
        let text = resp
            .text()
            .await
            .map_err(|e| HelixError::dependency(format!("kratos body: {e}")))?;
        serde_json::from_str::<KratosSession>(&text).map_err(|e| {
            HelixError::dependency(format!(
                "kratos decode: {e}; body_prefix={}",
                &text[..text.len().min(200)]
            ))
        })
    }

    async fn introspect_oauth(&self, access_token: &str) -> HelixResult<HydraIntrospection> {
        let url = format!(
            "{}/admin/oauth2/introspect",
            self.cfg.hydra_admin_url.trim_end_matches('/')
        );
        let has_creds = self.cfg.hydra_introspect_client_id.is_some()
            && self.cfg.hydra_introspect_client_secret.is_some();
        if self.cfg.strict && !has_creds {
            return Err(HelixError::unauthorized(
                "hydra introspection credentials required outside local",
            ));
        }
        let mut req = self
            .http
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .body(format!("token={access_token}"));
        if let (Some(id), Some(secret)) = (
            self.cfg.hydra_introspect_client_id.as_ref(),
            self.cfg.hydra_introspect_client_secret.as_ref(),
        ) {
            req = req.basic_auth(id, Some(secret));
        }
        let resp = req
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("hydra introspect: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(HelixError::unauthorized(format!(
                "hydra introspect {status}: {body}"
            )));
        }
        resp.json::<HydraIntrospection>()
            .await
            .map_err(|e| HelixError::dependency(format!("hydra introspect decode: {e}")))
    }
}

#[derive(Debug, Deserialize)]
struct HydraIntrospection {
    active: bool,
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    ext: Option<serde_json::Value>,
}

fn principal_from_oauth(intro: &HydraIntrospection, cfg: &AuthConfig) -> HelixResult<Principal> {
    if !intro.active {
        return Err(HelixError::unauthorized("oauth token inactive"));
    }
    let sub = intro
        .sub
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(intro.client_id.as_deref())
        .ok_or_else(|| HelixError::unauthorized("oauth token missing sub"))?;

    let user_id = uuid::Uuid::parse_str(sub)
        .map(UserId::from_uuid)
        .unwrap_or_else(|_| {
            UserId::from_uuid(uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_DNS,
                format!("helixforge-oauth-sub:{sub}").as_bytes(),
            ))
        });

    let tenant_id: TenantId = intro
        .ext
        .as_ref()
        .and_then(|e| e.get("tenant_id"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            if cfg.strict {
                HelixError::unauthorized("oauth token missing tenant_id")
            } else {
                HelixError::unauthorized("oauth token missing tenant_id (local fallback disabled)")
            }
        })?;

    let scopes = intro
        .scope
        .as_deref()
        .and_then(Scope::parse_list)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            if cfg.strict {
                HelixError::unauthorized("oauth token missing scope")
            } else {
                HelixError::unauthorized("oauth token missing scope (local fallback disabled)")
            }
        })?;

    let residency = intro
        .ext
        .as_ref()
        .and_then(|e| e.get("residency_region"))
        .and_then(|v| v.as_str())
        .unwrap_or(&cfg.default_residency)
        .to_string();

    if cfg.strict && !cfg.residency_allowlist.iter().any(|r| r == &residency) {
        return Err(HelixError::forbidden(format!(
            "oauth residency {residency} not in allowlist"
        )));
    }

    Ok(Principal {
        user_id,
        tenant_id,
        org_id: None,
        scopes,
        session_id: Some(format!(
            "oauth:{}",
            intro.client_id.as_deref().unwrap_or("client")
        )),
        residency_region: residency,
    })
}

#[derive(Debug, Deserialize)]
struct KratosSession {
    id: String,
    #[serde(default)]
    #[allow(dead_code)]
    active: Option<bool>,
    identity: KratosIdentity,
}

#[derive(Debug, Deserialize)]
struct KratosIdentity {
    id: String,
    #[serde(default)]
    metadata_public: Option<serde_json::Value>,
    #[serde(default)]
    traits: serde_json::Value,
}

fn principal_from_kratos(session: &KratosSession, cfg: &AuthConfig) -> HelixResult<Principal> {
    let user_id = uuid::Uuid::parse_str(&session.identity.id)
        .map(UserId::from_uuid)
        .unwrap_or_else(|_| UserId::new());

    let _email = session
        .identity
        .traits
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let meta = session.identity.metadata_public.as_ref();

    let tenant_id: TenantId = meta
        .and_then(|m| m.get("tenant_id"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| HelixError::unauthorized("identity missing tenant_id in metadata_public"))?;

    let scopes = meta
        .and_then(|m| m.get("scopes"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .filter_map(Scope::parse_token)
                .collect::<Vec<_>>()
        })
        .filter(|s| !s.is_empty())
        .ok_or_else(|| HelixError::unauthorized("identity missing scopes in metadata_public"))?;

    let residency = meta
        .and_then(|m| m.get("residency_region"))
        .and_then(|v| v.as_str())
        .unwrap_or(&cfg.default_residency)
        .to_string();

    if cfg.strict && !cfg.residency_allowlist.iter().any(|r| r == &residency) {
        return Err(HelixError::forbidden(format!(
            "identity residency {residency} not in allowlist"
        )));
    }

    Ok(Principal {
        user_id,
        tenant_id,
        org_id: None,
        scopes,
        session_id: Some(session.id.clone()),
        residency_region: residency,
    })
}

/// Admin + public helpers for live Ory Kratos.
#[derive(Clone)]
pub struct KratosAdmin {
    public_url: String,
    admin_url: String,
    http: reqwest::Client,
}

impl KratosAdmin {
    pub fn new(public_url: impl Into<String>, admin_url: impl Into<String>) -> Self {
        Self {
            public_url: public_url.into(),
            admin_url: admin_url.into(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("kratos admin http"),
        }
    }

    pub async fn ready(&self) -> bool {
        let url = format!("{}/health/ready", self.public_url.trim_end_matches('/'));
        self.http
            .get(url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn create_identity(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        residency_region: &str,
        scopes: &[&str],
    ) -> HelixResult<serde_json::Value> {
        let url = format!("{}/admin/identities", self.admin_url.trim_end_matches('/'));
        let metadata = serde_json::json!({
            "residency_region": residency_region,
            "tenant_id": tenant_id,
            "scopes": scopes
        });
        let body = serde_json::json!({
            "schema_id": "default",
            "traits": { "email": email },
            "metadata_public": metadata,
            "credentials": {
                "password": { "config": { "password": password } }
            }
        });
        let resp = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("kratos create identity: {e}")))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(HelixError::dependency(format!(
                "kratos create identity {status}: {text}"
            )));
        }
        serde_json::from_str(&text)
            .map_err(|e| HelixError::dependency(format!("kratos identity decode: {e}")))
    }

    pub async fn login_password(
        &self,
        email: &str,
        password: &str,
    ) -> HelixResult<KratosLoginResult> {
        let flow_url = format!(
            "{}/self-service/login/api",
            self.public_url.trim_end_matches('/')
        );
        let flow_resp = self
            .http
            .get(&flow_url)
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("kratos login flow: {e}")))?;
        if !flow_resp.status().is_success() {
            return Err(HelixError::dependency(format!(
                "kratos login flow status {}",
                flow_resp.status()
            )));
        }
        let flow: serde_json::Value = flow_resp
            .json()
            .await
            .map_err(|e| HelixError::dependency(format!("kratos flow decode: {e}")))?;
        let flow_id = flow
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HelixError::dependency("kratos flow missing id"))?;

        let submit_url = format!(
            "{}/self-service/login?flow={flow_id}",
            self.public_url.trim_end_matches('/')
        );
        let submit = self
            .http
            .post(submit_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "method": "password",
                "identifier": email,
                "password": password
            }))
            .send()
            .await
            .map_err(|e| HelixError::dependency(format!("kratos login submit: {e}")))?;
        let status = submit.status();
        let body: serde_json::Value = submit
            .json()
            .await
            .map_err(|e| HelixError::dependency(format!("kratos login body: {e}")))?;
        if !status.is_success() {
            return Err(HelixError::unauthorized(format!(
                "kratos login failed {status}: {body}"
            )));
        }
        let session_token = body
            .get("session_token")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| HelixError::dependency("kratos login missing session_token"))?;
        let session = body.get("session").cloned().unwrap_or(body);
        Ok(KratosLoginResult {
            session_token,
            session,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KratosLoginResult {
    pub session_token: String,
    pub session: serde_json::Value,
}

#[async_trait]
impl IdentityProvider for HybridIdentityProvider {
    async fn resolve_session(
        &self,
        session_token: Option<&str>,
        dev_user: Option<&str>,
    ) -> HelixResult<Principal> {
        // If any credential token is presented, never fall back to dev (Kimi P0).
        if let Some(token) = session_token {
            match self.whoami(token).await {
                Ok(session) => {
                    return principal_from_kratos(&session, &self.cfg);
                }
                Err(kratos_err) => match self.introspect_oauth(token).await {
                    Ok(intro) => {
                        return principal_from_oauth(&intro, &self.cfg);
                    }
                    Err(oauth_err) => {
                        debug!(%kratos_err, %oauth_err, "credential rejected — fail closed");
                        return Err(HelixError::unauthorized("invalid session or oauth token"));
                    }
                },
            }
        }

        // Dev path only when no token and explicitly allowed.
        if self.cfg.allow_dev_headers {
            return self.dev.resolve_session(None, dev_user).await;
        }

        Err(HelixError::unauthorized("authentication required"))
    }

    async fn health(&self) -> HelixResult<AuthHealth> {
        let kratos_url = format!(
            "{}/health/ready",
            self.cfg.kratos_public_url.trim_end_matches('/')
        );
        let hydra_url = format!(
            "{}/health/ready",
            self.cfg.hydra_public_url.trim_end_matches('/')
        );
        let kratos_reachable = self
            .health_http
            .get(&kratos_url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        let hydra_reachable = self
            .health_http
            .get(&hydra_url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        let mode = if kratos_reachable && hydra_reachable {
            "kratos+hydra"
        } else if kratos_reachable {
            "kratos"
        } else if hydra_reachable {
            "hydra"
        } else if self.cfg.allow_dev_headers {
            "dev-fallback"
        } else {
            "ory-required"
        };
        Ok(AuthHealth {
            mode: mode.into(),
            kratos_reachable,
            hydra_reachable,
            dev_headers_allowed: self.cfg.allow_dev_headers,
        })
    }

    fn dev_headers_allowed(&self) -> bool {
        self.cfg.allow_dev_headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dev_requires_user_header() {
        let p = DevIdentityProvider::default();
        assert!(p.resolve_session(None, None).await.is_err());
        let ok = p
            .resolve_session(None, Some("alice@x.local"))
            .await
            .unwrap();
        assert!(ok.has_scope(&Scope::Read));
        assert!(!ok.has_scope(&Scope::Platform));
    }

    #[tokio::test]
    async fn ops_dev_user_gets_platform() {
        let p = DevIdentityProvider::default();
        let ok = p
            .resolve_session(None, Some("ops@helixforge.local"))
            .await
            .unwrap();
        assert!(ok.has_scope(&Scope::Platform));
    }

    #[tokio::test]
    async fn shared_local_tenant_across_users() {
        let client = AuthClient::dev();
        let a = client.resolve(None, Some("a@x.local")).await.unwrap();
        let b = client.resolve(None, Some("b@x.local")).await.unwrap();
        assert_eq!(a.tenant_id, b.tenant_id);
        assert_ne!(a.user_id, b.user_id);
    }

    #[test]
    fn default_config_disallows_dev_headers() {
        assert!(!AuthConfig::default().allow_dev_headers);
    }

    #[test]
    fn from_env_flags_requires_local_and_flag() {
        // Without env flag, never allow even if environment string is local.
        std::env::remove_var("HELIX_ALLOW_DEV_HEADERS");
        let cfg = AuthConfig::from_env_flags("local", "local");
        assert!(!cfg.allow_dev_headers);
        let cfg2 = AuthConfig::from_env_flags("prod", "eu-west");
        assert!(!cfg2.allow_dev_headers);
    }

    #[tokio::test]
    async fn hybrid_fail_closed_without_token_or_dev() {
        let cfg = AuthConfig {
            allow_dev_headers: false,
            ..Default::default()
        };
        let p = HybridIdentityProvider::new(cfg);
        let err = p.resolve_session(None, Some("alice@x.local")).await;
        assert!(err.is_err());
    }

    #[test]
    fn strict_mode_requires_hydra_introspection_creds() {
        let cfg = AuthConfig {
            strict: true,
            ..Default::default()
        };
        // No introspection credentials → should fail before network.
        let p = HybridIdentityProvider::new(cfg);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(p.introspect_oauth("dummy-token"))
            .expect_err("should fail closed without hydra creds");
        assert!(err.to_string().contains("credentials required"));
    }

    #[test]
    fn principal_from_kratos_fails_closed_without_tenant_or_scopes() {
        let cfg = AuthConfig {
            strict: true,
            residency_allowlist: vec!["eu-west".into()],
            ..Default::default()
        };
        let session = KratosSession {
            id: "test-session".into(),
            active: None,
            identity: KratosIdentity {
                id: uuid::Uuid::new_v4().to_string(),
                metadata_public: Some(serde_json::json!({})),
                traits: serde_json::json!({}),
            },
        };
        assert!(principal_from_kratos(&session, &cfg).is_err());
    }

    #[test]
    fn principal_from_kratos_validates_residency() {
        let cfg = AuthConfig {
            strict: true,
            residency_allowlist: vec!["eu-west".into()],
            default_residency: "eu-west".into(),
            ..Default::default()
        };
        let tid = uuid::Uuid::new_v4();
        let session = KratosSession {
            id: "test-session".into(),
            active: None,
            identity: KratosIdentity {
                id: uuid::Uuid::new_v4().to_string(),
                metadata_public: Some(serde_json::json!({
                    "tenant_id": tid.to_string(),
                    "scopes": ["read"],
                    "residency_region": "us-east"
                })),
                traits: serde_json::json!({}),
            },
        };
        let err = principal_from_kratos(&session, &cfg).expect_err("residency not allowed");
        assert!(err.to_string().contains("residency"));
    }
}
