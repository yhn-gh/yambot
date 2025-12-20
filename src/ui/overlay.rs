use super::Chatbot;
use egui::{Button, Color32, RichText, ScrollArea, Ui};

impl Chatbot {
    pub fn show_overlay(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Overlay Settings");
            ui.add_space(10.0);

            // Overlay Enable/Disable
            ui.group(|ui| {
                ui.label(RichText::new("Overlay Server").strong().size(16.0));
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

                // Enable/Disable button
                let button_text = if self.overlay_enabled {
                    "Disable Overlay"
                } else {
                    "Enable Overlay"
                };

                if ui.add(Button::new(button_text)).clicked() {
                    let message = if self.overlay_enabled {
                        super::FrontendToBackendMessage::DisableOverlay
                    } else {
                        super::FrontendToBackendMessage::EnableOverlay
                    };
                    let _ = self.frontend_tx.try_send(message);
                }
            });

            ui.add_space(10.0);

            // Port Configuration
            ui.group(|ui| {
                ui.label(RichText::new("Server Configuration").strong().size(16.0));
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

                ui.label(RichText::new("Overlay URL:").strong());
                let url = format!("http://localhost:{}", self.overlay_port);
                ui.horizontal(|ui| {
                    let mut url_mut = url.clone();
                    ui.text_edit_singleline(&mut url_mut);
                    if ui.button("Copy").clicked() {
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

            ui.add_space(10.0);

            // Quick Actions
            ui.group(|ui| {
                ui.label(RichText::new("Quick Actions").strong().size(16.0));
                ui.add_space(5.0);

                if ui
                    .add_enabled(
                        self.overlay_enabled,
                        Button::new("Test Spin Wheel"),
                    )
                    .clicked()
                {
                    let _ = self.frontend_tx.try_send(
                        super::FrontendToBackendMessage::TestOverlayWheel,
                    );
                }
            });

            ui.add_space(10.0);

            // Help Section
            ui.group(|ui| {
                ui.label(RichText::new("Help").strong().size(16.0));
                ui.add_space(5.0);

                ui.label("1. Enable the overlay server above");
                ui.label("2. Copy the overlay URL");
                ui.label("3. In OBS, add a Browser Source");
                ui.label("4. Paste the URL and set dimensions (1920x1080)");
            });
        });
    }
}
