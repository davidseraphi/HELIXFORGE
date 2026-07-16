use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use shared_core::{ApiResponse, HelixError};

pub fn map_error(err: HelixError) -> Response {
    let status =
        StatusCode::from_u16(err.code.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(ApiResponse::<()>::err(err))).into_response()
}

/// Axum-friendly wrapper so handlers can `?` HelixError.
pub struct ApiError(pub HelixError);

impl From<HelixError> for ApiError {
    fn from(value: HelixError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        map_error(self.0)
    }
}
