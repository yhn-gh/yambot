use tungstenite::Message;
use serde_json::{json, Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::StreamExt;
use tungstenite::client::IntoClientRequest;
use std::collections::HashMap;
use std::sync::Arc;
use crate::backend;
use super::helix::Subscription;

pub struct EventSubConnection {
    pub session: Option<String>,
    pub rx: mpsc::UnboundedReceiver<Event>,
}

#[derive(Debug)]
struct Event {
    subscription: Subscription,
    event: Value,
}

enum EventSubMessage {
    SessionId(String),
    Event(Option<Event>),
}

type WelcomeDone = Option<oneshot::Sender<String>>;

impl EventSubConnection {
    // TODO In case session dies should make a new one
    pub(super) async fn serve() -> Result<Self, tungstenite::Error> {
        let request = "wss://eventsub.wss.twitch.tv/ws".into_client_request().unwrap();
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (tx, rx) = mpsc::unbounded_channel();
        
        let (tx_session, rx_session) = tokio::sync::oneshot::channel();
        let mut tx_session: WelcomeDone = tx_session.into();
        
        tokio::spawn(async move {
            while let Some(recv) = ws_stream.next().await {
                // remove unwrap
                let recv = recv.unwrap();
                let msg: Option<EventSubMessage> = match recv {
                    Message::Text(b) => Self::handle_message_bytes(b.as_bytes()).await.ok(),
                    _ => None,
                };
                // do declaratively this
                if let Some(event) = msg {
                    match event {
                        EventSubMessage::SessionId(id) => {
                            if let Some(sender) = tx_session.take() {
                                sender.send(id).ok();
                            };
                        },
                        EventSubMessage::Event(event) => {
                            Self::handle_sub_event(tx.clone(), event).await
                        },
                    };
                };
            };
        });
        
        let session = rx_session.await.ok();
        log::info!("Session ID: {:?}", session);

        Ok(Self {
            session,
            rx,
        })
    }

    async fn handle_sub_event(tx: mpsc::UnboundedSender<Event>, event: Option<Event>) {
        if let Some(event) = event {
            match event.subscription {
                Subscription::ChannelChatMessage => {
                    log::info!("{:?}", event);
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
            "session_welcome" => {
                let id: String = serde_json::from_value(payload["session"]["id"].take())?;
                EventSubMessage::SessionId(id)
            },
            "notification" => {
                let payload = Event::parse(payload);
                EventSubMessage::Event(payload)
            },
            _ => EventSubMessage::Event(None),
        };
        Ok(msg)
    }
}
impl Event {
    fn parse(mut msg: Value) -> Option<Self> {
        let subscription = msg["subscription"]["type"].take();
        let subscription = subscription.as_str()?;
        let event =  msg["event"].take();
        let notification = Self {
            subscription: subscription.try_into().ok()?,
            event,
        };
        Some(notification)
    }
}
