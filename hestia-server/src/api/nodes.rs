use std::time::Duration;

use axum::extract::State;
use axum::Json;
use serde::Serialize;
use serde_json::Value;

use crate::agents::Agent;
use crate::AppState;

/// Per-request timeout for aggregation reads, so a slow node fails fast.
const AGGREGATION_TIMEOUT: Duration = Duration::from_secs(5);

/// A node's status as seen by the server: reachability plus its metrics.
#[derive(Debug, Serialize)]
pub struct NodeView {
    pub name: String,
    pub url: String,
    pub online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// GET /v1/nodes — fan out to every agent's `/v1/metrics` concurrently and
/// report each node's status.
pub async fn list(State(state): State<AppState>) -> Json<Vec<NodeView>> {
    let nodes = futures::future::join_all(
        state
            .registry
            .agents()
            .iter()
            .map(|agent| fetch_node(&state.http, agent)),
    )
    .await;

    Json(nodes)
}

async fn fetch_node(http: &reqwest::Client, agent: &Agent) -> NodeView {
    let url = format!("{}/v1/metrics", agent.base_url);

    match http.get(&url).timeout(AGGREGATION_TIMEOUT).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Value>().await {
            Ok(metrics) => NodeView {
                name: agent.name.clone(),
                url: agent.base_url.clone(),
                online: true,
                metrics: Some(metrics),
                error: None,
            },
            Err(e) => NodeView {
                name: agent.name.clone(),
                url: agent.base_url.clone(),
                online: false,
                metrics: None,
                error: Some(format!("invalid metrics response: {e}")),
            },
        },
        Ok(resp) => NodeView {
            name: agent.name.clone(),
            url: agent.base_url.clone(),
            online: false,
            metrics: None,
            error: Some(format!("agent returned {}", resp.status())),
        },
        Err(e) => NodeView {
            name: agent.name.clone(),
            url: agent.base_url.clone(),
            online: false,
            metrics: None,
            error: Some(e.to_string()),
        },
    }
}
