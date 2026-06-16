use axum::extract::ws::{Message as ClientMsg, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as AgentMsg;

use crate::error::AppError;
use crate::AppState;

/// GET /v1/nodes/{node}/containers/{id}/logs — proxy a log stream from the
/// owning node's agent to the client over WebSocket.
pub async fn logs(
    State(state): State<AppState>,
    Path((node, id)): Path<(String, String)>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let agent = state
        .registry
        .get(&node)
        .ok_or_else(|| AppError::UnknownNode(node.clone()))?;

    // The agent speaks plain ws; map its http(s) base accordingly.
    let base_ws = agent
        .base_url
        .replacen("http://", "ws://", 1)
        .replacen("https://", "wss://", 1);
    let upstream = format!("{base_ws}/v1/containers/{id}/logs");

    Ok(ws.on_upgrade(move |socket| bridge(socket, upstream)))
}

/// Pipe messages between the downstream client socket and the upstream agent
/// socket until either side closes.
async fn bridge(client: WebSocket, upstream_url: String) {
    let upstream = match connect_async(&upstream_url).await {
        Ok((stream, _resp)) => stream,
        Err(e) => {
            let mut client = client;
            let _ = client
                .send(ClientMsg::Text(format!("failed to reach agent: {e}").into()))
                .await;
            let _ = client.send(ClientMsg::Close(None)).await;
            return;
        }
    };

    let (mut client_tx, mut client_rx) = client.split();
    let (mut agent_tx, mut agent_rx) = upstream.split();

    loop {
        tokio::select! {
            // Agent -> client (the log lines).
            msg = agent_rx.next() => match msg {
                Some(Ok(AgentMsg::Text(t))) => {
                    if client_tx.send(ClientMsg::Text(t.as_str().into())).await.is_err() {
                        break;
                    }
                }
                Some(Ok(AgentMsg::Binary(b))) => {
                    if client_tx.send(ClientMsg::Binary(b)).await.is_err() {
                        break;
                    }
                }
                Some(Ok(AgentMsg::Close(_))) | None => break,
                Some(Ok(_)) => {} // ignore ping/pong/raw frames
                Some(Err(_)) => break,
            },
            // Client -> agent (typically just the closing handshake).
            msg = client_rx.next() => match msg {
                Some(Ok(ClientMsg::Close(_))) | None => break,
                Some(Ok(_)) => {}
                Some(Err(_)) => break,
            },
        }
    }

    let _ = agent_tx.send(AgentMsg::Close(None)).await;
    let _ = client_tx.send(ClientMsg::Close(None)).await;
}
