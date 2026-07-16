//! HelixCore Auth Adapter — Ory Kratos/Hydra façade + local dev sessions.

use audit_log::AuditEvent;
use auth_client::KratosAdmin;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use service_kit::{serve_with_shutdown, ApiError, AppState, RequireAuth, ServiceBuilder};
use shared_core::ids::{TenantId, UserId};
use shared_core::tenancy::{Actor, Role, Scope};
use shared_core::{ApiResponse, HelixError, HelixResult};
use subtle::ConstantTimeEq;

#[tokio::main]
async fn main() -> HelixResult<()> {
    let builder = ServiceBuilder::new("auth-adapter", 8085).await?;
    let addr = builder.config().listen_addr;
    let state = builder.into_state();

    let app = ServiceBuilder::base_router(state.clone()).merge(
        Router::new()
            .route("/v1/session/whoami", get(whoami))
            .route("/v1/session/dev-login", post(dev_login))
            .route("/v1/session/introspect", get(introspect))
            .route("/v1/session/scopes", get(list_scopes))
            .route("/v1/auth/health", get(auth_health))
            .route("/v1/auth/residency", get(residency_info))
            .route("/v1/ory/register", post(ory_register))
            .route("/v1/ory/login", post(ory_login))
            .route("/v1/ory/status", get(ory_status))
            .route("/v1/oidc/status", get(oidc_status))
            .route("/v1/oidc/introspect", post(oidc_introspect))
            .route("/v1/oidc/clients", post(oidc_create_client))
            .with_state(state.clone()),
    );

    serve_with_shutdown(addr, app, "auth-adapter", state.clone()).await
}

fn kratos(state: &AppState) -> KratosAdmin {
    KratosAdmin::new(
        state.clients.config.ory_kratos_public.clone(),
        state.clients.config.ory_kratos_admin.clone(),
    )
}

async fn whoami(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<shared_core::tenancy::Principal>>, ApiError> {
    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: shared_core::tenancy::Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "auth.whoami".into(),
            resource_type: "session".into(),
            resource_id: principal.session_id.clone().unwrap_or_default(),
            metadata: serde_json::json!({"scopes": principal.scopes.iter().map(Scope::as_str).collect::<Vec<_>>()}),
            residency_region: principal.residency_region.clone(),
        })
        .await;
    Ok(Json(ApiResponse::ok(principal)))
}

async fn introspect(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: shared_core::tenancy::Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "auth.introspect".into(),
            resource_type: "session".into(),
            resource_id: principal.session_id.clone().unwrap_or_default(),
            metadata: serde_json::json!({"service_residency": state.clients.config.data_residency_region}),
            residency_region: principal.residency_region.clone(),
        })
        .await;
    let auth = state.clients.auth.health().await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "principal": principal,
        "scopes": principal.scopes.iter().map(Scope::as_str).collect::<Vec<_>>(),
        "auth_mode": auth.mode,
        "kratos_reachable": auth.kratos_reachable,
        "service_residency": state.clients.config.data_residency_region,
        "environment": state.clients.config.environment,
        "dev_headers_allowed": auth.dev_headers_allowed
    }))))
}

async fn list_scopes(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: shared_core::tenancy::Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "auth.list_scopes".into(),
            resource_type: "session".into(),
            resource_id: principal.session_id.clone().unwrap_or_default(),
            metadata: serde_json::json!({"granted": principal.scopes.iter().map(Scope::as_str).collect::<Vec<_>>()}),
            residency_region: principal.residency_region.clone(),
        })
        .await;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "granted": principal.scopes.iter().map(Scope::as_str).collect::<Vec<_>>(),
        "catalog": ["read", "write", "admin", "platform", "audit_read"],
        "hints": {
            "x_helix_dev_scopes": "local only, master-key gated",
            "cookie": "ory_kratos_session",
            "bearer": "Authorization: Bearer <kratos session_token>",
            "ory_login": "POST /v1/ory/login when Kratos is up"
        }
    }))))
}

