use crate::backend::sfx::SoundsManager;
use crate::backend::twitch_api;
use backend::config::AppConfig;
use eframe::egui::{self};
use rodio::{Decoder, OutputStream};
use rodio::{OutputStreamHandle, Sink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use ui::{BackendToFrontendMessage, FrontendToBackendMessage};

pub mod backend;
pub mod ui;
use log::{error, info};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub message_id: String,
    pub message_text: String,
    pub badges: Vec<String>,
    pub username: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let (backend_tx, frontend_rx) = tokio::sync::mpsc::channel(100);
    let (frontend_tx, backend_rx) = tokio::sync::mpsc::channel(100);
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_resizable(false),
        ..Default::default()
    };
    let config = backend::config::load_config();

    let sounds_manager = SoundsManager::new()
        .await
        .expect("Sound manager initialization");

    let (_stream, stream_handle) = sounds_manager.get_stream();
    
    tokio::spawn(async move {
        handle_frontend_to_backend_messages(backend_rx, backend_tx.clone(), stream_handle).await;
    });
    info!("Starting chatbot");
    let _ = eframe::run_native(
        "Yambot",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_style(egui::Style {
                visuals: egui::Visuals::dark(),
                ..egui::Style::default()
            });
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // read values from env or other config file that will be updated later on
            Ok(Box::new(ui::Chatbot::new(
                config.chatbot,
                frontend_tx,
                frontend_rx,
                config.sfx,
                config.tts,
            )))
        }),
    )
    .map_err(|e| error!("Error: {:?}", e));
}

async fn handle_twitch_connection(
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    mut client: &mut twitch_api::Client,
    mut close: &mut Option<tokio::sync::oneshot::Sender<()>>, 
) {
    let mut rx = client.eventsub.rx.take().unwrap();
    let mut close = close.take().unwrap();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = close.closed() => break,
                Some(recv) = rx.recv() => {
                    if matches!(recv.subscription, twitch_api::helix::Subscription::ChannelChatMessage) {
                        let user = recv.event["chatter_user_name"].as_str().unwrap();
                        let message_body = recv.event["message"]["text"].as_str().unwrap();
                        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                            ui::LogLevel::INFO,
                            format!("{} sent: {}", user, message_body)
                        ));
                    };
                },
            };
        } 
    });
}


async fn handle_frontend_to_backend_messages(
    mut backend_rx: tokio::sync::mpsc::Receiver<FrontendToBackendMessage>,
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    stream_handle: rodio::OutputStreamHandle,
) {
    let _stream_handle = Arc::new(stream_handle);
    // Instead of this I should spin up separate thread that deals
    // with shared data and logging stuff; it should take cloned
    // sender while this function should have Option<Client> should be
    // as a mut argument;
    
    let mut twitch_handler: Option<twitch_api::Client> = None;
    let (mut close_tx, mut close_rx) = tokio::sync::oneshot::channel();
    let mut close_tx = Some(close_tx);


    while let Some(message) = backend_rx.recv().await {
        match message {
            FrontendToBackendMessage::UpdateTTSConfig(config) => {
                let current_config: AppConfig = backend::config::load_config();
                backend::config::save_config(
                    &(AppConfig {
                        chatbot: current_config.chatbot,
                        sfx: current_config.sfx,
                        tts: config,
                    }),
                );
                let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    "TTS config updated".to_string(),
                ));
            }
            FrontendToBackendMessage::UpdateSfxConfig(config) => {
                let current_config: AppConfig = backend::config::load_config();
                backend::config::save_config(
                    &(AppConfig {
                        chatbot: current_config.chatbot,
                        sfx: config,
                        tts: current_config.tts,
                    }),
                );
                let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    "SFX config updated".to_string(),
                ));
            }
            FrontendToBackendMessage::UpdateConfig(config) => {
                let mut helix_client = twitch_api::HelixClient::new(config).await;
                let id = helix_client.request_user_id().await;
                
                let mut config = std::mem::take(&mut helix_client.config);

                let (id, callback) = match id {
                    Ok(id) => (Some(id), None),
                    Err(e) => (None, Some(e)),
                };

                config.user_id = id;
                let _ = backend_tx.try_send(BackendToFrontendMessage::GetUserId(callback));

                let current_config: AppConfig = backend::config::load_config();
                backend::config::save_config(&AppConfig {
                    chatbot: config,
                    sfx: current_config.sfx,
                    tts: current_config.tts,
                });
                
                let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    "Chatbot config updated".to_string(),
                ));
            }
            FrontendToBackendMessage::ConnectToChat(_) => {
                log::info!("Attempting to create Twitch API connection");
                let config = backend::config::load_config().chatbot;
                twitch_handler = twitch_api::Client::new(config).await.ok();
                
                handle_twitch_connection(backend_tx.clone(), &mut twitch_handler.as_mut().unwrap(), &mut close_tx).await;
            }
            FrontendToBackendMessage::DisconnectFromChat(_) => {
                if let Some(client) = twitch_handler.take() {
                    log::info!("Dropping Twitch client");
                    close_rx.close();
                }
            }
            _ => {
                println!("Received other message: {:?}", message);
            }
        }
    }
}

async fn play_sound(sound_file: String, stream_handle: Arc<OutputStreamHandle>) {
    let sound_path = "./assets/sounds/".to_string() + &sound_file;
    if let Ok(file) = File::open(Path::new(&sound_path)) {
        let source = Decoder::new(BufReader::new(file)).unwrap();
        let sink = Sink::try_new(&*stream_handle).unwrap();
        sink.set_volume(0.5);
        sink.append(source);
        sink.detach();
    } else {
        println!("Could not open sound file: {}", sound_path);
    }
}
