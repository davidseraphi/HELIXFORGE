//! Zero-trust principals, actors, and scopes.

use serde::{Deserialize, Serialize};

use crate::ids::{OrgId, TenantId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Read non-sensitive metadata
    Read,
    /// Mutate tenant-owned resources
    Write,
    /// Admin operations within a tenant
    Admin,
    /// Platform-operator only
    Platform,
    /// Immutable audit read
    AuditRead,
}

impl Scope {
    /// Parse a single scope token (`read`, `write`, `admin`, `platform`, `audit_read`).
    pub fn parse_token(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "admin" => Some(Self::Admin),
            "platform" => Some(Self::Platform),
            "audit_read" | "audit-read" | "auditread" => Some(Self::AuditRead),
            _ => None,
        }
    }

    /// Parse comma/space-separated scopes; empty input → None.
    pub fn parse_list(s: &str) -> Option<Vec<Self>> {
        let scopes: Vec<Self> = s
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|t| !t.is_empty())
            .filter_map(Self::parse_token)
            .collect();
        if scopes.is_empty() {
            None
        } else {
            Some(scopes)
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Admin => "admin",
            Self::Platform => "platform",
            Self::AuditRead => "audit_read",
        }
    }
}

/// Least-privilege default for a freshly registered user.
pub const DEFAULT_USER_SCOPES: &[Scope] = &[Scope::Read, Scope::Write, Scope::AuditRead];

/// Tenant membership role. Roles are split: admin does not automatically include
/// secret export, audit rewrite, billing override, break-glass, signing, or
/// permanent delete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Owner,
    Admin,
    Member,
    Guest,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Guest => "guest",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "owner" => Some(Self::Owner),
            "admin" => Some(Self::Admin),
            "member" => Some(Self::Member),
            "guest" => Some(Self::Guest),
            _ => None,
        }
    }

    /// Map a role to the scopes it grants by default.
    pub fn default_scopes(&self) -> Vec<Scope> {
        match self {
            Self::Owner | Self::Admin => {
                vec![Scope::Read, Scope::Write, Scope::Admin, Scope::AuditRead]
            }
            Self::Member => DEFAULT_USER_SCOPES.to_vec(),
            Self::Guest => vec![Scope::Read],
        }
    }
}

/// A person's binding to a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membership {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub role: Role,
    pub invited_by: Option<UserId>,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub org_id: Option<OrgId>,
    pub scopes: Vec<Scope>,
    pub session_id: Option<String>,
    /// Data residency region the principal is allowed to access.
    pub residency_region: String,
}

impl Principal {
    pub fn has_scope(&self, scope: &Scope) -> bool {
        self.scopes
            .iter()
            .any(|s| s == scope || matches!(s, Scope::Platform))
    }

    pub fn require_scope(&self, scope: Scope) -> Result<(), crate::error::HelixError> {
        if self.has_scope(&scope) {
            Ok(())
        } else {
            Err(crate::error::HelixError::forbidden(format!(
                "missing scope {scope:?}"
            )))
        }
    }

    /// Permanent deletion requires explicit admin or platform authority.
    pub fn can_permanently_delete(&self) -> bool {
        self.has_scope(&Scope::Admin) || self.has_scope(&Scope::Platform)
    }

    /// Narrow or replace scopes (local/dev testing with `X-Helix-Dev-Scopes`).
    pub fn with_scopes(mut self, scopes: Vec<Scope>) -> Self {
        self.scopes = scopes;
        self
    }

    pub fn with_residency(mut self, region: impl Into<String>) -> Self {
        self.residency_region = region.into();
        self
    }
}

/// Who performed an action (for audit).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Actor {
    User {
        user_id: UserId,
        tenant_id: TenantId,
    },
    Service {
        service: String,
    },
    System {
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{TenantId, UserId};

    #[test]
    fn scope_parse_and_require() {
        let scopes = Scope::parse_list("read,admin").unwrap();
        assert_eq!(scopes, vec![Scope::Read, Scope::Admin]);
        let p = Principal {
            user_id: UserId::new(),
            tenant_id: TenantId::new(),
            org_id: None,
            scopes,
            session_id: None,
            residency_region: "eu-west".into(),
        };
        assert!(p.require_scope(Scope::Read).is_ok());
        assert!(p.require_scope(Scope::Write).is_err());
        assert_eq!(p.with_residency("us-east").residency_region, "us-east");
    }

    #[test]
    fn permanent_delete_authority_gated() {
        let base = || Principal {
            user_id: UserId::new(),
            tenant_id: TenantId::new(),
            org_id: None,
            scopes: vec![Scope::Read, Scope::Write],
            session_id: None,
            residency_region: "eu-west".into(),
        };
        assert!(!base().can_permanently_delete());
        assert!(base()
            .with_scopes(vec![Scope::Read, Scope::Write, Scope::Admin])
            .can_permanently_delete());
        assert!(base()
            .with_scopes(vec![Scope::Read, Scope::Platform])
            .can_permanently_delete());
    }
}