async fn residency_info(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let service_region = &state.clients.config.data_residency_region;
    let match_ok = principal.residency_region == *service_region || service_region == "local";
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "principal_region": principal.residency_region,
        "service_region": service_region,
        "allowed": match_ok,
        "policy": "mismatch fails closed when service residency is not local"
    }))))
}

#[derive(Deserialize)]
struct DevLogin {
    email: String,
    #[serde(default)]
    scopes: Option<String>,
    #[serde(default)]
    residency: Option<String>,
}

async fn dev_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<DevLogin>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    // Local only + explicit HELIX_ALLOW_DEV_HEADERS (Kimi P0).
    let cfg = &state.clients.config;
    if cfg.environment != "local" {
        return Err(HelixError::forbidden("dev-login disabled outside local").into());
    }
    if !cfg.allow_dev_headers {
        return Err(HelixError::forbidden("set HELIX_ALLOW_DEV_HEADERS=1 for dev-login").into());
    }
    // Optional operator token when HELIX_DEV_LOGIN_TOKEN is set.
    if let Some(expected) = cfg.dev_login_token.as_deref() {
        let provided = headers
            .get("x-helix-dev-login-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let eq = expected.as_bytes().ct_eq(provided.as_bytes());
        if eq.unwrap_u8() == 0 {
            return Err(HelixError::forbidden("invalid dev-login operator token").into());
        }
    }
    let mut principal = state.clients.auth.resolve(None, Some(&body.email)).await?;
    // Scope escalation only for ops@ or when HELIX_DEV_PLATFORM=1; ignore arbitrary Platform/Admin.
    if let Some(raw) = body.scopes.as_deref() {
        if let Some(mut scopes) = Scope::parse_list(raw) {
            let elevate = cfg.dev_platform || body.email.starts_with("ops@");
            if !elevate {
                scopes.retain(|s| !matches!(s, Scope::Platform | Scope::Admin));
                if scopes.is_empty() {
                    scopes = vec![Scope::Read, Scope::Write, Scope::AuditRead];
                }
            }
            principal = principal.with_scopes(scopes);
        }
    }
    if let Some(region) = body
        .residency
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        principal = principal.with_residency(region);
    }
    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "auth.dev_login".into(),
            resource_type: "session".into(),
            resource_id: body.email.clone(),
            metadata: serde_json::json!({
                "scopes": principal.scopes.iter().map(Scope::as_str).collect::<Vec<_>>(),
                "residency": principal.residency_region,
            }),
            residency_region: principal.residency_region.clone(),
        })
        .await;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "principal": principal,
        "hint": "Pass X-Helix-Dev-User with this email; Platform requires HELIX_DEV_PLATFORM=1 or ops@"
    }))))
}

async fn auth_health(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<auth_client::AuthHealth>>, ApiError> {
    Ok(Json(ApiResponse::ok(state.clients.auth.health().await?)))
}

async fn ory_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let k = kratos(&state);
    let ready = k.ready().await;
    let auth = state.clients.auth.health().await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "kratos_public": state.clients.config.ory_kratos_public,
        "kratos_admin": state.clients.config.ory_kratos_admin,
        "hydra_public": state.clients.config.ory_hydra_public,
        "hydra_admin": state.clients.config.ory_hydra_admin,
        "ready": ready,
        "kratos_reachable": auth.kratos_reachable,
        "hydra_reachable": auth.hydra_reachable,
        "auth_mode": auth.mode,
        "hint": if ready {
            "POST /v1/ory/register then /v1/ory/login for live session_token; OIDC via /v1/oidc/*"
        } else {
            "docker compose --profile ory up -d  (kratos + hydra)"
        }
    }))))
}

async fn oidc_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let auth = state.clients.auth.health().await?;
    let public = state.clients.config.ory_hydra_public.trim_end_matches('/');
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "hydra_reachable": auth.hydra_reachable,
        "issuer": format!("{public}/"),
        "discovery": format!("{public}/.well-known/openid-configuration"),
        "token": format!("{public}/oauth2/token"),
        "auth": format!("{public}/oauth2/auth"),
        "introspect": "/v1/oidc/introspect",
        "create_client": "POST /v1/oidc/clients (Admin)"
    }))))
}

