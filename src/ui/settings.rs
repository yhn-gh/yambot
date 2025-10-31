use super::{Chatbot, ChatbotConfig, FrontendToBackendMessage};
use crate::backend::sfx::Format;

impl Chatbot {
    pub fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Channel name:");
                ui.text_edit_singleline(&mut self.config.channel_name);
            });
            ui.horizontal(|ui| {
                ui.label("Auth token:");
                ui.add(egui::TextEdit::singleline(&mut self.config.auth_token).password(true))
            });
            ui.horizontal(|ui| {
                ui.label("Refresh token:");
                ui.add(egui::TextEdit::singleline(&mut self.config.refresh_token).password(true))
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let format = match self.config.sound_format {
                    Format::Wav => ".wav",
                    Format::Opus => ".opus",
                    Format::Mp3 => ".mp3",
                };

                ui.label("Choose sound format:");
                egui::ComboBox::from_label("")
                    .selected_text(format)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.sound_format, Format::Wav, ".wav");
                        ui.selectable_value(&mut self.config.sound_format, Format::Opus, ".opus");
                        ui.selectable_value(&mut self.config.sound_format, Format::Mp3, ".mp3");
                    });
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Welcome message:");
                ui.text_edit_singleline(&mut self.config.welcome_message);
            });
            ui.label("(Optional: Message to send when bot connects. Leave empty to disable)");
            ui.add_space(10.0);
            if ui.button("Save").clicked() {
                let _ = self
                    .frontend_tx
                    .try_send(FrontendToBackendMessage::UpdateConfig(ChatbotConfig {
                        channel_name: self.config.channel_name.clone(),
                        auth_token: self.config.auth_token.clone(),
                        refresh_token: self.config.refresh_token.clone(),
                        sound_format: self.config.sound_format.clone(),
                        welcome_message: self.config.welcome_message.clone(),
                    }))
                    .unwrap();
            }
        });
    }
}
