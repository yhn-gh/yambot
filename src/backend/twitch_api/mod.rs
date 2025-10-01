pub mod helix;
pub mod eventsub;

use helix::{HelixClient, Subscription};
use eventsub::EventSubConnection;
use tokio::sync::{mpsc, oneshot};

pub struct Client {
    helix: HelixClient,
    eventsub: EventSubConnection,
    //    close: Option<oneshot::Sender<()>>,
}

impl Client {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let helix = HelixClient::new().await;
        let eventsub = EventSubConnection::serve().await?;

        if let Some(session_id) = &eventsub.session {
            helix.subscribe(Subscription::ChannelChatMessage, &session_id).await?;
        };
        Ok(Self {
            helix,
            eventsub,
        })
    }
}
