use egui::{ CentralPanel, Color32, TopBottomPanel };
use serde::{ Deserialize, Serialize };

pub mod home;
pub mod settings;
pub mod sfx;
pub mod tts;

enum Section {
    Home,
    Sfx,
    Tts,
    Settings(settings::Settings),
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
    PlaySound(String),
}

#[derive(Debug)]
pub enum BackendToFrontendMessage {
    ConnectionSuccess(String),
    ConnectionFailure(String),
    TTSLangListUpdated,
    SFXListUpdated,
    ChatMessageReceived(String),
    CreateLog(LogLevel, String),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    // https://github.com/emilk/egui/discussions/4670
    volume: f64,
    enabled: bool,
    permited_roles: PermitedRoles,
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
    pub client_id: String,
    pub sound_format: crate::backend::sfx::Format,
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
}

impl Chatbot {
    pub fn new(
        config: ChatbotConfig,
        frontend_tx: tokio::sync::mpsc::Sender<FrontendToBackendMessage>,
        frontend_rx: tokio::sync::mpsc::Receiver<BackendToFrontendMessage>,
        sfx_config: Config,
        tts_config: Config
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
        }
    }
}

impl eframe::App for Chatbot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(25.0);
                ui.spacing_mut().item_spacing.x = 5.0;
                ui.horizontal(|ui| {
                    ui.image(egui::include_image!("../../assets/img/logo.png"));
                    ui.label("Yambot");
                });
                ui.add_space(ui.available_width() - (ui.available_width() - 495.0));
                ui.horizontal(|ui| {
                    if ui.button("HOME").clicked() {
                        self.selected_section = Section::Home;
                    }
                    if ui.button("SFX").clicked() {
                        self.selected_section = Section::Sfx;
                    }
                    if ui.button("TTS").clicked() {
                        self.selected_section = Section::Tts;
                    }
                    if ui.button("SETTINGS").clicked() {
                        self.selected_section = Section::Settings(settings::Settings::default());
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            match self.selected_section {
                Section::Home => self.show_home(ui),
                Section::Sfx => self.show_sfx(ui),
                Section::Tts => self.show_tts(ui),
                Section::Settings(_) => self.show_settings(ui),
            }
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
                _ => {
                    println!("Received message");
                }
            }
        }

        ctx.request_repaint();
    }
}
