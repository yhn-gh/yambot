use tungstenite::Message;
use serde_json::{Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{Stream, StreamExt};
use tungstenite::client::IntoClientRequest;
use super::helix::Subscription;

pub struct EventSubConnection;

#[derive(Debug)]
pub struct Event {
    pub subscription: Subscription,
    pub event: Value,
}

#[derive(Debug)]
enum EventSubMessage {
    SessionId(String),
    Event(Option<Event>),
    Close(tungstenite::Utf8Bytes),
}

impl EventSubConnection {
    pub(super) async fn serve() -> tungstenite::Result<(String, impl Stream<Item = Event>)> {
        let request = "wss://eventsub.wss.twitch.tv/ws".into_client_request()?;
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(request).await?;

        let mut stream = Self::wrap_stream(ws_stream);
        
        let session = (&mut stream).filter_map(|x| {
            match x {
                EventSubMessage::SessionId(id) => Some(id),
                _ => None,
            }
        }).next().await.unwrap();

        let stream = stream.filter_map(|x| {
            match x {
                EventSubMessage::Event(event) => event,
                _ => None,
            }
        });

        Ok((session, stream))
    }

    fn wrap_stream<S>(stream: S) -> impl Stream<Item = EventSubMessage> where
        S: Stream<Item = tungstenite::Result<Message>>
    {
        stream.filter_map(|x| {
            match x.ok()? {
                Message::Text(bytes) => EventSubMessage::from_message_bytes(&bytes).ok(),
                Message::Close(close) => close.map(|x| EventSubMessage::Close(x.reason)),
                _ => None,
            }
        })
    }
}

impl EventSubMessage {
    fn from_message_bytes(bytes: &str) -> serde_json::Result<EventSubMessage> {
        let mut map: Map<String, Value> = serde_json::from_str(bytes)?;
        let metadata: Value = map["metadata"].take();
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
        Some(Self {
            subscription: subscription.try_into().ok()?,
            event,
        })
    }
}
