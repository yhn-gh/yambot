use crate::backend::tts::{TTSQueue, TTSQueueItem};
use crate::ui::{BackendToFrontendMessage, TTSQueueItemUI};
use log::{error, info};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// Audio playback request for SFX system
#[derive(Debug, Clone)]
pub struct AudioPlaybackRequest {
    pub file_path: String,
    pub volume: f32,
    pub is_full_path: bool,
}

// Channel for sending audio playback requests
// Using std::sync::mpsc::Sender wrapped for compatibility with async code
#[derive(Clone)]
pub struct AudioPlaybackSender(pub std::sync::mpsc::Sender<AudioPlaybackRequest>);

impl AudioPlaybackSender {
    pub fn send_sound(
        &self,
        sound: String,
        volume: f32,
    ) -> Result<(), std::sync::mpsc::SendError<AudioPlaybackRequest>> {
        self.0.send(AudioPlaybackRequest {
            file_path: sound,
            volume,
            is_full_path: false,
        })
    }
}

// Dedicated audio playback task that owns the OutputStream
// This solves the Send issue on macOS by keeping OutputStream in a single blocking thread
// Handles both sound effects and TTS audio files
pub fn audio_playback_task(
    rx: std::sync::mpsc::Receiver<AudioPlaybackRequest>,
    stream: OutputStream,
) {
    while let Ok(request) = rx.recv() {
        let audio_path = if request.is_full_path {
            request.file_path
        } else {
            "./assets/sounds/".to_string() + &request.file_path
        };

        if let Ok(file) = File::open(Path::new(&audio_path)) {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                let sink = Sink::connect_new(stream.mixer());
                sink.set_volume(request.volume);
                sink.append(source);
                sink.detach();
            } else {
                error!("Could not decode audio file: {}", audio_path);
            }
        } else {
            error!("Could not open audio file: {}", audio_path);
        }
    }
}

// Dedicated TTS player task that watches the queue and plays TTS sequentially
pub async fn tts_player_task(
    queue: TTSQueue,
    backend_tx: tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    info!("TTS player task started");

    loop {
        // Wait for an item in the queue
        if let Some(item) = queue.pop().await {
            // Check if user is ignored
            if queue.is_user_ignored(&item.request.username).await {
                info!("Skipping TTS for ignored user: {}", item.request.username);
                continue;
            }

            // Set as currently playing
            queue.set_currently_playing(Some(item.clone())).await;

            // Send updated queue to frontend
            send_queue_update(&queue, &backend_tx).await;

            // Load current volume from config
            let volume = {
                let config = crate::backend::config::load_config();
                config.tts.volume as f32
            };

            info!(
                "Playing TTS for user {} in language {}: {} chunk(s)",
                item.request.username,
                item.request.language,
                item.audio_chunks.len()
            );

            // Play audio chunks from memory
            play_tts_item(&item, volume, &queue).await;

            // Clear skip flag
            queue.clear_skip();

            // Clear currently playing
            queue.set_currently_playing(None).await;

            // Send updated queue to frontend
            send_queue_update(&queue, &backend_tx).await;
        } else {
            // Queue is empty, wait a bit before checking again
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

async fn send_queue_update(
    queue: &TTSQueue,
    backend_tx: &tokio::sync::mpsc::Sender<BackendToFrontendMessage>,
) {
    let queue_items = queue.get_all_with_current().await;
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

async fn play_tts_item(item: &TTSQueueItem, volume: f32, queue: &TTSQueue) {
    let audio_chunks = item.audio_chunks.clone();
    let chunk_count = audio_chunks.len();
    let skip_flag = queue.get_skip_flag();

    match tokio::task::spawn_blocking(move || {
        // Create audio stream for TTS playback
        let stream = match rodio::OutputStreamBuilder::open_default_stream() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to open TTS audio stream: {}", e);
                return Err(format!("Failed to open audio stream: {}", e));
            }
        };

        // Play each audio chunk synchronously
        for (index, chunk) in audio_chunks.iter().enumerate() {
            // Check skip flag before playing each chunk
            if skip_flag.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Skip detected, stopping playback");
                return Ok(());
            }

            let cursor = std::io::Cursor::new(chunk.audio_data.clone());
            if let Ok(source) = Decoder::new(BufReader::new(cursor)) {
                let sink = Sink::connect_new(stream.mixer());
                sink.set_volume(volume);
                sink.append(source);

                // Poll while waiting for playback to finish, checking skip flag
                while !sink.empty() {
                    if skip_flag.load(std::sync::atomic::Ordering::SeqCst) {
                        info!("Skip detected during playback, stopping");
                        sink.stop();
                        return Ok(());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }

                info!("Finished playing TTS chunk {}/{}", index + 1, chunk_count);
            } else {
                error!(
                    "Could not decode TTS audio chunk {}/{}",
                    index + 1,
                    chunk_count
                );
            }

            // Small delay between chunks
            if chunk_count > 1 && index < chunk_count - 1 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        Ok(())
    })
    .await
    {
        Ok(Ok(())) => {
            info!("Finished TTS for user {}", item.request.username);
        }
        Ok(Err(e)) => {
            error!("TTS playback error: {}", e);
        }
        Err(e) => {
            error!("TTS task join error: {}", e);
        }
    }
}
