use axum::Json;
use serde_json::{json, Value};

/// GET /health — server liveness probe.
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "hestia-server",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
