use super::{ FrontendToBackendMessage, Chatbot };
use egui::{Widget, Ui, Response};
use crate::backend::sfx::Format;

static INVALID_CREDENTIALS:&'static str = "\
Credentials inputted above were incorrect;
The user ID will only be cached after a successful chat connection
or when you reverify by pressing the button above,
Credentials were saved, but you should still check whether they are right.
";

static CONNECTION_ERROR:&'static str = "\
Could not reach the Twitch API to verify access and client tokens.
The user ID will only be cached after a successful chat connection
or when you reverify by pressing the button above.
";

struct Verify<'a> {
    clicks: &'a u8,
    verified: &'a bool,
}

impl<'a> Verify<'a> {
    pub fn new(clicks: &'a u8, verified: &'a bool) -> Self {
        Self {
            clicks,
            verified,
        }
    }
}

impl<'a> Widget for Verify<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let clicks = self.clicks;

        let label: String = match clicks {
            0.. if *self.verified => "Verified!".into(),
            0 => "Save".into(),
            1 => "Verifying..".into(),
            2.. => format!("Verifying ({clicks})"),
        };
        egui::Button::new(label).ui(ui)
    }
}
pub enum Label {
    Unreached,
    Invalid,
}

#[derive(Default)]
pub struct Settings {
    clicks: u8,
    pub verified: bool,
    // auth_visibility: bool,
    // client_visibility: bool,
    pub label: Option<Label>,
}

impl Chatbot {
    pub fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // TODO make this not suck ass
            let super::Section::Settings(state) = &mut self.selected_section else {panic!()};

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
            // TODO move to sfx tab
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

            if ui.add(Verify::new(&state.clicks, &state.verified)).clicked() {
                // would panic on integer overflow
                state.clicks += 1;
                let _ = self
                    .frontend_tx
                    .try_send(FrontendToBackendMessage::UpdateConfig(self.config.clone()));
            };
            if let Some(label) = &state.label {
                state.clicks = 0;
                match label {
                    Label::Invalid => {
                        ui.heading("Invalid credentials");
                        ui.label(INVALID_CREDENTIALS)
                    },
                    Label::Unreached => {
                        ui.heading("Connection error");
                        ui.label(CONNECTION_ERROR)
                    },
                };
            };
        });
    }
}
