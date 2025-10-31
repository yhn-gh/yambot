use super::context::CommandContext;
use crate::backend::twitch::ChatMessageEvent;

/// Parser for extracting commands from chat messages
#[derive(Debug, Default)]
pub struct CommandParser {
    /// The prefix used for commands (e.g., "!")
    pub prefix: String,
}

impl CommandParser {
    /// Create a new command parser with a given prefix
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    /// Create a command parser with the default "!" prefix
    pub fn with_default_prefix() -> Self {
        Self {
            prefix: "!".to_string(),
        }
    }

    /// Check if a message is a command
    pub fn is_command(&self, message: &str) -> bool {
        message.trim().starts_with(&self.prefix)
    }

    /// Parse a command from a chat message
    pub fn parse(&self, message: ChatMessageEvent) -> Option<CommandContext> {
        let text = message.message.text.trim();

        if !self.is_command(text) {
            return None;
        }

        // Remove the prefix
        let without_prefix = text.strip_prefix(&self.prefix)?;

        // Split into parts
        let parts: Vec<&str> = without_prefix.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        let command_name = parts[0].to_lowercase();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        Some(CommandContext::new(message, command_name, args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_command() {
        let parser = CommandParser::with_default_prefix();
        assert!(parser.is_command("!hello"));
        assert!(parser.is_command("  !hello  "));
        assert!(!parser.is_command("hello"));
        assert!(!parser.is_command("hello !world"));
    }
}
