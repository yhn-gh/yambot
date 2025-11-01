pub mod helix;
pub mod eventsub;

pub use helix::{HelixClient, Subscription};
pub use eventsub::Event;
use eventsub::EventSubConnection;
use serde::{Serialize, Deserialize};
use crate::ui::ChatbotConfig;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

pub struct Client {
    helix: HelixClient,
    session: String,
    tx: mpsc::UnboundedSender<Event>,
    pub rx: Option<mpsc::UnboundedReceiver<Event>>,
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
        let mut helix = HelixClient::new(config).await;
        let (session, mut stream) = EventSubConnection::serve().await?;

        helix.subscribe(Subscription::ChannelChatMessage, &session).await?;

        let (tx, rx) = mpsc::unbounded_channel();
        let weak_tx = tx.clone().downgrade();
        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match weak_tx.upgrade() {
                    Some(tx) => tx.send(item),
                    None => break,
                };
            };
        });

        Ok(Self {
            helix,
            session,
            tx,
            rx: Some(rx),
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
