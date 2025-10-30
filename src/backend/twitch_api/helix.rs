use serde_json::{json, Map, Value};
use crate::ui::ChatbotConfig;

const HELIX_URI: &'static str = "https://api.twitch.tv/helix";

#[non_exhaustive]
#[derive(Debug)]
pub enum Subscription {
    ChannelChatMessage
}

pub struct HelixClient {
    pub client: reqwest::Client,
    pub config: ChatbotConfig,
    // subscriptions: HashSet<Subscription>
}

impl HelixClient {
    pub(crate) async fn new(config: ChatbotConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config: config,
        }
    }
    
    pub async fn request_user_id(&self) -> Result<String, super::Error> {
        let mut body: Map<String, Value> = self.client
            .get(format!("{HELIX_URI}/users?login={}",&self.config.channel_name))
            .bearer_auth(&self.config.auth_token)
            .header("Client-Id", &self.config.client_id)
            .send()
            .await?.json().await?;
        
        body.get_mut("data")
            .and_then(|x| x.get_mut(0)) // data returns 0-len array
            .and_then(|x| x.get_mut("id"))
            .and_then(|x| x.as_str())
            .map(|x| x.into())
            .ok_or(super::Error::InvalidData)
            
    }
    
    pub(super) async fn subscribe(&self, sub: Subscription, session_id: &str) -> reqwest::Result<()> {
        let map = json!({
            "type": sub.as_str(),
            "version": "1",
            "condition": sub.condition(self).await,
            "transport": {
                "method": "websocket",
                "session_id": session_id
            }
        });
        
        self.client.post(format!("{HELIX_URI}/eventsub/subscriptions"))
            .bearer_auth(&self.config.auth_token)
            .header("Client-Id", &self.config.client_id)
            .json(&map)
            .send()
            .await?;
            
        Ok(())
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
    
    // condition value's convention for subscribing per
    // https://dev.twitch.tv/docs/eventsub/eventsub-reference/#conditions
    async fn condition(&self, client: &HelixClient) -> Value {
        let user_id = &client.config.user_id;
        match self {
            Subscription::ChannelChatMessage => json!({
                "broadcaster_user_id": user_id,
                "user_id": user_id
            }),
        }
    }
}
