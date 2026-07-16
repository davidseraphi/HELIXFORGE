//! HelixForge shared core — IDs, errors, tenancy, config, hashing.
//!
//! All product backends and HelixCore services depend on this crate.

pub mod config;
pub mod error;
pub mod hash;
pub mod ids;
pub mod pagination;
pub mod project;
pub mod response;
pub mod semantic_state;
pub mod tenancy;
pub mod time;

pub use config::{CoreConfig, DbPoolConfig, ServiceEndpoints};
pub use error::{ErrorCode, HelixError, HelixResult};
pub use ids::{AuditId, OrgId, ProjectSlug, RequestId, SessionId, TenantId, UserId, WorkspaceId};
pub use pagination::{Page, PageRequest};
pub use project::{ProductMeta, ProductTier, PRODUCT_CATALOG};
pub use response::ApiResponse;
pub use semantic_state::{ProductMaturity, SemanticState};
pub use tenancy::{Actor, Principal, Scope};
pub use time::UtcTimestamp;
