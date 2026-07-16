//! Unified error type for HelixForge services.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type HelixResult<T> = Result<T, HelixError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    Validation,
    RateLimited,
    Unavailable,
    Internal,
    Dependency,
    AuditIntegrity,
}

impl ErrorCode {
    pub fn http_status(self) -> u16 {
        match self {
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::Validation => 422,
            Self::RateLimited => 429,
            Self::Unavailable => 503,
            Self::Internal | Self::Dependency | Self::AuditIntegrity => 500,
        }
    }
}

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("{code:?}: {message}")]
pub struct HelixError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl HelixError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
            request_id: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unauthorized, msg)
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Forbidden, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, msg)
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Validation, msg)
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, msg)
    }

    pub fn dependency(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Dependency, msg)
    }

    pub fn unavailable(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unavailable, msg)
    }
}

impl From<anyhow::Error> for HelixError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal(err.to_string())
    }
}
