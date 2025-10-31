use serde::{ Deserialize, Serialize };
use std::fs;
use std::path::Path;

use crate::backend::commands::CommandRegistry;
use crate::ui::{ ChatbotConfig, Config };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub chatbot: ChatbotConfig,
    pub sfx: Config,
    pub tts: Config,
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
                eprintln!("Failed to parse commands.toml: {}", e);
                eprintln!("File content: {}", content);
                CommandRegistry::new()
            })
        }
        Err(e) => {
            eprintln!("Failed to read commands.toml: {}", e);
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
                eprintln!("Failed to write commands.toml: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize commands: {}", e);
        }
    }
}
