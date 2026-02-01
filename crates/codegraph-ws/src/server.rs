//! WebSocket server implementation

use crate::handlers::route_message;
use crate::protocol::WsMessage;
use crate::state::SharedState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

/// Create the WebSocket router
pub fn create_router(state: Arc<SharedState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .with_state(state)
}

/// Health check handler
async fn health_handler() -> &'static str {
    "OK"
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<SharedState>) {
    let (mut sender, mut receiver) = socket.split();

    info!("New WebSocket connection established");

    // Create channel for sending responses
    let (tx, mut rx) = mpsc::channel::<WsMessage>(32);

    // Spawn task to forward responses to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            match serde_json::to_string(&response) {
                Ok(json) => {
                    if let Err(e) = sender.send(Message::Text(json.into())).await {
                        error!("Failed to send WebSocket message: {}", e);
                        break;
                    }
                }
                Err(e) => error!("Failed to serialize response: {}", e),
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse the message
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => {
                        let state_clone = state.clone();
                        let tx_clone = tx.clone();

                        // Route to appropriate handler
                        tokio::spawn(async move {
                            if let Some(response) = route_message(state_clone, ws_msg).await {
                                if let Err(e) = tx_clone.send(response).await {
                                    error!("Failed to send response to channel: {}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);

                        // Send error response
                        let error_response = WsMessage::error(
                            uuid::Uuid::nil(),
                            crate::protocol::ErrorPayload::new(
                                crate::protocol::error_codes::PARSE_ERROR,
                                format!("Invalid message format: {}", e),
                            ),
                        );
                        let _ = tx.send(error_response).await;
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                warn!("Received binary message, ignoring");
            }
            Ok(Message::Ping(_)) => {
                // Pong is handled automatically by axum
            }
            Ok(Message::Pong(_)) => {
                // Pong received
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by client");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Close the sender channel
    drop(tx);

    // Wait for send task to complete
    let _ = send_task.await;

    info!("WebSocket connection closed");
}

/// Start the WebSocket server
pub async fn serve(state: SharedState, port: u16) -> anyhow::Result<()> {
    let state = Arc::new(state);
    let app = create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting WebSocket server on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
