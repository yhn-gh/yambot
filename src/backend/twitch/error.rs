use std::fmt;

/// Errors that can occur during Twitch operations
#[derive(Debug)]
pub enum TwitchError {
    /// WebSocket connection error
    WebSocketError(String),

    /// HTTP request error
    HttpError(String),

    /// JSON parsing error
    JsonError(String),

    /// Authentication error (invalid token, missing scopes)
    AuthError(String),

    /// Configuration error
    ConfigError(String),

    /// EventSub subscription error
    SubscriptionError(String),

    /// Connection closed unexpectedly
    ConnectionClosed(u16, String),

    /// Rate limit exceeded
    RateLimitExceeded(String),

    /// Channel send error
    ChannelError(String),
}

impl fmt::Display for TwitchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TwitchError::WebSocketError(msg) => write!(f, "WebSocket error: {}", msg),
            TwitchError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            TwitchError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            TwitchError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            TwitchError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            TwitchError::SubscriptionError(msg) => write!(f, "Subscription error: {}", msg),
            TwitchError::ConnectionClosed(code, reason) => {
                write!(f, "Connection closed: code={}, reason={}", code, reason)
            }
            TwitchError::RateLimitExceeded(msg) => write!(f, "Rate limit exceeded: {}", msg),
            TwitchError::ChannelError(msg) => write!(f, "Channel error: {}", msg),
        }
    }
}

impl std::error::Error for TwitchError {}

impl From<serde_json::Error> for TwitchError {
    fn from(err: serde_json::Error) -> Self {
        TwitchError::JsonError(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for TwitchError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        TwitchError::WebSocketError(err.to_string())
    }
}

impl From<reqwest::Error> for TwitchError {
    fn from(err: reqwest::Error) -> Self {
        TwitchError::HttpError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TwitchError>;
