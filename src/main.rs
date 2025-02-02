use backend::config::AppConfig;
use backend::sounds::{Sounds, FILES};
use backend::watcher::Watcher;
use eframe::egui::{self};
use rodio::{Decoder, OutputStream};
use rodio::{OutputStreamHandle, Sink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
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

impl From<PrivmsgMessage> for ChatMessage {
    fn from(privmsg: PrivmsgMessage) -> Self {
        let badges = privmsg
            .badges
            .into_iter()
            .map(|badge| format!("{}-{}", badge.name, badge.version))
            .collect();
        ChatMessage {
            message_id: privmsg.message_id,
            message_text: privmsg.message_text,
            badges,
            username: privmsg.sender.login,
        }
    }
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
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let config = backend::config::load_config();
    tokio::spawn(async move {
        handle_frontend_to_backend_messages(backend_rx, backend_tx.clone(), stream_handle).await;
    });
    info!("Starting chatbot");
    let mut watcher = Watcher::serve();
    let _ = watcher.watch(Path::new("./assets/sounds/"));
    Sounds::serve().unwrap();
    let _ = watcher.push_files().await;

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

async fn handle_twitch_messages(channel_name: String, stream_handle: Arc<OutputStreamHandle>) {
    // TODO: add messages to local db
    let mut messages: Vec<ChatMessage> = Vec::new();
    let config: ClientConfig<StaticLoginCredentials> = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);
    client.join(channel_name.clone()).unwrap();

    // For now :)
    //    let mut watcher = Watcher::serve();
    //    let _ = watcher.watch(Path::new("./assets/sounds"));
    //    let sounds = Sounds::collect(watcher);

    while let Some(message) = incoming_messages.recv().await {
        match message {
            twitch_irc::message::ServerMessage::Privmsg(privmsg) => {
                let chat_message: ChatMessage = privmsg.into();
                println!("Message: {:?}", chat_message);
                if chat_message.message_text.starts_with('!') {
                    let command = chat_message.message_text.split_whitespace().next().unwrap();
                    let command = &command[1..];
                    if FILES.lock().unwrap().contains(command) {
                        let file = &format!("{}.{}", command, Sounds::get_format());
                        play_sound(file, stream_handle.clone()).await;
                    }
                }
                messages.push(chat_message);
            }
            twitch_irc::message::ServerMessage::Join(join_msg) => {
                println!("User joined: {}", join_msg.user_login);
            }
            twitch_irc::message::ServerMessage::Part(part_msg) => {
                println!("User left: {}", part_msg.user_login);
            }
            twitch_irc::message::ServerMessage::Whisper(whisper_message) => {
                println!(
                    "User {}, whispered message {}",
                    whisper_message.sender.login, whisper_message.message_text
                );
            }
            _ => {
                println!("Received other message: {:?}", message);
            }
        }
    }
}
async fn handle_frontend_to_backend_messages(
    mut backend_rx: tokio::sync::mpsc::Receiver<FrontendToBackendMessage>,
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    stream_handle: rodio::OutputStreamHandle,
) {
    let stream_handle = Arc::new(stream_handle);

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
                let current_config: AppConfig = backend::config::load_config();
                backend::config::save_config(
                    &(AppConfig {
                        chatbot: config,
                        sfx: current_config.sfx,
                        tts: current_config.tts,
                    }),
                );
                let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    "Chatbot config updated".to_string(),
                ));
            }
            FrontendToBackendMessage::ConnectToChat(channel_name) => {
                let _stream_handle = stream_handle.clone();
                tokio::spawn(async move {
                    handle_twitch_messages(channel_name, _stream_handle).await;
                });
            }
            _ => {
                println!("Received other message: {:?}", message);
            }
        }
    }
}

async fn play_sound(sound_file: &str, stream_handle: Arc<OutputStreamHandle>) {
    let sound_path = "./assets/sounds/".to_string() + sound_file;
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
