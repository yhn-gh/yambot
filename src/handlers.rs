use crate::audio::AudioPlaybackSender;
use crate::backend::commands::{CommandExecutor, CommandParser, CommandRegistry, CommandResult};
use crate::backend::config::AppConfig;
use crate::backend::tts::{
    LanguageConfig, TTSAudioChunk, TTSQueue, TTSQueueItem, TTSRequest, TTSService,
};
use crate::backend::twitch::{TwitchClient, TwitchClientEvent, TwitchConfig};
use crate::ui::{
    BackendToFrontendMessage, ChatbotConfig, Config, FrontendToBackendMessage, LogLevel,
    TTSQueueItemUI,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub message_id: String,
    pub message_text: String,
    pub badges: Vec<String>,
    pub username: String,
    pub user_id: String,
    pub color: String,
}

impl From<crate::backend::twitch::ChatMessageEvent> for ChatMessage {
    fn from(msg: crate::backend::twitch::ChatMessageEvent) -> Self {
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

pub async fn handle_twitch_messages(
    config: TwitchConfig,
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    audio_tx: AudioPlaybackSender,
    command_registry: Arc<RwLock<CommandRegistry>>,
    tts_queue: TTSQueue,
    tts_service: Arc<TTSService>,
    language_config: Arc<RwLock<LanguageConfig>>,
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
                .send(BackendToFrontendMessage::ConnectionSuccess(
                    "Connected".to_string(),
                ))
                .await;
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(
                    LogLevel::INFO,
                    "Successfully connected to Twitch chat".to_string(),
                ))
                .await;

            // Send welcome message if configured
            if let Some(ref msg) = welcome_message {
                send_welcome_message(&mut client, msg, &backend_tx).await;
            }
        }
        Err(e) => {
            let _ = backend_tx
                .send(BackendToFrontendMessage::ConnectionFailure(
                    "Connection Failed".to_string(),
                ))
                .await;
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(
                    LogLevel::ERROR,
                    format!("Failed to connect: {}", e),
                ))
                .await;
            return;
        }
    }

    // Handle incoming events
    while let Some(event) = rx.recv().await {
        handle_twitch_event(
            event,
            &mut messages,
            &backend_tx,
            &mut client,
            &audio_tx,
            &command_registry,
            &command_parser,
            &tts_queue,
            &tts_service,
            &language_config,
        )
        .await;
    }
}

