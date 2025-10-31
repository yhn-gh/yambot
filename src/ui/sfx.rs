use egui::Color32;

use super::Chatbot;
use crate::backend::sfx::FILES;

impl Chatbot {
    pub fn show_sfx(&mut self, ui: &mut egui::Ui) {
        ui.set_height(ui.available_height());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("SFX status: ");
                    if ui.button(if self.sfx_config.enabled { "ON" } else { "OFF" }).clicked() {
                        if self.sfx_config.enabled {
                            self.sfx_config.enabled = false;
                        } else {
                            self.sfx_config.enabled = true;
                        }
                        self.frontend_tx
                            .try_send(
                                super::FrontendToBackendMessage::UpdateSfxConfig(
                                    self.sfx_config.clone()
                                )
                            )
                            .unwrap();
                    }
                });
                ui.add_space(10.0);
                ui.label("SFX volume (0-1 range):");
                if ui.add(egui::Slider::new(&mut self.sfx_config.volume, 0.0..=1.0)).drag_stopped() {
                    self.frontend_tx
                        .try_send(
                            super::FrontendToBackendMessage::UpdateSfxConfig(
                                self.sfx_config.clone()
                            )
                        )
                        .unwrap();
                }
                ui.add_space(10.0);
                ui.label("SFX permissions:");
                if ui.checkbox(&mut self.sfx_config.permited_roles.subs, "Subs").changed() {
                    self.frontend_tx
                        .try_send(
                            super::FrontendToBackendMessage::UpdateSfxConfig(
                                self.sfx_config.clone()
                            )
                        )
                        .unwrap();
                }
                if ui.checkbox(&mut self.sfx_config.permited_roles.vips, "VIPS").changed() {
                    self.frontend_tx
                        .try_send(
                            super::FrontendToBackendMessage::UpdateSfxConfig(
                                self.sfx_config.clone()
                            )
                        )
                        .unwrap();
                }
                if ui.checkbox(&mut self.sfx_config.permited_roles.mods, "Mods").changed() {
                    self.frontend_tx
                        .try_send(
                            super::FrontendToBackendMessage::UpdateSfxConfig(
                                self.sfx_config.clone()
                            )
                        )
                        .unwrap();
                }
                ui.add_space(350.0);
            });
            ui.add_space(250.0);
            ui.separator();
            ui.vertical(|ui| {
                ui.set_height(ui.available_height());
                ui.heading(
                    egui::widget_text::RichText::new("Available sounds").color(Color32::WHITE)
                );
                let files = FILES.lock().unwrap();
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 100.0)
                    .max_width(ui.available_width())
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        for (i, file) in files.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label((i + 1).to_string());
                                ui.label(file);
                            });
                            ui.separator();
                        }
                    });
            });
        });
    }
}
