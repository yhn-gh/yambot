use tungstenite::Message;
use serde_json::{json, Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::StreamExt;
use tungstenite::client::IntoClientRequest;
use std::collections::{HashMap, HashSet};
use crate::backend;

const HELIX_URI: &'static str = "https://api.twitch.tv/helix";

#[non_exhaustive]
#[derive(Debug)]
pub enum Subscription {
    ChannelChatMessage
}

pub struct HelixClient {
    client: reqwest::Client,
    auth_token: String,
    client_id: String,
    user_id: Option<u32>,
    subscriptions: HashSet<Subscription>
}

impl HelixClient {
    pub(super) async fn new() -> Self {
        let config = backend::config::load_config().chatbot;
        
        Self {
            client: reqwest::Client::new(),
            auth_token: config.auth_token,
            client_id: config.client_id,
            user_id: Self::get_user_id().await.ok(),
            subscriptions: HashSet::new(),
        }
    }
    
    pub(super) async fn subscribe(&self, sub: Subscription, session_id: &str) -> reqwest::Result<()> {
        let client = &self.client;
        let map = json!({
            "type": sub.as_str(),
            "version": "1",
            "condition": sub.condition(&self).await,
            "transport": {
                "method": "websocket",
                "session_id": session_id
            }
        });
        
        let post = client.post(format!("{HELIX_URI}/eventsub/subscriptions"))
            .bearer_auth(&self.auth_token)
            .header("Client-Id", &self.client_id)
            .json(&map);
        
        let res = post.send().await?.text().await?;
        // should handle Non-authorized, Bad Request case
        Ok(())
    }

    // Todo caching of user id
    pub async fn get_user_id() -> reqwest::Result<u32> {
        let config = backend::config::load_config().chatbot;

        let mut body = reqwest::Client::new()
            .get(format!("{HELIX_URI}/users?login={}",&config.channel_name))
            .bearer_auth(&config.auth_token)
            .header("Client-Id", &config.client_id)
            .send()
            .await?.json::<Map<String, Value>>().await?;
        
        // returns 0-len array over an Object instead of just an Object
        let data = &mut body["data"][0];

        let id = data["id"].take().as_str()
            .expect("Unexpected user data from Twitch API")
            .parse()
            .unwrap();
        Ok(id)
    }
}

impl TryFrom<&str> for Subscription {
    // TODO change to io::Error
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = match s {
            "channel.chat.message" => Self::ChannelChatMessage,
            _ => return Err(()),
        };
        Ok(s)
    }
}


impl Subscription {
    fn as_str(&self) -> &str {
        match &self {
            Self::ChannelChatMessage => "channel.chat.message"
        }
    }
    
    // returns condition value convention for subscribing
    async fn condition(&self, client: &HelixClient) -> Value {
        let user_id = client.user_id.unwrap_or_default().to_string();
        match self {
            Subscription::ChannelChatMessage => json!({
                "broadcaster_user_id": user_id,
                "user_id": user_id
            }),
        }
    }
}
