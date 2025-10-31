use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::error::{Result, TwitchError};
use super::messages::{EventSubMessage, Payload, TwitchEvent};

const EVENTSUB_WS_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

/// WebSocket connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

/// Messages from the WebSocket handler
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Connected,
    Disconnected,
    SessionId(String),
    Event(TwitchEvent),
    Error(String),
    Reconnect(String),
}

/// Shared state for WebSocket connection
#[derive(Clone)]
struct SharedState {
    last_message_time: Arc<RwLock<Instant>>,
    keepalive_timeout: Arc<RwLock<Duration>>,
}

/// WebSocket connection handler for Twitch EventSub
#[derive(Clone)]
pub struct WebSocketHandler {
    url: String,
    state: ConnectionState,
    session_id: Option<String>,
    shared: SharedState,
}

impl WebSocketHandler {
    pub fn new() -> Self {
        Self {
            url: EVENTSUB_WS_URL.to_string(),
            state: ConnectionState::Disconnected,
            session_id: None,
            shared: SharedState {
                last_message_time: Arc::new(RwLock::new(Instant::now())),
                keepalive_timeout: Arc::new(RwLock::new(Duration::from_secs(10))),
            },
        }
    }

    /// Start the WebSocket connection and message handling loop
    pub async fn connect(&mut self, tx: mpsc::Sender<WebSocketMessage>) -> Result<()> {
        self.state = ConnectionState::Connecting;

        let (ws_stream, _) = connect_async(&self.url).await?;

        self.state = ConnectionState::Connected;
        let _ = tx.send(WebSocketMessage::Connected).await;

        let (mut write, mut read) = ws_stream.split();

        // Message handling loop
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    *self.shared.last_message_time.write().await = Instant::now();

                    match self.handle_message(&text, &tx).await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Error handling message: {}", e);
                            let _ = tx.send(WebSocketMessage::Error(e.to_string())).await;
                        }
                    }
                }
                Ok(Message::Close(frame)) => {
                    let code = frame.as_ref().map(|f| f.code.into()).unwrap_or(1000);
                    let reason = frame
                        .as_ref()
                        .map(|f| f.reason.to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    log::warn!("WebSocket closed: code={}, reason={}", code, reason);

                    self.state = ConnectionState::Disconnected;
                    let _ = tx.send(WebSocketMessage::Disconnected).await;

                    // Handle specific close codes
                    match code {
                        4000..=4007 => {
                            // Twitch specific close codes - log and potentially reconnect
                            log::error!("Twitch close code {}: {}", code, reason);
                        }
                        _ => {}
                    }

                    break;
                }
                Ok(Message::Ping(payload)) => {
                    // Respond to ping with pong
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        log::error!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Pong received, connection is alive
                }
                Err(e) => {
                    log::error!("WebSocket error: {}", e);
                    self.state = ConnectionState::Disconnected;
                    let _ = tx.send(WebSocketMessage::Error(e.to_string())).await;
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        &mut self,
        text: &str,
        tx: &mpsc::Sender<WebSocketMessage>,
    ) -> Result<()> {
        let message: EventSubMessage = serde_json::from_str(text)?;

        log::debug!("Received message type: {}", message.metadata.message_type);

        match message.payload {
            Payload::Welcome(welcome) => {
                self.session_id = Some(welcome.session.id.clone());
                *self.shared.keepalive_timeout.write().await =
                    Duration::from_secs(welcome.session.keepalive_timeout_seconds);

                let _ = tx
                    .send(WebSocketMessage::SessionId(welcome.session.id))
                    .await;
            }

            Payload::Notification(notification) => {
                // Parse the event based on subscription type
                let event = self.parse_event(
                    &notification.subscription.subscription_type,
                    notification.event,
                )?;
                let _ = tx.send(WebSocketMessage::Event(event)).await;
            }

            Payload::Reconnect(reconnect) => {
                if let Some(reconnect_url) = reconnect.session.reconnect_url {
                    log::warn!("Server requested reconnect to: {}", reconnect_url);
                    self.url = reconnect_url.clone();
                    self.state = ConnectionState::Reconnecting;
                    let _ = tx.send(WebSocketMessage::Reconnect(reconnect_url)).await;
                }
            }

            Payload::Keepalive(_) => {}

            Payload::Revocation(_revocation) => {}
        }

        Ok(())
    }

    /// Parse event data based on subscription type
    fn parse_event(
        &self,
        subscription_type: &str,
        event: serde_json::Value,
    ) -> Result<TwitchEvent> {
        match subscription_type {
            "channel.chat.message" => {
                let chat_event = serde_json::from_value(event)?;
                Ok(TwitchEvent::ChatMessage(chat_event))
            }
            "channel.chat.message_delete" => {
                let delete_event = serde_json::from_value(event)?;
                Ok(TwitchEvent::MessageDelete(delete_event))
            }
            "channel.chat.clear_user_messages" => {
                let clear_user_event = serde_json::from_value(event)?;
                Ok(TwitchEvent::ClearUserMessages(clear_user_event))
            }
            "channel.chat.clear" => {
                let clear_event = serde_json::from_value(event)?;
                Ok(TwitchEvent::ChatClear(clear_event))
            }
            "channel.chat_settings.update" => {
                let settings_event = serde_json::from_value(event)?;
                Ok(TwitchEvent::ChatSettingsUpdate(settings_event))
            }
            "channel.ban" => {
                let ban_event = serde_json::from_value(event)?;
                Ok(TwitchEvent::ChannelBan(ban_event))
            }
            "channel.unban" => {
                let unban_event = serde_json::from_value(event)?;
                Ok(TwitchEvent::ChannelUnban(unban_event))
            }
            _ => {
                log::warn!("Unknown subscription type: {}", subscription_type);
                Err(TwitchError::JsonError(format!(
                    "Unknown subscription type: {}",
                    subscription_type
                )))
            }
        }
    }

    /// Get the current session ID
    #[allow(dead_code)] // Reserved for future WebSocket session management
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get the current connection state
    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Check if keepalive timeout has been exceeded
    pub async fn is_keepalive_timeout(&self) -> bool {
        let last_message_time = *self.shared.last_message_time.read().await;
        let keepalive_timeout = *self.shared.keepalive_timeout.read().await;
        last_message_time.elapsed() > keepalive_timeout + Duration::from_secs(5)
    }

    /// Set a new URL for reconnection
    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }
}

/// Reconnect with exponential backoff
pub async fn reconnect_with_backoff(
    handler: &mut WebSocketHandler,
    tx: mpsc::Sender<WebSocketMessage>,
    max_retries: u32,
) -> Result<()> {
    let mut retries = 0;
    let base_delay = Duration::from_secs(1);

    while retries < max_retries {
        retries += 1;
        let delay = base_delay * 2_u32.pow(retries - 1).min(6); // Max 64 seconds

        sleep(delay).await;

        match handler.connect(tx.clone()).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                log::error!("Reconnection failed: {}", e);
                if retries >= max_retries {
                    return Err(e);
                }
            }
        }
    }

    Err(TwitchError::WebSocketError(
        "Max reconnection attempts reached".to_string(),
    ))
}
