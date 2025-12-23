use super::Chatbot;
use egui::{Button, Color32, RichText, Ui};

impl Chatbot {
    pub fn show_overlay(&mut self, ui: &mut Ui) {
        ui.heading("Overlay Settings");
        ui.add_space(10.0);

        let available_width = ui.available_width();

        // Use Grid layout for better control
        egui::Grid::new("overlay_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .striped(false)
            .show(ui, |ui| {
                // Left column
                ui.vertical(|ui| {
                    ui.set_width(available_width * 0.48);

                    // Server Status
                    ui.group(|ui| {
                        ui.heading("Server Status");
                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label("Status:");
                            if self.overlay_enabled {
                                ui.label(
                                    RichText::new("●")
                                        .color(Color32::from_rgb(0, 255, 0))
                                        .size(16.0),
                                );
                                ui.label("Running");
                            } else {
                                ui.label(
                                    RichText::new("●")
                                        .color(Color32::from_rgb(255, 0, 0))
                                        .size(16.0),
                                );
                                ui.label("Stopped");
                            }
                        });

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label("Port:");
                            ui.add(
                                egui::DragValue::new(&mut self.overlay_port)
                                    .speed(1.0)
                                    .range(1024..=65535),
                            );
                        });

                        ui.add_space(5.0);

                        let button_text = if self.overlay_enabled {
                            "Disable Overlay Server"
                        } else {
                            "Enable Overlay Server"
                        };

                        if ui.button(button_text).clicked() {
                            let message = if self.overlay_enabled {
                                super::FrontendToBackendMessage::DisableOverlay
                            } else {
                                super::FrontendToBackendMessage::EnableOverlay
                            };
                            let _ = self.frontend_tx.try_send(message);
                        }
                    });

                    ui.add_space(10.0);

                    // OBS Browser Source
                    ui.group(|ui| {
                        ui.heading("OBS Browser Source");
                        ui.add_space(5.0);

                        ui.label(RichText::new("Overlay URL:").strong());
                        let url = format!("http://localhost:{}", self.overlay_port);

                        ui.horizontal(|ui| {
                            let mut url_mut = url.clone();
                            ui.add(
                                egui::TextEdit::singleline(&mut url_mut)
                                    .desired_width(ui.available_width() - 70.0),
                            );
                            if ui.button("📋 Copy").clicked() {
                                ui.ctx().copy_text(url.clone());
                            }
                        });

                        ui.add_space(5.0);
                        ui.label(
                            RichText::new("Add this URL as a Browser Source in OBS")
                                .italics()
                                .color(Color32::GRAY),
                        );
                    });
                });

                // Right column
                ui.vertical(|ui| {
                    ui.set_width(available_width * 0.48);

                    // Testing
                    ui.group(|ui| {
                        ui.heading("Testing");
                        ui.add_space(5.0);

                        if ui
                            .add_enabled(self.overlay_enabled, Button::new("🎡 Test Spin Wheel"))
                            .clicked()
                        {
                            let _ = self
                                .frontend_tx
                                .try_send(super::FrontendToBackendMessage::TestOverlayWheel);
                        }

                        if !self.overlay_enabled {
                            ui.add_space(5.0);
                            ui.label(
                                RichText::new("Enable overlay server first to test")
                                    .italics()
                                    .color(Color32::GRAY),
                            );
                        }
                    });

                    ui.add_space(10.0);

                    // Setup Instructions
                    ui.group(|ui| {
                        ui.heading("Setup Instructions");
                        ui.add_space(5.0);

                        ui.label("1. Enable the overlay server");
                        ui.label("2. Click 'Copy' to copy the URL");
                        ui.label("3. In OBS, add Browser Source");
                        ui.label("4. Paste URL (1920x1080)");
                        ui.label("5. Test with the button");
                    });
                });
            });
    }
}
