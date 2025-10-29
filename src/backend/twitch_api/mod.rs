pub mod helix;
pub mod eventsub;

pub use helix::{HelixClient, Subscription};
use eventsub::EventSubConnection;
use tokio::sync::{mpsc, oneshot};
use serde::{Serialize, Deserialize};
use crate::ui::ChatbotConfig;

pub struct Client {
    helix: HelixClient,
    eventsub: EventSubConnection,
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
        let (mut helix, eventsub) = tokio::join!(
            HelixClient::new(config),
            EventSubConnection::serve());

        let eventsub = eventsub?;

        if helix.config.user_id.is_none() {
            let user_id = helix.request_user_id().await;
            helix.config.user_id = user_id.ok();
        }

        
        let session_id = eventsub.session.as_ref().unwrap();
        helix.subscribe(Subscription::ChannelChatMessage, session_id).await?;

        Ok(Self {
            helix,
            eventsub,
        })
    }
}
