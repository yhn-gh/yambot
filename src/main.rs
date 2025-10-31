use crate::backend::commands::{CommandExecutor, CommandParser, CommandRegistry, CommandResult};
use crate::backend::sfx::SoundsManager;
use crate::backend::twitch::{
    ChatMessageEvent, TwitchClient, TwitchClientEvent, TwitchConfig, TwitchEvent,
};
use backend::config::AppConfig;
use eframe::egui::{self};
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use ui::{BackendToFrontendMessage, FrontendToBackendMessage};

pub mod backend;
pub mod ui;
use log::{error, info};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub message_id: String,
    pub message_text: String,
    pub badges: Vec<String>,
    pub username: String,
    pub user_id: String,
    pub color: String,
}

impl From<ChatMessageEvent> for ChatMessage {
    fn from(msg: ChatMessageEvent) -> Self {
        let badges = msg
            .badges
            .into_iter()
            .map(|badge| format!("{}-{}", badge.set_id, badge.id))
            .collect();

        ChatMessage {
            message_id: msg.message_id,
            message_text: msg.message.text,
            badges,
            username: msg.chatter_user_login,
            user_id: msg.chatter_user_id,
            color: msg.color,
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
    let config = backend::config::load_config();
    let command_registry = backend::config::load_commands();

    // Wrap command registry in Arc<RwLock> for sharing across tasks
    let shared_registry = Arc::new(RwLock::new(command_registry));

    let sounds_manager = SoundsManager::new()
        .await
        .expect("Sound manager initialization");

    let stream = sounds_manager.get_stream();
    let registry_clone = shared_registry.clone();
    tokio::spawn(async move {
        handle_frontend_to_backend_messages(backend_rx, backend_tx.clone(), stream, registry_clone)
            .await;
    });
    info!("Starting chatbot");

    // Get initial commands for UI
    let commands = {
        let registry = shared_registry.read().await;
        registry.list().iter().map(|c| (*c).clone()).collect()
    };

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
                commands,
            )))
        }),
    )
    .map_err(|e| error!("Error: {:?}", e));
}

