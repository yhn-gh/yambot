use super::queue::{TTSAudioChunk, TTSQueue, TTSRequest};
use log::info;
use urlencoding::encode;

const MAX_TEXT_LENGTH: usize = 200;

pub struct TTSService {
    queue: TTSQueue,
}

impl TTSService {
    pub fn new(queue: TTSQueue) -> Self {
        Self { queue }
    }

    /// Fetch TTS audio data as bytes from Google Translate API
    pub async fn fetch_tts_audio(
        &self,
        text: &str,
        language: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let encoded_text = encode(text);
        let url = format!(
            "https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl={}&client=tw-ob",
            encoded_text, language
        );

        // Download the TTS audio
        let response = reqwest::get(&url).await?;

        if !response.status().is_success() {
            return Err(format!("Failed to generate TTS: HTTP {}", response.status()).into());
        }

        let bytes = response.bytes().await?;

        info!(
            "Fetched TTS audio for text: '{}' in language: {} ({} bytes)",
            text,
            language,
            bytes.len()
        );

        Ok(bytes.to_vec())
    }


    /// Split text into chunks if longer than MAX_TEXT_LENGTH
    pub fn split_text(&self, text: &str) -> Vec<String> {
        if text.len() <= MAX_TEXT_LENGTH {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for word in text.split_whitespace() {
            if current_chunk.len() + word.len() + 1 > MAX_TEXT_LENGTH {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                    current_chunk.clear();
                }
            }
            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(word);
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }

    /// Process TTS request (fetch audio for all chunks)
    /// Returns list of audio chunks
    pub async fn process_request(
        &self,
        request: &TTSRequest,
    ) -> Result<Vec<TTSAudioChunk>, Box<dyn std::error::Error + Send + Sync>> {
        let chunks = self.split_text(&request.text);
        let mut audio_chunks = Vec::new();

        for chunk in chunks.iter() {
            let audio_data = self.fetch_tts_audio(chunk, &request.language).await?;
            audio_chunks.push(TTSAudioChunk { audio_data });
        }

        Ok(audio_chunks)
    }

    pub fn queue(&self) -> &TTSQueue {
        &self.queue
    }
}
