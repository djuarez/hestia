use axum::Json;
use serde_json::{json, Value};

/// GET /health — liveness probe used by `hestia-server`'s agent registry.
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "hestia-agent",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
