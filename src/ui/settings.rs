use super::{ FrontendToBackendMessage, Chatbot, ChatbotConfig };
use egui::{Widget, Ui, Response};
use crate::backend::sfx::Format;

static VERIFICATION_FAILURE:&'static str = "\
Verification failed
Could not reach the Twitch API to verify access and client tokens.
The user ID will only be cached after a successful chat connection
or when you reverify by pressing the button above.
";


struct Verify<'a> {
    clicks: &'a mut u8,
}

impl<'a> Verify<'a> {
    pub fn new(clicks: &'a mut u8) -> Self {
        Self {
            clicks,
        }
    }
}

impl<'a> Widget for Verify<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut clicks = self.clicks;
        let label: String = match clicks {
            0 => "Save".into(),
            1 => "Verifying..".into(),
            2.. => format!("Verifying ({clicks})")
        };
        egui::Button::new(label).ui(ui)
    }
}

#[derive(Default)]
pub struct Settings {
    clicks: u8,
    unreached: bool,
}

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
                ui.label("Client Id:");
                ui.add(egui::TextEdit::singleline(&mut self.config.client_id).password(true))
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

            if let super::Section::Settings(state) = &mut self.selected_section {
                if ui.add(Verify::new(&mut state.clicks)).clicked() {
                    state.clicks += 1;
                    let _ = self
                        .frontend_tx
                        .try_send(FrontendToBackendMessage::UpdateConfig(ChatbotConfig {
                            channel_name: self.config.channel_name.clone(),
                            auth_token: self.config.auth_token.clone(),
                            client_id: self.config.client_id.clone(),
                            sound_format: self.config.sound_format.clone(),
                        }))
                        .unwrap();
                };
                if state.unreached {
                    ui.label(VERIFICATION_FAILURE);
                };
            }; 
        });
    }
}