async fn handle_twitch_messages(
    config: TwitchConfig,
    backend_tx: tokio::sync::mpsc::Sender<ui::BackendToFrontendMessage>,
    command_registry: Arc<RwLock<CommandRegistry>>,
    welcome_message: Option<String>,
) {
    // TODO: add messages to local db
    let mut messages: Vec<ChatMessage> = Vec::new();
    let command_parser = CommandParser::with_default_prefix();

    // Create event channel
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // Create and connect Twitch client
    let mut client = TwitchClient::new(config);

    match client.connect(tx).await {
        Ok(_) => {
            let _ = backend_tx
                .send(ui::BackendToFrontendMessage::ConnectionSuccess(
                    "Connected".to_string(),
                ))
                .await;
            let _ = backend_tx
                .send(ui::BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    "Successfully connected to Twitch chat".to_string(),
                ))
                .await;

            // Send welcome message if configured
            if let Some(ref msg) = welcome_message {
                if !msg.trim().is_empty() {
                    log::info!("Attempting to send welcome message: {}", msg);

                    // Wait a moment for subscriptions to settle
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    match client.send_message(msg).await {
                        Ok(_) => {
                            log::info!("Welcome message sent successfully");
                            let _ = backend_tx
                                .send(ui::BackendToFrontendMessage::CreateLog(
                                    ui::LogLevel::INFO,
                                    format!("✓ Sent welcome message: {}", msg),
                                ))
                                .await;
                        }
                        Err(e) => {
                            log::error!("Failed to send welcome message: {}", e);
                            let error_str = e.to_string();

                            let user_msg = if error_str.contains("403")
                                || error_str.contains("Forbidden")
                            {
                                format!("❌ Cannot send welcome message - Missing OAuth scope 'user:write:chat'. Please re-authorize with write permissions.")
                            } else {
                                format!("❌ Failed to send welcome message: {}", e)
                            };

                            let _ = backend_tx
                                .send(ui::BackendToFrontendMessage::CreateLog(
                                    ui::LogLevel::ERROR,
                                    user_msg,
                                ))
                                .await;
                        }
                    }
                }
            }
        }
        Err(e) => {
            let _ = backend_tx
                .send(ui::BackendToFrontendMessage::ConnectionFailure(
                    "Connection Failed".to_string(),
                ))
                .await;
            let _ = backend_tx
                .send(ui::BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::ERROR,
                    format!("Failed to connect: {}", e),
                ))
                .await;
            return;
        }
    }

    // Handle incoming events
    while let Some(event) = rx.recv().await {
        match event {
            TwitchClientEvent::Connected => {
                let _ = backend_tx
                    .send(ui::BackendToFrontendMessage::CreateLog(
                        ui::LogLevel::INFO,
                        "EventSub ready - listening for chat events".to_string(),
                    ))
                    .await;
            }

            TwitchClientEvent::ChatEvent(chat_event) => match chat_event {
                TwitchEvent::ChatMessage(msg) => {
                    let chat_message: ChatMessage = msg.clone().into();
                    // Check if message is a command
                    if let Some(context) = command_parser.parse(msg.clone()) {
                        // Lock the registry and execute command
                        let result = {
                            let mut registry = command_registry.write().await;
                            let mut executor = CommandExecutor::new(registry.clone());
                            let result = executor.execute(&context);

                            // Update cooldowns in the shared registry
                            *registry = executor.registry().clone();
                            result
                        };

                        match result {
                            CommandResult::Success(Some(action)) => {
                                // Parse the action and handle it
                                if let Some(play_sound) = action.strip_prefix("play_sound:") {
                                    let _ = backend_tx
                                        .send(BackendToFrontendMessage::PlaySound(
                                            play_sound.to_string(),
                                        ))
                                        .await;
                                } else if let Some(send_msg) = action.strip_prefix("send:") {
                                    if let Err(e) = client.send_message(send_msg).await {
                                        let _ = backend_tx
                                            .send(BackendToFrontendMessage::CreateLog(
                                                ui::LogLevel::ERROR,
                                                format!("Failed to send message: {}", e),
                                            ))
                                            .await;
                                    }
                                } else if let Some(reply_parts) = action.strip_prefix("reply:") {
                                    let parts: Vec<&str> = reply_parts.splitn(2, ':').collect();
                                    if parts.len() == 2 {
                                        let message_id = parts[0];
                                        let reply_msg = parts[1];
                                        if let Err(e) =
                                            client.reply_to_message(reply_msg, message_id).await
                                        {
                                            error!("Failed to reply: {}", e);
                                            let _ = backend_tx
                                                .send(BackendToFrontendMessage::CreateLog(
                                                    ui::LogLevel::ERROR,
                                                    format!("Failed to reply: {}", e),
                                                ))
                                                .await;
                                        }
                                    }
                                } else if let Some(_tts_msg) = action.strip_prefix("tts:") {
                                    let _ = backend_tx
                                        .send(BackendToFrontendMessage::CreateLog(
                                            ui::LogLevel::INFO,
                                            "TTS not yet implemented".to_string(),
                                        ))
                                        .await;
                                }
                            }
                            CommandResult::Success(None) => {}
                            CommandResult::Error(e) => {
                                let _ = backend_tx
                                    .send(BackendToFrontendMessage::CreateLog(
                                        ui::LogLevel::ERROR,
                                        format!("Command error: {}", e),
                                    ))
                                    .await;
                            }
                            CommandResult::NotFound => {
                                // Command not found, ignore silently
                            }
                            CommandResult::PermissionDenied => {
                                let _ = backend_tx
                                    .send(BackendToFrontendMessage::CreateLog(
                                        ui::LogLevel::WARN,
                                        format!(
                                            "User {} tried to use command !{} without permission",
                                            context.username(),
                                            context.command_name
                                        ),
                                    ))
                                    .await;
                            }
                            CommandResult::OnCooldown(_remaining) => {}
                        }
                    }

                    messages.push(chat_message);
                }

                TwitchEvent::MessageDelete(delete) => {
                    println!(
                        "Message {} from {} was deleted",
                        delete.message_id, delete.target_user_name
                    );
                }

                TwitchEvent::ClearUserMessages(clear) => {
                    println!(
                        "Messages from {} were cleared (ban/timeout)",
                        clear.target_user_name
                    );
                }

                TwitchEvent::ChatClear(clear) => {
                    println!(
                        "Chat was cleared in {}'s channel",
                        clear.broadcaster_user_name
                    );
                }

                TwitchEvent::ChatSettingsUpdate(settings) => {
                    println!(
                        "Chat settings updated: slow_mode={}, sub_only={}",
                        settings.slow_mode, settings.subscriber_mode
                    );
                }

                TwitchEvent::ChannelBan(ban) => {
                    let ban_type = if ban.is_permanent {
                        "permanently banned"
                    } else {
                        "timed out"
                    };
                    let duration_info = if let Some(ref ends_at) = ban.ends_at {
                        format!(" (until {})", ends_at)
                    } else {
                        String::new()
                    };

                    println!(
                        "🔨 {} was {} by {}: {}{}",
                        ban.user_name, ban_type, ban.moderator_user_name, ban.reason, duration_info
                    );

                    let _ = backend_tx
                        .send(BackendToFrontendMessage::CreateLog(
                            ui::LogLevel::WARN,
                            format!(
                                "{} was {} by {}: {}{}",
                                ban.user_name,
                                ban_type,
                                ban.moderator_user_name,
                                ban.reason,
                                duration_info
                            ),
                        ))
                        .await;
                }

                TwitchEvent::ChannelUnban(unban) => {
                    println!(
                        "✅ {} was unbanned by {}",
                        unban.user_name, unban.moderator_user_name
                    );

                    let _ = backend_tx
                        .send(BackendToFrontendMessage::CreateLog(
                            ui::LogLevel::INFO,
                            format!(
                                "{} was unbanned by {}",
                                unban.user_name, unban.moderator_user_name
                            ),
                        ))
                        .await;
                }
            },

            TwitchClientEvent::TokensRefreshed(access_token, refresh_token) => {
                // Load current config
                let mut current_config = backend::config::load_config();

                // Update tokens
                current_config.chatbot.auth_token = access_token;
                current_config.chatbot.refresh_token = refresh_token;

                // Save updated config
                backend::config::save_config(&current_config);
            }

            TwitchClientEvent::Disconnected => {
                let _ = backend_tx
                    .send(ui::BackendToFrontendMessage::ConnectionFailure(
                        "Disconnected".to_string(),
                    ))
                    .await;
                let _ = backend_tx
                    .send(ui::BackendToFrontendMessage::CreateLog(
                        ui::LogLevel::ERROR,
                        "Disconnected from Twitch".to_string(),
                    ))
                    .await;
                break;
            }

            TwitchClientEvent::Warning(w) => {
                let _ = backend_tx
                    .send(ui::BackendToFrontendMessage::CreateLog(
                        ui::LogLevel::WARN,
                        w,
                    ))
                    .await;
            }

            TwitchClientEvent::Error(e) => {
                let _ = backend_tx
                    .send(ui::BackendToFrontendMessage::CreateLog(
                        ui::LogLevel::ERROR,
                        format!("Twitch error: {}", e),
                    ))
                    .await;
            }
        }
    }
}
async fn handle_frontend_to_backend_messages(
    mut backend_rx: tokio::sync::mpsc::Receiver<FrontendToBackendMessage>,
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    stream: OutputStream,
    command_registry: Arc<RwLock<CommandRegistry>>,
) {
    let stream = Arc::new(stream);

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
            FrontendToBackendMessage::ConnectToChat(_channel_name) => {
                // Load config to get auth_token and client_id
                let config = backend::config::load_config();
                let twitch_config = TwitchConfig {
                    channel_name: config.chatbot.channel_name.clone(),
                    auth_token: config.chatbot.auth_token.clone(),
                    refresh_token: config.chatbot.refresh_token.clone(),
                };

                // Get welcome message if configured
                let welcome_message = if config.chatbot.welcome_message.trim().is_empty() {
                    None
                } else {
                    Some(config.chatbot.welcome_message.clone())
                };

                let backend_tx_clone = backend_tx.clone();
                let registry_clone = command_registry.clone();
                tokio::spawn(async move {
                    handle_twitch_messages(
                        twitch_config,
                        backend_tx_clone,
                        registry_clone,
                        welcome_message,
                    )
                    .await;
                });

                let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    "Connecting to Twitch...".to_string(),
                ));
            }
            FrontendToBackendMessage::PlaySound(sound_file) => {
                let stream_clone = stream.clone();
                tokio::spawn(async move {
                    play_sound(sound_file, stream_clone).await;
                });
            }
            FrontendToBackendMessage::AddCommand(command) => {
                {
                    let mut registry = command_registry.write().await;
                    registry.register(command);
                    backend::config::save_commands(&registry);
                }
                let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    "Command added".to_string(),
                ));
                let _ = backend_tx.try_send(BackendToFrontendMessage::CommandsUpdated);
            }
            FrontendToBackendMessage::RemoveCommand(trigger) => {
                {
                    let mut registry = command_registry.write().await;
                    registry.unregister(&trigger);
                    backend::config::save_commands(&registry);
                }
                let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    format!("Command '{}' removed", trigger),
                ));
                let _ = backend_tx.try_send(BackendToFrontendMessage::CommandsUpdated);
            }
            FrontendToBackendMessage::UpdateCommand(command) => {
                {
                    let mut registry = command_registry.write().await;
                    registry.register(command);
                    backend::config::save_commands(&registry);
                }
                let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                    ui::LogLevel::INFO,
                    "Command updated".to_string(),
                ));
                let _ = backend_tx.try_send(BackendToFrontendMessage::CommandsUpdated);
            }
            FrontendToBackendMessage::ToggleCommand(trigger, enabled) => {
                let mut registry = command_registry.write().await;
                if let Some(cmd) = registry.get_mut(&trigger) {
                    cmd.enabled = enabled;
                    backend::config::save_commands(&registry);
                    let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
                        ui::LogLevel::INFO,
                        format!(
                            "Command '{}' {}",
                            trigger,
                            if enabled { "enabled" } else { "disabled" }
                        ),
                    ));
                }
            }
            _ => {
                println!("Received other message: {:?}", message);
            }
        }
    }
}

async fn play_sound(sound_file: String, stream: Arc<OutputStream>) {
    let sound_path = "./assets/sounds/".to_string() + &sound_file;
    if let Ok(file) = File::open(Path::new(&sound_path)) {
        let source = Decoder::new(BufReader::new(file)).unwrap();
        let sink = Sink::connect_new(&stream.mixer());
        sink.set_volume(0.5);
        sink.append(source);
        sink.detach();
    } else {
        println!("Could not open sound file: {}", sound_path);
    }
}
