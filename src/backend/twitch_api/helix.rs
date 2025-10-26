use tungstenite::Message;
use serde_json::{json, Map, Value};
use tokio::sync::{mpsc, oneshot};
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
    pub(super) client: reqwest::Client,
    pub(super) auth_token: String,
    pub(super) client_id: String,
    pub(super) channel_name: String,
    pub(super) user_id: Option<String>,
    // subscriptions: HashSet<Subscription>
}

impl HelixClient {
    pub(crate) async fn new() -> Self {
        let config = backend::config::load_config().chatbot;
        Self {
            client: reqwest::Client::new(),
            auth_token: config.auth_token,
            client_id: config.client_id,
            channel_name: config.channel_name,
            // can be None and then get_user_id be done in other
            // function with reqwest::Result<T>
            user_id: config.user_id,
            // subscriptions: HashSet::new(),
        }
    }
    pub async fn request_user_id(&self) -> Result<String, Error> {
        // move to separate function
        let mut body = reqwest::Client::new()
            .get(format!("{HELIX_URI}/users?login={}",&self.channel_name))
            .bearer_auth(&self.auth_token)
            .header("Client-Id", &self.client_id)
            .send()
            .await?.json::<Map<String, Value>>().await?;
        
        let data = &mut body.get_mut("data").ok_or(Error::InvalidData)?;
        
        // data returns 0-len array over an Object instead of just an Object
        data[0]["id"].take().as_str().map(|x| x.into()).ok_or(Error::InvalidData)
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

}

#[derive(Debug)]
pub enum Error {
    InvalidData,
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
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
        let user_id = &client.user_id;
        match self {
            Subscription::ChannelChatMessage => json!({
                "broadcaster_user_id": user_id,
                "user_id": user_id
            }),
        }
    }
}