#[derive(Deserialize)]
struct IntrospectBody {
    token: String,
}

/// Introspect an OAuth2 access token (Hydra) and map to a Helix principal when active.
async fn oidc_introspect(
    State(state): State<AppState>,
    RequireAuth(caller): RequireAuth,
    Json(body): Json<IntrospectBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    // Resolve via hybrid path (kratos first, then hydra introspect).
    let principal = state
        .clients
        .auth
        .resolve(Some(body.token.trim()), None)
        .await?;
    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(caller.tenant_id),
            actor: Actor::User {
                user_id: caller.user_id,
                tenant_id: caller.tenant_id,
            },
            action: "auth.oidc_introspect".into(),
            resource_type: "oauth_token".into(),
            resource_id: format!(
                "{:.8}...{:.8}",
                body.token,
                &body.token[body.token.len().saturating_sub(8)..]
            ),
            metadata: serde_json::json!({
                "subject_tenant_id": principal.tenant_id,
                "subject_user_id": principal.user_id,
            }),
            residency_region: caller.residency_region.clone(),
        })
        .await;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "active": true,
        "principal": principal
    }))))
}

#[derive(Deserialize)]
struct CreateOidcClientBody {
    client_name: String,
    #[serde(default)]
    redirect_uris: Option<Vec<String>>,
    #[serde(default)]
    grant_types: Option<Vec<String>>,
    #[serde(default)]
    scope: Option<String>,
}

/// Register a public/confidential OAuth2 client in Hydra (Admin).
async fn oidc_create_client(
    State(state): State<AppState>,
    RequireAuth(principal): RequireAuth,
    Json(body): Json<CreateOidcClientBody>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    principal.require_scope(Scope::Admin)?;
    let admin = state.clients.config.ory_hydra_admin.trim_end_matches('/');
    let url = format!("{admin}/admin/clients");
    let admin_id = state.clients.config.hydra_admin_client_id.clone();
    let admin_secret = state.clients.config.hydra_admin_client_secret.clone();
    let has_admin_creds = admin_id.as_ref().zip(admin_secret.as_ref()).is_some();
    if state.clients.config.environment != "local" && !has_admin_creds {
        return Err(HelixError::forbidden("hydra admin credentials required outside local").into());
    }
    let client_name = body.client_name.clone();
    let redirect = body
        .redirect_uris
        .clone()
        .unwrap_or_else(|| vec!["http://127.0.0.1:3000/callback".into()]);
    let grant_types = body.grant_types.clone().unwrap_or_else(|| {
        vec![
            "authorization_code".into(),
            "refresh_token".into(),
            "client_credentials".into(),
        ]
    });
    let scope = body
        .scope
        .clone()
        .unwrap_or_else(|| "openid offline read write".into());
    let payload = serde_json::json!({
        "client_name": client_name,
        "redirect_uris": redirect.clone(),
        "grant_types": grant_types.clone(),
        "response_types": ["code", "id_token"],
        "scope": scope.clone(),
        "token_endpoint_auth_method": "client_secret_post",
    });
    let client = reqwest::Client::new();
    let mut req = client.post(url).json(&payload);
    if let (Some(id), Some(secret)) = (admin_id, admin_secret) {
        req = req.basic_auth(id, Some(secret));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| HelixError::dependency(format!("hydra create client: {e}")))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(HelixError::dependency(format!("hydra create client {status}: {text}")).into());
    }
    let created: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| HelixError::dependency(format!("hydra client decode: {e}")))?;
    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "auth.oidc_create_client".into(),
            resource_type: "oidc_client".into(),
            resource_id: client_name.clone(),
            metadata: serde_json::json!({
                "grant_types": grant_types,
                "scope": scope,
                "redirect_uris": redirect,
            }),
            residency_region: principal.residency_region.clone(),
        })
        .await;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "client": created,
        "note": "Store client_secret now if returned; use client_credentials or auth code flow"
    }))))
}

