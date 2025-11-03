use egui::{CentralPanel, Color32, TopBottomPanel};
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod home;
pub mod settings;
pub mod sfx;
pub mod tts;

enum Section {
    Home,
    Sfx,
    Tts,
    Commands,
    Settings,
}
#[derive(Debug)]
pub enum FrontendToBackendMessage {
    RemoveTTSLang(String),
    AddTTSLang(String),
    UpdateConfig(ChatbotConfig),
    UpdateSfxConfig(Config),
    UpdateTTSConfig(Config),
    ConnectToChat(String),
    DisconnectFromChat(String),
    AddCommand(crate::backend::commands::Command),
    RemoveCommand(String),
    UpdateCommand(crate::backend::commands::Command),
    ToggleCommand(String, bool),
    GetTTSQueue,
    SkipTTSMessage(String), // Skip by message ID
    SkipCurrentTTS,
}

#[derive(Debug, Clone)]
pub struct TTSQueueItemUI {
    pub id: String,
    pub username: String,
    pub text: String,
    pub language: String,
}

#[derive(Debug)]
pub enum BackendToFrontendMessage {
    ConnectionSuccess(String),
    ConnectionFailure(String),
    TTSLangListUpdated(Vec<crate::backend::tts::Language>),
    SFXListUpdated,
    ChatMessageReceived(String),
    CreateLog(LogLevel, String),
    CommandExecuted(String, String), // (command_name, result)
    CommandsUpdated,
    TTSQueueUpdated(Vec<TTSQueueItemUI>),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    // https://github.com/emilk/egui/discussions/4670
    pub volume: f64,
    pub enabled: bool,
    pub permited_roles: PermitedRoles,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PermitedRoles {
    pub subs: bool,
    pub vips: bool,
    pub mods: bool,
}

struct ChatbotUILabels {
    bot_status: String,
    connect_button: String,
}

#[derive(Debug)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
}

impl LogLevel {
    fn color(&self) -> Color32 {
        match self {
            LogLevel::INFO => Color32::from_rgb(0, 255, 0),
            LogLevel::WARN => Color32::from_rgb(255, 255, 0),
            LogLevel::ERROR => Color32::from_rgb(255, 50, 0),
        }
    }
}
struct LogMessage {
    message: String,
    timestamp: String,
    log_level: LogLevel,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatbotConfig {
    pub channel_name: String,
    pub auth_token: String,
    pub refresh_token: String,
    pub sound_format: crate::backend::sfx::Format,
    #[serde(default)]
    pub welcome_message: String,
}

pub struct Chatbot {
    config: ChatbotConfig,
    selected_section: Section,
    frontend_tx: tokio::sync::mpsc::Sender<FrontendToBackendMessage>,
    frontend_rx: tokio::sync::mpsc::Receiver<BackendToFrontendMessage>,
    labels: ChatbotUILabels,
    log_messages: Vec<LogMessage>,
    sfx_config: Config,
    tts_config: Config,
    tts_languages: Vec<crate::backend::tts::Language>,
    tts_queue: Vec<TTSQueueItemUI>,
    commands: Vec<crate::backend::commands::Command>,
    editing_command: Option<EditingCommand>,
}

pub struct EditingCommand {
    pub original_trigger: String,
    pub trigger: String,
    pub description: String,
    pub permission: usize, // Index into permission options
    pub cooldown: String,
    pub action_type: usize, // Index into action type options
    pub action_param: String,
}

impl Chatbot {
    pub fn new(
        config: ChatbotConfig,
        frontend_tx: tokio::sync::mpsc::Sender<FrontendToBackendMessage>,
        frontend_rx: tokio::sync::mpsc::Receiver<BackendToFrontendMessage>,
        sfx_config: Config,
        tts_config: Config,
        tts_languages: Vec<crate::backend::tts::Language>,
        commands: Vec<crate::backend::commands::Command>,
    ) -> Self {
        Self {
            config,
            selected_section: Section::Home,
            frontend_tx: frontend_tx,
            frontend_rx: frontend_rx,
            labels: ChatbotUILabels {
                bot_status: "Disconnected".to_string(),
                connect_button: "Connect".to_string(),
            },
            log_messages: Vec::new(),
            sfx_config,
            tts_config,
            tts_languages,
            tts_queue: Vec::new(),
            commands,
            editing_command: None,
        }
    }
}

impl eframe::App for Chatbot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;

                // Left section: Logo and title
                ui.horizontal(|ui| {
                    ui.image(egui::include_image!("../../assets/img/logo.png"));
                    ui.heading("Yambot");
                });

