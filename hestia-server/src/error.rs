use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Top-level error type for server handlers.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unknown node `{0}`")]
    UnknownNode(String),
    #[error("upstream agent error: {0}")]
    Upstream(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::UnknownNode(_) => StatusCode::NOT_FOUND,
            // The agent is unreachable / misbehaving: surface a gateway error.
            AppError::Upstream(_) => StatusCode::BAD_GATEWAY,
        };

        tracing::error!(error = %self, "request failed");

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
