use egui::Color32;

use super::{Chatbot, FrontendToBackendMessage, LogLevel, LogMessage};

impl Chatbot {
    pub fn show_home(&mut self, ui: &mut egui::Ui) {
        ui.set_min_height(ui.max_rect().height());
        ui.set_min_width(ui.max_rect().width());
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [120.0, 35.0],
                    egui::Button::new(&self.labels.connect_button),
                )
                .clicked()
            {
                if self.labels.connect_button == "Connect" {
                    if self.config.auth_token == "" {
                        self.log_messages.push(LogMessage {
                            message: "Tried to connect to the chat without auth token".to_string(),
                            timestamp: chrono::Local::now().to_string(),
                            log_level: LogLevel::ERROR,
                        });
                        return;
                    }
                    // Set status to "Connecting..." and wait for backend response
                    self.labels.bot_status = "Connecting...".to_string();
                    let _ = self
                        .frontend_tx
                        .try_send(FrontendToBackendMessage::ConnectToChat(
                            self.config.channel_name.clone(),
                        ))
                        .unwrap();
                } else {
                    self.labels.connect_button = "Connect".to_string();
                    let _ = self
                        .frontend_tx
                        .try_send(FrontendToBackendMessage::DisconnectFromChat(
                            self.config.channel_name.clone(),
                        ))
                        .unwrap();
                    self.labels.bot_status = "Disconnected".to_string();
                }
            }
        });
        ui.separator();
        ui.heading(egui::widget_text::RichText::new("Bot logs").color(Color32::WHITE));
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                for mesasge in self.log_messages.iter() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(&mesasge.timestamp);
                        ui.add(
                            egui::Label::new(
                                egui::widget_text::RichText::new(&mesasge.message)
                                    .color(mesasge.log_level.color()),
                            )
                            .wrap(),
                        );
                    });
                    ui.separator();
                }
            });
    }
}