                // Center section: Navigation buttons
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;

                            // HOME button
                            let home_btn = if matches!(self.selected_section, Section::Home) {
                                egui::Button::new(egui::RichText::new("HOME").strong())
                                    .fill(Color32::from_rgb(60, 60, 80))
                            } else {
                                egui::Button::new("HOME")
                            };
                            if ui.add_sized([80.0, 30.0], home_btn).clicked() {
                                self.selected_section = Section::Home;
                            }

                            // SFX button
                            let sfx_btn = if matches!(self.selected_section, Section::Sfx) {
                                egui::Button::new(egui::RichText::new("SFX").strong())
                                    .fill(Color32::from_rgb(60, 60, 80))
                            } else {
                                egui::Button::new("SFX")
                            };
                            if ui.add_sized([80.0, 30.0], sfx_btn).clicked() {
                                self.selected_section = Section::Sfx;
                            }

                            // TTS button
                            let tts_btn = if matches!(self.selected_section, Section::Tts) {
                                egui::Button::new(egui::RichText::new("TTS").strong())
                                    .fill(Color32::from_rgb(60, 60, 80))
                            } else {
                                egui::Button::new("TTS")
                            };
                            if ui.add_sized([80.0, 30.0], tts_btn).clicked() {
                                self.selected_section = Section::Tts;
                            }

                            // COMMANDS button
                            let commands_btn = if matches!(self.selected_section, Section::Commands)
                            {
                                egui::Button::new(egui::RichText::new("COMMANDS").strong())
                                    .fill(Color32::from_rgb(60, 60, 80))
                            } else {
                                egui::Button::new("COMMANDS")
                            };
                            if ui.add_sized([95.0, 30.0], commands_btn).clicked() {
                                self.selected_section = Section::Commands;
                            }

                            // SETTINGS button
                            let settings_btn = if matches!(self.selected_section, Section::Settings)
                            {
                                egui::Button::new(egui::RichText::new("SETTINGS").strong())
                                    .fill(Color32::from_rgb(60, 60, 80))
                            } else {
                                egui::Button::new("SETTINGS")
                            };
                            if ui.add_sized([90.0, 30.0], settings_btn).clicked() {
                                self.selected_section = Section::Settings;
                            }
                        });
                    },
                );

                // Right section: Status or empty space for balance
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Status: {}", self.labels.bot_status));
                });
            });

            ui.add_space(5.0);
        });

        CentralPanel::default().show(ctx, |ui| match self.selected_section {
            Section::Home => self.show_home(ui),
            Section::Sfx => self.show_sfx(ui),
            Section::Tts => self.show_tts(ui),
            Section::Commands => self.show_commands(ui),
            Section::Settings => self.show_settings(ui),
        });

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                ui.hyperlink_to("Source code", "https://www.github.com/xyamii/yambot");
            });
        });

        while let Ok(message) = self.frontend_rx.try_recv() {
            match message {
                BackendToFrontendMessage::ConnectionSuccess(response) => {
                    self.labels.bot_status = response;
                    self.labels.connect_button = "Disconnect".to_string();
                }
                BackendToFrontendMessage::ConnectionFailure(response) => {
                    self.labels.bot_status = response;
                    self.labels.connect_button = "Connect".to_string();
                }
                BackendToFrontendMessage::CreateLog(level, message) => {
                    self.log_messages.push(LogMessage {
                        message,
                        timestamp: chrono::Local::now().to_string(),
                        log_level: level,
                    });
                }
                BackendToFrontendMessage::CommandExecuted(command, result) => {
                    self.log_messages.push(LogMessage {
                        message: format!("Command '{}' executed: {}", command, result),
                        timestamp: chrono::Local::now().to_string(),
                        log_level: LogLevel::INFO,
                    });
                }
                BackendToFrontendMessage::CommandsUpdated => {
                    // Command list will be updated on the backend
                }
                BackendToFrontendMessage::TTSLangListUpdated(updated_langs) => {
                    // Update TTS languages with the new list from backend
                    self.tts_languages = updated_langs;
                }
                BackendToFrontendMessage::TTSQueueUpdated(queue) => {
                    self.tts_queue = queue;
                }
                BackendToFrontendMessage::SFXListUpdated => {
                    // Sound list has been updated by the file watcher
                    // The UI will automatically reflect changes since it reads from FILES every frame
                }
                BackendToFrontendMessage::ChatMessageReceived(_) => {
                    // Chat message received
                }
            }
        }

        ctx.request_repaint();
    }
}
