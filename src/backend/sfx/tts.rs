use serde::{Deserialize, Serialize};
use std::sync::Arc;

const TTSCONFIG_PATH: &str = "./assets/tts.json";

pub struct TTSMessage {
    text: String,
    lang: Language,
}

pub struct TTSHandler {
    queue: std::collections::VecDeque<TTSMessage>,
    in_rx: tokio::sync::mpsc::Receiver<TTSMessage>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Language {
    alias: String,
    code: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct TTSConfig {
    languages: Vec<Language>,
    banned_phrases: Vec<String>,
}

impl TTSMessage {
    pub async fn request_tts(&self) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let lang = &self.lang;
        let recv = reqwest::get(format!(
            "https://translate.google.com/translate_tts?ie=UTF-8&tl={}&client=tw-ob&q={}",
            lang.code, self.text
        ))
        .await?;
        Ok(recv)
    }
}

#[tokio::test]
async fn test_recv() -> Result<(), Box<dyn std::error::Error>> {
    let lang = Language {
        alias: "us".into(),
        code: "en-US".into(),
    };

    let msg = TTSMessage {
        text: "test".into(),
        lang,
    };

    let tts = msg.request_tts().await?.bytes().await?;

    tokio::fs::write("test.mp3", tts).await?;

    Ok(())
}

impl TTSConfig {
    async fn save(&self) -> Result<(), std::io::Error> {
        let config = serde_json::to_vec(self)?;
        tokio::fs::write(TTSCONFIG_PATH, &config).await?;
        Ok(())
    }
}