#[derive(Deserialize)]
struct OryRegister {
    email: String,
    password: String,
    #[serde(default)]
    tenant_id: Option<String>,
}

/// Create Kratos identity via Admin API (live Ory path).
async fn ory_register(
    State(state): State<AppState>,
    Json(body): Json<OryRegister>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    if body.password.len() < 8 {
        return Err(HelixError::validation("password min 8 chars").into());
    }
    // Public registration creates a fresh tenant. Claiming an existing tenant is
    // only allowed through a validated invite/admin flow (not public self-serve).
    if body.tenant_id.is_some() {
        return Err(HelixError::validation(
            "public registration cannot specify a tenant_id; use an invite link to join an existing tenant",
        )
        .into());
    }
    let k = kratos(&state);
    if !k.ready().await {
        return Err(
            HelixError::unavailable("Kratos not reachable — start compose profile ory").into(),
        );
    }

    // Create a fresh tenant for every public registration. Joining an existing
    // tenant requires a validated invite token (not implemented in this slice).
    let tenant_id = TenantId::new();
    let tenant_region = state.clients.config.data_residency_region.clone();
    if let Some(tenants) = state.clients.tenants.as_ref() {
        tenants
            .create(tenant_id, &body.email, &tenant_region, None)
            .await?;
    }

    let identity = k
        .create_identity(
            &body.email,
            &body.password,
            &tenant_id.to_string(),
            &state.clients.config.data_residency_region,
            &["read", "write", "audit_read"],
        )
        .await?;

    let user_id = identity
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(UserId::from_uuid)
        .unwrap_or_else(UserId::new);

    if let Some(memberships) = state.clients.memberships.as_ref() {
        memberships
            .create(tenant_id, user_id, Role::Owner, None)
            .await?;
    }

    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: None,
            actor: Actor::System {
                reason: "ory_register".into(),
            },
            action: "auth.ory_register".into(),
            resource_type: "identity".into(),
            resource_id: body.email.clone(),
            metadata: serde_json::json!({ "tenant_id": tenant_id.to_string() }),
            residency_region: state.clients.config.data_residency_region.clone(),
        })
        .await;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "identity": identity,
        "tenant_id": tenant_id.to_string(),
        "user_id": user_id.to_string(),
        "next": "POST /v1/ory/login with same email/password"
    }))))
}

#[derive(Deserialize)]
struct OryLogin {
    email: String,
    password: String,
}

/// Password login against live Kratos; returns session_token for Bearer auth.
async fn ory_login(
    State(state): State<AppState>,
    Json(body): Json<OryLogin>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let k = kratos(&state);
    if !k.ready().await {
        return Err(
            HelixError::unavailable("Kratos not reachable — start compose profile ory").into(),
        );
    }
    let login = k.login_password(&body.email, &body.password).await?;
    // Prefer live whoami via session token; if hybrid falls back, still return token + session.
    let principal = match state
        .clients
        .auth
        .resolve(Some(&login.session_token), None)
        .await
    {
        Ok(p)
            if p.session_id
                .as_deref()
                .is_some_and(|s| !s.starts_with("dev-session:")) =>
        {
            p
        }
        Ok(p) => p, // may still be valid kratos if session_id format differs
        Err(e) => {
            return Err(e.into());
        }
    };
    let _ = state
        .clients
        .audit
        .append(AuditEvent {
            tenant_id: Some(principal.tenant_id),
            actor: Actor::User {
                user_id: principal.user_id,
                tenant_id: principal.tenant_id,
            },
            action: "auth.ory_login".into(),
            resource_type: "session".into(),
            resource_id: body.email.clone(),
            metadata: serde_json::json!({ "session_id": principal.session_id }),
            residency_region: principal.residency_region.clone(),
        })
        .await;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "session_token": login.session_token,
        "session": login.session,
        "principal": principal,
        "auth_backend": "ory_kratos",
        "usage": {
            "header": "Authorization: Bearer <session_token>",
            "or": "X-Session-Token: <session_token>",
            "cookie": "ory_kratos_session=<session_token>"
        }
    }))))
}
