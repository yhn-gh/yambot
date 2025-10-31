use super::Chatbot;

impl Chatbot {
    pub fn show_tts(&mut self, ui: &mut egui::Ui) {
        ui.set_height(ui.available_height());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("TTS status: ");
                    if ui.button(if self.tts_config.enabled { "ON" } else { "OFF" }).clicked() {
                        if self.tts_config.enabled {
                            self.tts_config.enabled = false;
                        } else {
                            self.tts_config.enabled = true;
                        }
                        self.frontend_tx
                            .try_send(
                                super::FrontendToBackendMessage::UpdateTTSConfig(
                                    self.tts_config.clone()
                                )
                            )
                            .unwrap();
                    }
                });
                ui.add_space(10.0);
                ui.label("TTS volume (0-1 range):");
                // funny cus this returns giant floating point numbers
                if ui.add(egui::Slider::new(&mut self.tts_config.volume, 0.0..=1.0)).drag_stopped() {
                    self.frontend_tx
                        .try_send(
                            super::FrontendToBackendMessage::UpdateTTSConfig(
                                self.tts_config.clone()
                            )
                        )
                        .unwrap();
                }
                ui.add_space(10.0);
                ui.label("TTS permissions:");
                if ui.checkbox(&mut self.tts_config.permited_roles.subs, "Subs").changed() {
                    self.frontend_tx
                        .try_send(
                            super::FrontendToBackendMessage::UpdateTTSConfig(
                                self.tts_config.clone()
                            )
                        )
                        .unwrap();
                }
                if ui.checkbox(&mut self.tts_config.permited_roles.vips, "VIPS").changed() {
                    self.frontend_tx
                        .try_send(
                            super::FrontendToBackendMessage::UpdateTTSConfig(
                                self.tts_config.clone()
                            )
                        )
                        .unwrap();
                }
                if ui.checkbox(&mut self.tts_config.permited_roles.mods, "Mods").changed() {
                    self.frontend_tx
                        .try_send(
                            super::FrontendToBackendMessage::UpdateTTSConfig(
                                self.tts_config.clone()
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
                let available_height = ui.available_height();
                let table = egui_extras::TableBuilder
                    ::new(ui)
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
}
