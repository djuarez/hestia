use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path, RawQuery, State};
use axum::http::{header, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;

use crate::agents::Agent;
use crate::error::AppError;
use crate::AppState;

/// GET /v1/containers — aggregate containers from every agent, tagging each
/// with the node it lives on. Agents that are unreachable are skipped.
pub async fn list(State(state): State<AppState>) -> Json<Vec<Value>> {
    let per_agent = futures::future::join_all(
        state
            .registry
            .agents()
            .iter()
            .map(|agent| fetch_containers(&state.http, agent)),
    )
    .await;

    let mut all = Vec::new();
    for (agent, containers) in per_agent {
        for mut container in containers {
            // Tag each container with its node so the UI can route actions.
            if let Value::Object(map) = &mut container {
                map.insert("node".into(), Value::String(agent.clone()));
            }
            all.push(container);
        }
    }

    Json(all)
}

// --- Action proxying --------------------------------------------------------
//
// Routes are nested under `/v1/nodes/{node}/containers/...`. The client knows
// the node from the aggregated container list, so the server just forwards the
// request to that node's agent and returns the agent's response verbatim.

/// POST /v1/nodes/{node}/containers — create a container on `node`.
pub async fn create(
    State(state): State<AppState>,
    Path(node): Path<String>,
    Json(body): Json<Value>,
) -> Result<Response, AppError> {
    let agent = resolve(&state, &node)?;
    let url = format!("{}/v1/containers", agent.base_url);
    forward(&state.http, Method::POST, url, Some(body)).await
}

/// DELETE /v1/nodes/{node}/containers/{id} — delete a container on `node`
/// (forwards the `?force=` query through to the agent).
pub async fn delete(
    State(state): State<AppState>,
    Path((node, id)): Path<(String, String)>,
    RawQuery(query): RawQuery,
) -> Result<Response, AppError> {
    let agent = resolve(&state, &node)?;
    let mut url = format!("{}/v1/containers/{}", agent.base_url, id);
    if let Some(q) = query.filter(|q| !q.is_empty()) {
        url.push('?');
        url.push_str(&q);
    }
    forward(&state.http, Method::DELETE, url, None).await
}

/// POST /v1/nodes/{node}/containers/{id}/start
pub async fn start(
    State(state): State<AppState>,
    Path((node, id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let agent = resolve(&state, &node)?;
    let url = format!("{}/v1/containers/{}/start", agent.base_url, id);
    forward(&state.http, Method::POST, url, None).await
}

/// POST /v1/nodes/{node}/containers/{id}/stop
pub async fn stop(
    State(state): State<AppState>,
    Path((node, id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let agent = resolve(&state, &node)?;
    let url = format!("{}/v1/containers/{}/stop", agent.base_url, id);
    forward(&state.http, Method::POST, url, None).await
}

/// Resolve a node name to its agent, or 404 if it isn't registered.
fn resolve<'a>(state: &'a AppState, node: &str) -> Result<&'a Agent, AppError> {
    state
        .registry
        .get(node)
        .ok_or_else(|| AppError::UnknownNode(node.to_string()))
}

/// Forward a request to an agent and mirror its status and JSON body back to
/// the caller.
async fn forward(
    http: &reqwest::Client,
    method: Method,
    url: String,
    body: Option<Value>,
) -> Result<Response, AppError> {
    let mut req = http.request(method, &url);
    if let Some(body) = body {
        req = req.json(&body);
    }

    let resp = req.send().await.map_err(AppError::Upstream)?;
    let status: StatusCode = resp.status();
    let bytes: Bytes = resp.bytes().await.map_err(AppError::Upstream)?;

    Ok((status, [(header::CONTENT_TYPE, "application/json")], bytes).into_response())
}

/// Fetch one agent's containers, returning the node name alongside them.
/// On any error the node simply contributes no containers.
async fn fetch_containers(http: &reqwest::Client, agent: &Agent) -> (String, Vec<Value>) {
    let url = format!("{}/v1/containers", agent.base_url);

    let containers = match http.get(&url).timeout(Duration::from_secs(5)).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Vec<Value>>().await.unwrap_or_else(|e| {
                tracing::warn!(node = %agent.name, error = %e, "bad containers response");
                Vec::new()
            })
        }
        Ok(resp) => {
            tracing::warn!(node = %agent.name, status = %resp.status(), "agent error");
            Vec::new()
        }
        Err(e) => {
            tracing::warn!(node = %agent.name, error = %e, "agent unreachable");
            Vec::new()
        }
    };

    (agent.name.clone(), containers)
}
