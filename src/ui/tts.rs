use super::Chatbot;

impl Chatbot {
    pub fn show_tts(&mut self, ui: &mut egui::Ui) {
        ui.set_height(ui.available_height());
        ui.horizontal(|ui| {
            // Left panel - Settings and Queue (fixed width)
            ui.vertical(|ui| {
                ui.set_width(400.0);
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label("TTS status: ");
                    if ui
                        .button(if self.tts_config.enabled { "ON" } else { "OFF" })
                        .clicked()
                    {
                        if self.tts_config.enabled {
                            self.tts_config.enabled = false;
                        } else {
                            self.tts_config.enabled = true;
                        }
                        self.frontend_tx
                            .try_send(super::FrontendToBackendMessage::UpdateTTSConfig(
                                self.tts_config.clone(),
                            ))
                            .unwrap();
                    }
                });
                ui.add_space(10.0);
                ui.label("TTS volume (0-1 range):");
                // funny cus this returns giant floating point numbers
                if ui
                    .add(egui::Slider::new(&mut self.tts_config.volume, 0.0..=1.0))
                    .drag_stopped()
                {
                    self.frontend_tx
                        .try_send(super::FrontendToBackendMessage::UpdateTTSConfig(
                            self.tts_config.clone(),
                        ))
                        .unwrap();
                }
                ui.add_space(10.0);
                ui.label("TTS permissions:");
                if ui
                    .checkbox(&mut self.tts_config.permited_roles.subs, "Subs")
                    .changed()
                {
                    self.frontend_tx
                        .try_send(super::FrontendToBackendMessage::UpdateTTSConfig(
                            self.tts_config.clone(),
                        ))
                        .unwrap();
                }
                if ui
                    .checkbox(&mut self.tts_config.permited_roles.vips, "VIPS")
                    .changed()
                {
                    self.frontend_tx
                        .try_send(super::FrontendToBackendMessage::UpdateTTSConfig(
                            self.tts_config.clone(),
                        ))
                        .unwrap();
                }
                if ui
                    .checkbox(&mut self.tts_config.permited_roles.mods, "Mods")
                    .changed()
                {
                    self.frontend_tx
                        .try_send(super::FrontendToBackendMessage::UpdateTTSConfig(
                            self.tts_config.clone(),
                        ))
                        .unwrap();
                }
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // TTS Queue Preview Section
                ui.label(egui::RichText::new("TTS Queue").strong().size(16.0));
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
                    .max_height(300.0)
                    .min_scrolled_height(250.0)
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
                ui.add_space(200.0);
            });
            ui.separator();

            // Right panel - Language Selection
            ui.vertical(|ui| {
                ui.set_height(ui.available_height());
                ui.label(egui::RichText::new("Enabled Languages").strong().size(16.0));
                ui.add_space(10.0);

                let available_height = ui.available_height() - 250.0;
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
                            ui.strong("Code");
                        });
                        header.col(|ui| {
                            ui.strong("Language name");
                        });
                        header.col(|ui| {
                            ui.strong("Enabled");
                        });
                    })
                    .body(|mut body| {
                        let languages = self.tts_languages.clone();
                        for language in languages.iter() {
                            let row_height = 18.0;
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    ui.label(&language.code);
                                });
                                row.col(|ui| {
                                    ui.label(&language.name);
                                });
                                row.col(|ui| {
                                    let mut enabled = language.enabled;
                                    if ui.checkbox(&mut enabled, "").changed() {
                                        if enabled {
                                            self.frontend_tx
                                                .try_send(
                                                    super::FrontendToBackendMessage::AddTTSLang(
                                                        language.code.clone(),
                                                    ),
                                                )
                                                .unwrap();
                                        } else {
                                            self.frontend_tx
                                                .try_send(
                                                    super::FrontendToBackendMessage::RemoveTTSLang(
                                                        language.code.clone(),
                                                    ),
                                                )
                                                .unwrap();
                                        }
                                    }
                                });
                            });
                        }
                    })
            });
        });
    }
}
