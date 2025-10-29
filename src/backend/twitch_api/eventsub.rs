use tungstenite::Message;
use serde_json::{Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{StreamExt};
use tungstenite::client::IntoClientRequest;
use super::helix::Subscription;

pub struct EventSubConnection {
    pub session: Option<String>,
    pub rx: Option<mpsc::UnboundedReceiver<Event>>,
}

#[derive(Debug)]
pub struct Event {
    pub subscription: Subscription,
    pub event: Value,
}

enum EventSubMessage {
    SessionId(String),
    Event(Option<Event>),
    Close(tungstenite::Utf8Bytes),
}

impl EventSubConnection {
    pub(super) async fn serve() -> Result<Self, tungstenite::Error> {
        let request = "wss://eventsub.wss.twitch.tv/ws".into_client_request().unwrap();
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (tx, rx) = mpsc::unbounded_channel();
        // tokio::SetOnce<T> would be more ergonomical(?) but its for tokio >=1.47.0
        let (session_tx, session_rx) = oneshot::channel();

        let mut session_tx = Some(session_tx);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tx.closed() => break,
                    Some(recv) = ws_stream.next() => {
                        Self::handle_message(recv.unwrap(), &tx, &mut session_tx).await;
                    },
                };
            }
        });
        
        Ok(Self {
            session: session_rx.await.ok(),
            rx: Some(rx),
        })
    }

    async fn handle_message(recv: Message, tx: &mpsc::UnboundedSender<Event>, session_tx: &mut Option<oneshot::Sender<String>>) {
        let msg: Option<EventSubMessage> = match recv {
            Message::Text(b) => Self::handle_message_bytes(b.as_bytes()).await.ok(),
            Message::Close(c) => {
                c.map(|x| EventSubMessage::Close(x.reason))
            },
            _ => None,
        };
        if let Some(event) = msg {
            match event {
                EventSubMessage::SessionId(id) => {
                    if let Some(tx) = session_tx.take() {
                        tx.send(id).ok();
                    }
                },
                EventSubMessage::Event(event) => {
                    Self::handle_sub_event(tx, event).await;
                },
                EventSubMessage::Close(reason) => {
                    log::info!("Closing Websocket connection; Reason: {:?}", reason);
                },
            };
        };
    }

    async fn handle_sub_event(tx: &mpsc::UnboundedSender<Event>, event: Option<Event>) {
        if let Some(event) = event {
            match event.subscription {
                Subscription::ChannelChatMessage => {
                    // log::info!("{:?}", event.event);
                    dbg!(tx.send(event).unwrap());
                },
            };
        };
    }
    
    async fn handle_message_bytes(bytes: &[u8]) -> serde_json::Result<EventSubMessage> {
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