async fn send_welcome_message(
    client: &mut TwitchClient,
    msg: &str,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    if !msg.trim().is_empty() {
        info!("Attempting to send welcome message: {}", msg);

        // Wait a moment for subscriptions to settle
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        match client.send_message(msg).await {
            Ok(_) => {
                info!("Welcome message sent successfully");
                let _ = backend_tx
                    .send(BackendToFrontendMessage::CreateLog(
                        LogLevel::INFO,
                        format!("✓ Sent welcome message: {}", msg),
                    ))
                    .await;
            }
            Err(e) => {
                error!("Failed to send welcome message: {}", e);
                let error_str = e.to_string();

                let user_msg = if error_str.contains("403") || error_str.contains("Forbidden") {
                    format!("❌ Cannot send welcome message - Missing OAuth scope 'user:write:chat'. Please re-authorize with write permissions.")
                } else {
                    format!("❌ Failed to send welcome message: {}", e)
                };

                let _ = backend_tx
                    .send(BackendToFrontendMessage::CreateLog(
                        LogLevel::ERROR,
                        user_msg,
                    ))
                    .await;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_twitch_event(
    event: TwitchClientEvent,
    messages: &mut Vec<ChatMessage>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    client: &mut TwitchClient,
    audio_tx: &AudioPlaybackSender,
    command_registry: &Arc<RwLock<CommandRegistry>>,
    command_parser: &CommandParser,
    tts_queue: &TTSQueue,
    tts_service: &Arc<TTSService>,
    language_config: &Arc<RwLock<LanguageConfig>>,
) {
    match event {
        TwitchClientEvent::Connected => {
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(
                    LogLevel::INFO,
                    "EventSub ready - listening for chat events".to_string(),
                ))
                .await;
        }

        TwitchClientEvent::ChatEvent(chat_event) => match chat_event {
            crate::backend::twitch::TwitchEvent::ChatMessage(msg) => {
                let chat_message: ChatMessage = msg.clone().into();

                // Check if message is a TTS command
                if handle_tts_command(&msg, tts_queue, tts_service, language_config, backend_tx)
                    .await
                {
                    messages.push(chat_message);
                    return;
                }

                // Check if message is a command
                if let Some(context) = command_parser.parse(msg.clone()) {
                    handle_command(context, command_registry, client, backend_tx, audio_tx).await;
                }

                messages.push(chat_message);
            }

            crate::backend::twitch::TwitchEvent::MessageDelete(delete) => {
                info!(
                    "Message {} from {} was deleted",
                    delete.message_id, delete.target_user_name
                );
            }

            crate::backend::twitch::TwitchEvent::ClearUserMessages(clear) => {
                info!(
                    "Messages from {} were cleared (ban/timeout)",
                    clear.target_user_name
                );
            }

            crate::backend::twitch::TwitchEvent::ChatClear(clear) => {
                info!(
                    "Chat was cleared in {}'s channel",
                    clear.broadcaster_user_name
                );
            }

            crate::backend::twitch::TwitchEvent::ChatSettingsUpdate(settings) => {
                info!(
                    "Chat settings updated: slow_mode={}, sub_only={}",
                    settings.slow_mode, settings.subscriber_mode
                );
            }

            crate::backend::twitch::TwitchEvent::ChannelBan(ban) => {
                handle_ban_event(&ban, backend_tx).await;
            }

            crate::backend::twitch::TwitchEvent::ChannelUnban(unban) => {
                info!(
                    "✅ {} was unbanned by {}",
                    unban.user_name, unban.moderator_user_name
                );

                let _ = backend_tx
                    .send(BackendToFrontendMessage::CreateLog(
                        LogLevel::INFO,
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
            let mut current_config = crate::backend::config::load_config();

            // Update tokens
            current_config.chatbot.auth_token = access_token;
            current_config.chatbot.refresh_token = refresh_token;

            // Save updated config
            crate::backend::config::save_config(&current_config);
        }

        TwitchClientEvent::Disconnected => {
            let _ = backend_tx
                .send(BackendToFrontendMessage::ConnectionFailure(
                    "Disconnected".to_string(),
                ))
                .await;
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(
                    LogLevel::ERROR,
                    "Disconnected from Twitch".to_string(),
                ))
                .await;
        }

        TwitchClientEvent::Warning(w) => {
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(LogLevel::WARN, w))
                .await;
        }

        TwitchClientEvent::Error(e) => {
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(
                    LogLevel::ERROR,
                    format!("Twitch error: {}", e),
                ))
                .await;
        }
    }
}

async fn handle_tts_command(
    msg: &crate::backend::twitch::ChatMessageEvent,
    tts_queue: &TTSQueue,
    tts_service: &Arc<TTSService>,
    language_config: &Arc<RwLock<LanguageConfig>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) -> bool {
    let message_text = msg.message.text.trim().to_lowercase();
    if message_text.starts_with('!') && message_text.len() > 1 {
        let parts: Vec<&str> = message_text.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let potential_lang_code = &parts[0][1..]; // Remove the '!' prefix
            let tts_text = parts[1];

            // Check if this is a valid language code
            let lang_config = language_config.read().await;
            if let Some(language) = lang_config.get_language(potential_lang_code) {
                if language.enabled {
                    // Check TTS config and permissions
                    let config = crate::backend::config::load_config();
                    if config.tts.enabled {
                        // Check user permissions
                        let has_permission = msg.badges.iter().any(|badge| {
                            (badge.set_id == "subscriber" || badge.set_id == "founder")
                                && config.tts.permited_roles.subs
                                || badge.set_id == "vip" && config.tts.permited_roles.vips
                                || badge.set_id == "moderator" && config.tts.permited_roles.mods
                                || badge.set_id == "broadcaster"
                        });

                        if !has_permission {
                            return true;
                        }
                        if tts_queue.is_user_ignored(&msg.chatter_user_login).await {
                            return true;
                        }

                        let tts_request = TTSRequest {
                            id: msg.message_id.clone(),
                            username: msg.chatter_user_login.clone(),
                            language: potential_lang_code.to_string(),
                            text: tts_text.to_string(),
                            timestamp: chrono::Utc::now(),
                        };

                        // Generate TTS files asynchronously
                        spawn_tts_generation(
                            tts_request,
                            tts_service.clone(),
                            tts_queue.clone(),
                            backend_tx.clone(),
                        );
                    }
                }
                // If it's a valid language code, don't process as regular command
                return true;
            }
        }
    }
    false
}

fn spawn_tts_generation(
    tts_request: TTSRequest,
    tts_service: Arc<TTSService>,
    tts_queue: TTSQueue,
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    tokio::spawn(async move {
        // Split text into chunks
        let text_chunks = tts_service.split_text(&tts_request.text);
        let chunk_count = text_chunks.len();

        // Process each chunk as a separate queue item
        for (chunk_index, text_chunk) in text_chunks.into_iter().enumerate() {
            // Create unique ID for this chunk
            let chunk_id = if chunk_count > 1 {
                format!("{}-{}", tts_request.id, chunk_index)
            } else {
                tts_request.id.clone()
            };

            // Fetch audio for this chunk
            match tts_service
                .fetch_tts_audio(&text_chunk, &tts_request.language)
                .await
            {
                Ok(audio_data) => {
                    let chunk_request = TTSRequest {
                        id: chunk_id,
                        username: tts_request.username.clone(),
                        language: tts_request.language.clone(),
                        text: text_chunk,
                        timestamp: tts_request.timestamp,
                    };

                    let queue_item = TTSQueueItem {
                        request: chunk_request,
                        audio_chunks: vec![TTSAudioChunk { audio_data }],
                    };

                    tts_queue.add(queue_item).await;

                    // Send updated queue to frontend (including currently playing)
                    let queue_items = tts_queue.get_all_with_current().await;
                    let ui_queue: Vec<TTSQueueItemUI> = queue_items
                        .into_iter()
                        .map(|item| TTSQueueItemUI {
                            id: item.request.id,
                            username: item.request.username,
                            text: item.request.text,
                            language: item.request.language,
                        })
                        .collect();
                    let _ = backend_tx
                        .send(BackendToFrontendMessage::TTSQueueUpdated(ui_queue))
                        .await;
                }
                Err(e) => {
                    error!(
                        "Failed to fetch TTS audio for chunk {}/{}: {}",
                        chunk_index + 1,
                        chunk_count,
                        e
                    );
                    let _ = backend_tx
                        .send(BackendToFrontendMessage::CreateLog(
                            LogLevel::ERROR,
                            format!("Failed to generate TTS chunk: {}", e),
                        ))
                        .await;
                }
            }
        }
    });
}

async fn handle_command(
    context: crate::backend::commands::CommandContext,
    command_registry: &Arc<RwLock<CommandRegistry>>,
    client: &mut TwitchClient,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    audio_tx: &AudioPlaybackSender,
) {
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
            handle_command_action(action, client, backend_tx).await;
        }
        CommandResult::Success(None) => {}
        CommandResult::Error(e) => {
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(
                    LogLevel::ERROR,
                    format!("Command error: {}", e),
                ))
                .await;
        }
        CommandResult::NotFound => {
            handle_sound_file(&context, audio_tx);
        }
        CommandResult::PermissionDenied => {
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(
                    LogLevel::WARN,
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

async fn handle_command_action(
    action: String,
    client: &mut TwitchClient,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    if let Some(send_msg) = action.strip_prefix("send:") {
        if let Err(e) = client.send_message(send_msg).await {
            let _ = backend_tx
                .send(BackendToFrontendMessage::CreateLog(
                    LogLevel::ERROR,
                    format!("Failed to send message: {}", e),
                ))
                .await;
        }
    } else if let Some(reply_parts) = action.strip_prefix("reply:") {
        let parts: Vec<&str> = reply_parts.splitn(2, ':').collect();
        if parts.len() == 2 {
            let message_id = parts[0];
            let reply_msg = parts[1];
            if let Err(e) = client.reply_to_message(reply_msg, message_id).await {
                error!("Failed to reply: {}", e);
                let _ = backend_tx
                    .send(BackendToFrontendMessage::CreateLog(
                        LogLevel::ERROR,
                        format!("Failed to reply: {}", e),
                    ))
                    .await;
            }
        }
    }
}

fn handle_sound_file(
    context: &crate::backend::commands::CommandContext,
    audio_tx: &AudioPlaybackSender,
) {
    // Check if there's a sound file with this name
    let sound_format = crate::backend::sfx::Soundlist::get_format();
    let sound_path = format!("./assets/sounds/{}.{}", context.command_name, sound_format);

    if std::path::Path::new(&sound_path).exists() {
        // Check if user has permission to play sounds
        let config = crate::backend::config::load_config();
        let has_permission = context.badges().iter().any(|badge| {
            (badge.set_id == "subscriber" || badge.set_id == "founder")
                && config.sfx.permited_roles.subs
                || badge.set_id == "vip" && config.sfx.permited_roles.vips
                || badge.set_id == "moderator" && config.sfx.permited_roles.mods
                || badge.set_id == "broadcaster"
        });

        if has_permission && config.sfx.enabled {
            // Play the sound with volume from sfx config
            let sound_file = format!("{}.{}", context.command_name, sound_format);
            let _ = audio_tx.send_sound(sound_file, config.sfx.volume as f32);
        }
    }
}

async fn handle_ban_event(
    ban: &crate::backend::twitch::ChannelBanEvent,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
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

    info!(
        "🔨 {} was {} by {}: {}{}",
        ban.user_name, ban_type, ban.moderator_user_name, ban.reason, duration_info
    );

    let _ = backend_tx
        .send(BackendToFrontendMessage::CreateLog(
            LogLevel::WARN,
            format!(
                "{} was {} by {}: {}{}",
                ban.user_name, ban_type, ban.moderator_user_name, ban.reason, duration_info
            ),
        ))
        .await;
}

pub async fn handle_frontend_to_backend_messages(
    mut backend_rx: tokio::sync::mpsc::Receiver<FrontendToBackendMessage>,
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    audio_tx: AudioPlaybackSender,
    command_registry: Arc<RwLock<CommandRegistry>>,
    tts_queue: TTSQueue,
    tts_service: Arc<TTSService>,
    language_config: Arc<RwLock<LanguageConfig>>,
    overlay_ws_state: crate::backend::overlay::WebSocketState,
) {
    // Store the handle to the twitch message handler task so we can abort it on disconnect
    let mut twitch_task_handle: Option<tokio::task::JoinHandle<()>> = None;
    while let Some(message) = backend_rx.recv().await {
        match message {
            FrontendToBackendMessage::AddTTSLang(lang_code) => {
                handle_add_tts_lang(lang_code, &language_config, &backend_tx).await;
            }
            FrontendToBackendMessage::RemoveTTSLang(lang_code) => {
                handle_remove_tts_lang(lang_code, &language_config, &backend_tx).await;
            }
            FrontendToBackendMessage::UpdateTTSConfig(config) => {
                update_tts_config(config, &backend_tx);
            }
            FrontendToBackendMessage::UpdateSfxConfig(config) => {
                update_sfx_config(config, &backend_tx);
            }
            FrontendToBackendMessage::UpdateConfig(config) => {
                update_chatbot_config(config, &backend_tx);
            }
            FrontendToBackendMessage::ConnectToChat(_channel_name) => {
                connect_to_chat(
                    &mut twitch_task_handle,
                    &backend_tx,
                    &audio_tx,
                    &command_registry,
                    &tts_queue,
                    &tts_service,
                    &language_config,
                )
                .await;
            }
            FrontendToBackendMessage::AddCommand(command) => {
                add_command(command, &command_registry, &backend_tx).await;
            }
            FrontendToBackendMessage::RemoveCommand(trigger) => {
                remove_command(trigger, &command_registry, &backend_tx).await;
            }
            FrontendToBackendMessage::UpdateCommand(command) => {
                update_command(command, &command_registry, &backend_tx).await;
            }
            FrontendToBackendMessage::ToggleCommand(trigger, enabled) => {
                toggle_command(trigger, enabled, &command_registry, &backend_tx).await;
            }
            FrontendToBackendMessage::GetTTSQueue => {
                send_tts_queue(&tts_queue, &backend_tx).await;
            }
            FrontendToBackendMessage::SkipTTSMessage(message_id) => {
                skip_tts_message(message_id, &tts_queue, &backend_tx).await;
            }
            FrontendToBackendMessage::SkipCurrentTTS => {
                skip_current_tts(&tts_queue, &backend_tx).await;
            }
            FrontendToBackendMessage::DisconnectFromChat(_channel_name) => {
                disconnect_from_chat(&mut twitch_task_handle, &backend_tx);
            }
            FrontendToBackendMessage::EnableOverlay => {
                handle_enable_overlay(&backend_tx, &overlay_ws_state).await;
            }
            FrontendToBackendMessage::DisableOverlay => {
                handle_disable_overlay(&backend_tx).await;
            }
            FrontendToBackendMessage::TestOverlayWheel => {
                handle_test_overlay_wheel(&overlay_ws_state, &backend_tx).await;
            }
        }
    }
}

async fn handle_add_tts_lang(
    lang_code: String,
    language_config: &Arc<RwLock<LanguageConfig>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    let mut config = language_config.write().await;
    config.enable_language(&lang_code);
    if let Err(e) = crate::backend::tts::save_language_config(&config) {
        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
            LogLevel::ERROR,
            format!("Failed to save language config: {}", e),
        ));
    } else {
        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
            LogLevel::INFO,
            format!("Language {} enabled", lang_code),
        ));
        // Send updated language list to frontend
        let updated_langs = config
            .get_all_languages()
            .iter()
            .map(|l| (*l).clone())
            .collect();
        let _ = backend_tx.try_send(BackendToFrontendMessage::TTSLangListUpdated(updated_langs));
    }
}

