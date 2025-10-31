use serde::{Deserialize, Serialize};

/// WebSocket message received from Twitch EventSub
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventSubMessage {
    pub metadata: Metadata,
    pub payload: Payload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub message_id: String,
    pub message_type: String,
    pub message_timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Payload {
    Welcome(WelcomePayload),
    Notification(NotificationPayload),
    Reconnect(ReconnectPayload),
    Keepalive(KeepalivePayload),
    Revocation(RevocationPayload),
}

/// Session welcome message payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WelcomePayload {
    pub session: Session,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub id: String,
    pub status: String,
    pub keepalive_timeout_seconds: u64,
    pub reconnect_url: Option<String>,
    pub connected_at: String,
}

/// Notification payload containing events
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationPayload {
    pub subscription: Subscription,
    pub event: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Subscription {
    pub id: String,
    #[serde(rename = "type")]
    pub subscription_type: String,
    pub version: String,
    pub status: String,
    pub cost: u32,
    pub condition: serde_json::Value,
    pub created_at: String,
}

/// Reconnect payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReconnectPayload {
    pub session: Session,
}

/// Keepalive payload (empty)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeepalivePayload {}

/// Revocation payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevocationPayload {
    pub subscription: Subscription,
}

/// Chat message event from channel.chat.message subscription
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessageEvent {
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub chatter_user_id: String,
    pub chatter_user_login: String,
    pub chatter_user_name: String,
    pub message_id: String,
    pub message: Message,
    pub color: String,
    pub badges: Vec<Badge>,
    pub message_type: String,
    pub cheer: Option<Cheer>,
    pub reply: Option<Reply>,
    pub channel_points_custom_reward_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub text: String,
    pub fragments: Vec<MessageFragment>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageFragment {
    #[serde(rename = "type")]
    pub fragment_type: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cheermote: Option<Cheermote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emote: Option<Emote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention: Option<Mention>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cheermote {
    pub prefix: String,
    pub bits: u32,
    pub tier: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Emote {
    pub id: String,
    pub emote_set_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mention {
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Badge {
    pub set_id: String,
    pub id: String,
    pub info: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cheer {
    pub bits: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Reply {
    pub parent_message_id: String,
    pub parent_message_body: String,
    pub parent_user_id: String,
    pub parent_user_name: String,
    pub parent_user_login: String,
    pub thread_message_id: String,
    pub thread_user_id: String,
    pub thread_user_name: String,
    pub thread_user_login: String,
}

/// Message delete event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageDeleteEvent {
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub target_user_id: String,
    pub target_user_login: String,
    pub target_user_name: String,
    pub message_id: String,
}

/// Clear user messages event (ban/timeout)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearUserMessagesEvent {
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub target_user_id: String,
    pub target_user_login: String,
    pub target_user_name: String,
}

/// Chat clear event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatClearEvent {
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
}

/// Chat settings update event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatSettingsUpdateEvent {
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub emote_mode: bool,
    pub follower_mode: bool,
    pub follower_mode_duration_minutes: Option<u32>,
    pub slow_mode: bool,
    pub slow_mode_wait_time_seconds: Option<u32>,
    pub subscriber_mode: bool,
    pub unique_chat_mode: bool,
}

/// Channel ban event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChannelBanEvent {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub moderator_user_id: String,
    pub moderator_user_login: String,
    pub moderator_user_name: String,
    pub reason: String,
    pub banned_at: String,
    pub ends_at: Option<String>,
    pub is_permanent: bool,
}

/// Channel unban event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChannelUnbanEvent {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub moderator_user_id: String,
    pub moderator_user_login: String,
    pub moderator_user_name: String,
}

/// Events that can be received from Twitch
#[derive(Debug, Clone)]
pub enum TwitchEvent {
    ChatMessage(ChatMessageEvent),
    MessageDelete(MessageDeleteEvent),
    ClearUserMessages(ClearUserMessagesEvent),
    ChatClear(ChatClearEvent),
    ChatSettingsUpdate(ChatSettingsUpdateEvent),
    ChannelBan(ChannelBanEvent),
    ChannelUnban(ChannelUnbanEvent),
}
