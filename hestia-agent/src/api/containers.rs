use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::container::{Container, CreateContainerRequest};
use crate::error::AppError;
use crate::AppState;

/// GET /v1/containers — list all containers on this node.
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Container>>, AppError> {
    let containers = state.containers.list().await?;
    Ok(Json(containers))
}

/// POST /v1/containers — create a container (optionally starting it).
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateContainerRequest>,
) -> Result<(StatusCode, Json<Container>), AppError> {
    let container = state.containers.create(&req).await?;
    Ok((StatusCode::CREATED, Json(container)))
}

#[derive(Debug, Deserialize)]
pub struct DeleteParams {
    /// Remove the container even if it is running.
    #[serde(default)]
    force: bool,
}

/// DELETE /v1/containers/{id} — delete a container (`?force=true` to force).
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<DeleteParams>,
) -> Result<Json<Value>, AppError> {
    state.containers.delete(&id, params.force).await?;
    Ok(Json(json!({ "id": id, "action": "delete", "ok": true })))
}

/// POST /v1/containers/{id}/start — start a stopped container.
pub async fn start(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    state.containers.start(&id).await?;
    Ok(Json(json!({ "id": id, "action": "start", "ok": true })))
}

/// POST /v1/containers/{id}/stop — stop a running container.
pub async fn stop(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    state.containers.stop(&id).await?;
    Ok(Json(json!({ "id": id, "action": "stop", "ok": true })))
}