async fn handle_remove_tts_lang(
    lang_code: String,
    language_config: &Arc<RwLock<LanguageConfig>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    let mut config = language_config.write().await;
    config.disable_language(&lang_code);
    if let Err(e) = crate::backend::tts::save_language_config(&config) {
        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
            LogLevel::ERROR,
            format!("Failed to save language config: {}", e),
        ));
    } else {
        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
            LogLevel::INFO,
            format!("Language {} disabled", lang_code),
        ));
        // Send updated language list to frontend
        let updated_langs = config
            .get_all_languages()
            .iter()
            .map(|l| (*l).clone())
            .collect();
        let _ = backend_tx.try_send(BackendToFrontendMessage::TTSLangListUpdated(updated_langs));
    }
}

fn update_tts_config(
    config: Config,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    let current_config: AppConfig = crate::backend::config::load_config();
    crate::backend::config::save_config(&AppConfig {
        chatbot: current_config.chatbot,
        sfx: current_config.sfx,
        tts: config,
        overlay: current_config.overlay,
    });
    let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        "TTS config updated".to_string(),
    ));
}

fn update_sfx_config(
    config: Config,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    let current_config: AppConfig = crate::backend::config::load_config();
    crate::backend::config::save_config(&AppConfig {
        chatbot: current_config.chatbot,
        sfx: config,
        tts: current_config.tts,
        overlay: current_config.overlay,
    });
    let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        "SFX config updated".to_string(),
    ));
}

