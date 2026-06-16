use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::AppState;

/// GET /v1/containers/{id}/logs — upgrade to a WebSocket and stream the
/// container's logs (tail + follow) until the client disconnects.
pub async fn logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| stream(socket, state, id))
}

async fn stream(mut socket: WebSocket, state: AppState, id: String) {
    // Tail the last 100 lines, then follow new output.
    let mut child = match state.containers.logs_command(&id, true, Some(100)).spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = socket
                .send(Message::Text(
                    format!("failed to start log stream: {e}").into(),
                ))
                .await;
            return;
        }
    };

    let Some(stdout) = child.stdout.take() else {
        return;
    };
    let mut reader = BufReader::new(stdout).lines();

    loop {
        tokio::select! {
            // A new log line from the container process.
            line = reader.next_line() => match line {
                Ok(Some(line)) => {
                    if socket.send(Message::Text(line.into())).await.is_err() {
                        break; // client went away
                    }
                }
                // EOF (process exited) or read error: stop streaming.
                Ok(None) | Err(_) => break,
            },
            // The client closed the socket (or it errored).
            msg = socket.recv() => {
                if matches!(msg, None | Some(Ok(Message::Close(_))) | Some(Err(_))) {
                    break;
                }
            }
        }
    }

    // Close the socket cleanly (no-op if the client already went away).
    let _ = socket.send(Message::Close(None)).await;
    // Don't leave a `container logs --follow` process running behind us.
    let _ = child.kill().await;
}
