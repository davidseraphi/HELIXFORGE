//! Standard API envelope used by all HelixForge HTTP services.

use serde::{Deserialize, Serialize};

use crate::error::HelixError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Ok {
        data: T,
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
    },
    Err {
        error: HelixError,
    },
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self::Ok {
            data,
            request_id: None,
        }
    }

    pub fn ok_with_request_id(data: T, request_id: impl Into<String>) -> Self {
        Self::Ok {
            data,
            request_id: Some(request_id.into()),
        }
    }

    pub fn err(error: HelixError) -> Self {
        Self::Err { error }
    }
}
