mod containers;
mod health;
mod logs;

use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;

/// Build the agent's HTTP router with all routes and middleware wired up.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route(
            "/v1/containers",
            get(containers::list).post(containers::create),
        )
        .route("/v1/containers/{id}", delete(containers::delete))
        .route("/v1/containers/{id}/start", post(containers::start))
        .route("/v1/containers/{id}/stop", post(containers::stop))
        .route("/v1/containers/{id}/logs", get(logs::logs))
        .route("/v1/metrics", get(crate::metrics::handler))
        .layer(TraceLayer::new_for_http())
        // CORS is permissive for now; lock this down once auth lands.
        .layer(CorsLayer::permissive())
        .with_state(state)
}
