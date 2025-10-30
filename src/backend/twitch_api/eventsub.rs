use tungstenite::Message;
use serde_json::{Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{Stream, StreamExt};
use tungstenite::client::IntoClientRequest;
use super::helix::Subscription;

pub struct EventSubConnection {
    // TODO should be just String
    pub session: Option<String>,
    pub rx: mpsc::UnboundedReceiver<Event>,
}

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
    pub(super) async fn serve() -> tungstenite::Result<Self> {
        let request = "wss://eventsub.wss.twitch.tv/ws".into_client_request()?;
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (tx, rx) = mpsc::unbounded_channel();

        let mut stream = Self::wrap_stream(ws_stream);
        
        let session = (&mut stream).filter_map(|x| {
            match x {
                EventSubMessage::SessionId(id) => Some(id),
                _ => None,
            }
        }).next().await;
       

        tokio::spawn(async move {
                while let Some(msg) = stream.next().await {
                    match msg {
                        
                        EventSubMessage::Event(event) => {
                            let _ = event.map(|x| tx.send(x));
                        },
                        EventSubMessage::Close(reason) => {
                            log::info!("Closing Websocket connection; Reason: {:?}", reason);
                            break;
                        },
                        EventSubMessage::SessionId(_) => (),
                    };
            };
        });
        
        Ok(Self {
            session,
            rx,
        })
    }

    fn wrap_stream<S>(stream: S) -> impl Stream<Item = EventSubMessage> where
        S: Stream<Item = tungstenite::Result<Message>>
    {
        stream.filter_map(|x| {
            match x.ok()? {
                Message::Text(b) => Self::handle_message_bytes(b.as_bytes()).ok(),
                Message::Close(c) => {
                    c.map(|x| EventSubMessage::Close(x.reason))
                },
                _ => None,
            }
        })
    }

    fn handle_message_bytes(bytes: &[u8]) -> serde_json::Result<EventSubMessage> {
        let mut map: Map<String, Value> = serde_json::from_slice(bytes)?;
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
        let notification = Self {
            subscription: subscription.try_into().ok()?,
            event,
        };
        Some(notification)
    }
}
