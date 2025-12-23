use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::backend::commands::CommandRegistry;
use crate::ui::{ChatbotConfig, Config};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub ui: UiConfig,
    pub chatbot: ChatbotConfig,
    pub sfx: Config,
    pub tts: Config,
    #[serde(default)]
    pub overlay: OverlayConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
        }
    }
}

fn default_theme() -> String {
    "Twilight".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OverlayConfig {
    #[serde(default = "default_overlay_enabled")]
    pub enabled: bool,
    #[serde(default = "default_overlay_port")]
    pub port: u16,
    #[serde(default)]
    pub reward_bindings: HashMap<String, RewardAction>,
    #[serde(default)]
    pub positions: OverlayPositions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OverlayPositions {
    #[serde(default)]
    pub wheel: ElementPosition,
    #[serde(default)]
    pub alert: ElementPosition,
    #[serde(default)]
    pub image: ElementPosition,
    #[serde(default)]
    pub text: ElementPosition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ElementPosition {
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_scale")]
    pub scale: f32,
}

impl Default for OverlayPositions {
    fn default() -> Self {
        Self {
            wheel: ElementPosition { x: 50.0, y: 50.0, scale: 1.0 },
            alert: ElementPosition { x: 85.0, y: 10.0, scale: 1.0 },
            image: ElementPosition { x: 50.0, y: 50.0, scale: 1.0 },
            text: ElementPosition { x: 50.0, y: 80.0, scale: 1.0 },
        }
    }
}

impl Default for ElementPosition {
    fn default() -> Self {
        Self { x: 50.0, y: 50.0, scale: 1.0 }
    }
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RewardAction {
    PlaySound(String),
    SpinWheel { items: Vec<String> },
    ShowImage { url: String, duration_ms: u32 },
    ShowText { text: String, duration_ms: u32 },
    TriggerEffect(String),
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enabled: default_overlay_enabled(),
            port: default_overlay_port(),
            reward_bindings: HashMap::new(),
            positions: OverlayPositions::default(),
        }
    }
}

fn default_overlay_enabled() -> bool {
    false
}

fn default_overlay_port() -> u16 {
    3000
}

impl AppConfig {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

pub fn load_config() -> AppConfig {
    let project_root = project_root::get_project_root().unwrap();
    let config_path = project_root.join("config.toml");
    let config: AppConfig = AppConfig::from_file(config_path).unwrap();

    return config;
}

pub fn save_config(config: &AppConfig) {
    let project_root = project_root::get_project_root().unwrap();
    let config_path = project_root.join("config.toml");
    config.to_file(config_path).unwrap();
}

pub fn load_commands() -> CommandRegistry {
    let project_root = project_root::get_project_root().unwrap();
    let commands_path = project_root.join("commands.toml");

    // If file doesn't exist, return empty registry
    if !commands_path.exists() {
        return CommandRegistry::new();
    }

    match fs::read_to_string(&commands_path) {
        Ok(content) => {
            // If file is empty or only whitespace, return empty registry
            if content.trim().is_empty() {
                return CommandRegistry::new();
            }

            toml::from_str(&content).unwrap_or_else(|e| {
                log::error!("Failed to parse commands.toml: {}", e);
                log::error!("File content: {}", content);
                CommandRegistry::new()
            })
        }
        Err(e) => {
            log::error!("Failed to read commands.toml: {}", e);
            CommandRegistry::new()
        }
    }
}

pub fn save_commands(commands: &CommandRegistry) {
    let project_root = project_root::get_project_root().unwrap();
    let commands_path = project_root.join("commands.toml");

    match toml::to_string_pretty(commands) {
        Ok(content) => {
            if let Err(e) = fs::write(&commands_path, content) {
                log::error!("Failed to write commands.toml: {}", e);
            }
        }
        Err(e) => {
            log::error!("Failed to serialize commands: {}", e);
        }
    }
}
