use eframe::egui::{self, CentralPanel, TopBottomPanel};
use egui::{Color32, Label};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

enum Section {
    Home,
    Sfx,
    Tts,
    Settings,
}
#[derive(Debug)]
enum Roles {
    DEFAULT,
    VIP,
    SUBSCRIBER,
    MODERATOR,
}

enum BackendMessageAction {
    RemoveTTSLang(String),
    AddTTSLang(String),
    UpdateConfig {
        channel_name: String,
        auth_token: String,
    },
    UpdateSfxConfig(Config),
    UpdateTTSConfig(Config),
    ConnectToChat(String),
    DisconnectFromChat(String),
}

#[derive(Debug)]
enum FrontendMessageAction {
    GetTTSLangs,
    GetTTSConfig(Config),
    GetSfxConfig(Config),
    GetSfxList,
}
#[derive(Debug)]
struct Config {
    volume: u8,
    enabled: bool,
    permited_roles: Vec<Roles>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub message_id: String,
    pub message_text: String,
    pub badges: Vec<String>,
    pub username: String,
}

impl From<PrivmsgMessage> for ChatMessage {
    fn from(privmsg: PrivmsgMessage) -> Self {
        let badges = privmsg
            .badges
            .into_iter()
            .map(|badge| format!("{}-{}", badge.name, badge.version))
            .collect();
        ChatMessage {
            message_id: privmsg.message_id,
            message_text: privmsg.message_text,
            badges,
            username: privmsg.sender.login,
        }
    }
}
struct ChatbotUILabels {
    bot_status: String,
    connect_button: String,
    tts_status: ButtonStatus,
    sfx_status: ButtonStatus,
}
#[derive(PartialEq)]
enum ButtonStatus {
    ON,
    OFF
}

impl ButtonStatus {
    fn to_string(&self) -> String {
        match self {
            ButtonStatus::ON => "ON".to_string(),
            ButtonStatus::OFF => "OFF".to_string(),
        }
    }
}

enum LogLevel {
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

struct ChatbotConfig {
    channel_name: String,
    auth_token: String,
}

struct Chatbot {
    config: ChatbotConfig,
    selected_section: Section,
    frontend_tx: tokio::sync::mpsc::Sender<BackendMessageAction>,
    frontend_rx: tokio::sync::mpsc::Receiver<FrontendMessageAction>,
    labels: ChatbotUILabels,
    log_messages: Vec<LogMessage>,
}

impl Chatbot {
    fn new(
        channel_name: String,
        auth_token: String,
        frontend_tx: tokio::sync::mpsc::Sender<BackendMessageAction>,
        frontend_rx: tokio::sync::mpsc::Receiver<FrontendMessageAction>,
    ) -> Self {
        Self {
            config: ChatbotConfig {
                channel_name: channel_name.clone(),
                auth_token: auth_token.clone(),
            },
            selected_section: Section::Home,
            frontend_tx: frontend_tx,
            frontend_rx: frontend_rx,
            labels: ChatbotUILabels {
                bot_status: "Disconnected".to_string(),
                connect_button: "Connect".to_string(),
                tts_status: ButtonStatus::ON,
                sfx_status: ButtonStatus::ON,
            },
            log_messages: Vec::new(),
        }
    }

    fn show_home(&mut self, ui: &mut egui::Ui) {
        ui.set_min_height(ui.max_rect().height());
        ui.set_min_width(ui.max_rect().width());
        ui.horizontal(|ui| {
            if ui.button(&self.labels.connect_button).clicked() {
                if self.labels.connect_button == "Connect" {
                    if self.config.auth_token == "" {
                        self.log_messages.push(LogMessage {
                            message: "Tried to connect to the chat without auth token".to_string(),
                            timestamp: chrono::Local::now().to_string(),
                            log_level: LogLevel::ERROR,
                        });
                        return;
                    }
                    self.labels.connect_button = "Disconnect".to_string();
                    let _ = self.frontend_tx.send(BackendMessageAction::ConnectToChat(
                        self.config.channel_name.clone(),
                    ));
                    self.labels.bot_status = "Connected".to_string();
                } else {
                    self.labels.connect_button = "Connect".to_string();
                    let _ = self
                        .frontend_tx
                        .send(BackendMessageAction::DisconnectFromChat(
                            self.config.channel_name.clone(),
                        ));
                    self.labels.bot_status = "Disconnected".to_string();
                };
            }
            ui.label(format!("Status: {}", self.labels.bot_status));
        });
        ui.separator();
        ui.heading(egui::widget_text::RichText::new("Bot logs").color(Color32::WHITE));
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 100.0)
            .max_width(ui.available_width())
            .auto_shrink(false)
            .show(ui, |ui| {
                for mesasge in self.log_messages.iter() {
                    ui.horizontal(|ui| {
                        ui.label(&mesasge.timestamp);
                        ui.label(
                            egui::widget_text::RichText::new(&mesasge.message)
                                .color(mesasge.log_level.color()),
                        );
                    });
                    ui.separator();
                }
            });
    }

