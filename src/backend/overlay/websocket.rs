use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Maximum number of messages that can be buffered in the broadcast channel
const CHANNEL_CAPACITY: usize = 100;

/// Default scale value for overlay elements
fn default_scale() -> f32 {
    1.0
}

/// Shared state for WebSocket connections
#[derive(Clone)]
pub struct WebSocketState {
    /// Broadcast channel for sending events to all connected overlays
    tx: broadcast::Sender<OverlayEvent>,
    /// Counter for connected clients
    client_count: Arc<RwLock<usize>>,
    /// Channel for receiving messages from overlay clients
    client_message_tx: Option<tokio::sync::mpsc::UnboundedSender<OverlayClientMessage>>,
}

impl WebSocketState {
    /// Create a new WebSocket state
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            tx,
            client_count: Arc::new(RwLock::new(0)),
            client_message_tx: None,
        }
    }

    /// Set a channel to receive messages from overlay clients
    pub fn set_client_message_channel(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<OverlayClientMessage>,
    ) {
        self.client_message_tx = Some(tx);
    }

    /// Send an event to all connected overlay clients
    pub async fn broadcast(&self, event: OverlayEvent) {
        if let Err(e) = self.tx.send(event) {
            log::warn!("Failed to broadcast overlay event: {}", e);
        }
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        *self.client_count.read().await
    }

    /// Get a broadcast receiver
    fn subscribe(&self) -> broadcast::Receiver<OverlayEvent> {
        self.tx.subscribe()
    }

    /// Send a client message to the backend
    fn send_client_message(&self, message: OverlayClientMessage) {
        if let Some(ref tx) = self.client_message_tx {
            if let Err(e) = tx.send(message) {
                log::warn!("Failed to send client message to backend: {}", e);
            }
        }
    }
}

/// Events that can be sent to the overlay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OverlayEvent {
    /// A command was executed
    CommandExecuted {
        command: String,
        user_name: String,
    },
    /// A TTS message is being played
    TtsMessage {
        user_name: String,
        message: String,
        language: String,
    },
    /// A sound effect is being played
    SoundPlayed {
        sound_name: String,
    },
    /// Trigger a specific action based on reward binding
    TriggerAction {
        action_type: String,
        data: serde_json::Value,
    },
    /// Ping to keep connection alive
    Ping,
    /// Configuration update - send overlay positions to client
    ConfigUpdate {
        positions: serde_json::Value,
    },
}

/// Messages that can be received from the overlay client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OverlayClientMessage {
    /// Wheel spin completed with result
    WheelResult {
        result: String,
        action: Option<WheelAction>,
    },
    /// Overlay position update
    PositionUpdate {
        element: String,
        x: f32,
        y: f32,
        #[serde(default = "default_scale")]
        scale: f32,
    },
    /// Request current configuration
    RequestConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WheelAction {
    Ban { username: String, reason: String },
    Timeout { username: String, duration: u32, reason: String },
    Unban { username: String },
    RunCommand { command: String },
    Nothing,
}

/// WebSocket handler for overlay connections
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle a single WebSocket connection
async fn handle_socket(socket: WebSocket, state: WebSocketState) {
    // Increment client count
    {
        let mut count = state.client_count.write().await;
        *count += 1;
        log::info!("Overlay client connected. Total clients: {}", *count);
    }

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.subscribe();

    // Task to receive events from the broadcast channel and send to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            // Serialize event to JSON
            let json = match serde_json::to_string(&event) {
                Ok(json) => json,
                Err(e) => {
                    log::error!("Failed to serialize overlay event: {}", e);
                    continue;
                }
            };

            // Send to client
            if sender.send(Message::Text(json)).await.is_err() {
                log::debug!("Client disconnected during send");
                break;
            }
        }
    });

    // Task to receive messages from client
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    log::debug!("Received message from overlay client: {}", text);
                    // Parse and handle client messages
                    match serde_json::from_str::<OverlayClientMessage>(&text) {
                        Ok(client_msg) => {
                            log::info!("Overlay client message: {:?}", client_msg);
                            state_clone.send_client_message(client_msg);
                        }
                        Err(e) => {
                            log::warn!("Failed to parse overlay client message: {}", e);
                        }
                    }
                }
                Message::Close(_) => {
                    log::debug!("Client sent close message");
                    break;
                }
                Message::Ping(_) => {
                    log::trace!("Received ping from client");
                    // Pong is automatically sent by axum
                }
                Message::Pong(_) => {
                    log::trace!("Received pong from client");
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete (which means the connection is closed)
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    // Decrement client count
    {
        let mut count = state.client_count.write().await;
        *count = count.saturating_sub(1);
        log::info!("Overlay client disconnected. Total clients: {}", *count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_state_creation() {
        let state = WebSocketState::new();
        assert_eq!(state.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_broadcast() {
        let state = WebSocketState::new();
        let event = OverlayEvent::Ping;
        state.broadcast(event).await;
        // Just ensure it doesn't panic
    }
}
