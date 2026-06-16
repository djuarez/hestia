use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::container::ContainerError;

/// Top-level error type for HTTP handlers. Every variant knows how to turn
/// itself into a JSON response with an appropriate status code.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Container(#[from] ContainerError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Container(ContainerError::NotFound(_)) => StatusCode::NOT_FOUND,
            AppError::Container(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        tracing::error!(error = %self, "request failed");

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