    fn show_sfx(&mut self, ui: &mut egui::Ui) {
        ui.set_height(ui.available_height());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("SFX status: ");
                    if ui.button(self.labels.sfx_status.to_string()).clicked() {
                         if self.labels.sfx_status ==  ButtonStatus::ON {
                            self.labels.sfx_status = ButtonStatus::OFF;
                        } else {
                            self.labels.sfx_status = ButtonStatus::ON;
                        }
                    }
                });
                ui.add_space(10.0);
                ui.label("SFX volume (0-1 range):");
                ui.add(egui::Slider::new(&mut 0.92, 0.0..=1.0));
                ui.add_space(10.0);
                ui.label("SFX permissions:");
                ui.checkbox(&mut false, "Subs");
                ui.checkbox(&mut false, "VIPS");
                ui.checkbox(&mut false, "Mods");
                ui.add_space(350.0);
            });
            ui.add_space(250.0);
            ui.separator();
            ui.vertical(|ui| {
                ui.set_height(ui.available_height());
                ui.heading(egui::widget_text::RichText::new("Available sounds").color(Color32::WHITE));
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 100.0)
                    .max_width(ui.available_width())
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        for i in 0..100 {
                            ui.horizontal(|ui| {
                                ui.label(i.to_string());
                                ui.label(
                                    "sound name"
                                );
                            });
                            ui.separator();
                        }
                    });
            });
        });
    }

    fn show_tts(&mut self, ui: &mut egui::Ui) {
        ui.set_height(ui.available_height());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("TTS status: ");
                    if ui.button(self.labels.tts_status.to_string()).clicked() {
                         if self.labels.tts_status ==  ButtonStatus::ON {
                            self.labels.tts_status = ButtonStatus::OFF;
                        } else {
                            self.labels.tts_status = ButtonStatus::ON;
                        }
                    }
                });
                ui.add_space(10.0);
                ui.label("TTS volume (0-1 range):");
                ui.add(egui::Slider::new(&mut 0.92, 0.0..=1.0));
                ui.add_space(10.0);
                ui.label("TTS permissions:");
                ui.checkbox(&mut false, "Subs");
                ui.checkbox(&mut false, "VIPS");
                ui.checkbox(&mut false, "Mods");
                ui.add_space(350.0);
            });
            ui.add_space(250.0);
            ui.separator();
            ui.vertical(|ui| {
                ui.set_height(ui.available_height());
                let available_height = ui.available_height();
                let table = egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::auto())
                    .column(egui_extras::Column::initial(200.0))
                    .column(egui_extras::Column::auto())
                    .min_scrolled_height(0.0)
                    .max_scroll_height(available_height);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("No.");
                        });
                        header.col(|ui| {
                            ui.strong("Language name");
                        });
                        header.col(|ui| {
                            ui.strong("Enabled");
                        });
                    })
                    .body(|mut body| {
                        for row_index in 1..100 {
                            let row_height = 18.0;
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    ui.label(row_index.to_string());
                                });
                                row.col(|ui| {
                                    ui.label("test");
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut false, "");
                                });
                            });
                        }
                    })
            });
        });
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Channel name:");
                ui.text_edit_singleline(&mut self.config.channel_name);
            });
            ui.horizontal(|ui| {
                ui.label("Auth token:");
                ui.text_edit_singleline(&mut self.config.auth_token);
                
            });
            if ui.button("Save").clicked() {
                let _ = self.frontend_tx.send(BackendMessageAction::UpdateConfig {
                    channel_name: self.config.channel_name.clone(),
                    auth_token: self.config.auth_token.clone(),
                });
            }
        });
    }
}

impl eframe::App for Chatbot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(25.0);
                ui.spacing_mut().item_spacing.x = 5.0;
                ui.horizontal(|ui| {
                    ui.image(egui::include_image!("../assets/img/logo.png"));
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
                        self.selected_section = Section::Settings;
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| match self.selected_section {
            Section::Home => self.show_home(ui),
            Section::Sfx => self.show_sfx(ui),
            Section::Tts => self.show_tts(ui),
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
                FrontendMessageAction::GetSfxConfig(config) => {
                    println!("Getting sfx config {:?}", config);
                }
                FrontendMessageAction::GetTTSConfig(config) => {
                    println!("Getting tts config {:?}", config);
                }
                _ => {
                    println!("Received message");
                }
            }
        }

        ctx.request_repaint();
    }
}

#[tokio::main]
async fn main() {
    let (backend_tx, frontend_rx) = tokio::sync::mpsc::channel(100);
    let (frontend_tx, backend_rx) = tokio::sync::mpsc::channel(100);
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_resizable(false),
        ..Default::default()
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
            Box::new(Chatbot::new(
                "".to_string(),
                String::new(),
                frontend_tx,
                frontend_rx,
            ))
        }),
    );
}

async fn handle_messages(channel_name: String, messages: Arc<Mutex<Vec<ChatMessage>>>) {
    let config: ClientConfig<StaticLoginCredentials> = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);
    client.join(channel_name.clone()).unwrap();

    while let Some(message) = incoming_messages.recv().await {
        match message {
            twitch_irc::message::ServerMessage::Privmsg(privmsg) => {
                let chat_message: ChatMessage = privmsg.into();
                println!("Message: {:?}", chat_message);
                messages.lock().unwrap().push(chat_message);
            }
            twitch_irc::message::ServerMessage::Join(join_msg) => {
                println!("User joined: {}", join_msg.user_login);
            }
            twitch_irc::message::ServerMessage::Part(part_msg) => {
                println!("User left: {}", part_msg.user_login);
            }
            twitch_irc::message::ServerMessage::Whisper(whisper_message) => {
                println!(
                    "User {}, whispered message {}",
                    whisper_message.sender.login, whisper_message.message_text
                );
            }
            _ => {
                println!("Received other message: {:?}", message);
            }
        }
    }
}
