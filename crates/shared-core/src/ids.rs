//! Strongly-typed identifiers used across HelixForge.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident, $prefix:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            pub fn as_uuid(&self) -> Uuid {
                self.0
            }

            pub fn prefix() -> &'static str {
                $prefix
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}:{}", $prefix, self.0)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let raw = s.strip_prefix(concat!($prefix, ":")).unwrap_or(s);
                Ok(Self(Uuid::parse_str(raw)?))
            }
        }
    };
}

id_type!(TenantId, "ten");
id_type!(OrgId, "org");
id_type!(UserId, "usr");
id_type!(ProjectId, "prj");
id_type!(SessionId, "ses");
id_type!(WorkspaceId, "ws");
id_type!(RequestId, "req");
id_type!(AuditId, "aud");
id_type!(JobId, "job");

/// Product project slug (e.g. `helix-collab`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectSlug(pub String);

impl ProjectSlug {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_id_roundtrip_display() {
        let id = TenantId::new();
        let s = id.to_string();
        assert!(s.starts_with("ten:"));
        let parsed: TenantId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }
}
