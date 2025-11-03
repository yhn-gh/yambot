use eframe::egui;
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod audio;
pub mod backend;
pub mod handlers;
pub mod ui;

use audio::{audio_playback_task, tts_player_task, AudioPlaybackSender};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

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

    // Initialize SoundsManager to start file watching
    // Spawn it in a task to keep it alive for the entire application lifetime
    let backend_tx_for_sounds = backend_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let _sounds_manager = backend::sfx::SoundsManager::new(backend_tx_for_sounds)
                .await
                .expect("Failed to initialize SoundsManager");

            // Keep the watcher alive forever
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        });
    });

    // Wrap command registry in Arc<RwLock> for sharing across tasks
    let shared_registry = Arc::new(RwLock::new(command_registry));

    // Create audio playback channel and spawn dedicated audio task in a blocking thread
    // This solves the OutputStream Send issue on macOS by creating OutputStream in a dedicated thread
    let (audio_tx, audio_rx) = std::sync::mpsc::channel::<audio::AudioPlaybackRequest>();
    let audio_tx = AudioPlaybackSender(audio_tx);
    std::thread::spawn(move || {
        // Create the OutputStream inside the thread to avoid Send issues on macOS
        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .expect("Failed to open default audio stream");
        audio_playback_task(audio_rx, stream);
    });

    // Initialize TTS system
    let tts_queue = backend::tts::TTSQueue::new();
    let tts_service = Arc::new(backend::tts::TTSService::new(tts_queue.clone()));
    let language_config = Arc::new(RwLock::new(backend::tts::load_language_config()));

    // Start TTS player task using tokio
    let tts_queue_for_player = tts_queue.clone();
    let backend_tx_for_player = backend_tx.clone();
    tokio::spawn(async move {
        tts_player_task(tts_queue_for_player, backend_tx_for_player).await;
    });

    let registry_clone = shared_registry.clone();
    let audio_tx_clone = audio_tx.clone();
    let tts_queue_clone = tts_queue.clone();
    let tts_service_clone = tts_service.clone();
    let language_config_clone = language_config.clone();
    tokio::spawn(async move {
        handlers::handle_frontend_to_backend_messages(
            backend_rx,
            backend_tx.clone(),
            audio_tx_clone,
            registry_clone,
            tts_queue_clone,
            tts_service_clone,
            language_config_clone,
        )
        .await;
    });
    info!("Starting chatbot");

    // Get initial commands for UI
    let commands = {
        let registry = shared_registry.read().await;
        registry.list().iter().map(|c| (*c).clone()).collect()
    };

    // Get TTS languages for UI
    let tts_languages = {
        let lang_cfg = language_config.read().await;
        lang_cfg
            .get_all_languages()
            .iter()
            .map(|l| (*l).clone())
            .collect()
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
                tts_languages,
                commands,
            )))
        }),
    )
    .map_err(|e| log::error!("Error: {:?}", e));
}
