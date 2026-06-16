mod containers;
mod health;
mod logs;
mod nodes;

use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;

/// Build the server's HTTP router with all routes and middleware wired up.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/v1/nodes", get(nodes::list))
        .route("/v1/containers", get(containers::list))
        // Action proxying: forwarded to the owning node's agent.
        .route("/v1/nodes/{node}/containers", post(containers::create))
        .route(
            "/v1/nodes/{node}/containers/{id}",
            delete(containers::delete),
        )
        .route(
            "/v1/nodes/{node}/containers/{id}/start",
            post(containers::start),
        )
        .route(
            "/v1/nodes/{node}/containers/{id}/stop",
            post(containers::stop),
        )
        .route(
            "/v1/nodes/{node}/containers/{id}/logs",
            get(logs::logs),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