fn update_chatbot_config(
    config: ChatbotConfig,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    let current_config: AppConfig = crate::backend::config::load_config();
    crate::backend::config::save_config(&AppConfig {
        chatbot: config,
        sfx: current_config.sfx,
        tts: current_config.tts,
        overlay: current_config.overlay,
    });
    let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        "Chatbot config updated".to_string(),
    ));
}

#[allow(clippy::too_many_arguments)]
async fn connect_to_chat(
    twitch_task_handle: &mut Option<tokio::task::JoinHandle<()>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    audio_tx: &AudioPlaybackSender,
    command_registry: &Arc<RwLock<CommandRegistry>>,
    tts_queue: &TTSQueue,
    tts_service: &Arc<TTSService>,
    language_config: &Arc<RwLock<LanguageConfig>>,
) {
    // Abort any existing connection first
    if let Some(handle) = twitch_task_handle.take() {
        handle.abort();
        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
            LogLevel::INFO,
            "Disconnecting previous session...".to_string(),
        ));
    }

    // Load config to get auth_token and client_id
    let config = crate::backend::config::load_config();
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
    let audio_tx_clone = audio_tx.clone();
    let registry_clone = command_registry.clone();
    let tts_queue_clone = tts_queue.clone();
    let tts_service_clone = tts_service.clone();
    let language_config_clone = language_config.clone();

    // Spawn the twitch handler task and store the handle
    let handle = tokio::spawn(async move {
        handle_twitch_messages(
            twitch_config,
            backend_tx_clone,
            audio_tx_clone,
            registry_clone,
            tts_queue_clone,
            tts_service_clone,
            language_config_clone,
            welcome_message,
        )
        .await;
    });
    *twitch_task_handle = Some(handle);

    let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        "Connecting to Twitch...".to_string(),
    ));
}

