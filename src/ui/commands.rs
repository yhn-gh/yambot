use super::{Chatbot, EditingCommand};
use crate::backend::commands::{Command, CommandAction, CommandPermission};
use crate::ui::FrontendToBackendMessage;
use egui::{ScrollArea, Ui};

impl Chatbot {
    pub fn show_commands(&mut self, ui: &mut Ui) {
        ui.heading("Command Management");
        ui.separator();

        // Command editor/creator section
        let is_editing = self.editing_command.is_some();
        if is_editing {
            self.show_command_editor(ui);
            ui.separator();
        } else {
            // Add new command section
            ui.group(|ui| {
                ui.heading("Add New Command");
                ui.horizontal(|ui| {
                    if ui.button("Add Example Commands").clicked() {
                        self.add_example_commands();
                    }
                    if ui.button("Create New Command").clicked() {
                        self.start_creating_command();
                    }
                });
            });
            ui.separator();
        }

        // Commands list
        ui.heading("Registered Commands");

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if self.commands.is_empty() {
                    ui.label("No commands registered yet. Add some commands to get started!");
                } else {
                    let mut command_to_delete: Option<usize> = None;
                    let mut command_to_toggle: Option<(String, bool)> = None;
                    let mut command_to_edit: Option<usize> = None;

                    for (idx, command) in self.commands.iter().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(format!("!{}", command.trigger));
                                    ui.label(format!("Description: {}", command.description));
                                    ui.label(format!("Permission: {:?}", command.permission));
                                    ui.label(format!(
                                        "Cooldown: {}s",
                                        if command.cooldown == 0 {
                                            "None".to_string()
                                        } else {
                                            command.cooldown.to_string()
                                        }
                                    ));
                                    ui.label(format!("Action: {}", Self::format_action(&command.action)));
                                    ui.label(format!(
                                        "Status: {}",
                                        if command.enabled { "Enabled" } else { "Disabled" }
                                    ));
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("Delete").clicked() {
                                            command_to_delete = Some(idx);
                                        }

                                        if ui.button("Edit").clicked() {
                                            command_to_edit = Some(idx);
                                        }

                                        if command.enabled {
                                            if ui.button("Disable").clicked() {
                                                command_to_toggle = Some((command.trigger.clone(), false));
                                            }
                                        } else {
                                            if ui.button("Enable").clicked() {
                                                command_to_toggle = Some((command.trigger.clone(), true));
                                            }
                                        }
                                    },
                                );
                            });
                        });

                        ui.add_space(5.0);
                    }

                    // Process actions after iteration
                    if let Some(idx) = command_to_delete {
                        self.delete_command(idx);
                    }
                    if let Some((trigger, enabled)) = command_to_toggle {
                        self.toggle_command(&trigger, enabled);
                    }
                    if let Some(idx) = command_to_edit {
                        self.start_editing_command(idx);
                    }
                }
            });
    }

    fn format_action(action: &CommandAction) -> String {
        match action {
            CommandAction::TextToSpeech { message } => format!("TTS: {}", message),
            CommandAction::SendMessage { message } => format!("Send: {}", message),
            CommandAction::Reply { message } => format!("Reply: {}", message),
            CommandAction::Multiple { actions } => {
                format!("Multiple actions ({})", actions.len())
            }
        }
    }

    fn add_example_commands(&mut self) {
        // Example 1: Simple reply command
        let hello_cmd = Command::new(
            "hello".to_string(),
            "Greet the user".to_string(),
            CommandPermission::Everyone,
            CommandAction::Reply {
                message: "Hello {user}! Welcome to the stream!".to_string(),
            },
        );

        // Example 2: TTS command with permission
        let lurk_cmd = Command::new(
            "lurk".to_string(),
            "Announce lurking".to_string(),
            CommandPermission::Everyone,
            CommandAction::SendMessage {
                message: "{user} is now lurking!".to_string(),
            },
        )
        .with_cooldown(5);

        // Example 3: Mod-only command
        let shoutout_cmd = Command::new(
            "so".to_string(),
            "Shoutout another streamer".to_string(),
            CommandPermission::Moderator,
            CommandAction::SendMessage {
                message: "Check out {args} at https://twitch.tv/{args}".to_string(),
            },
        );

        // Send commands to backend
        let _ = self
            .frontend_tx
            .try_send(FrontendToBackendMessage::AddCommand(hello_cmd.clone()));
        let _ = self
            .frontend_tx
            .try_send(FrontendToBackendMessage::AddCommand(lurk_cmd.clone()));
        let _ = self
            .frontend_tx
            .try_send(FrontendToBackendMessage::AddCommand(shoutout_cmd.clone()));

        // Update local state
        self.commands.push(hello_cmd);
        self.commands.push(lurk_cmd);
        self.commands.push(shoutout_cmd);
    }

    fn delete_command(&mut self, idx: usize) {
        if let Some(command) = self.commands.get(idx) {
            let _ = self
                .frontend_tx
                .try_send(FrontendToBackendMessage::RemoveCommand(
                    command.trigger.clone(),
                ));
            self.commands.remove(idx);
        }
    }

    fn toggle_command(&mut self, trigger: &str, enabled: bool) {
        let _ = self
            .frontend_tx
            .try_send(FrontendToBackendMessage::ToggleCommand(
                trigger.to_string(),
                enabled,
            ));

        // Update local state
        if let Some(cmd) = self.commands.iter_mut().find(|c| c.trigger == trigger) {
            cmd.enabled = enabled;
        }
    }

    fn start_creating_command(&mut self) {
        self.editing_command = Some(EditingCommand {
            original_trigger: String::new(),
            trigger: String::new(),
            description: String::new(),
            permission: 0, // Everyone
            cooldown: "0".to_string(),
            action_type: 0, // Reply
            action_param: String::new(),
        });
    }

    fn start_editing_command(&mut self, idx: usize) {
        if let Some(command) = self.commands.get(idx) {
            let (action_type, action_param) = match &command.action {
                CommandAction::Reply { message } => (0, message.clone()),
                CommandAction::SendMessage { message } => (1, message.clone()),
                CommandAction::TextToSpeech { message } => (2, message.clone()),
                CommandAction::Multiple { .. } => (0, String::new()), // Default to Reply for complex actions
            };

            let permission = match command.permission {
                CommandPermission::Everyone => 0,
                CommandPermission::Subscriber => 1,
                CommandPermission::Vip => 2,
                CommandPermission::Moderator => 3,
                CommandPermission::Broadcaster => 4,
            };

            self.editing_command = Some(EditingCommand {
                original_trigger: command.trigger.clone(),
                trigger: command.trigger.clone(),
                description: command.description.clone(),
                permission,
                cooldown: command.cooldown.to_string(),
                action_type,
                action_param,
            });
        }
    }

    fn show_command_editor(&mut self, ui: &mut Ui) {
        let mut save_clicked = false;
        let mut cancel_clicked = false;

        if let Some(editing) = &mut self.editing_command {
            ui.group(|ui| {
                ui.heading(if editing.original_trigger.is_empty() {
                    "Create New Command"
                } else {
                    "Edit Command"
                });

                ui.horizontal(|ui| {
                    ui.label("Trigger:");
                    ui.text_edit_singleline(&mut editing.trigger);
                    ui.label("(without !)");
                });

                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.text_edit_singleline(&mut editing.description);
                });

                ui.horizontal(|ui| {
                    ui.label("Permission:");
                    egui::ComboBox::from_id_salt("permission_combo")
                        .selected_text(Self::permission_name(editing.permission))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut editing.permission, 0, "Everyone");
                            ui.selectable_value(&mut editing.permission, 1, "Subscriber");
                            ui.selectable_value(&mut editing.permission, 2, "VIP");
                            ui.selectable_value(&mut editing.permission, 3, "Moderator");
                            ui.selectable_value(&mut editing.permission, 4, "Broadcaster");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Cooldown (seconds):");
                    ui.text_edit_singleline(&mut editing.cooldown);
                });

                ui.horizontal(|ui| {
                    ui.label("Action Type:");
                    egui::ComboBox::from_id_salt("action_type_combo")
                        .selected_text(Self::action_type_name(editing.action_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut editing.action_type, 0, "Reply");
                            ui.selectable_value(&mut editing.action_type, 1, "Send Message");
                            ui.selectable_value(&mut editing.action_type, 2, "Text-to-Speech");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(Self::action_param_label(editing.action_type));
                    ui.text_edit_singleline(&mut editing.action_param);
                });

                ui.label("Available placeholders: {user}, {userid}, {args}, {command}");

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save_clicked = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel_clicked = true;
                    }
                });
            });
        }

        // Process button clicks after the borrow ends
        if save_clicked {
            self.save_edited_command();
        }
        if cancel_clicked {
            self.editing_command = None;
        }
    }

    fn permission_name(idx: usize) -> &'static str {
        match idx {
            0 => "Everyone",
            1 => "Subscriber",
            2 => "VIP",
            3 => "Moderator",
            4 => "Broadcaster",
            _ => "Unknown",
        }
    }

    fn action_type_name(idx: usize) -> &'static str {
        match idx {
            0 => "Reply",
            1 => "Send Message",
            2 => "Text-to-Speech",
            _ => "Unknown",
        }
    }

    fn action_param_label(idx: usize) -> &'static str {
        match idx {
            0 => "Reply message:",
            1 => "Message:",
            2 => "TTS message:",
            _ => "Parameter:",
        }
    }

    fn save_edited_command(&mut self) {
        if let Some(editing) = self.editing_command.take() {
            // Validate inputs
            if editing.trigger.trim().is_empty() {
                return;
            }

            let permission = match editing.permission {
                0 => CommandPermission::Everyone,
                1 => CommandPermission::Subscriber,
                2 => CommandPermission::Vip,
                3 => CommandPermission::Moderator,
                4 => CommandPermission::Broadcaster,
                _ => CommandPermission::Everyone,
            };

            let action = match editing.action_type {
                0 => CommandAction::Reply {
                    message: editing.action_param,
                },
                1 => CommandAction::SendMessage {
                    message: editing.action_param,
                },
                2 => CommandAction::TextToSpeech {
                    message: editing.action_param,
                },
                _ => CommandAction::Reply {
                    message: editing.action_param,
                },
            };

            let cooldown = editing.cooldown.parse::<u64>().unwrap_or(0);

            let command = Command::new(
                editing.trigger.clone(),
                editing.description.clone(),
                permission,
                action,
            )
            .with_cooldown(cooldown);

            // If we're editing an existing command, remove the old one first
            if !editing.original_trigger.is_empty() {
                let _ = self
                    .frontend_tx
                    .try_send(FrontendToBackendMessage::RemoveCommand(
                        editing.original_trigger.clone(),
                    ));
                self.commands
                    .retain(|c| c.trigger != editing.original_trigger);
            }

            // Add the new/updated command
            let _ = self
                .frontend_tx
                .try_send(FrontendToBackendMessage::AddCommand(command.clone()));
            self.commands.push(command);
        }
    }
}
