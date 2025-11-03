pub mod languages;
pub mod queue;
pub mod service;

pub use languages::{Language, LanguageConfig};
pub use queue::{TTSAudioChunk, TTSQueue, TTSQueueItem, TTSRequest};
pub use service::TTSService;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const LANGUAGES_CONFIG_FILE: &str = "tts_languages.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSConfig {
    pub languages: LanguageConfig,
}

impl TTSConfig {
    pub fn new() -> Self {
        Self {
            languages: LanguageConfig::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: TTSConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

impl Default for TTSConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Load TTS language configuration
/// Loads from tts_languages.toml file which should contain all language definitions
pub fn load_language_config() -> LanguageConfig {
    let project_root = project_root::get_project_root().unwrap();
    let config_path = project_root.join(LANGUAGES_CONFIG_FILE);

    if config_path.exists() {
        match TTSConfig::from_file(&config_path) {
            Ok(config) => config.languages,
            Err(e) => {
                log::error!("Failed to load TTS language config from {}: {}", config_path.display(), e);
                log::error!("Please ensure tts_languages.toml exists and is properly formatted");
                panic!("Cannot start without valid tts_languages.toml file");
            }
        }
    } else {
        log::error!("TTS language config file not found at: {}", config_path.display());
        log::error!("Please ensure tts_languages.toml exists with all language definitions");
        panic!("Cannot start without tts_languages.toml file");
    }
}

/// Save TTS language configuration
pub fn save_language_config(config: &LanguageConfig) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = project_root::get_project_root().unwrap();
    let config_path = project_root.join(LANGUAGES_CONFIG_FILE);

    let tts_config = TTSConfig {
        languages: config.clone(),
    };

    tts_config.to_file(config_path)?;
    Ok(())
}