async fn add_command(
    command: crate::backend::commands::Command,
    command_registry: &Arc<RwLock<CommandRegistry>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    {
        let mut registry = command_registry.write().await;
        registry.register(command);
        crate::backend::config::save_commands(&registry);
    }
    let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        "Command added".to_string(),
    ));
    let _ = backend_tx.try_send(BackendToFrontendMessage::CommandsUpdated);
}

async fn remove_command(
    trigger: String,
    command_registry: &Arc<RwLock<CommandRegistry>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    {
        let mut registry = command_registry.write().await;
        registry.unregister(&trigger);
        crate::backend::config::save_commands(&registry);
    }
    let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        format!("Command '{}' removed", trigger),
    ));
    let _ = backend_tx.try_send(BackendToFrontendMessage::CommandsUpdated);
}

async fn update_command(
    command: crate::backend::commands::Command,
    command_registry: &Arc<RwLock<CommandRegistry>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    {
        let mut registry = command_registry.write().await;
        registry.register(command);
        crate::backend::config::save_commands(&registry);
    }
    let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        "Command updated".to_string(),
    ));
    let _ = backend_tx.try_send(BackendToFrontendMessage::CommandsUpdated);
}

async fn toggle_command(
    trigger: String,
    enabled: bool,
    command_registry: &Arc<RwLock<CommandRegistry>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    let mut registry = command_registry.write().await;
    if let Some(cmd) = registry.get_mut(&trigger) {
        cmd.enabled = enabled;
        crate::backend::config::save_commands(&registry);
        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
            LogLevel::INFO,
            format!(
                "Command '{}' {}",
                trigger,
                if enabled { "enabled" } else { "disabled" }
            ),
        ));
    }
}

