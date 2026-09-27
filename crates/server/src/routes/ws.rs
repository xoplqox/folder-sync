use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use folder_sync_core::batch::{BatchRun, ProgressEvent};
use serde::Serialize;
use tokio::sync::broadcast;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/events", get(handler))
}

async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Every message carries both the semantic event (for one-shot UI reactions
/// like a "batch completed" toast) and a full queue snapshot (so the client
/// never has to reconcile incremental deltas itself).
#[derive(Serialize)]
struct WsMessage {
    event: ProgressEvent,
    run: BatchRun,
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let initial = WsMessage {
        event: ProgressEvent::Snapshot,
        run: state.0.batch_queue.snapshot(),
    };
    if send(&mut socket, &initial).await.is_err() {
        return;
    }

    let mut rx = state.0.progress_tx.subscribe();
    loop {
        tokio::select! {
            event = rx.recv() => {
                let event = match event {
                    Ok(event) => event,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                let msg = WsMessage { event, run: state.0.batch_queue.snapshot() };
                if send(&mut socket, &msg).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(_)) => {} // clients don't send anything meaningful; ignore
                    _ => break,
                }
            }
        }
    }
}

async fn send(socket: &mut WebSocket, msg: &WsMessage) -> Result<(), axum::Error> {
    let payload = serde_json::to_string(msg).expect("WsMessage is always serializable");
    socket.send(Message::Text(payload)).await
}
