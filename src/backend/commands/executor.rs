use super::{CommandAction, CommandContext, CommandRegistry};

/// Result of a command execution
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Command executed successfully with an optional response
    Success(Option<String>),
    /// Command failed with an error message
    Error(String),
    /// Command was not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// Command is on cooldown
    OnCooldown(u64), // remaining seconds
}

/// Executor for running commands
#[derive(Debug)]
pub struct CommandExecutor {
    registry: CommandRegistry,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    /// Get a reference to the registry
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Get a mutable reference to the registry
    pub fn registry_mut(&mut self) -> &mut CommandRegistry {
        &mut self.registry
    }

    /// Execute a command
    pub fn execute(&mut self, context: &CommandContext) -> CommandResult {
        // Get the command
        let command = match self.registry.get(&context.command_name) {
            Some(cmd) => cmd,
            None => return CommandResult::NotFound,
        };

        // Check if enabled
        if !command.enabled {
            return CommandResult::NotFound;
        }

        // Check permissions
        if !command.permission.has_permission(context.badges()) {
            return CommandResult::PermissionDenied;
        }

        // Check cooldown
        if self.registry.is_on_cooldown(&context.command_name) {
            if let Some(remaining) = self.registry.remaining_cooldown(&context.command_name) {
                return CommandResult::OnCooldown(remaining);
            }
        }

        // Execute the action
        let result = self.execute_action(&command.action, context);

        // Update cooldown
        if matches!(result, CommandResult::Success(_)) {
            self.registry.update_cooldown(&context.command_name);
        }

        result
    }

    /// Execute a command action
    fn execute_action(&self, action: &CommandAction, context: &CommandContext) -> CommandResult {
        match action {
            CommandAction::TextToSpeech { message } => {
                let processed = context.replace_placeholders(message);
                CommandResult::Success(Some(format!("tts:{}", processed)))
            }
            CommandAction::SendMessage { message } => {
                let processed = context.replace_placeholders(message);
                CommandResult::Success(Some(format!("send:{}", processed)))
            }
            CommandAction::Reply { message } => {
                let processed = context.replace_placeholders(message);
                CommandResult::Success(Some(format!(
                    "reply:{}:{}",
                    context.message_id(),
                    processed
                )))
            }
            CommandAction::Multiple { actions } => {
                let mut results = Vec::new();
                for action in actions {
                    match self.execute_action(action, context) {
                        CommandResult::Success(Some(msg)) => results.push(msg),
                        CommandResult::Success(None) => {}
                        CommandResult::Error(e) => return CommandResult::Error(e),
                        other => return other,
                    }
                }
                if results.is_empty() {
                    CommandResult::Success(None)
                } else {
                    CommandResult::Success(Some(results.join("|")))
                }
            }
        }
    }
}