async fn send_tts_queue(
    tts_queue: &TTSQueue,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    // Get all items from queue (including currently playing) and send to frontend
    let queue_items = tts_queue.get_all_with_current().await;
    let ui_queue: Vec<TTSQueueItemUI> = queue_items
        .into_iter()
        .map(|item| TTSQueueItemUI {
            id: item.request.id,
            username: item.request.username,
            text: item.request.text,
            language: item.request.language,
        })
        .collect();
    let _ = backend_tx.try_send(BackendToFrontendMessage::TTSQueueUpdated(ui_queue));
}

async fn skip_tts_message(
    message_id: String,
    tts_queue: &TTSQueue,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    // Check if it's the currently playing item
    let is_current = if let Some(current) = tts_queue.get_currently_playing().await {
        current.request.id == message_id
    } else {
        false
    };

    if is_current {
        // Skip currently playing
        tts_queue.skip_current().await;
    }

    // Send updated queue
    send_tts_queue(tts_queue, backend_tx).await;
}

async fn skip_current_tts(
    tts_queue: &TTSQueue,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    tts_queue.skip_current().await;

    // Send updated queue
    send_tts_queue(tts_queue, backend_tx).await;
}

fn disconnect_from_chat(
    twitch_task_handle: &mut Option<tokio::task::JoinHandle<()>>,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    // Abort the twitch message handler task if it's running
    if let Some(handle) = twitch_task_handle.take() {
        handle.abort();
        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
            LogLevel::INFO,
            "Disconnected from Twitch".to_string(),
        ));
    } else {
        let _ = backend_tx.try_send(BackendToFrontendMessage::CreateLog(
            LogLevel::WARN,
            "Not connected to Twitch".to_string(),
        ));
    }
}

// Overlay handler functions

async fn handle_enable_overlay(
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
    _overlay_ws_state: &crate::backend::overlay::WebSocketState,
) {
    // Update config to enable overlay
    let mut config = crate::backend::config::load_config();
    config.overlay.enabled = true;
    crate::backend::config::save_config(&config);

    let _ = backend_tx.send(BackendToFrontendMessage::OverlayStatusChanged(true)).await;
    let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
        LogLevel::WARN,
        "Overlay enabled. Please restart the application for changes to take effect.".to_string(),
    )).await;
}

async fn handle_disable_overlay(
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    // Update config to disable overlay
    let mut config = crate::backend::config::load_config();
    config.overlay.enabled = false;
    crate::backend::config::save_config(&config);

    let _ = backend_tx.send(BackendToFrontendMessage::OverlayStatusChanged(false)).await;
    let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
        LogLevel::WARN,
        "Overlay disabled. Please restart the application for changes to take effect.".to_string(),
    )).await;
}

