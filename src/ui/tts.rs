use super::Chatbot;

impl Chatbot {
    pub fn show_tts(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();

        ui.horizontal_top(|ui| {
            // Left panel - Settings and Queue (2/3 width for more queue space)
            ui.vertical(|ui| {
                ui.set_width(available_width * 0.64);

                // TTS Settings in Grid for compact layout
                egui::Grid::new("tts_settings_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        // Status
                        ui.label("Status:");
                        if ui
                            .button(if self.tts_config.enabled { "ON" } else { "OFF" })
                            .clicked()
                        {
                            self.tts_config.enabled = !self.tts_config.enabled;
                            let _ = self.frontend_tx.try_send(
                                super::FrontendToBackendMessage::UpdateTTSConfig(
                                    self.tts_config.clone(),
                                )
                            );
                        }
                        ui.end_row();

                        // Volume
                        ui.label("Volume:");
                        if ui
                            .add(egui::Slider::new(&mut self.tts_config.volume, 0.0..=1.0))
                            .drag_stopped()
                        {
                            let _ = self.frontend_tx.try_send(
                                super::FrontendToBackendMessage::UpdateTTSConfig(
                                    self.tts_config.clone(),
                                )
                            );
                        }
                        ui.end_row();

                        // Permissions
                        ui.label("Permissions:");
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut self.tts_config.permited_roles.subs, "Subs").changed() {
                                let _ = self.frontend_tx.try_send(
                                    super::FrontendToBackendMessage::UpdateTTSConfig(
                                        self.tts_config.clone(),
                                    )
                                );
                            }
                            if ui.checkbox(&mut self.tts_config.permited_roles.vips, "VIPs").changed() {
                                let _ = self.frontend_tx.try_send(
                                    super::FrontendToBackendMessage::UpdateTTSConfig(
                                        self.tts_config.clone(),
                                    )
                                );
                            }
                            if ui.checkbox(&mut self.tts_config.permited_roles.mods, "Mods").changed() {
                                let _ = self.frontend_tx.try_send(
                                    super::FrontendToBackendMessage::UpdateTTSConfig(
                                        self.tts_config.clone(),
                                    )
                                );
                            }
                        });
                        ui.end_row();
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // TTS Queue Preview Section
                ui.heading("TTS Queue");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("Refresh Queue").clicked() {
                        let _ = self
                            .frontend_tx
                            .try_send(super::FrontendToBackendMessage::GetTTSQueue);
                    }
                    if ui.button("Skip Current").clicked() {
                        let _ = self
                            .frontend_tx
                            .try_send(super::FrontendToBackendMessage::SkipCurrentTTS);
                    }
                });
                ui.add_space(5.0);

                // Queue display with scrollable area
                egui::ScrollArea::vertical()
                    .id_salt("tts_queue_scroll")
                    .show(ui, |ui| {
                        if self.tts_queue.is_empty() {
                            ui.label("Queue is empty");
                        } else {
                            for (index, queue_item) in self.tts_queue.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        let status_text = if index == 0 {
                                            egui::RichText::new("[Playing]")
                                                .color(egui::Color32::GREEN)
                                        } else {
                                            egui::RichText::new(format!("[{}]", index))
                                        };
                                        ui.label(status_text);

                                        ui.label(format!(
                                            "{} ({})",
                                            queue_item.username, queue_item.language
                                        ));

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button("Skip").clicked() {
                                                    let _ = self.frontend_tx.try_send(
                                                    super::FrontendToBackendMessage::SkipTTSMessage(
                                                        queue_item.id.clone()
                                                    )
                                                );
                                                }
                                            },
                                        );
                                    });

                                    // Show full text with word wrap
                                    ui.label(format!("\"{}\"", queue_item.text));
                                });
                            }
                        }
                    });
            });

            ui.separator();

            // Right panel - Language Selection (1/3 width)
            ui.vertical(|ui| {
                ui.set_width(available_width * 0.32);

                ui.heading("Enabled Languages");
                ui.add_space(10.0);

                egui::ScrollArea::vertical()
                    .id_salt("tts_languages_scroll")
                    .show(ui, |ui| {
                        ui.set_width(available_width);
                        if self.tts_languages.is_empty() {
                            ui.label("No languages loaded.");
                        } else {
                            for lang in &self.tts_languages {
                                ui.horizontal(|ui| {
                                    ui.label(&lang.code);
                                    ui.label(&lang.name);
                                    let mut enabled = lang.enabled;
                                    if ui.checkbox(&mut enabled, "").changed() {
                                        if enabled {
                                            let _ = self.frontend_tx.try_send(
                                                super::FrontendToBackendMessage::AddTTSLang(
                                                    lang.code.clone(),
                                                ),
                                            );
                                        } else {
                                            let _ = self.frontend_tx.try_send(
                                                super::FrontendToBackendMessage::RemoveTTSLang(
                                                    lang.code.clone(),
                                                ),
                                            );
                                        }
                                    }
                                });
                            }
                        }
                    });
            });
        });
    }
}
