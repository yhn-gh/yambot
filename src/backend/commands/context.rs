use crate::backend::twitch::ChatMessageEvent;

/// Context provided to command execution
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// The chat message that triggered the command
    pub message: ChatMessageEvent,
    /// The command name (without the ! prefix)
    pub command_name: String,
    /// Arguments passed to the command
    pub args: Vec<String>,
}

impl CommandContext {
    /// Create a new command context
    pub fn new(message: ChatMessageEvent, command_name: String, args: Vec<String>) -> Self {
        Self {
            message,
            command_name,
            args,
        }
    }

    /// Get the username of the person who sent the command
    pub fn username(&self) -> &str {
        &self.message.chatter_user_login
    }

    /// Get the user ID of the person who sent the command
    pub fn user_id(&self) -> &str {
        &self.message.chatter_user_id
    }

    /// Get the user's badges
    pub fn badges(&self) -> &[crate::backend::twitch::Badge] {
        &self.message.badges
    }

    /// Get the message text
    pub fn message_text(&self) -> &str {
        &self.message.message.text
    }

    /// Get the message ID
    pub fn message_id(&self) -> &str {
        &self.message.message_id
    }

    /// Replace placeholders in a string with context values
    pub fn replace_placeholders(&self, template: &str) -> String {
        template
            .replace("{user}", self.username())
            .replace("{userid}", self.user_id())
            .replace("{args}", &self.args.join(" "))
            .replace("{command}", &self.command_name)
    }
}