async fn handle_test_overlay_wheel(
    overlay_ws_state: &crate::backend::overlay::WebSocketState,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    use crate::backend::overlay::OverlayEvent;

    let test_items = vec![
        "Prize 1".to_string(),
        "Prize 2".to_string(),
        "Prize 3".to_string(),
        "Prize 4".to_string(),
        "Prize 5".to_string(),
        "Prize 6".to_string(),
    ];

    let event = OverlayEvent::TriggerAction {
        action_type: "spin_wheel".to_string(),
        data: serde_json::json!({
            "items": test_items
        }),
    };

    overlay_ws_state.broadcast(event).await;

    let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        "Test wheel spin sent to overlay".to_string(),
    )).await;
}

/// Handle messages from overlay clients (wheel results, position updates, etc.)
pub async fn handle_overlay_client_messages(
    mut rx: tokio::sync::mpsc::UnboundedReceiver<crate::backend::overlay::websocket::OverlayClientMessage>,
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    use crate::backend::overlay::websocket::OverlayClientMessage;

    while let Some(message) = rx.recv().await {
        match message {
            OverlayClientMessage::WheelResult { result, action } => {
                log::info!("Wheel result received: {} with action: {:?}", result, action);

                if let Some(wheel_action) = action {
                    handle_wheel_action(wheel_action, &backend_tx).await;
                }

                let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
                    LogLevel::INFO,
                    format!("Wheel landed on: {}", result),
                )).await;
            }
            OverlayClientMessage::PositionUpdate { element, x, y, scale } => {
                log::info!("Position update for {}: ({}, {}) scale: {}", element, x, y, scale);
                handle_position_update(element, x, y, scale, &backend_tx).await;
            }
            OverlayClientMessage::RequestConfig => {
                log::debug!("Overlay requested configuration");
                // Could send current positions here if needed
            }
        }
    }
}

async fn handle_wheel_action(
    action: crate::backend::overlay::websocket::WheelAction,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    use crate::backend::overlay::websocket::WheelAction;

    match action {
        WheelAction::Ban { username, reason } => {
            let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
                LogLevel::WARN,
                format!("Wheel action: BAN {} - {}", username, reason),
            )).await;
            // TODO: Implement actual ban via Twitch client
            // This would require passing the TwitchClient to this handler
        }
        WheelAction::Timeout { username, duration, reason } => {
            let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
                LogLevel::WARN,
                format!("Wheel action: TIMEOUT {} for {}s - {}", username, duration, reason),
            )).await;
            // TODO: Implement actual timeout via Twitch client
        }
        WheelAction::Unban { username } => {
            let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
                LogLevel::INFO,
                format!("Wheel action: UNBAN {}", username),
            )).await;
            // TODO: Implement actual unban via Twitch client
        }
        WheelAction::RunCommand { command } => {
            let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
                LogLevel::INFO,
                format!("Wheel action: RUN COMMAND {}", command),
            )).await;
            // TODO: Execute chat command
        }
        WheelAction::Nothing => {
            let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
                LogLevel::INFO,
                "Wheel action: Nothing happens".to_string(),
            )).await;
        }
    }
}

async fn handle_position_update(
    element: String,
    x: f32,
    y: f32,
    scale: f32,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    // Update config with new position and scale
    let mut config = crate::backend::config::load_config();

    match element.as_str() {
        "wheel" => {
            config.overlay.positions.wheel.x = x;
            config.overlay.positions.wheel.y = y;
            config.overlay.positions.wheel.scale = scale;
        }
        "alert" => {
            config.overlay.positions.alert.x = x;
            config.overlay.positions.alert.y = y;
            config.overlay.positions.alert.scale = scale;
        }
        "image" => {
            config.overlay.positions.image.x = x;
            config.overlay.positions.image.y = y;
            config.overlay.positions.image.scale = scale;
        }
        "text" => {
            config.overlay.positions.text.x = x;
            config.overlay.positions.text.y = y;
            config.overlay.positions.text.scale = scale;
        }
        _ => {
            log::warn!("Unknown overlay element: {}", element);
            return;
        }
    }

    crate::backend::config::save_config(&config);

    let _ = backend_tx.send(BackendToFrontendMessage::CreateLog(
        LogLevel::INFO,
        format!("Updated {} position to ({:.1}%, {:.1}%) with scale {:.2}x", element, x, y, scale),
    )).await;
}
