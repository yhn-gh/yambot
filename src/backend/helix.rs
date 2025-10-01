use tungstenite::Message;
use serde_json::{Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::StreamExt;
use tungstenite::client::IntoClientRequest;
use std::collections::HashMap;

pub struct EventSubConnection {
    session: Option<String>,
    pub rx: mpsc::UnboundedReceiver<Event>,
}

#[non_exhaustive]
enum Subscription {
    ChannelChatMessage
}

enum EventSubMessage {
    None,
    SessionId(String),
    Event(Option<Event>),
}

struct Event {
    subscription: Subscription,
    event: Value,
}

type WelcomeDone = Option<oneshot::Sender<String>>;

impl EventSubConnection {
    // TODO In case session dies should make a new one
    pub async fn new() -> Result<Self, tungstenite::Error> {
        let request = "wss://eventsub.wss.twitch.tv/ws".into_client_request().unwrap();
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (tx, rx) = mpsc::unbounded_channel();
        
        let (tx_session, rx_session) = tokio::sync::oneshot::channel();
        let mut tx_session: WelcomeDone = tx_session.into();
        
        tokio::spawn(async move {
            while let Some(recv) = ws_stream.next().await {
                let msg: Option<EventSubMessage> = match recv.unwrap() {
                    Message::Text(b) => Self::handle_message_bytes(b.as_bytes()).await.ok(),
                    _ => None,
                };
                if let Some(event) = msg {
                    match event {
                        EventSubMessage::SessionId(id) => {
                            if let Some(sender) = (&mut tx_session).take() {
                                sender.send(id).ok();
                            };
                        },
                        EventSubMessage::Event(event) => {
                            Self::handle_sub_event(tx.clone(), event).await
                        },
                        EventSubMessage::None => (),
                    };
                };
            };
        });
        
        let session = rx_session.await.ok();
        log::info!("{:?}", session);

        Ok(Self {
            session,
            rx,
        })
    }

    pub async fn subscribe(&self, sub: Subscription) -> reqwest::Result<()> {
        let session_id = &self.session;
        todo!();
        Ok(())
    }

    async fn handle_sub_event(tx: mpsc::UnboundedSender<Event>, event: Option<Event>) {
        if let Some(event) = event {
            match event.subscription {
                Subscription::ChannelChatMessage => {
                },
                _ => (),
            };
        };
    }
    
    async fn handle_message_bytes(bytes: &[u8]) -> serde_json::Result<EventSubMessage> {
        let mut map: Map<String, Value> = serde_json::from_slice(bytes)?;
        let mut metadata: Value = map["metadata"].take();
        let mut payload: Value = map["payload"].take();

        let msg = match metadata["message_type"].as_str().expect("Twitch API returned unexpected data") {
            "session_welcome" => EventSubMessage::SessionId(payload["session"]["id"].take().to_string()),
            "notification" => EventSubMessage::Event(payload.try_into().ok()),
            _ => EventSubMessage::None,
        };
        Ok(msg)
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

impl TryFrom<Value> for Event {
    // TODO change to io::Error
    type Error = ();
    fn try_from(mut msg: Value) -> Result<Self, Self::Error> {
        let subscription =  msg["subscription"].get_mut("type").ok_or(())?.to_string();
        let event =  msg.get_mut("event").ok_or(())?.take();
        let notification = Self {
            subscription: (&*subscription).try_into()?,
            event,
        };
        Ok(notification)
    }
}
