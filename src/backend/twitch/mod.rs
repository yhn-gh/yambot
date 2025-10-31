/// Twitch EventSub WebSocket integration module
///
/// This module provides a complete implementation of Twitch chat integration using
/// EventSub WebSocket and Helix API. It supports:
/// - Receiving chat messages and events via WebSocket
/// - Sending chat messages via HTTP API
/// - Moderation actions (delete, ban, timeout)
/// - Chat settings management
/// - Automatic reconnection handling
///
/// # Example Usage
///
/// ```rust,no_run
/// use yambot::backend::twitch::{TwitchClient, TwitchConfig, TwitchClientEvent};
/// use tokio::sync::mpsc;
///
/// #[tokio::main]
/// async fn main() {
///     let config = TwitchConfig {
///         channel_name: "your_channel".to_string(),
///         auth_token: "your_oauth_token".to_string(),
///         client_id: "your_client_id".to_string(),
///     };
///
///     let (tx, mut rx) = mpsc::channel(100);
///     let mut client = TwitchClient::new(config);
///
///     // Connect to Twitch
///     client.connect(tx).await.unwrap();
///
///     // Listen for events
///     while let Some(event) = rx.recv().await {
///         match event {
///             TwitchClientEvent::ChatEvent(chat_event) => {
///                 // Handle chat event
///             }
///             _ => {}
///         }
///     }
/// }
/// ```

mod api;
mod auth;
mod client;
mod error;
mod eventsub;
mod messages;
mod websocket;

// Re-export public types
pub use auth::{refresh_access_token, validate_token, TokenResponse, CLIENT_ID};
pub use client::{TwitchClient, TwitchClientEvent, TwitchConfig};
pub use error::{Result, TwitchError};
pub use messages::{
    Badge, ChatMessageEvent, TwitchEvent, MessageDeleteEvent,
    ClearUserMessagesEvent, ChatClearEvent, ChatSettingsUpdateEvent,
    ChannelBanEvent, ChannelUnbanEvent,
};
