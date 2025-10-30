pub mod helix;
pub mod eventsub;

pub use helix::{HelixClient, Subscription};
pub use eventsub::Event;
use eventsub::EventSubConnection;
use serde::{Serialize, Deserialize};
use crate::ui::ChatbotConfig;

pub struct Client {
    helix: HelixClient,
    pub eventsub: EventSubConnection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    // pub message_id: String,
    pub message_text: String,
    pub username: String,
    pub badges: Vec<String>,
}

impl Client {
    pub async fn new(config: ChatbotConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (mut helix, mut eventsub) = (
            HelixClient::new(config).await,
            EventSubConnection::serve().await?
        );

        if helix.config.user_id.is_none() {
            let user_id = helix.request_user_id().await;
            helix.config.user_id = user_id.ok();
        }

        if let Some(session_id) = eventsub.session.as_ref() {
            helix.subscribe(Subscription::ChannelChatMessage, session_id).await?;
        }

        Ok(Self {
            helix,
            eventsub,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidData,
    ReqwestError(reqwest::Error),
    TungsteniteError(tungstenite::Error),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
    }
}
